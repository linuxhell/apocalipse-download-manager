use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "pt-BR")]
    PortugueseBrazil,
    #[serde(rename = "zh-CN")]
    ChineseSimplified,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::English, Self::PortugueseBrazil, Self::ChineseSimplified];
    pub const fn code(self) -> &'static str {
        match self { Self::English => "en", Self::PortugueseBrazil => "pt-BR", Self::ChineseSimplified => "zh-CN" }
    }
}

pub struct Translator { language: Language }

impl Translator {
    pub const fn new(language: Language) -> Self { Self { language } }
    pub const fn language(&self) -> Language { self.language }
    pub fn text<'a>(&self, key: &'a str) -> &'a str {
        match (self.language, key) {
            (Language::PortugueseBrazil, "downloads") => "Downloads",
            (Language::PortugueseBrazil, "settings") => "Configurações",
            (Language::PortugueseBrazil, "video") => "Vídeo",
            (Language::PortugueseBrazil, "audio") => "Áudio",
            (Language::PortugueseBrazil, "images") => "Imagens",
            (Language::PortugueseBrazil, "convert_ts") => "Converter TS para MP4",
            (Language::PortugueseBrazil, "fast_remux") => "Conversão rápida sem perda",
            (Language::PortugueseBrazil, "compatibility_mode") => "Modo de compatibilidade",
            (Language::PortugueseBrazil, "watch_while_downloading") => "Assistir enquanto baixa",
            (Language::PortugueseBrazil, "media_player") => "Reprodutor de mídia",
            (Language::PortugueseBrazil, "site_credentials") => "Credenciais de sites",
            (Language::PortugueseBrazil, "add_site") => "Adicionar site",
            (Language::PortugueseBrazil, "remove_site") => "Remover site",
            (Language::ChineseSimplified, "downloads") => "下载",
            (Language::ChineseSimplified, "settings") => "设置",
            (Language::ChineseSimplified, "video") => "视频",
            (Language::ChineseSimplified, "audio") => "音频",
            (Language::ChineseSimplified, "images") => "图片",
            (Language::ChineseSimplified, "convert_ts") => "将 TS 转换为 MP4",
            (Language::ChineseSimplified, "fast_remux") => "无损快速封装",
            (Language::ChineseSimplified, "compatibility_mode") => "兼容模式",
            (Language::ChineseSimplified, "watch_while_downloading") => "边下边看",
            (Language::ChineseSimplified, "media_player") => "媒体播放器",
            (Language::ChineseSimplified, "site_credentials") => "网站凭据",
            (Language::ChineseSimplified, "add_site") => "添加网站",
            (Language::ChineseSimplified, "remove_site") => "移除网站",
            (_, "downloads") => "Downloads",
            (_, "settings") => "Settings",
            (_, "video") => "Video",
            (_, "audio") => "Audio",
            (_, "images") => "Images",
            (_, "convert_ts") => "Convert TS to MP4",
            (_, "fast_remux") => "Fast lossless remux",
            (_, "compatibility_mode") => "Compatibility mode",
            (_, "watch_while_downloading") => "Watch while downloading",
            (_, "media_player") => "Media player",
            (_, "site_credentials") => "Site credentials",
            (_, "add_site") => "Add site",
            (_, "remove_site") => "Remove site",
            _ => key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn english_is_default_and_all_locales_cover_core_keys() {
        assert_eq!(Language::default(), Language::English);
        for language in Language::ALL {
            let tr = Translator::new(language);
            for key in ["downloads", "settings", "video", "audio", "images", "convert_ts", "fast_remux", "compatibility_mode", "watch_while_downloading", "media_player", "site_credentials", "add_site", "remove_site"] { assert_ne!(tr.text(key), key); }
        }
    }
}
