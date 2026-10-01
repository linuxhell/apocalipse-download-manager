use anyhow::{Context, Result, bail};
use apocalipse_transport_lab::{dictionary, multipath};
use std::{path::Path, time::Duration};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("serve") if args.len() == 5 => {
            let token = std::env::var("ADM_LINK_LAB_TOKEN")
                .context("set ADM_LINK_LAB_TOKEN to a random 32+ character token")?;
            let (endpoint, cert) = multipath::server(args[2].parse()?)?;
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[4])
                .await?;
            file.write_all(&cert).await?;
            eprintln!(
                "Listening on {}; certificate saved. Share certificate and token securely.",
                endpoint.local_addr()?
            );
            tokio::time::timeout(
                Duration::from_secs(600),
                multipath::serve_once(&endpoint, Path::new(&args[3]), &token),
            )
            .await??;
        }
        Some("get") if args.len() >= 5 => {
            let token = std::env::var("ADM_LINK_LAB_TOKEN").context("set ADM_LINK_LAB_TOKEN")?;
            let cert = tokio::fs::read(&args[3]).await?;
            let endpoint = multipath::client("0.0.0.0:0".parse()?, cert)?;
            let local_ips = args[5..]
                .iter()
                .map(|v| v.parse())
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let report = tokio::time::timeout(
                Duration::from_secs(600),
                multipath::download(
                    &endpoint,
                    args[2].parse()?,
                    &local_ips,
                    &token,
                    Path::new(&args[4]),
                    64 * 1024 * 1024 * 1024,
                ),
            )
            .await??;
            println!("{}", serde_json::to_string(&report)?);
        }
        Some("dictionary-get") if args.len() == 6 => {
            let output = dictionary::fetch(
                &args[2].parse()?,
                &args[3].parse()?,
                &args[4],
                256 * 1024 * 1024,
            )
            .await?;
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[5])
                .await?;
            file.write_all(&output).await?;
            file.sync_all().await?;
            println!("Verified {} bytes", output.len());
        }
        #[cfg(feature = "moqt")]
        Some("moqt-capture") if args.len() == 7 => {
            let count = apocalipse_transport_lab::moqt::capture(
                args[2].parse()?,
                &args[3],
                &args[4],
                Path::new(&args[5]),
                Duration::from_secs(args[6].parse()?),
                256 * 1024 * 1024,
            )
            .await?;
            println!("Captured {count} objects (raw archive; not an MP4)");
        }
        _ => bail!(
            "usage:\n  serve <bind-ip:port> <selected-file> <new-certificate.der>\n  get <server-ip:port> <trusted-certificate.der> <new-output> [additional-local-ip ...]\n  dictionary-get <https-dictionary-url> <https-artifact-url> <expected-sha256> <new-output>\n  moqt-capture <https-relay-url> <broadcast> <track> <new-archive> <seconds> (requires --features moqt)"
        ),
    }
    Ok(())
}
