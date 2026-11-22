use crate::modules::hook_event::domain::hook_event::HookEvent;
use crate::modules::hook_event::domain::hook_input::HookInput;
use crate::modules::hook_event::domain::transition::WAITING_NOTIFICATIONS;

/// What an event leaves on the session's card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardNote<'a> {
    /// The text to preview, from a documented field of the hook's input.
    pub message: Option<&'a str>,
    /// Whether it counts as unread until the user looks at the session.
    pub asks_for_attention: bool,
    /// Whether the text is a request that stands only while Claude waits
    /// for the user, rather than the last thing said.
    pub is_pending: bool,
}

/// `None` for the events that say nothing to the user.
pub fn card_note(event: HookEvent, input: &HookInput) -> Option<CardNote<'_>> {
    match event {
        HookEvent::PromptSubmit => Some(CardNote {
            message: input.prompt.as_deref(),
            asks_for_attention: false,
            is_pending: false,
        }),
        HookEvent::Stop => Some(CardNote {
            message: input.last_assistant_message.as_deref(),
            asks_for_attention: true,
            is_pending: false,
        }),
        // Only the ones that block Claude on the user. The rest inform
        // (a login, a quota, a finished agent), and the idle reminder
        // repeats a turn that was already counted: none of them is worth
        // a count, nor replaces what Claude said last.
        HookEvent::Notification => input
            .notification_type
            .as_deref()
            .filter(|kind| WAITING_NOTIFICATIONS.contains(kind))
            .map(|_| CardNote {
                message: input.message.as_deref(),
                asks_for_attention: true,
                is_pending: true,
            }),
        HookEvent::SessionStart
        | HookEvent::SessionEnd
        | HookEvent::PostToolUse
        | HookEvent::PostToolUseFailure => None,
    }
}
