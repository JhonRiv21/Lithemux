use crate::modules::settings::application::read_setting::read_setting;
use crate::modules::settings::domain::config_policy::{
    CONFIGURABLE_KEYS, checked_value, configurable_key,
};
use crate::modules::settings::domain::setting_key::SettingKey;
use crate::modules::settings::error::SettingsError;
use crate::modules::settings::ports::settings_repository::SettingsRepository;

pub fn get_config(
    settings: &dyn SettingsRepository,
    key: &str,
) -> Result<Option<String>, SettingsError> {
    read_setting(settings, known(key)?)
}

/// Returns the value as stored.
pub fn set_config(
    settings: &dyn SettingsRepository,
    key: &str,
    value: &str,
) -> Result<String, SettingsError> {
    let key = known(key)?;
    let stored = checked_value(key, value).map_err(|reason| SettingsError::InvalidValue {
        key: key.as_str(),
        value: value.to_string(),
        reason,
    })?;
    settings.set(key.as_str(), &stored)?;
    Ok(stored)
}

pub fn unset_config(settings: &dyn SettingsRepository, key: &str) -> Result<(), SettingsError> {
    settings.remove(known(key)?.as_str())
}

fn known(key: &str) -> Result<SettingKey, SettingsError> {
    configurable_key(key).ok_or_else(|| SettingsError::UnknownKey {
        key: key.to_string(),
        valid: CONFIGURABLE_KEYS.map(SettingKey::as_str).join(", "),
    })
}
