use crate::modules::settings::domain::config_policy::{checked_value, configurable_key};
use crate::modules::settings::domain::setting_key::SettingKey;

// PLAN §0.6 + D-28: `lithemux config` knows the keys of the user and the
// one the launcher writes. PLAN §2.6 (block 4) adds the ones of the settings
// window, `language` among them: the command line and the window change
// the same settings. (Until block 4 this test kept `language` and
// `hibernate_after_hours` out, because nothing read them yet.)
#[test]
fn the_command_line_knows_the_settings_of_the_user_and_the_launchers_own() {
    assert_eq!(
        configurable_key("last_claude_path"),
        Some(SettingKey::LastClaudePath)
    );
    assert!(checked_value(SettingKey::LastClaudePath, "/bin/anything").is_err());
    for (name, key) in [
        ("claude_path", SettingKey::ClaudePath),
        ("context_language", SettingKey::ContextLanguage),
        ("language", SettingKey::Language),
        ("hibernate_after_hours", SettingKey::HibernateAfterHours),
        ("memory_valve", SettingKey::MemoryValve),
        ("scrollback", SettingKey::Scrollback),
        ("font_family", SettingKey::FontFamily),
        ("font_size", SettingKey::FontSize),
        ("theme", SettingKey::Theme),
        ("version_check_hours", SettingKey::VersionCheckHours),
    ] {
        assert_eq!(configurable_key(name), Some(key), "{name}");
    }
    for other in ["CLAUDE_PATH", "Language", "colour", ""] {
        assert_eq!(configurable_key(other), None, "{other:?}");
    }
}

// PLAN §2.6: interface language, English or Spanish.
#[test]
fn language_accepts_only_es_or_en() {
    assert_eq!(
        checked_value(SettingKey::Language, " ES "),
        Ok("es".to_string())
    );
    assert_eq!(
        checked_value(SettingKey::Language, "en"),
        Ok("en".to_string())
    );
    assert!(checked_value(SettingKey::Language, "fr").is_err());
}

// PLAN §2.6: hours to hibernate, the valve, scrollback, font, theme.
#[test]
fn the_other_settings_accept_only_values_that_make_sense() {
    let ok = |key, value: &str| checked_value(key, value).ok();
    assert_eq!(
        ok(SettingKey::HibernateAfterHours, "48").as_deref(),
        Some("48")
    );
    assert_eq!(
        ok(SettingKey::HibernateAfterHours, " 1 ").as_deref(),
        Some("1")
    );
    for wrong in ["0", "-3", "two", "1.5", "9000", ""] {
        assert_eq!(ok(SettingKey::HibernateAfterHours, wrong), None, "{wrong}");
    }
    assert_eq!(ok(SettingKey::MemoryValve, "ON").as_deref(), Some("on"));
    assert_eq!(ok(SettingKey::MemoryValve, "off").as_deref(), Some("off"));
    assert_eq!(ok(SettingKey::MemoryValve, "yes"), None);
    assert_eq!(ok(SettingKey::Scrollback, "5000").as_deref(), Some("5000"));
    assert_eq!(ok(SettingKey::Scrollback, "10"), None);
    assert_eq!(ok(SettingKey::Scrollback, "9999999"), None);
    assert_eq!(ok(SettingKey::FontSize, "13").as_deref(), Some("13"));
    assert_eq!(ok(SettingKey::FontSize, "4"), None);
    assert_eq!(ok(SettingKey::FontSize, "200"), None);
    assert_eq!(
        ok(SettingKey::FontFamily, " JetBrains Mono ").as_deref(),
        Some("JetBrains Mono")
    );
    assert_eq!(ok(SettingKey::FontFamily, ""), None);
    for theme in ["light", "dark", "system"] {
        assert_eq!(ok(SettingKey::Theme, theme).as_deref(), Some(theme));
    }
    assert_eq!(ok(SettingKey::Theme, "blue"), None);
    // PLAN §3.2: how often the version is checked, in hours.
    assert_eq!(ok(SettingKey::VersionCheckHours, "6").as_deref(), Some("6"));
    for wrong in ["0", "200", "often"] {
        assert_eq!(ok(SettingKey::VersionCheckHours, wrong), None, "{wrong}");
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
