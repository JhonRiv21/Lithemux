use crate::modules::hook_event::domain::hook_event::HookEvent;

/// Tells the app, when it is running, that a session changed. Stored state
/// stays the source of truth, so delivery is best effort and never blocks.
pub trait AppNotifier {
    fn notify(&self, session_id: &str, event: HookEvent);
}
