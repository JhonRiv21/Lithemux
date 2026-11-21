#[derive(Debug, thiserror::Error)]
pub enum PathsError {
    #[error("could not determine the lithemux data directory")]
    NoDataDir,
}
