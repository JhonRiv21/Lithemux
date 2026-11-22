#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    /// The value of the `language` setting. Anything unknown is English.
    pub fn from_setting(value: Option<&str>) -> Self {
        match value {
            Some("es") => Language::Spanish,
            _ => Language::English,
        }
    }
}

impl From<lithemux_core::modules::settings::InterfaceLanguage> for Language {
    fn from(language: lithemux_core::modules::settings::InterfaceLanguage) -> Self {
        match language {
            lithemux_core::modules::settings::InterfaceLanguage::English => Language::English,
            lithemux_core::modules::settings::InterfaceLanguage::Spanish => Language::Spanish,
        }
    }
}
