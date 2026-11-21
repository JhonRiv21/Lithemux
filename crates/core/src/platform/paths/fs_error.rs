use std::io;

#[derive(Debug, thiserror::Error)]
#[error("{context}: {source}")]
pub struct FsError {
    pub context: String,
    #[source]
    pub source: io::Error,
}

impl FsError {
    pub fn new(context: impl Into<String>, source: io::Error) -> Self {
        FsError {
            context: context.into(),
            source,
        }
    }
}
