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
