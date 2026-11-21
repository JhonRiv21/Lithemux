use crate::modules::session::SessionError;
use crate::modules::settings::SettingsError;

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("session \"{0}\" not found")]
    SessionNotFound(String),

    #[error(transparent)]
    Session(#[from] SessionError),

    #[error(transparent)]
    Settings(#[from] SettingsError),
}
