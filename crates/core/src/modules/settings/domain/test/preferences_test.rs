use std::collections::HashMap;

use crate::modules::settings::domain::preferences::{InterfaceLanguage, Preferences, ThemeChoice};
use crate::modules::settings::domain::setting_key::SettingKey;

fn from(values: &[(SettingKey, &str)]) -> Preferences {
    let stored: HashMap<&str, String> = values
        .iter()
        .map(|(key, value)| (key.as_str(), (*value).to_string()))
        .collect();
    Preferences::from_stored(|key| stored.get(key.as_str()).cloned())
}

// PLAN §2.6 and §2.4: English, 48 h, the valve on, system theme.
#[test]
fn with_nothing_set_the_defaults_of_the_plan_apply() {
    let preferences = from(&[]);
    assert_eq!(preferences.language, InterfaceLanguage::English);
    assert_eq!(preferences.context_language, None);
    assert_eq!(preferences.hibernate_after_hours, 48);
    assert!(preferences.memory_valve);
    assert_eq!(preferences.scrollback, 5_000);
    assert_eq!(preferences.claude_path, None);
    assert_eq!(preferences.theme, ThemeChoice::System);
    assert_eq!(preferences.version_check_hours, 6);
    assert_eq!(preferences, Preferences::default());
}

// PLAN §2.6
#[test]
fn what_is_set_is_what_the_app_uses() {
    let preferences = from(&[
        (SettingKey::Language, "es"),
        (SettingKey::ContextLanguage, "en"),
        (SettingKey::HibernateAfterHours, "6"),
        (SettingKey::MemoryValve, "off"),
        (SettingKey::Scrollback, "20000"),
        (SettingKey::FontFamily, "JetBrains Mono"),
        (SettingKey::FontSize, "15"),
        (SettingKey::ClaudePath, "/opt/claude"),
        (SettingKey::Theme, "light"),
    ]);
    assert_eq!(preferences.language, InterfaceLanguage::Spanish);
    assert_eq!(
        preferences.context_language,
        Some(InterfaceLanguage::English)
    );
    assert_eq!(preferences.hibernate_after_hours, 6);
    assert!(!preferences.memory_valve);
    assert_eq!(preferences.scrollback, 20_000);
    assert_eq!(preferences.font_family, "JetBrains Mono");
    assert_eq!(preferences.font_size, 15);
    assert_eq!(preferences.claude_path.as_deref(), Some("/opt/claude"));
    assert_eq!(preferences.theme, ThemeChoice::Light);
}

// PLAN principle 3: whatever the table holds, the app starts.
#[test]
fn a_value_stored_wrong_counts_as_unset() {
    let preferences = from(&[
        (SettingKey::Language, "klingon"),
        (SettingKey::HibernateAfterHours, "soon"),
        (SettingKey::MemoryValve, "maybe"),
        (SettingKey::Scrollback, "-1"),
        (SettingKey::FontSize, "900"),
        (SettingKey::ClaudePath, "claude"),
        (SettingKey::Theme, "neon"),
    ]);
    assert_eq!(preferences, Preferences::default());
}
