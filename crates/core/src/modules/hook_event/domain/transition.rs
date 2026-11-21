use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;
use crate::modules::session::State;

/// Notification types that mean Claude is blocked on the user. `idle_prompt`
/// is not one of them: it fires 60 s into any idle period, and a session that
/// is merely idle must stay eligible for hibernation.
pub const WAITING_NOTIFICATIONS: [&str; 3] = [
    "permission_prompt",
    "elicitation_dialog",
    "elicitation_url_dialog",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub state: State,
    /// Whether the event counts as activity (`last_activity_at`).
    pub touches_activity: bool,
    /// A current state that the transition must leave alone.
    pub preserves: Option<State>,
}

/// What an event does to the session's state; `None` when it changes nothing.
pub fn transition(event: HookEvent, input: &HookInput) -> Option<Transition> {
    let change = |state, touches_activity| Transition {
        state,
        touches_activity,
        preserves: None,
    };
    match event {
        HookEvent::SessionStart | HookEvent::Stop => Some(change(State::Idle, true)),
        HookEvent::PromptSubmit => Some(change(State::Working, true)),
        HookEvent::Notification => input
            .notification_type
            .as_deref()
            .filter(|kind| WAITING_NOTIFICATIONS.contains(kind))
            .map(|_| change(State::Waiting, false)),
        // The app writes `hibernated` before it stops a session on purpose.
        HookEvent::SessionEnd => Some(Transition {
            state: State::Ended,
            touches_activity: false,
            preserves: Some(State::Hibernated),
        }),
    }
}
