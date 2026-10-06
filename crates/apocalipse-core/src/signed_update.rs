use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UpdateArtifact {
    pub target: String,
    pub url: String,
    pub length: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UpdateManifest {
    pub schema: u32,
    pub sequence: u64,
    pub version: String,
    pub expires_at: u64,
    pub artifacts: Vec<UpdateArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedUpdateManifest {
    pub signed: UpdateManifest,
    pub signatures: Vec<String>,
}

pub fn verify_update_manifest(
    encoded: &[u8],
    trusted_public_keys: &[String],
    minimum_sequence: u64,
    now: u64,
) -> Result<UpdateManifest> {
    let envelope: SignedUpdateManifest =
        serde_json::from_slice(encoded).context("invalid update manifest")?;
    if envelope.signed.schema != 1 {
        bail!("unsupported update manifest schema")
    }
    if envelope.signed.sequence < minimum_sequence {
        bail!("update metadata rollback detected")
    }
    if envelope.signed.expires_at <= now {
        bail!("update metadata has expired")
    }
    if envelope.signed.artifacts.is_empty() {
        bail!("update manifest has no artifacts")
    }
    for artifact in &envelope.signed.artifacts {
        if artifact.target.is_empty()
            || artifact.length == 0
            || artifact.sha256.len() != 64
            || !artifact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !artifact.url.starts_with("https://")
        {
            bail!("invalid update artifact metadata")
        }
    }
    let canonical = serde_json::to_vec(&envelope.signed)?;
    let verified = trusted_public_keys.iter().any(|encoded_key| {
        let Ok(bytes) = STANDARD.decode(encoded_key) else {
            return false;
        };
        let Ok(bytes) = <Vec<u8> as TryInto<[u8; 32]>>::try_into(bytes) else {
            return false;
        };
        let Ok(key) = VerifyingKey::from_bytes(&bytes) else {
            return false;
        };
        envelope.signatures.iter().any(|encoded_signature| {
            STANDARD
                .decode(encoded_signature)
                .ok()
                .and_then(|bytes| Signature::from_slice(&bytes).ok())
                .is_some_and(|signature| key.verify(&canonical, &signature).is_ok())
        })
    });
    if !verified {
        bail!("update manifest signature is not trusted")
    }
    Ok(envelope.signed)
}

/// Manifest target naming for a managed tool: `<tool>/<platform>-<architecture>`,
/// e.g. `yt-dlp/windows-x86_64`. The artifact hash is the SHA-256 of the exact
/// release asset the app downloads (the archive when the tool ships as one).
pub fn tool_artifact_target(tool: &str, platform: &str, architecture: &str) -> String {
    format!("{tool}/{platform}-{architecture}")
}

/// Verifies a downloaded tool asset against a signed manifest: the signature,
/// anti-rollback sequence and expiry are checked first, then the asset's size
/// and SHA-256 must match the entry for `target`. Returns the manifest sequence
/// so the caller can persist it as the new rollback floor.
pub fn verify_tool_download(
    encoded_manifest: &[u8],
    trusted_public_keys: &[String],
    minimum_sequence: u64,
    now: u64,
    target: &str,
    bytes: &[u8],
) -> Result<u64> {
    use sha2::{Digest, Sha256};
    let manifest =
        verify_update_manifest(encoded_manifest, trusted_public_keys, minimum_sequence, now)?;
    let artifact = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.target == target)
        .with_context(|| format!("{target} is not listed in the signed manifest"))?;
    if artifact.length != bytes.len() as u64 {
        bail!("{target} size does not match the signed manifest")
    }
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !artifact.sha256.eq_ignore_ascii_case(&actual) {
        bail!("{target} SHA-256 does not match the signed manifest")
    }
    Ok(manifest.sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn signed(sequence: u64, expires_at: u64) -> (Vec<u8>, String) {
        let key = SigningKey::from_bytes(&[7; 32]);
        let manifest = UpdateManifest {
            schema: 1,
            sequence,
            version: "0.5.0".into(),
            expires_at,
            artifacts: vec![UpdateArtifact {
                target: "x86_64-pc-windows-msvc".into(),
                url: "https://downloads.example/apocalipse.exe".into(),
                length: 42,
                sha256: "a".repeat(64),
            }],
        };
        let signature = key.sign(&serde_json::to_vec(&manifest).unwrap());
        let envelope = SignedUpdateManifest {
            signed: manifest,
            signatures: vec![STANDARD.encode(signature.to_bytes())],
        };
        (
            serde_json::to_vec(&envelope).unwrap(),
            STANDARD.encode(key.verifying_key().to_bytes()),
        )
    }

    #[test]
    fn accepts_trusted_manifest_and_rejects_rollback() {
        let (encoded, public_key) = signed(12, 2_000);
        assert_eq!(
            verify_update_manifest(&encoded, &[public_key.clone()], 12, 1_000)
                .unwrap()
                .sequence,
            12
        );
        assert!(verify_update_manifest(&encoded, &[public_key], 13, 1_000).is_err());
    }

    #[test]
    fn rejects_tampering_and_expiration() {
        let (encoded, public_key) = signed(12, 2_000);
        assert!(verify_update_manifest(&encoded, &[public_key.clone()], 0, 2_000).is_err());
        let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        value["signed"]["version"] = serde_json::json!("9.9.9");
        assert!(verify_update_manifest(
            &serde_json::to_vec(&value).unwrap(),
            &[public_key],
            0,
            1_000
        )
        .is_err());
    }

    fn signed_tool(bytes: &[u8], sequence: u64) -> (Vec<u8>, String) {
        use sha2::{Digest, Sha256};
        let key = SigningKey::from_bytes(&[9; 32]);
        let manifest = UpdateManifest {
            schema: 1,
            sequence,
            version: "tools-1".into(),
            expires_at: 5_000,
            artifacts: vec![UpdateArtifact {
                target: tool_artifact_target("yt-dlp", "windows", "x86_64"),
                url: "https://github.com/yt-dlp/yt-dlp/releases/download/x/yt-dlp.exe".into(),
                length: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(bytes)),
            }],
        };
        let signature = key.sign(&serde_json::to_vec(&manifest).unwrap());
        let envelope = SignedUpdateManifest {
            signed: manifest,
            signatures: vec![STANDARD.encode(signature.to_bytes())],
        };
        (
            serde_json::to_vec(&envelope).unwrap(),
            STANDARD.encode(key.verifying_key().to_bytes()),
        )
    }

    #[test]
    fn tool_download_must_match_the_signed_hash_and_size() {
        let payload = vec![7u8; 64];
        let (encoded, key) = signed_tool(&payload, 3);
        let target = tool_artifact_target("yt-dlp", "windows", "x86_64");
        assert_eq!(
            verify_tool_download(&encoded, &[key.clone()], 3, 1_000, &target, &payload).unwrap(),
            3
        );
        let mut tampered = payload.clone();
        tampered[0] ^= 1;
        assert!(
            verify_tool_download(&encoded, &[key.clone()], 0, 1_000, &target, &tampered).is_err()
        );
        assert!(
            verify_tool_download(&encoded, &[key.clone()], 0, 1_000, &target, &payload[..32])
                .is_err()
        );
        assert!(verify_tool_download(
            &encoded,
            &[key.clone()],
            0,
            1_000,
            "ffmpeg/windows-x86_64",
            &payload
        )
        .is_err());
        assert!(
            verify_tool_download(&encoded, &[key.clone()], 4, 1_000, &target, &payload).is_err()
        );
        assert!(verify_tool_download(&encoded, &[key], 0, 5_000, &target, &payload).is_err());
    }

    #[test]
    fn tool_download_rejects_an_untrusted_signer() {
        let payload = vec![1u8; 16];
        let (encoded, _) = signed_tool(&payload, 1);
        let other = STANDARD.encode(SigningKey::from_bytes(&[3; 32]).verifying_key().to_bytes());
        let target = tool_artifact_target("yt-dlp", "windows", "x86_64");
        assert!(verify_tool_download(&encoded, &[other], 0, 1_000, &target, &payload).is_err());
    }
}
