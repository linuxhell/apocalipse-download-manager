//! Authenticated selected-file transport used by the optional QUIC Link mode.
use anyhow::{Context, Result, bail, ensure};
use noq::{
    ClientConfig, Connection, Endpoint, FourTuple, PathStatus, ServerConfig, TransportConfig,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::Arc,
    time::Duration,
};
use subtle::ConstantTimeEq;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_HEADER: usize = 4096;
const ALPN: &[u8] = b"apocalipse-link/1";

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferReport {
    pub bytes: u64,
    pub sha256: String,
    pub multipath_negotiated: bool,
    pub additional_paths: usize,
    pub failed_paths: usize,
}

#[derive(Serialize, Deserialize)]
struct Request {
    token: String,
}
#[derive(Serialize, Deserialize)]
struct Header {
    bytes: u64,
    sha256: String,
}

fn transport() -> Arc<TransportConfig> {
    let mut config = TransportConfig::default();
    config.max_concurrent_multipath_paths(4);
    config.max_concurrent_bidi_streams(1u8.into());
    config.max_concurrent_uni_streams(0u8.into());
    config.max_idle_timeout(Some(Duration::from_secs(120).try_into().unwrap()));
    Arc::new(config)
}

/// The DER certificate must be delivered through a trusted channel before use.
pub fn server(bind: SocketAddr) -> Result<(Endpoint, Vec<u8>)> {
    crate::init_crypto();
    let certified = rcgen::generate_simple_self_signed(vec!["apocalipse-link.local".into()])?;
    let certificate = certified.cert.der().to_vec();
    let mut tls = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(certificate.clone())],
            PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()).into(),
        )?;
    tls.alpn_protocols = vec![ALPN.to_vec()];
    let mut config = ServerConfig::with_crypto(Arc::new(
        noq::crypto::rustls::QuicServerConfig::try_from(tls)?,
    ));
    config.transport = transport();
    Ok((Endpoint::server(config, bind)?, certificate))
}

pub fn client(bind: SocketAddr, certificate: Vec<u8>) -> Result<Endpoint> {
    crate::init_crypto();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(CertificateDer::from(certificate))?;
    let mut tls = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls.alpn_protocols = vec![ALPN.to_vec()];
    let mut config = ClientConfig::new(Arc::new(noq::crypto::rustls::QuicClientConfig::try_from(
        tls,
    )?));
    config.transport_config(transport());
    let endpoint = Endpoint::client(bind)?;
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}

async fn write_json<T: Serialize>(send: &mut noq::SendStream, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() <= MAX_HEADER, "header too large");
    send.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
    send.write_all(&bytes).await?;
    Ok(())
}

async fn read_json<T: for<'a> Deserialize<'a>>(recv: &mut noq::RecvStream) -> Result<T> {
    let mut length = [0; 4];
    recv.read_exact(&mut length).await?;
    let length = u32::from_be_bytes(length) as usize;
    ensure!(length <= MAX_HEADER, "header too large");
    let mut bytes = vec![0; length];
    recv.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

async fn hash_file(path: &Path) -> Result<Header> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut digest = Sha256::new();
    let mut bytes = 0;
    let mut buffer = vec![0; 256 * 1024];
    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        digest.update(&buffer[..count]);
    }
    Ok(Header {
        bytes,
        sha256: format!("{:x}", digest.finalize()),
    })
}

/// Serves exactly the selected file, never a client-provided filesystem path.
pub async fn serve_once(endpoint: &Endpoint, path: &Path, token: &str) -> Result<()> {
    ensure!(
        token.len() >= 32,
        "use a random token of at least 32 characters"
    );
    let incoming = tokio::time::timeout(Duration::from_secs(30), endpoint.accept())
        .await?
        .context("endpoint closed")?;
    let connection = tokio::time::timeout(Duration::from_secs(10), incoming).await??;
    let (mut send, mut recv) =
        tokio::time::timeout(Duration::from_secs(10), connection.accept_bi()).await??;
    let request: Request =
        tokio::time::timeout(Duration::from_secs(10), read_json(&mut recv)).await??;
    if !bool::from(request.token.as_bytes().ct_eq(token.as_bytes())) {
        connection.close(1u8.into(), b"authentication failed");
        bail!("authentication failed");
    }
    let header = hash_file(path).await?;
    write_json(&mut send, &header).await?;
    let mut file = tokio::fs::File::open(path).await?;
    tokio::io::copy(&mut file, &mut send).await?;
    let mut ack = [0u8; 1];
    recv.read_exact(&mut ack).await?;
    ensure!(ack == [1], "receiver did not verify transfer");
    send.write_all(&[1]).await?;
    send.finish()?;
    // Keep the server-side connection alive until the verified receiver closes it.
    let closed = connection.closed().await;
    ensure!(
        matches!(closed, noq::ConnectionError::ApplicationClosed(ref close)
        if close.error_code == 0u8.into()),
        "transfer confirmation was not received: {closed}"
    );
    Ok(())
}

/// Adds validated network paths to this SAME QUIC connection.
pub async fn add_paths(
    connection: &Connection,
    remote: SocketAddr,
    local_ips: &[IpAddr],
) -> Result<usize> {
    ensure!(local_ips.len() <= 3, "at most three additional paths");
    ensure!(
        local_ips.is_empty() || connection.is_multipath_enabled(),
        "peer did not negotiate multipath"
    );
    let mut count = 0;
    for ip in local_ips {
        ensure!(
            !ip.is_unspecified() && !ip.is_multicast(),
            "invalid source IP"
        );
        // Reject addresses not assigned to this machine before asking the QUIC
        // socket to select one (some platforms silently ignore source hints).
        let _probe = std::net::UdpSocket::bind(SocketAddr::new(*ip, 0))
            .context("source IP is not assigned locally")?;
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                match connection
                    .open_path(FourTuple::new(remote, Some(*ip)), PathStatus::Available)
                    .await
                {
                    Ok(path) => break Ok::<_, noq::PathError>(path),
                    // NEW_CONNECTION_ID can arrive after the TLS handshake completes.
                    Err(noq::PathError::RemoteCidsExhausted) => {
                        tokio::time::sleep(Duration::from_millis(20)).await
                    }
                    Err(error) => break Err(error),
                }
            }
        })
        .await??;
        count += 1;
    }
    Ok(count)
}

pub async fn download(
    endpoint: &Endpoint,
    remote: SocketAddr,
    local_ips: &[IpAddr],
    token: &str,
    destination: &Path,
    max_bytes: u64,
) -> Result<TransferReport> {
    download_with_progress(
        endpoint,
        remote,
        local_ips,
        token,
        destination,
        max_bytes,
        |_, _| Ok(()),
    )
    .await
}

/// Unusable extra interfaces do not discard the working primary QUIC path.
/// The callback can pause or abort; temporary output is removed on abort.
#[allow(clippy::too_many_arguments)]
pub async fn download_with_progress(
    endpoint: &Endpoint,
    remote: SocketAddr,
    local_ips: &[IpAddr],
    token: &str,
    destination: &Path,
    max_bytes: u64,
    mut progress: impl FnMut(u64, u64) -> Result<()>,
) -> Result<TransferReport> {
    ensure!(local_ips.len() <= 3, "at most three additional paths");
    ensure!(
        local_ips
            .iter()
            .all(|ip| !ip.is_unspecified() && !ip.is_multicast()),
        "invalid source IP"
    );
    let connection = tokio::time::timeout(
        Duration::from_secs(8),
        endpoint.connect(remote, "apocalipse-link.local")?,
    )
    .await??;
    let mut additional_paths = 0;
    let mut failed_paths = 0;
    for ip in local_ips {
        match add_paths(&connection, remote, &[*ip]).await {
            Ok(count) => additional_paths += count,
            Err(_) => failed_paths += 1,
        }
    }
    let negotiated = connection.is_multipath_enabled();
    let (mut send, mut recv) = connection.open_bi().await?;
    write_json(
        &mut send,
        &Request {
            token: token.into(),
        },
    )
    .await?;
    let header: Header = read_json(&mut recv).await?;
    progress(0, header.bytes)?;
    ensure!(
        header.bytes <= max_bytes,
        "transfer exceeds configured size limit"
    );
    ensure!(!destination.try_exists()?, "destination already exists");
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temporary = tempfile::NamedTempFile::new_in(parent)?;
    let mut file = tokio::fs::File::from_std(temporary.reopen()?);
    let outcome = async {
        let mut remaining = header.bytes;
        let mut digest = Sha256::new();
        let mut buffer = vec![0; 256 * 1024];
        while remaining > 0 {
            let limit = remaining.min(buffer.len() as u64) as usize;
            let count = recv
                .read(&mut buffer[..limit])
                .await?
                .context("truncated transfer")?;
            ensure!(count > 0, "truncated transfer");
            file.write_all(&buffer[..count]).await?;
            digest.update(&buffer[..count]);
            remaining -= count as u64;
            progress(header.bytes - remaining, header.bytes)?;
        }
        let sha256 = format!("{:x}", digest.finalize());
        ensure!(sha256 == header.sha256, "SHA-256 verification failed");
        file.sync_all().await?;
        send.write_all(&[1]).await?;
        send.finish()?;
        let mut confirmation = [0u8; 1];
        recv.read_exact(&mut confirmation).await?;
        ensure!(confirmation == [1], "missing transfer confirmation");
        ensure!(
            recv.read(&mut [0u8; 1]).await?.is_none(),
            "extra transfer bytes"
        );
        Ok::<_, anyhow::Error>(TransferReport {
            bytes: header.bytes,
            sha256,
            multipath_negotiated: negotiated,
            additional_paths,
            failed_paths,
        })
    }
    .await;
    drop(file);
    let outcome = outcome.and_then(|report| {
        temporary
            .persist_noclobber(destination)
            .map_err(|e| e.error)?;
        Ok(report)
    });
    if outcome.is_ok() {
        connection.close(0u8.into(), b"complete");
    } else {
        connection.close(1u8.into(), b"transfer failed");
    }
    outcome
}
