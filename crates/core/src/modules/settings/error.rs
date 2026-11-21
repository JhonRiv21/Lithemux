use crate::platform::store::error::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("unknown setting \"{key}\"; the valid ones are: {valid}")]
    UnknownKey { key: String, valid: String },

    #[error("invalid value \"{value}\" for {key}: {reason}")]
    InvalidValue {
        key: &'static str,
        value: String,
        reason: &'static str,
    },

    #[error(transparent)]
    Store(#[from] StoreError),
}
