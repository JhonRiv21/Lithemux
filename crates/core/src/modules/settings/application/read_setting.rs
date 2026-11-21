use crate::modules::settings::domain::setting_key::SettingKey;
use crate::modules::settings::error::SettingsError;
use crate::modules::settings::ports::settings_repository::SettingsRepository;

/// A blank value counts as unset.
pub fn read_setting(
    settings: &dyn SettingsRepository,
    key: SettingKey,
) -> Result<Option<String>, SettingsError> {
    Ok(settings
        .get(key.as_str())?
        .filter(|value| !value.trim().is_empty()))
}
