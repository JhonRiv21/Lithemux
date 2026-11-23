use crate::platform::paths::fs_error::FsError;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Fs(#[from] FsError),

    #[error(
        "some sessions share a Claude conversation, which lithemux no longer allows; \
         remove all but one of each before upgrading ({})",
        .0.join("; ")
    )]
    SharedConversations(Vec<String>),

    #[error("database schema version {found} is newer than this build supports ({supported})")]
    SchemaTooNew { found: i64, supported: i64 },
}
