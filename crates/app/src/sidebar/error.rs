use lithemux_core::modules::category::CategoryError;
use lithemux_core::modules::session::SessionError;

#[derive(Debug, thiserror::Error)]
pub enum SidebarError {
    #[error("\"{0}\" has no letters or digits to make a name from")]
    UnusableName(String),

    #[error("that session or category no longer exists")]
    Gone,

    #[error(transparent)]
    Category(#[from] CategoryError),

    #[error(transparent)]
    Session(#[from] SessionError),
}
