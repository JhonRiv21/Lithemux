use crate::modules::settings::domain::setting_key::SettingKey;

/// The settings a user can change from the command line.
pub const CONFIGURABLE_KEYS: [SettingKey; 2] =
    [SettingKey::ClaudePath, SettingKey::ContextLanguage];

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
        _ => Err("this setting cannot be changed from the command line"),
    }
}
