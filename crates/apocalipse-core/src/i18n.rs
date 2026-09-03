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
            (Language::ChineseSimplified, "downloads") => "下载",
            (Language::ChineseSimplified, "settings") => "设置",
            (Language::ChineseSimplified, "video") => "视频",
            (Language::ChineseSimplified, "audio") => "音频",
            (Language::ChineseSimplified, "images") => "图片",
            (_, "downloads") => "Downloads",
            (_, "settings") => "Settings",
            (_, "video") => "Video",
            (_, "audio") => "Audio",
            (_, "images") => "Images",
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
            for key in ["downloads", "settings", "video", "audio", "images"] { assert_ne!(tr.text(key), key); }
        }
    }
}
