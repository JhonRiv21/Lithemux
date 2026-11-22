use crate::modules::workspace::domain::layout::Layout;
use crate::modules::workspace::domain::stored_layout::{from_stored, to_stored};
use crate::modules::workspace::error::WorkspaceError;
use crate::modules::workspace::ports::layout_repository::LayoutRepository;

pub fn save_layout(layouts: &dyn LayoutRepository, layout: &Layout) -> Result<(), WorkspaceError> {
    layouts.save(&to_stored(layout))
}

/// The layout saved last, without the tabs of sessions that no longer
/// exist. A single empty pane when there is none, or it cannot be read.
pub fn restore_layout(
    layouts: &dyn LayoutRepository,
    session_exists: impl Fn(&str) -> bool,
) -> Result<Layout, WorkspaceError> {
    let stored = layouts.load()?;
    let mut layout = stored
        .as_deref()
        .and_then(from_stored)
        .unwrap_or_else(Layout::single);
    layout.keep_sessions(session_exists);
    Ok(layout)
}
