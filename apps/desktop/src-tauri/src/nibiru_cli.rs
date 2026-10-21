use serde_json::Value;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::io::AsyncReadExt;
#[path = "nibiru_process.rs"]
mod process_group;
pub(crate) use process_group::spawn as spawn_supervised;
const MAX_JSON: u64 = 2 * 1024 * 1024;
/// Drain both pipes concurrently, bounded by bytes and time. Every exit drops
/// the process-group/job guard, including cancellation and aborting the future.
pub async fn run_json(path: &Path, args: &[String], timeout: Duration) -> Result<Value, String> {
    run_json_observed(path, args, timeout, None, |_| {}).await
}
pub async fn run_json_observed<F: Fn(Value) + Send + Sync>(
    path: &Path,
    args: &[String],
    timeout: Duration,
    cancel: Option<&mut tokio::sync::oneshot::Receiver<()>>,
    observe: F,
) -> Result<Value, String> {
    let mut command = tokio::process::Command::new(path);
    command.args(args);
    run_command_observed(command, timeout, cancel, observe).await
}

async fn run_command_observed<F: Fn(Value) + Send + Sync>(
    mut command: tokio::process::Command,
    timeout: Duration,
    cancel: Option<&mut tokio::sync::oneshot::Receiver<()>>,
    observe: F,
) -> Result<Value, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let (mut child, group) = process_group::spawn(&mut command)?;
    let stdout = child.stdout.take().ok_or("nibiru_stdout_missing")?;
    let stderr = child.stderr.take().ok_or("nibiru_stderr_missing")?;
    let read_out = async move {
        let mut out = Vec::new();
        let mut stdout = stdout.take(MAX_JSON + 1);
        stdout
            .read_to_end(&mut out)
            .await
            .map_err(|_| "nibiru_output_read_failed")?;
        if out.len() as u64 > MAX_JSON {
            return Err("nibiru_output_too_large");
        }
        Ok(out)
    };
    let read_err = async move {
        let mut stderr = stderr;
        let mut buffer = [0; 8192];
        let mut line = Vec::new();
        let mut oversized = false;
        loop {
            let n = stderr
                .read(&mut buffer)
                .await
                .map_err(|_| "nibiru_output_read_failed")?;
            if n == 0 {
                break;
            }
            for byte in &buffer[..n] {
                if *byte == b'\n' || *byte == b'\r' {
                    if let Some(event) = serde_json::from_slice::<Value>(&line)
                        .ok()
                        .filter(|_| !oversized)
                    {
                        if event["schema_version"] == 1
                            && event["event"] == "progress"
                            && event["engine"] == "nibiru-m3u8"
                        {
                            observe(event);
                        }
                    }
                    line.clear();
                    oversized = false;
                } else if line.len() < 65536 {
                    line.push(*byte);
                } else {
                    oversized = true;
                }
            }
        }
        Ok(())
    };
    let outcome = {
        let work = tokio::time::timeout(timeout, async {
            let (output, ()) = tokio::try_join!(read_out, read_err)?;
            // Keep the Unix leader unreaped while descendants hold the pipes,
            // preserving its PID/group identity until the guard terminates it.
            let status = child.wait().await.map_err(|_| "nibiru_wait_failed")?;
            Ok::<_, &str>((output, status))
        });
        tokio::pin!(work);
        let cancelled = async {
            match cancel {
                Some(cancel) => {
                    let _ = cancel.await;
                }
                None => std::future::pending::<()>().await,
            }
        };
        tokio::select! {_=cancelled=>Err("nibiru_cancelled"),result=&mut work=>result.map_err(|_|"nibiru_operation_timed_out").and_then(|r|r)}
    };
    // Kill descendants even if the immediate child exited successfully and
    // closed its pipes. The group/job is owned independently of Child::id().
    drop(group);
    let (output, status) = match outcome {
        Ok(value) => value,
        Err(error) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(error.into());
        }
    };
    if !status.success() {
        // Raw subprocess output can contain tokens and filesystem paths; keep it out of UI logs.
        return Err(format!(
            "nibiru_operation_failed:{}",
            status.code().unwrap_or(-1)
        ));
    }
    let value: Value = serde_json::from_slice(&output).map_err(|_| "nibiru_invalid_json")?;
    if value.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err("nibiru_schema_unsupported".into());
    }
    Ok(value)
}

#[cfg(all(test, any(unix, windows)))]
mod supervision_tests {
    use super::*;
    use std::{path::PathBuf, process::Command, time::Instant};

    fn fixture_command(mode: &str, directory: &Path) -> Command {
        let test = concat!(module_path!(), "::subprocess_fixture");
        let test = test.split_once("::").unwrap().1;
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", test, "--ignored", "--nocapture"])
            .env("ADM_NIBIRU_FIXTURE_MODE", mode)
            .env("ADM_NIBIRU_FIXTURE_DIRECTORY", directory);
        command
    }

    // A real descendant of the test process, launched through the production
    // supervisor. Its heartbeat distinguishes a live orphan from a zombie.
    #[test]
    #[ignore = "internal subprocess fixture; launched by supervision tests"]
    #[allow(
        clippy::zombie_processes,
        reason = "fixture must leave a descendant after its parent exits"
    )]
    fn subprocess_fixture() {
        let Ok(mode) = std::env::var("ADM_NIBIRU_FIXTURE_MODE") else {
            return;
        };
        let directory = PathBuf::from(std::env::var_os("ADM_NIBIRU_FIXTURE_DIRECTORY").unwrap());
        if mode == "leaf" {
            std::fs::write(directory.join("leaf.pid"), std::process::id().to_string()).unwrap();
            for tick in 0..500 {
                std::fs::write(directory.join("heartbeat"), tick.to_string()).unwrap();
                std::thread::sleep(Duration::from_millis(20));
            }
        } else {
            let mut child = fixture_command("leaf", &directory).spawn().unwrap();
            if mode != "exit" {
                let _ = child.wait();
            }
        }
    }

    async fn wait_for_heartbeat(directory: &Path) {
        let started = Instant::now();
        while !directory.join("heartbeat").is_file() {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "descendant did not start"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn assert_heartbeat_stopped(directory: &Path) {
        wait_for_heartbeat(directory).await;
        // Job termination is requested synchronously, but process teardown can
        // take a scheduling turn. Compare after that initial grace period.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let path = directory.join("heartbeat");
        let before = std::fs::read(&path).unwrap();
        tokio::time::sleep(Duration::from_millis(150)).await;
        let after = std::fs::read(path).unwrap();
        assert_eq!(
            before, after,
            "descendant continued after supervisor exited"
        );
    }

    #[tokio::test]
    async fn timeout_terminates_descendant_after_immediate_parent_exits() {
        let directory = tempfile::tempdir().unwrap();
        let result = run_command_observed(
            fixture_command("exit", directory.path()).into(),
            Duration::from_secs(2),
            None,
            |_| {},
        )
        .await;
        assert_eq!(result.unwrap_err(), "nibiru_operation_timed_out");
        assert_heartbeat_stopped(directory.path()).await;
    }

    #[tokio::test]
    async fn cancellation_terminates_descendants() {
        let directory = tempfile::tempdir().unwrap();
        let command = fixture_command("wait", directory.path());
        let (sender, mut receiver) = tokio::sync::oneshot::channel();
        let operation = tokio::spawn(async move {
            run_command_observed(
                command.into(),
                Duration::from_secs(20),
                Some(&mut receiver),
                |_| {},
            )
            .await
        });
        wait_for_heartbeat(directory.path()).await;
        sender.send(()).unwrap();
        assert_eq!(operation.await.unwrap().unwrap_err(), "nibiru_cancelled");
        assert_heartbeat_stopped(directory.path()).await;
    }

    #[tokio::test]
    async fn aborting_future_terminates_descendants() {
        let directory = tempfile::tempdir().unwrap();
        let command = fixture_command("wait", directory.path());
        let operation = tokio::spawn(async move {
            run_command_observed(command.into(), Duration::from_secs(20), None, |_| {}).await
        });
        wait_for_heartbeat(directory.path()).await;
        operation.abort();
        assert!(operation.await.unwrap_err().is_cancelled());
        assert_heartbeat_stopped(directory.path()).await;
    }
}

pub async fn capabilities(path: &Path, engine: &str) -> Result<Value, String> {
    let value = run_json(
        path,
        &["capabilities".into(), "--json".into()],
        Duration::from_secs(10),
    )
    .await?;
    if value.get("engine").and_then(Value::as_str) != Some(engine) {
        return Err("nibiru_engine_identity_mismatch".into());
    }
    Ok(value)
}

pub fn http_url(value: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| "invalid_media_url")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("invalid_media_url".into());
    }
    Ok(url)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[tokio::test]
    async fn child_timeout_terminates_the_process_group_without_waiting_for_pipe_eof() {
        let started = std::time::Instant::now();
        let result = run_json(
            Path::new("/bin/sh"),
            &["-c".into(), "sleep 60".into()],
            Duration::from_millis(50),
        )
        .await;
        assert_eq!(result.unwrap_err(), "nibiru_operation_timed_out");
        assert!(started.elapsed() < Duration::from_secs(2));
    }
    #[tokio::test]
    async fn schema_and_size_limits_reject_incompatible_or_runaway_tools() {
        let result = run_json(
            Path::new("/bin/sh"),
            &["-c".into(), "printf '{\"schema_version\":2}'".into()],
            Duration::from_secs(2),
        )
        .await;
        assert_eq!(result.unwrap_err(), "nibiru_schema_unsupported");
        let result = run_json(
            Path::new("/bin/sh"),
            &["-c".into(), "head -c 2097153 /dev/zero".into()],
            Duration::from_secs(2),
        )
        .await;
        assert_eq!(result.unwrap_err(), "nibiru_output_too_large");
    }
    #[tokio::test]
    async fn bounded_jsonl_progress_is_observed_while_large_stderr_is_drained() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        let count = Arc::new(AtomicUsize::new(0));
        let observed = count.clone();
        let script="head -c 100000 /dev/zero >&2; printf '\\n{\"schema_version\":1,\"event\":\"progress\",\"engine\":\"nibiru-m3u8\",\"bytes\":12}\\n' >&2; printf '{\"schema_version\":1}'";
        let value = run_json_observed(
            Path::new("/bin/sh"),
            &["-c".into(), script.into()],
            Duration::from_secs(2),
            None,
            move |event| {
                assert_eq!(event["bytes"], 12);
                observed.fetch_add(1, Ordering::Relaxed);
            },
        )
        .await
        .unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }
    #[tokio::test]
    async fn cancellation_stops_the_child_and_returns_a_pause_signal() {
        let (sender, mut receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(40)).await;
            let _ = sender.send(());
        });
        let result = run_json_observed(
            Path::new("/bin/sh"),
            &["-c".into(), "sleep 60".into()],
            Duration::from_secs(30),
            Some(&mut receiver),
            |_| {},
        )
        .await;
        assert_eq!(result.unwrap_err(), "nibiru_cancelled");
    }
}

#[cfg(test)]
mod publication_tests {
    use crate::native_output::{publish, Work};
    #[test]
    fn publication_preserves_competing_destination_and_retry_source() {
        let source_dir = tempfile::tempdir().unwrap();
        let destination_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("media.ts");
        let destination = destination_dir.path().join("media.ts");
        std::fs::write(&source, b"complete media").unwrap();
        std::fs::write(&destination, b"user file").unwrap();
        let (work, lease) = Work::new();
        assert!(publish(&source, &destination, 14, None, &work).is_err());
        assert_eq!(std::fs::read(&source).unwrap(), b"complete media");
        assert_eq!(std::fs::read(&destination).unwrap(), b"user file");
        std::fs::remove_file(&destination).unwrap();
        publish(&source, &destination, 14, None, &work).unwrap();
        lease.complete();
        assert_eq!(std::fs::read(&destination).unwrap(), b"complete media");
        assert!(!source.exists());
        assert_eq!(
            std::fs::read_dir(destination_dir.path()).unwrap().count(),
            1
        );
    }
}

#[cfg(test)]
mod native_m3u8_integration {
    use super::*;
    #[tokio::test]
    #[ignore = "requires NIBIRU_M3U8_BIN pointing to the compiled native engine"]
    async fn real_engine_download_progress_and_publication() {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };
        let executable =
            std::path::PathBuf::from(std::env::var_os("NIBIRU_M3U8_BIN").expect("NIBIRU_M3U8_BIN"));
        capabilities(&executable, "nibiru-m3u8").await.unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/index.m3u8", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let mut request = [0; 4096];
                    let n = socket.read(&mut request).await.unwrap();
                    let body = if String::from_utf8_lossy(&request[..n]).contains("/index.m3u8 ") {
                        b"#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXTINF:1,\nsegment.ts\n#EXT-X-ENDLIST\n".to_vec()
                    } else {
                        let mut bytes = vec![0; 376];
                        bytes[0] = 0x47;
                        bytes[188] = 0x47;
                        bytes
                    };
                    let _=socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).as_bytes()).await;
                    let _ = socket.write_all(&body).await;
                });
            }
        });
        let staging = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let output = staging.path().join("media.ts");
        let value = run_json(
            &executable,
            &[
                url,
                "--json".into(),
                "-o".into(),
                output.to_string_lossy().into_owned(),
            ],
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(value["result"]["complete"], true);
        assert_eq!(value["result"]["bytes"], 376);
        let destination = target.path().join("published.ts");
        let (work, lease) = crate::native_output::Work::new();
        crate::native_output::publish(&output, &destination, 376, None, &work).unwrap();
        lease.complete();
        assert_eq!(std::fs::metadata(destination).unwrap().len(), 376);
        server.abort();
    }
}
