use apocalipse_transport_lab::multipath;
use std::{net::SocketAddr, time::Duration};

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn transfers_with_an_extra_path_attempt_and_checks_hash() {
    tokio::time::timeout(Duration::from_secs(20), async {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.bin");
        let destination = directory.path().join("destination.bin");
        let data: Vec<u8> = (0..2 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
        tokio::fs::write(&source, &data).await.unwrap();
        let (server, cert) = multipath::server("127.0.0.1:0".parse().unwrap()).unwrap();
        let remote = server.local_addr().unwrap();
        let client = multipath::client("0.0.0.0:0".parse().unwrap(), cert).unwrap();
        let local_ips = ["127.0.0.2".parse().unwrap()];
        let (served, downloaded) = tokio::join!(
            multipath::serve_once(&server, &source, TOKEN),
            multipath::download(
                &client,
                remote,
                &local_ips,
                TOKEN,
                &destination,
                data.len() as u64
            )
        );
        assert!(served.is_ok(), "server={served:?}; client={downloaded:?}");
        let report = downloaded.unwrap();
        assert!(report.multipath_negotiated);
        #[cfg(not(target_os = "windows"))]
        assert_eq!(report.additional_paths, 1);
        // Windows does not reliably route an unassigned 127.0.0.2 source.
        // Validate both the attempted path and successful primary-path fallback.
        assert_eq!(report.additional_paths + report.failed_paths, 1);
        assert_eq!(report.bytes, data.len() as u64);
        assert_eq!(tokio::fs::read(&destination).await.unwrap(), data);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn rejects_wrong_token_without_creating_output() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.bin");
        let destination = directory.path().join("destination.bin");
        tokio::fs::write(&source, b"private data").await.unwrap();
        let (server, cert) = multipath::server("127.0.0.1:0".parse().unwrap()).unwrap();
        let remote: SocketAddr = server.local_addr().unwrap();
        let client = multipath::client("0.0.0.0:0".parse().unwrap(), cert).unwrap();
        let (served, downloaded) = tokio::join!(
            multipath::serve_once(&server, &source, TOKEN),
            multipath::download(&client, remote, &[], "wrong", &destination, 1024)
        );
        assert!(served.is_err());
        assert!(downloaded.is_err());
        assert!(!destination.exists());
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn existing_output_is_never_overwritten() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.bin");
        let destination = directory.path().join("destination.bin");
        tokio::fs::write(&source, b"new data").await.unwrap();
        tokio::fs::write(&destination, b"keep me").await.unwrap();
        let (server, cert) = multipath::server("127.0.0.1:0".parse().unwrap()).unwrap();
        let remote = server.local_addr().unwrap();
        let client = multipath::client("0.0.0.0:0".parse().unwrap(), cert).unwrap();
        let client_work = multipath::download(&client, remote, &[], TOKEN, &destination, 1024);
        let server_work = multipath::serve_once(&server, &source, TOKEN);
        let (downloaded, _) = tokio::join!(
            client_work,
            tokio::time::timeout(Duration::from_secs(2), server_work)
        );
        assert!(downloaded.is_err());
        assert_eq!(tokio::fs::read(destination).await.unwrap(), b"keep me");
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn unavailable_path_keeps_primary_transfer_and_callback_can_cancel() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.bin");
    let destination = directory.path().join("destination.bin");
    tokio::fs::write(&source, vec![7; 512 * 1024])
        .await
        .unwrap();
    let (server, cert) = multipath::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let remote = server.local_addr().unwrap();
    let client = multipath::client("0.0.0.0:0".parse().unwrap(), cert).unwrap();
    let unavailable = ["192.0.2.123".parse().unwrap()];
    let (served, report) = tokio::join!(
        multipath::serve_once(&server, &source, TOKEN),
        multipath::download(
            &client,
            remote,
            &unavailable,
            TOKEN,
            &destination,
            1024 * 1024
        )
    );
    assert!(served.is_ok());
    let report = report.unwrap();
    assert_eq!(report.failed_paths, 1);
    assert_eq!(report.bytes, 512 * 1024);
    assert_eq!(
        tokio::fs::read(&destination).await.unwrap(),
        vec![7; 512 * 1024]
    );
    tokio::fs::remove_file(&destination).await.unwrap();
    let (_, downloaded) = tokio::join!(
        tokio::time::timeout(
            Duration::from_secs(2),
            multipath::serve_once(&server, &source, TOKEN)
        ),
        multipath::download_with_progress(
            &client,
            remote,
            &[],
            TOKEN,
            &destination,
            1024 * 1024,
            |received, _| {
                if received > 0 {
                    anyhow::bail!("cancelled");
                }
                Ok(())
            }
        )
    );
    assert!(downloaded.is_err());
    assert!(!destination.exists());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}
