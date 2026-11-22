use crate::session_host::event_socket::session_id_of;

// PLAN §2.1: one short message per line, `{"session_id": …, "event": …}`
#[test]
fn a_well_formed_message_names_its_session() {
    assert_eq!(
        session_id_of("{\"session_id\":\"abc-1\",\"event\":\"stop\"}\n"),
        Some("abc-1".to_string())
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
        assert_eq!(session_id_of(message), None, "{message:?}");
    }
}
