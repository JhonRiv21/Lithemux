use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedClaude {
    pub path: PathBuf,
    /// Set when the preferred way of finding `claude` did not work.
    pub warning: Option<String>,
}
