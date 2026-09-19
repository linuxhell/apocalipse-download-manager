pub mod classifier;
pub mod credentials;
pub mod download;
pub mod i18n;
pub mod media;
pub mod metalink;
pub mod model;
pub mod preview;
pub mod private_cache;
pub mod signed_update;
pub mod strategy;
pub mod tools;
pub mod validation;

pub use classifier::{classify_url, DownloadKind};
pub use credentials::{AuthKind, CredentialMetadata, SecretStore, SensitiveSecret};
pub use download::{
    chunk_directory, cleanup_chunk_artifacts, partial_path, BandwidthLimiter, DownloadEngine,
    DownloadEvent, DownloadRequest,
};
pub use i18n::{Language, Translator};
pub use media::{convert_ts_to_mp4, ConversionMode, TsToMp4Request};
pub use metalink::{parse_metalink, MetalinkFile};
pub use model::{DownloadId, DownloadState, DownloadTask};
pub use preview::{launch_player, PlayerConfig, PreviewReadiness, TorrentPreviewPolicy};
pub use private_cache::EncryptedChunkCache;
pub use signed_update::{
    verify_update_manifest, SignedUpdateManifest, UpdateArtifact, UpdateManifest,
};
pub use strategy::{contextual_media_page, plan_download, Capabilities, Engine, StrategyPlan};
pub use validation::{validate_payload, PayloadExpectation};
