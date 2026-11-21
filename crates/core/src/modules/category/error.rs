use crate::platform::paths::fs_error::FsError;
use crate::platform::store::error::StoreError;

#[derive(Debug, thiserror::Error)]
#[error(
    "invalid slug \"{0}\": use lowercase letters, digits and hyphens, without a leading or trailing hyphen"
)]
pub struct InvalidSlug(pub String);

#[derive(Debug, thiserror::Error)]
pub enum CategoryError {
    #[error(transparent)]
    InvalidSlug(#[from] InvalidSlug),

    #[error("category \"{0}\" already exists")]
    AlreadyExists(String),

    #[error("category \"{0}\" not found")]
    NotFound(String),

    #[error(transparent)]
    Store(#[from] StoreError),

    #[error(transparent)]
    Fs(#[from] FsError),
}
