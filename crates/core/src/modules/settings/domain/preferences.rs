//! The settings as the app uses them: typed, with a default for whatever
//! is unset or was stored wrong.

use crate::modules::settings::domain::config_policy::checked_value;
use crate::modules::settings::domain::setting_key::SettingKey;

pub const DEFAULT_HIBERNATE_AFTER_HOURS: i64 = 48;
pub const DEFAULT_SCROLLBACK: usize = 5_000;
pub const DEFAULT_FONT_FAMILY: &str = "Menlo";
pub const DEFAULT_FONT_SIZE: u16 = 13;
pub const DEFAULT_VERSION_CHECK_HOURS: i64 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterfaceLanguage {
    #[default]
    English,
    Spanish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeChoice {
    Light,
    Dark,
    /// Whatever macOS is set to.
    #[default]
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preferences {
    pub language: InterfaceLanguage,
    /// `None`: the language of the interface.
    pub context_language: Option<InterfaceLanguage>,
    pub hibernate_after_hours: i64,
    pub memory_valve: bool,
    pub scrollback: usize,
    pub font_family: String,
    pub font_size: u16,
    /// `None`: the `claude` of the user's shell. As stored, even when it
    /// could not run: Settings shows it to be fixed, and the launch refuses
    /// it and says why.
    pub claude_path: Option<String>,
    pub theme: ThemeChoice,
    /// How often the installed `claude` is asked for its version.
    pub version_check_hours: i64,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            language: InterfaceLanguage::English,
            context_language: None,
            hibernate_after_hours: DEFAULT_HIBERNATE_AFTER_HOURS,
            memory_valve: true,
            scrollback: DEFAULT_SCROLLBACK,
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            font_size: DEFAULT_FONT_SIZE,
            claude_path: None,
            theme: ThemeChoice::System,
            version_check_hours: DEFAULT_VERSION_CHECK_HOURS,
        }
    }
}

fn language(value: &str) -> InterfaceLanguage {
    match value {
        "es" => InterfaceLanguage::Spanish,
        _ => InterfaceLanguage::English,
    }
}

impl Preferences {
    /// `stored` gives what the database has for a key. A value that would
    /// not be accepted today counts as unset, except `claude_path`.
    pub fn from_stored(stored: impl Fn(SettingKey) -> Option<String>) -> Self {
        let valid = |key| stored(key).and_then(|value| checked_value(key, &value).ok());
        let defaults = Preferences::default();
        Preferences {
            language: valid(SettingKey::Language).map_or(defaults.language, |v| language(&v)),
            context_language: valid(SettingKey::ContextLanguage).map(|v| language(&v)),
            hibernate_after_hours: valid(SettingKey::HibernateAfterHours)
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.hibernate_after_hours),
            memory_valve: valid(SettingKey::MemoryValve).is_none_or(|v| v == "on"),
            scrollback: valid(SettingKey::Scrollback)
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.scrollback),
            font_family: valid(SettingKey::FontFamily).unwrap_or(defaults.font_family),
            font_size: valid(SettingKey::FontSize)
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.font_size),
            claude_path: stored(SettingKey::ClaudePath),
            version_check_hours: valid(SettingKey::VersionCheckHours)
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.version_check_hours),
            theme: match valid(SettingKey::Theme).as_deref() {
                Some("light") => ThemeChoice::Light,
                Some("dark") => ThemeChoice::Dark,
                _ => ThemeChoice::System,
            },
        }
    }
}
