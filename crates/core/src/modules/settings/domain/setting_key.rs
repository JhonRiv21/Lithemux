#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKey {
    Language,
    ContextLanguage,
    ClaudePath,
    HibernateAfterHours,
}

impl SettingKey {
    pub fn as_str(self) -> &'static str {
        match self {
            SettingKey::Language => "language",
            SettingKey::ContextLanguage => "context_language",
            SettingKey::ClaudePath => "claude_path",
            SettingKey::HibernateAfterHours => "hibernate_after_hours",
        }
    }
}
