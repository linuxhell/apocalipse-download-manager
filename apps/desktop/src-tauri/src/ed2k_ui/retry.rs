//! Keeps an ED2K download going when one pass cannot finish. Sources come and go (peers are busy,
//! offline, or only have part of the file), so instead of stopping at the first stall the download
//! waits, asks for sources again and tries once more, until it completes or the user pauses it.
//! The verified parts are never discarded: each pass starts from what is already on disk.
use nibiru_xtreme::ed2k::{StallReason, Stalled};
use std::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// Why one pass ended without finishing.
#[derive(Debug)]
pub enum Failure {
    /// The user paused the task.
    Cancelled,
    /// Neither an ED2K server nor Kad is connected, so there is nobody to ask for sources.
    NotConnected,
    /// The engine reported why it could not finish this pass.
    Stalled { report: Stalled, progressed: bool },
    /// Asking the server or Kad for sources failed.
    Sources(String),
    /// Something waiting will not fix: disk errors, an existing destination, an invalid link.
    Fatal(String),
}

impl Failure {
    /// A short machine-readable reason the window turns into a translated message.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::NotConnected => "not_connected",
            Self::Sources(_) => "sources_failed",
            Self::Fatal(_) => "fatal",
            Self::Stalled { report, .. } => match report.reason {
                StallReason::NoSources => "no_sources",
                StallReason::Unreachable => "unreachable",
                StallReason::MetadataTimeout => "metadata_timeout",
                StallReason::MissingParts => "missing_parts",
                StallReason::QueueTimeout => "queue_timeout",
                StallReason::PeerRefused => "peer_refused",
                StallReason::PartFailed => "part_failed",
            },
        }
    }
    pub fn detail(&self) -> String {
        match self {
            Self::Stalled { report, .. } => report.to_string(),
            Self::Sources(text) | Self::Fatal(text) => text.clone(),
            Self::NotConnected => "not connected to an ED2K server or Kad".into(),
            Self::Cancelled => "cancelled".into(),
        }
    }
    fn progressed(&self) -> bool {
        matches!(
            self,
            Self::Stalled {
                progressed: true,
                ..
            }
        )
    }
}

/// What the window shows while the download waits for its next pass.
#[derive(Debug, Clone)]
pub struct Waiting {
    pub attempt: u32,
    pub wait: Duration,
    pub code: &'static str,
    pub detail: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Completed,
    Cancelled,
    Failed(String),
}

/// The pause before the next pass: short at first, then up to five minutes. A pass that fetched
/// something starts the sequence over, since the sources are evidently working.
pub fn backoff(attempt: u32) -> Duration {
    Duration::from_secs(match attempt {
        0 | 1 => 15,
        2 => 30,
        3 => 60,
        4 => 120,
        _ => 300,
    })
}

/// Runs passes until one completes, the task is cancelled or a failure is fatal.
///
/// `wait` sleeps for the given time but may return early (a server or Kad connection, or the
/// user asking to retry now, or a pause); `waiting` is told what the task is waiting for.
pub async fn run<A, F, W, WF, S>(
    mut attempt: A,
    mut wait: W,
    mut waiting: S,
    cancel: &AtomicBool,
) -> Outcome
where
    A: FnMut(u32) -> F,
    F: Future<Output = Result<(), Failure>>,
    W: FnMut(Duration) -> WF,
    WF: Future<Output = ()>,
    S: FnMut(Waiting),
{
    let mut failures = 0_u32;
    let mut pass = 0_u32;
    loop {
        if cancel.load(Ordering::Acquire) {
            return Outcome::Cancelled;
        }
        let failure = match attempt(pass).await {
            Ok(()) => return Outcome::Completed,
            Err(failure) => failure,
        };
        pass += 1;
        if cancel.load(Ordering::Acquire) {
            return Outcome::Cancelled;
        }
        match failure {
            Failure::Cancelled => return Outcome::Cancelled,
            Failure::Fatal(text) => return Outcome::Failed(text),
            failure => {
                failures = if failure.progressed() {
                    1
                } else {
                    failures + 1
                };
                let pause = backoff(failures);
                waiting(Waiting {
                    attempt: pass,
                    wait: pause,
                    code: failure.code(),
                    detail: failure.detail(),
                });
                wait(pause).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn stalled(reason: StallReason, progressed: bool) -> Failure {
        Failure::Stalled {
            report: Stalled {
                reason,
                sources: 12,
                unreachable: 0,
                metadata_timeouts: 7,
                refused: 0,
                lacking_parts: 5,
                queue_timeouts: 0,
                part_failures: 0,
                missing_parts: 16,
                total_parts: 150,
                last_error: Some("ED2K peer metadata timed out: deadline has elapsed".into()),
            },
            progressed,
        }
    }

    #[tokio::test]
    async fn a_stalled_download_waits_and_tries_again_instead_of_staying_stopped() {
        let cancel = AtomicBool::new(false);
        let passes = Arc::new(Mutex::new(Vec::new()));
        let waits = Arc::new(Mutex::new(Vec::new()));
        let (seen, slept) = (passes.clone(), waits.clone());
        let outcome = run(
            |pass| {
                let seen = seen.clone();
                async move {
                    seen.lock().unwrap().push(pass);
                    match pass {
                        0 => Err(stalled(StallReason::MetadataTimeout, false)),
                        1 => Err(Failure::NotConnected),
                        2 => Err(stalled(StallReason::MissingParts, false)),
                        _ => Ok(()),
                    }
                }
            },
            |pause| {
                let slept = slept.clone();
                async move { slept.lock().unwrap().push(pause) }
            },
            |_| {},
            &cancel,
        )
        .await;
        assert_eq!(outcome, Outcome::Completed);
        assert_eq!(*passes.lock().unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(
            *waits.lock().unwrap(),
            vec![
                Duration::from_secs(15),
                Duration::from_secs(30),
                Duration::from_secs(60)
            ],
            "the pause grows while nothing changes"
        );
    }

    #[tokio::test]
    async fn fetching_something_starts_the_pauses_over() {
        let cancel = AtomicBool::new(false);
        let waits = Arc::new(Mutex::new(Vec::new()));
        let slept = waits.clone();
        run(
            |pass| async move {
                match pass {
                    0..=2 => Err(stalled(StallReason::QueueTimeout, false)),
                    3 => Err(stalled(StallReason::MissingParts, true)),
                    4 => Err(stalled(StallReason::MissingParts, false)),
                    _ => Ok(()),
                }
            },
            |pause| {
                let slept = slept.clone();
                async move { slept.lock().unwrap().push(pause.as_secs()) }
            },
            |_| {},
            &cancel,
        )
        .await;
        assert_eq!(*waits.lock().unwrap(), vec![15, 30, 60, 15, 30]);
    }

    #[tokio::test]
    async fn the_user_can_pause_during_the_wait_and_fatal_errors_are_not_retried() {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let outcome = run(
            |_| async { Err(stalled(StallReason::NoSources, false)) },
            |_| {
                let flag = flag.clone();
                async move { flag.store(true, Ordering::Release) }
            },
            |_| {},
            &cancel,
        )
        .await;
        assert_eq!(outcome, Outcome::Cancelled);

        let cancel = AtomicBool::new(false);
        let tries = Arc::new(Mutex::new(0));
        let counted = tries.clone();
        let outcome = run(
            |_| {
                let counted = counted.clone();
                async move {
                    *counted.lock().unwrap() += 1;
                    Err(Failure::Fatal("destination_already_exists".into()))
                }
            },
            |_| async {},
            |_| {},
            &cancel,
        )
        .await;
        assert_eq!(
            outcome,
            Outcome::Failed("destination_already_exists".into())
        );
        assert_eq!(*tries.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn the_task_is_told_why_it_is_waiting() {
        let cancel = AtomicBool::new(false);
        let told = Arc::new(Mutex::new(Vec::new()));
        let log = told.clone();
        run(
            |pass| async move {
                if pass == 0 {
                    Err(stalled(StallReason::MetadataTimeout, false))
                } else {
                    Ok(())
                }
            },
            |_| async {},
            |waiting| log.lock().unwrap().push(waiting),
            &cancel,
        )
        .await;
        let told = told.lock().unwrap();
        assert_eq!(told.len(), 1);
        assert_eq!(told[0].code, "metadata_timeout");
        assert_eq!(told[0].attempt, 1);
        assert!(told[0].detail.contains("12 sources tried"));
        assert!(told[0].detail.contains("16 of 150 parts still missing"));
        assert!(told[0].detail.contains("metadata timed out"));
    }

    #[test]
    fn every_failure_has_a_distinct_reason_code() {
        let codes: Vec<_> = [
            StallReason::NoSources,
            StallReason::Unreachable,
            StallReason::MetadataTimeout,
            StallReason::MissingParts,
            StallReason::QueueTimeout,
            StallReason::PeerRefused,
            StallReason::PartFailed,
        ]
        .into_iter()
        .map(|reason| stalled(reason, false).code())
        .chain([
            Failure::NotConnected.code(),
            Failure::Sources(String::new()).code(),
        ])
        .collect();
        let unique: std::collections::BTreeSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len(), "{codes:?}");
    }
}
