pub mod classifier;
pub mod credentials;
pub mod download;
pub mod i18n;
pub mod media;
pub mod model;
pub mod preview;
pub mod strategy;
pub mod tools;
pub mod validation;

pub use classifier::{classify_url, DownloadKind};
pub use credentials::{AuthKind, CredentialMetadata, SecretStore, SensitiveSecret};
pub use download::{
    chunk_directory, cleanup_chunk_artifacts, partial_path, DownloadEngine, DownloadEvent,
    DownloadRequest,
};
pub use i18n::{Language, Translator};
pub use media::{convert_ts_to_mp4, ConversionMode, TsToMp4Request};
pub use model::{DownloadId, DownloadState, DownloadTask};
pub use preview::{launch_player, PlayerConfig, PreviewReadiness, TorrentPreviewPolicy};
pub use strategy::{plan_download, Capabilities, Engine, StrategyPlan};
pub use validation::{validate_payload, PayloadExpectation};
