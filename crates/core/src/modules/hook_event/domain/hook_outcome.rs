use crate::modules::session::State;

/// What one hook did to its session, as the app is told it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HookOutcome {
    /// The state the session went into, only when the event changed it.
    pub entered: Option<State>,
}
