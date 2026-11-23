use crate::modules::category::{CategoryError, InvalidSlug};
use crate::platform::store::error::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    InvalidSlug(#[from] InvalidSlug),

    #[error("invalid reference \"{0}\": expected <category>/<session>")]
    InvalidRef(String),

    #[error("session \"{0}\" already exists")]
    AlreadyExists(String),

    #[error("session \"{0}\" not found")]
    NotFound(String),

    #[error("another session already has the Claude conversation {0}")]
    ConversationTaken(String),

    #[error("messaging name \"{name}\" is already used by {owner}")]
    MessagingNameTaken { name: String, owner: String },

    #[error("category \"{category}\" already has a coordinator session ({coordinator})")]
    CoordinatorExists {
        category: String,
        coordinator: String,
    },

    #[error(transparent)]
    Category(#[from] CategoryError),

    #[error(transparent)]
    Store(#[from] StoreError),
}
