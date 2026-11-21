use crate::modules::context::domain::lang::Lang;
use crate::modules::context::error::ContextError;
use crate::modules::settings::{SettingKey, SettingsRepository, read_setting};

/// `context_language`, falling back to the interface `language`, then English.
pub fn resolve_language(settings: &dyn SettingsRepository) -> Result<Lang, ContextError> {
    for key in [SettingKey::ContextLanguage, SettingKey::Language] {
        if let Some(lang) = read_setting(settings, key)?
            .as_deref()
            .and_then(Lang::parse)
        {
            return Ok(lang);
        }
    }
    Ok(Lang::En)
}
