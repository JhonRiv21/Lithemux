use std::collections::HashMap;

use lithemux_core::modules::session::{Kind, Profile, Session, State};

use crate::app_shell::elsewhere::{open_elsewhere, shown_phase};
use crate::session_host::{Failure, Phase};

fn session(id: &str, conversation: Option<&str>) -> Session {
    Session {
        id: id.to_string(),
        category_id: "c".to_string(),
        slug: id.to_string(),
        role: String::new(),
        kind: Kind::Member,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: conversation.map(str::to_string),
        model: None,
        keep_alive: false,
        state: State::Idle,
        claude_version: None,
        last_activity_at: None,
        created_at: 0,
        sort: 0,
        title: None,
        pin_sort: None,
        preview: None,
        notice: None,
        unread: 0,
    }
}

// PLAN §3.4: la tarjeta muestra «abierta en otro lado» por sí sola
#[test]
fn a_session_whose_conversation_runs_in_another_process_is_open_elsewhere() {
    // Who runs each conversation, as the sweep found it.
    let held = HashMap::from([("conv-a".to_string(), 10), ("conv-b".to_string(), 11)]);
    let sessions = [
        session("a", Some("conv-a")),
        session("b", Some("conv-b")),
        session("c", Some("conv-c")),
        session("shell", None),
    ];

    let elsewhere = open_elsewhere(&sessions, &held, &|id| id == "b");

    assert_eq!(elsewhere, HashMap::from([("a".to_string(), 10)]));
}

// PLAN §3.4: what the app runs, or is starting, is never «elsewhere»
#[test]
fn the_host_s_own_program_wins() {
    for phase in [Phase::Opening, Phase::Running, Phase::Closing] {
        assert_eq!(shown_phase(Some(&phase), Some(7), false), Some(phase));
    }
}

// ronda 3B bug 4: «abierta en otro lado» lasts while somebody has it
#[test]
fn open_elsewhere_lasts_while_the_other_process_runs() {
    let told = Phase::OpenElsewhere { pid: Some(7) };

    // Its pid still runs, even before the sweep saw it.
    assert_eq!(shown_phase(Some(&told), None, true), Some(told.clone()));
    // The sweep still sees the conversation running.
    assert_eq!(shown_phase(Some(&told), Some(7), false), Some(told));
}

// ronda 3B bug 4: once the other one is gone, the card says what the
// database says
#[test]
fn open_elsewhere_ends_when_the_other_process_is_gone() {
    let told = Phase::OpenElsewhere { pid: Some(7) };
    let unknown = Phase::OpenElsewhere { pid: None };

    assert_eq!(shown_phase(Some(&told), None, false), None);
    assert_eq!(shown_phase(Some(&unknown), None, false), None);
}

// PLAN §3.4 + Fase 2 pendiente: sin pulsar Abrir
#[test]
fn a_session_not_running_here_shows_who_runs_it() {
    let elsewhere = Some(Phase::OpenElsewhere { pid: Some(7) });

    assert_eq!(shown_phase(None, Some(7), false), elsewhere);
    assert_eq!(shown_phase(Some(&Phase::Ended), Some(7), false), elsewhere);
    let failed = Phase::Failed(Failure::Other("x".to_string()));
    assert_eq!(shown_phase(Some(&failed), Some(7), false), elsewhere);
    assert_eq!(
        shown_phase(Some(&Phase::Ended), None, false),
        Some(Phase::Ended)
    );
    assert_eq!(shown_phase(None, None, false), None);
}
