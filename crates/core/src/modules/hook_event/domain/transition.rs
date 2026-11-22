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

/// The reminder Claude Code sends 60 s into any idle period.
const IDLE_REMINDER: &str = "idle_prompt";

/// `SessionStart` sources that open a conversation. `compact` is not one:
/// it fires in the middle of a turn.
const COMPACT_SOURCE: &str = "compact";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Set {
        state: State,
        /// Whether the event counts as activity (`last_activity_at`).
        touches_activity: bool,
        /// A current state that the transition must leave alone.
        preserves: Option<State>,
    },
    /// Counts as activity; the state stays as it is.
    Activity,
    /// Applies only to a session that is in `from`.
    Move { from: State, to: State },
}

/// What an event does to the session's state; `None` when it changes nothing.
pub fn transition(event: HookEvent, input: &HookInput) -> Option<Transition> {
    let set = |state, touches_activity| Transition::Set {
        state,
        touches_activity,
        preserves: None,
    };
    // The app writes `hibernated` before it stops a session on purpose.
    // What the program still reports on its way out must not undo that:
    // only the start of a new run takes a session out of hibernation.
    let late = |state, touches_activity| Transition::Set {
        state,
        touches_activity,
        preserves: Some(State::Hibernated),
    };
    match event {
        HookEvent::SessionStart if input.source.as_deref() == Some(COMPACT_SOURCE) => {
            Some(Transition::Activity)
        }
        HookEvent::SessionStart => Some(set(State::Idle, true)),
        HookEvent::Stop => Some(late(State::Idle, true)),
        HookEvent::PromptSubmit => Some(late(State::Working, true)),
        // Nothing documented fires when the user dismisses a permission
        // prompt; this reminder, 60 s into the idle period that follows,
        // is the first sign that Claude no longer waits for an answer.
        HookEvent::Notification if input.notification_type.as_deref() == Some(IDLE_REMINDER) => {
            Some(Transition::Move {
                from: State::Waiting,
                to: State::Idle,
            })
        }
        HookEvent::Notification => input
            .notification_type
            .as_deref()
            .filter(|kind| WAITING_NOTIFICATIONS.contains(kind))
            .map(|_| late(State::Waiting, false)),
        HookEvent::SessionEnd => Some(late(State::Ended, false)),
        // The permission was granted and the tool ran, well or badly: the
        // turn goes on.
        HookEvent::PostToolUse | HookEvent::PostToolUseFailure => Some(Transition::Move {
            from: State::Waiting,
            to: State::Working,
        }),
    }
}
