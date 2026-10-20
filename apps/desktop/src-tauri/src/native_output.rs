//! Cancellation and completion barriers for native engine output publication.
//!
//! A blocking copy outlives its async caller if that future is dropped. Every
//! phase therefore owns a lease, and removal waits for all leases before
//! deleting the destination or the engine's private workspace.
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File},
    hash::Hash,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use tokio::sync::Notify;

pub struct Work {
    cancelled: AtomicBool,
    active: AtomicUsize,
    completion: Notify,
    commit_gate: Mutex<()>,
}

pub struct Lease {
    work: Arc<Work>,
    cancel_on_drop: bool,
}

impl Work {
    pub fn new() -> (Arc<Self>, Lease) {
        let work = Arc::new(Self {
            cancelled: AtomicBool::new(false),
            active: AtomicUsize::new(1),
            completion: Notify::new(),
            commit_gate: Mutex::new(()),
        });
        let lease = Lease {
            work: work.clone(),
            cancel_on_drop: true,
        };
        (work, lease)
    }

    /// Acquire before spawning a blocking phase, while the caller has a lease.
    pub fn phase(self: &Arc<Self>) -> Lease {
        let previous = self.active.fetch_add(1, Ordering::AcqRel);
        assert!(previous > 0, "native work cannot restart after completion");
        Lease {
            work: self.clone(),
            cancel_on_drop: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire) != 0
    }

    pub fn cancel(&self) {
        // Serialize cancellation with the final rename, so publication either
        // precedes cancellation or is prevented by it. Copying holds no lock.
        let _gate = self.commit_gate.lock().unwrap_or_else(|e| e.into_inner());
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn checkpoint(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Acquire) {
            Err("nibiru_cancelled".into())
        } else {
            Ok(())
        }
    }

    pub async fn wait_done(&self) {
        loop {
            let notification = self.completion.notified();
            tokio::pin!(notification);
            notification.as_mut().enable();
            if !self.is_active() {
                return;
            }
            notification.await;
        }
    }
}

impl Lease {
    pub fn complete(mut self) {
        self.cancel_on_drop = false;
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.work.cancel();
        }
        if self.work.active.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.work.completion.notify_waiters();
        }
    }
}

/// Keep a removal reserved across awaits, including error and cancellation
/// paths. The scheduler checks the same set while holding its queue lock.
pub struct RemovalGuard<'a, Id: Eq + Hash> {
    removals: &'a Mutex<HashSet<Id>>,
    ids: Vec<Id>,
}

impl<'a, Id: Clone + Eq + Hash> RemovalGuard<'a, Id> {
    pub fn reserve(removals: &'a Mutex<HashSet<Id>>, ids: &[Id]) -> Result<Self, String> {
        let mut pending = removals.lock().map_err(|e| e.to_string())?;
        if ids.iter().any(|id| pending.contains(id)) {
            return Err("download_removal_in_progress".into());
        }
        pending.extend(ids.iter().cloned());
        Ok(Self {
            removals,
            ids: ids.to_vec(),
        })
    }
}

impl<Id: Eq + Hash> Drop for RemovalGuard<'_, Id> {
    fn drop(&mut self) {
        let mut pending = self.removals.lock().unwrap_or_else(|e| e.into_inner());
        for id in &self.ids {
            pending.remove(id);
        }
    }
}

pub fn workspace(queue_path: &Path, id: impl std::fmt::Display) -> Result<PathBuf, String> {
    Ok(queue_path
        .parent()
        .ok_or("invalid_queue_path")?
        .join("m3u8-work")
        .join(id.to_string()))
}

pub fn publish(
    source: &Path,
    destination: &Path,
    expected_bytes: u64,
    expected_sha256: Option<&str>,
    work: &Work,
) -> Result<u64, String> {
    publish_observed(
        source,
        destination,
        expected_bytes,
        expected_sha256,
        work,
        || {},
    )
}

fn publish_observed(
    source: &Path,
    destination: &Path,
    expected_bytes: u64,
    expected_sha256: Option<&str>,
    work: &Work,
    mut copied_chunk: impl FnMut(),
) -> Result<u64, String> {
    work.checkpoint()?;
    let mut input = File::open(source).map_err(|e| e.to_string())?;
    let metadata = input.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || expected_bytes == 0 || metadata.len() != expected_bytes {
        return Err("nibiru_m3u8_output_size_mismatch".into());
    }
    let parent = destination.parent().ok_or("invalid_destination")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0; 1024 * 1024];
    let mut copied = 0u64;
    loop {
        work.checkpoint()?;
        let count = input.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        copied = copied
            .checked_add(count as u64)
            .filter(|bytes| *bytes <= expected_bytes)
            .ok_or("nibiru_m3u8_output_size_mismatch")?;
        temporary
            .write_all(&buffer[..count])
            .map_err(|e| e.to_string())?;
        if expected_sha256.is_some() {
            hash.update(&buffer[..count]);
        }
        copied_chunk();
    }
    if copied != expected_bytes {
        return Err("nibiru_m3u8_output_size_mismatch".into());
    }
    if expected_sha256
        .is_some_and(|expected| !format!("{:x}", hash.finalize()).eq_ignore_ascii_case(expected))
    {
        return Err("download_checksum_mismatch".into());
    }
    work.checkpoint()?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    {
        let _gate = work.commit_gate.lock().map_err(|e| e.to_string())?;
        work.checkpoint()?;
        temporary
            .persist_noclobber(destination)
            .map_err(|e| e.to_string())?;
    }
    drop(input);
    // A cleanup failure must not turn an already-published valid output into a
    // failed download. Explicit task removal also removes this private cache.
    let _ = fs::remove_file(source);
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

    #[test]
    fn cancellation_during_copy_keeps_resume_source_and_publishes_nothing() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("cache.ts");
        let destination = directory.path().join("video.ts");
        let bytes = vec![0x47; 2 * 1024 * 1024 + 188];
        fs::write(&source, &bytes).unwrap();
        let (work, _lease) = Work::new();
        let result = publish_observed(
            &source,
            &destination,
            bytes.len() as u64,
            None,
            &work,
            || {
                work.cancel();
            },
        );
        assert_eq!(result.unwrap_err(), "nibiru_cancelled");
        assert!(!destination.exists());
        assert_eq!(fs::read(&source).unwrap(), bytes);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn verified_copy_is_atomic_and_never_overwrites_an_existing_destination() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("cache.ts");
        let destination = directory.path().join("video.ts");
        let bytes = b"verified media";
        fs::write(&source, bytes).unwrap();
        let (work, _lease) = Work::new();
        assert_eq!(
            publish(
                &source,
                &destination,
                bytes.len() as u64,
                Some("bad"),
                &work
            )
            .unwrap_err(),
            "download_checksum_mismatch"
        );
        assert!(!destination.exists());
        assert!(source.exists());
        let sha256 = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(
            publish(
                &source,
                &destination,
                bytes.len() as u64,
                Some(&sha256),
                &work
            )
            .unwrap(),
            bytes.len() as u64
        );
        assert_eq!(fs::read(&destination).unwrap(), bytes);
        assert!(!source.exists());
        fs::write(&source, b"replacement!!!").unwrap();
        assert!(publish(&source, &destination, 14, None, &work).is_err());
        assert_eq!(fs::read(&destination).unwrap(), bytes);
        assert!(source.exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[tokio::test]
    async fn removal_waits_for_a_blocking_copy_after_its_async_owner_is_dropped() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("cache.ts");
        let destination = directory.path().join("video.ts");
        fs::write(&source, vec![0x47; 2 * 1024 * 1024]).unwrap();
        let (work, parent) = Work::new();
        let phase = work.phase();
        let copying = work.clone();
        let (entered, copying_started) = tokio::sync::oneshot::channel();
        let (resume, blocked) = mpsc::channel();
        let copy_source = source.clone();
        let copy_destination = destination.clone();
        let copier = tokio::task::spawn_blocking(move || {
            let _phase = phase;
            let mut entered = Some(entered);
            publish_observed(
                &copy_source,
                &copy_destination,
                2 * 1024 * 1024,
                None,
                &copying,
                || {
                    if let Some(entered) = entered.take() {
                        let _ = entered.send(());
                        blocked.recv_timeout(Duration::from_secs(5)).unwrap();
                    }
                },
            )
        });
        copying_started.await.unwrap();
        drop(parent); // Simulate abortion of the async download future.
        assert!(
            tokio::time::timeout(Duration::from_millis(30), work.wait_done())
                .await
                .is_err()
        );
        resume.send(()).unwrap();
        assert_eq!(copier.await.unwrap().unwrap_err(), "nibiru_cancelled");
        tokio::time::timeout(Duration::from_secs(2), work.wait_done())
            .await
            .unwrap();
        assert!(!destination.exists());
        fs::remove_dir_all(directory.path()).unwrap();
        assert!(!source.exists());
    }
}
