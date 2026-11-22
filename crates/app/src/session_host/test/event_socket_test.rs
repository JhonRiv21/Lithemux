use lithemux_core::modules::hook_event::HookEvent;
use lithemux_core::modules::session::State;

use crate::session_host::event_socket::{HookNews, news_of};

// PLAN §2.1: one short message per line, `{"session_id": …, "event": …}`.
// PLAN §3.1: the event says what happened, for the sound when done.
#[test]
fn a_well_formed_message_names_its_session_and_what_happened() {
    assert_eq!(
        news_of("{\"session_id\":\"abc-1\",\"event\":\"stop\"}\n"),
        Some(HookNews {
            session_id: "abc-1".to_string(),
            event: Some(HookEvent::Stop),
            entered: None,
        })
    );
    assert_eq!(
        news_of("{\"session_id\":\"abc-1\",\"event\":\"notification\"}").and_then(|n| n.event),
        Some(HookEvent::Notification)
    );
}

// PLAN §3.1: and what it did to the session: a turn that ends rings.
#[test]
fn a_message_says_the_state_it_moved_into() {
    let news = news_of("{\"session_id\":\"abc-1\",\"event\":\"stop\",\"entered\":\"idle\"}");
    assert_eq!(news.and_then(|n| n.entered), Some(State::Idle));
    let stayed = news_of("{\"session_id\":\"abc-1\",\"event\":\"stop\",\"entered\":null}");
    assert_eq!(stayed.and_then(|n| n.entered), None);
}

// An event this build does not know still says that the session changed.
#[test]
fn an_event_that_is_not_known_still_names_its_session() {
    assert_eq!(
        news_of("{\"session_id\":\"abc-1\",\"event\":\"something-new\"}"),
        Some(HookNews {
            session_id: "abc-1".to_string(),
            event: None,
            entered: None,
        })
    );
}

// PLAN §2.1: malformed messages are ignored
#[test]
fn anything_else_names_no_session() {
    for message in [
        "",
        "not json",
        "{\"session_id\":\"abc-1\"}",
        "{\"event\":\"stop\"}",
        "{\"session_id\":7,\"event\":\"stop\"}",
        "{\"session_id\":\"\",\"event\":\"stop\"}",
        "{\"session_id\":\"abc-1\",\"event\":null}",
        "[\"abc-1\",\"stop\"]",
    ] {
        assert_eq!(news_of(message), None, "{message:?}");
    }
}
