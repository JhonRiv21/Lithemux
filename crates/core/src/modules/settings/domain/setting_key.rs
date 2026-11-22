#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKey {
    Language,
    ContextLanguage,
    ClaudePath,
    HibernateAfterHours,
    MemoryValve,
    Scrollback,
    FontFamily,
    FontSize,
    Theme,
    VersionCheckHours,
    /// Written by the launcher, not by the user: the `claude` the
    /// interactive shell reported last.
    LastClaudePath,
}

impl SettingKey {
    pub fn as_str(self) -> &'static str {
        match self {
            SettingKey::Language => "language",
            SettingKey::ContextLanguage => "context_language",
            SettingKey::ClaudePath => "claude_path",
            SettingKey::HibernateAfterHours => "hibernate_after_hours",
            SettingKey::MemoryValve => "memory_valve",
            SettingKey::Scrollback => "scrollback",
            SettingKey::FontFamily => "font_family",
            SettingKey::FontSize => "font_size",
            SettingKey::Theme => "theme",
            SettingKey::VersionCheckHours => "version_check_hours",
            SettingKey::LastClaudePath => "last_claude_path",
        }
    }
}
