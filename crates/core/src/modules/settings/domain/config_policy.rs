use crate::modules::settings::domain::setting_key::SettingKey;

/// The settings the command line knows. The user changes the first two;
/// `last_claude_path` is written by the launcher and can only be read or
/// unset from here.
pub const CONFIGURABLE_KEYS: [SettingKey; 3] = [
    SettingKey::ClaudePath,
    SettingKey::ContextLanguage,
    SettingKey::LastClaudePath,
];

pub fn configurable_key(name: &str) -> Option<SettingKey> {
    CONFIGURABLE_KEYS
        .into_iter()
        .find(|key| key.as_str() == name)
}

/// Returns the value as it should be stored, or what is wrong with it.
pub fn checked_value(key: SettingKey, value: &str) -> Result<String, &'static str> {
    let value = value.trim();
    match key {
        SettingKey::ClaudePath if value.starts_with('/') => Ok(value.to_string()),
        SettingKey::ClaudePath => Err("expected an absolute path"),
        SettingKey::ContextLanguage => match value.to_ascii_lowercase().as_str() {
            language @ ("es" | "en") => Ok(language.to_string()),
            _ => Err("expected `es` or `en`"),
        },
        SettingKey::LastClaudePath => Err("lithemux writes this one; it can only be read or unset"),
        _ => Err("this setting cannot be changed from the command line"),
    }
}
