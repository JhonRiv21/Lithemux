use crate::modules::settings::error::SettingsError;

pub trait SettingsRepository {
    fn get(&self, key: &str) -> Result<Option<String>, SettingsError>;
    /// Creates or replaces the value.
    fn set(&self, key: &str, value: &str) -> Result<(), SettingsError>;
    /// Removing a key that is not set is not an error.
    fn remove(&self, key: &str) -> Result<(), SettingsError>;
}
