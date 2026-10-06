//! Maintainer tool for the signed tool-update manifest.
//!
//!   cargo run -p apocalipse-core --example tools_manifest -- keygen
//!   cargo run -p apocalipse-core --example tools_manifest -- hash <file>
//!   ADM_UPDATE_SIGNING_SEED=<base64 seed> \
//!     cargo run -p apocalipse-core --example tools_manifest -- sign unsigned.json > tools-manifest.json
//!
//! `keygen` prints a private seed (keep it offline) and the public key to put in
//! `TRUSTED_TOOL_UPDATE_KEYS`. `hash` prints the length and SHA-256 of a release
//! asset for an `artifacts` entry. `sign` reads an unsigned `UpdateManifest` JSON
//! and prints the signed envelope to publish as `tools-manifest.json`.
use apocalipse_core::{SignedUpdateManifest, UpdateManifest};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("keygen") => {
            let mut seed = [0u8; 32];
            getrandom::fill(&mut seed)?;
            let key = SigningKey::from_bytes(&seed);
            println!(
                "private seed (secret, keep offline): {}",
                STANDARD.encode(seed)
            );
            println!(
                "public key (TRUSTED_TOOL_UPDATE_KEYS): {}",
                STANDARD.encode(key.verifying_key().to_bytes())
            );
        }
        Some("hash") => {
            let path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("usage: hash <file>"))?;
            let bytes = std::fs::read(path)?;
            println!("length: {}", bytes.len());
            println!("sha256: {:x}", Sha256::digest(&bytes));
        }
        Some("sign") => {
            let path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("usage: sign <unsigned.json>"))?;
            let seed = std::env::var("ADM_UPDATE_SIGNING_SEED")?;
            let seed: [u8; 32] = STANDARD
                .decode(seed.trim())?
                .try_into()
                .map_err(|_| anyhow::anyhow!("signing seed must be 32 bytes"))?;
            let manifest: UpdateManifest = serde_json::from_slice(&std::fs::read(path)?)?;
            let signature = SigningKey::from_bytes(&seed).sign(&serde_json::to_vec(&manifest)?);
            let envelope = SignedUpdateManifest {
                signed: manifest,
                signatures: vec![STANDARD.encode(signature.to_bytes())],
            };
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        _ => anyhow::bail!("usage: tools_manifest keygen | hash <file> | sign <unsigned.json>"),
    }
    Ok(())
}
