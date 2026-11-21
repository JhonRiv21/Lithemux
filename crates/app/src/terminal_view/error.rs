use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error("could not open a pseudo terminal: {0}")]
    OpenPty(String),
    #[error("could not start {program}: {message}")]
    Spawn { program: PathBuf, message: String },
    #[error("could not attach to the pseudo terminal: {0}")]
    Attach(String),
    #[error("text cannot be drawn: {0}")]
    TextRendering(String),
}
