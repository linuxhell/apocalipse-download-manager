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
}
