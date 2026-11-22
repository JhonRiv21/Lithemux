use crate::modules::settings::application::read_setting::read_setting;
use crate::modules::settings::domain::preferences::Preferences;
use crate::modules::settings::ports::settings_repository::SettingsRepository;

/// Every setting, typed. A setting that cannot be read is its default: the
/// app must start whatever the settings table holds.
pub fn load_preferences(settings: &dyn SettingsRepository) -> Preferences {
    Preferences::from_stored(|key| read_setting(settings, key).ok().flatten())
}
