use crate::{classify_url, DownloadKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Engine {
    NativeHttp,
    Aria2Rpc,
    YtDlp,
    NativeHls,
    NM3u8dlRe,
    NativeTorrent,
    AMule,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    pub aria2: bool,
    pub yt_dlp: bool,
    pub n_m3u8dl_re: bool,
    pub torrent: bool,
    pub amule: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyPlan {
    pub primary: Engine,
    pub fallbacks: Vec<Engine>,
    pub reason: &'static str,
}

pub fn plan_download(input: &str, capabilities: Capabilities) -> Option<StrategyPlan> {
    let kind = classify_url(input)?;
    let plan = match kind {
        DownloadKind::Http => StrategyPlan {
            primary: Engine::NativeHttp,
            fallbacks: capabilities
                .aria2
                .then_some(Engine::Aria2Rpc)
                .into_iter()
                .collect(),
            reason: "direct_http",
        },
        DownloadKind::MediaPage => StrategyPlan {
            primary: if capabilities.yt_dlp {
                Engine::YtDlp
            } else {
                Engine::NativeHttp
            },
            fallbacks: vec![Engine::NativeHttp],
            reason: if capabilities.yt_dlp {
                "media_extractor_available"
            } else {
                "media_extractor_missing"
            },
        },
        DownloadKind::Hls => StrategyPlan {
            primary: if capabilities.n_m3u8dl_re {
                Engine::NM3u8dlRe
            } else {
                Engine::NativeHls
            },
            fallbacks: vec![Engine::NativeHls],
            reason: "hls_manifest",
        },
        DownloadKind::Ftp => StrategyPlan {
            primary: Engine::Aria2Rpc,
            fallbacks: Vec::new(),
            reason: "ftp_transfer",
        },
        DownloadKind::Torrent | DownloadKind::Magnet => StrategyPlan {
            primary: if capabilities.torrent {
                Engine::NativeTorrent
            } else {
                Engine::Aria2Rpc
            },
            fallbacks: capabilities
                .aria2
                .then_some(Engine::Aria2Rpc)
                .into_iter()
                .collect(),
            reason: "peer_to_peer",
        },
        DownloadKind::Ed2k => StrategyPlan {
            primary: Engine::AMule,
            fallbacks: Vec::new(),
            reason: if capabilities.amule {
                "ed2k_adapter_available"
            } else {
                "ed2k_adapter_required"
            },
        },
    };
    Some(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_download_prefers_native_and_keeps_aria_as_fallback() {
        let plan = plan_download(
            "https://example.test/file.zip",
            Capabilities {
                aria2: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(plan.primary, Engine::NativeHttp);
        assert_eq!(plan.fallbacks, vec![Engine::Aria2Rpc]);
    }

    #[test]
    fn youtube_prefers_ytdlp_when_installed() {
        let plan = plan_download(
            "https://youtube.com/watch?v=x",
            Capabilities {
                yt_dlp: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(plan.primary, Engine::YtDlp);
        assert_eq!(plan.reason, "media_extractor_available");
    }
}
