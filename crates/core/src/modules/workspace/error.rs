use crate::platform::store::error::StoreError;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WorkspaceError {
    #[error("the workspace already has all the panes it can show")]
    TooManyPanes,

    #[error("there is no pane {0}")]
    UnknownPane(u32),

    #[error("the layout could not be kept: {0}")]
    Store(String),
}

impl From<StoreError> for WorkspaceError {
    fn from(error: StoreError) -> Self {
        WorkspaceError::Store(error.to_string())
    }
}
