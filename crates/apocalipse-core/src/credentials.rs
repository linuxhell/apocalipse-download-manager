use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthKind {
    HttpBasic,
    FormLogin,
    BrowserSession,
}

/// Serializable metadata only. The password/session secret is stored separately
/// in the operating system credential vault.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialMetadata {
    pub id: Uuid,
    pub origin: String,
    pub username: Option<String>,
    pub kind: AuthKind,
}

impl CredentialMetadata {
    pub fn new(origin: &str, username: Option<String>, kind: AuthKind) -> Result<Self> {
        let parsed = Url::parse(origin)?;
        if !matches!(parsed.scheme(), "https" | "http") || parsed.host_str().is_none() {
            bail!("credential origin must be an HTTP or HTTPS site");
        }
        if parsed.scheme() == "http"
            && !matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
        {
            bail!("credentials require HTTPS except for local services");
        }
        if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
            bail!("credential scope must be an origin without path, query or fragment");
        }
        Ok(Self {
            id: Uuid::new_v4(),
            origin: parsed.origin().ascii_serialization(),
            username,
            kind,
        })
    }

    pub fn applies_to(&self, target: &str) -> bool {
        Url::parse(target)
            .map(|url| url.origin().ascii_serialization() == self.origin)
            .unwrap_or(false)
    }
}

/// Secret bytes are wiped when dropped and deliberately cannot be serialized,
/// cloned or printed through `Debug`.
pub struct SensitiveSecret(Vec<u8>);

impl SensitiveSecret {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self(value.into())
    }
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SensitiveSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn save(&self, id: Uuid, secret: SensitiveSecret) -> Result<()>;
    async fn load(&self, id: Uuid) -> Result<Option<SensitiveSecret>>;
    async fn remove(&self, id: Uuid) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_is_limited_to_the_exact_origin() {
        let credential = CredentialMetadata::new(
            "https://example.com",
            Some("alice".into()),
            AuthKind::HttpBasic,
        )
        .unwrap();
        assert!(credential.applies_to("https://example.com/private/file.zip"));
        assert!(!credential.applies_to("https://cdn.example.com/file.zip"));
        assert!(!credential.applies_to("http://example.com/file.zip"));
    }

    #[test]
    fn rejects_insecure_remote_credentials_and_path_scopes() {
        assert!(CredentialMetadata::new("http://example.com", None, AuthKind::FormLogin).is_err());
        assert!(
            CredentialMetadata::new("https://example.com/login", None, AuthKind::FormLogin)
                .is_err()
        );
    }
}
