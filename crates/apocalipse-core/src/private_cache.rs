use anyhow::{bail, Context, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use zeroize::Zeroizing;

const CACHE_VERSION: u8 = 1;
const NONCE_BYTES: usize = 24;

pub struct EncryptedChunkCache {
    root: PathBuf,
    key: Zeroizing<[u8; 32]>,
    maximum_bytes: u64,
}

impl EncryptedChunkCache {
    pub fn new(root: impl Into<PathBuf>, key: [u8; 32], maximum_bytes: u64) -> Self {
        Self {
            root: root.into(),
            key: Zeroizing::new(key),
            maximum_bytes,
        }
    }

    pub fn digest(data: &[u8]) -> String {
        format!("{:x}", Sha256::digest(data))
    }

    fn path(&self, digest: &str) -> Result<PathBuf> {
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("invalid cache digest")
        }
        let digest = digest.to_ascii_lowercase();
        Ok(self
            .root
            .join(&digest[..2])
            .join(format!("{}.chunk", &digest[2..])))
    }

    pub async fn put(&self, data: &[u8]) -> Result<String> {
        let digest = Self::digest(data);
        let target = self.path(&digest)?;
        if fs::metadata(&target).await.is_ok() {
            return Ok(digest);
        }
        let mut nonce = [0_u8; NONCE_BYTES];
        getrandom::fill(&mut nonce).context("secure random generation failed")?;
        let cipher = XChaCha20Poly1305::new((&*self.key).into());
        let encrypted = cipher
            .encrypt(XNonce::from_slice(&nonce), data)
            .map_err(|_| anyhow::anyhow!("cache encryption failed"))?;
        let parent = target.parent().context("cache target has no parent")?;
        fs::create_dir_all(parent).await?;
        // Each writer owns a separate temporary file, including identical chunks.
        let temporary = Temporary(target.with_extension(format!("{}.tmp", uuid::Uuid::new_v4())));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary.0)
            .await?;
        file.write_all(&[CACHE_VERSION]).await?;
        file.write_all(&nonce).await?;
        file.write_all(&encrypted).await?;
        file.sync_all().await?;
        drop(file);
        fs::rename(&temporary.0, target).await?;
        self.enforce_limit().await?;
        Ok(digest)
    }

    pub async fn get(&self, digest: &str) -> Result<Option<Vec<u8>>> {
        let target = self.path(digest)?;
        let encrypted = match fs::read(&target).await {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        if encrypted.len() <= 1 + NONCE_BYTES || encrypted[0] != CACHE_VERSION {
            let _ = fs::remove_file(target).await;
            bail!("invalid encrypted cache entry")
        }
        let cipher = XChaCha20Poly1305::new((&*self.key).into());
        let clear = match cipher.decrypt(
            XNonce::from_slice(&encrypted[1..1 + NONCE_BYTES]),
            &encrypted[1 + NONCE_BYTES..],
        ) {
            Ok(clear) => clear,
            Err(_) => {
                let _ = fs::remove_file(&target).await;
                bail!("cache authentication failed");
            }
        };
        if Self::digest(&clear) != digest.to_ascii_lowercase() {
            let _ = fs::remove_file(target).await;
            bail!("cache digest mismatch")
        }
        Ok(Some(clear))
    }

    async fn enforce_limit(&self) -> Result<()> {
        if self.maximum_bytes == 0 {
            return Ok(());
        }
        let mut entries = Vec::new();
        collect_entries(&self.root, &mut entries).await?;
        let mut total = entries.iter().map(|(_, size, _)| *size).sum::<u64>();
        entries.sort_by_key(|(_, _, modified)| *modified);
        for (path, size, _) in entries {
            if total <= self.maximum_bytes {
                break;
            }
            if fs::remove_file(path).await.is_ok() {
                total = total.saturating_sub(size);
            }
        }
        Ok(())
    }
}

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn collect_entries(
    root: &Path,
    entries: &mut Vec<(PathBuf, u64, std::time::SystemTime)>,
) -> Result<()> {
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let mut reader = match fs::read_dir(directory).await {
            Ok(reader) => reader,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        while let Some(entry) = reader.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                directories.push(entry.path());
            } else if entry
                .path()
                .extension()
                .is_some_and(|value| value == "chunk")
            {
                entries.push((
                    entry.path(),
                    metadata.len(),
                    metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_is_encrypted_authenticated_and_content_addressed() {
        let root = std::env::temp_dir().join(format!("adm-cache-{}", uuid::Uuid::new_v4()));
        let cache = EncryptedChunkCache::new(&root, [9; 32], 1024 * 1024);
        let digest = cache.put(b"private chunk").await.unwrap();
        let stored = fs::read(cache.path(&digest).unwrap()).await.unwrap();
        assert!(!stored.windows(13).any(|window| window == b"private chunk"));
        assert_eq!(cache.get(&digest).await.unwrap().unwrap(), b"private chunk");
        let _ = fs::remove_dir_all(root).await;
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn cache_regression_concurrent_identical_puts_are_atomic() {
        let root = std::env::temp_dir().join(format!("adm-cache-race-{}", uuid::Uuid::new_v4()));
        let cache = Arc::new(EncryptedChunkCache::new(&root, [9; 32], 0));
        let gate = Arc::new(tokio::sync::Barrier::new(16));
        let mut tasks = Vec::new();
        for _ in 0..16 {
            let (cache, gate) = (cache.clone(), gate.clone());
            tasks.push(tokio::spawn(async move {
                gate.wait().await;
                cache.put(&vec![42; 64 * 1024]).await
            }));
        }
        for task in tasks {
            task.await.unwrap().unwrap();
        }
        let digest = EncryptedChunkCache::digest(&vec![42; 64 * 1024]);
        assert_eq!(
            cache.get(&digest).await.unwrap().unwrap(),
            vec![42; 64 * 1024]
        );
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn cache_regression_uppercase_digest_reads_the_same_entry() {
        let root = std::env::temp_dir().join(format!("adm-cache-case-{}", uuid::Uuid::new_v4()));
        let cache = EncryptedChunkCache::new(&root, [9; 32], 0);
        let digest = cache.put(b"payload").await.unwrap();
        assert_eq!(
            cache
                .get(&digest.to_ascii_uppercase())
                .await
                .unwrap()
                .unwrap(),
            b"payload"
        );
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn cache_regression_corrupt_entry_can_be_replaced() {
        let root = std::env::temp_dir().join(format!("adm-cache-corrupt-{}", uuid::Uuid::new_v4()));
        let cache = EncryptedChunkCache::new(&root, [9; 32], 0);
        let digest = cache.put(b"payload").await.unwrap();
        let path = cache.path(&digest).unwrap();
        let mut stored = fs::read(&path).await.unwrap();
        *stored.last_mut().unwrap() ^= 1;
        fs::write(&path, stored).await.unwrap();
        assert!(cache.get(&digest).await.is_err());
        cache.put(b"payload").await.unwrap();
        assert_eq!(cache.get(&digest).await.unwrap().unwrap(), b"payload");
        fs::remove_dir_all(root).await.unwrap();
    }
}
