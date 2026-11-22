use crate::modules::settings::domain::config_policy::{checked_value, configurable_key};
use crate::modules::settings::domain::setting_key::SettingKey;

// PLAN §0.6 + D-28: `lithemux config` knows the two keys of the user and the
// one the launcher writes
#[test]
fn the_command_line_knows_three_settings() {
    assert_eq!(
        configurable_key("last_claude_path"),
        Some(SettingKey::LastClaudePath)
    );
    assert!(checked_value(SettingKey::LastClaudePath, "/bin/anything").is_err());
    assert_eq!(
        configurable_key("claude_path"),
        Some(SettingKey::ClaudePath)
    );
    assert_eq!(
        configurable_key("context_language"),
        Some(SettingKey::ContextLanguage)
    );
    for other in ["language", "hibernate_after_hours", "CLAUDE_PATH", ""] {
        assert_eq!(configurable_key(other), None, "{other:?}");
    }
}

// PLAN §0.6: context_language is es|en
#[test]
fn context_language_accepts_only_es_or_en() {
    assert_eq!(
        checked_value(SettingKey::ContextLanguage, "es"),
        Ok("es".to_string())
    );
    assert_eq!(
        checked_value(SettingKey::ContextLanguage, " EN "),
        Ok("en".to_string())
    );
    assert!(checked_value(SettingKey::ContextLanguage, "fr").is_err());
    assert!(checked_value(SettingKey::ContextLanguage, "").is_err());
}

// PLAN §0.3: claude_path fixes the binary, so it must be unambiguous
#[test]
fn claude_path_must_be_an_absolute_path() {
    assert_eq!(
        checked_value(SettingKey::ClaudePath, "/usr/local/bin/claude"),
        Ok("/usr/local/bin/claude".to_string())
    );
    assert!(checked_value(SettingKey::ClaudePath, "claude").is_err());
    assert!(checked_value(SettingKey::ClaudePath, "~/bin/claude").is_err());
}
