use crate::modules::settings::domain::setting_key::SettingKey;
use crate::modules::settings::error::SettingsError;
use crate::modules::settings::ports::settings_repository::SettingsRepository;

pub fn write_setting(
    settings: &dyn SettingsRepository,
    key: SettingKey,
    value: &str,
) -> Result<(), SettingsError> {
    settings.set(key.as_str(), value)
}
