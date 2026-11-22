use crate::modules::workspace::error::WorkspaceError;

/// Where the one layout of the app is kept, as text.
pub trait LayoutRepository {
    fn load(&self) -> Result<Option<String>, WorkspaceError>;
    fn save(&self, layout: &str) -> Result<(), WorkspaceError>;
}
