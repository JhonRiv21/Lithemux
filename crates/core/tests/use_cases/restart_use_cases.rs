//! PLAN §2.5: «Al abrir la app… todas las sesiones de `claude` aparecen
//! hibernadas». PLAN principle 3: a restart of the Mac, a quit of the app
//! and a hibernation are treated the same.

use lithemux_core::modules::session::{
    Kind, NewSession, Profile, SessionRef, State, apply_state, create_session, find_session_by_id,
    hibernate_what_was_left_running, note_request,
};

use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::sample_data::{NOW, add_category, add_session};

fn world() -> InMemoryStore {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    add_category(&store, &boards, "demo");
    store
}

fn in_state(store: &InMemoryStore, slug: &str, state: State) -> String {
    let id = add_session(store, store, &format!("demo/{slug}"), Kind::Member).id;
    apply_state(store, &id, state, Some(NOW), None).unwrap();
    id
}

fn state_of(store: &InMemoryStore, id: &str) -> State {
    find_session_by_id(store, store, id)
        .unwrap()
        .unwrap()
        .1
        .state
}

#[test]
fn what_a_crash_left_as_running_is_hibernated() {
    let store = world();
    let ids: Vec<String> = [State::Working, State::Idle, State::Waiting]
        .iter()
        .enumerate()
        .map(|(n, state)| in_state(&store, &format!("s{n}"), *state))
        .collect();
    note_request(&store, &ids[2], Some("Claude needs your permission")).unwrap();

    let mut hibernated = hibernate_what_was_left_running(&store, |_| Some(())).unwrap();
    hibernated.sort();
    let mut expected = ids.clone();
    expected.sort();
    assert_eq!(hibernated, expected);
    for id in &ids {
        assert_eq!(state_of(&store, id), State::Hibernated);
    }
    // What it was waiting for went with the process that asked.
    let waiting = find_session_by_id(&store, &store, &ids[2])
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(waiting.notice, None);
    // The last activity is not touched: how long it has been quiet is true.
    assert_eq!(waiting.last_activity_at, Some(NOW));
}

#[test]
fn a_session_that_was_not_running_keeps_its_state() {
    let store = world();
    let never = in_state(&store, "never", State::New);
    let ended = in_state(&store, "ended", State::Ended);
    let asleep = in_state(&store, "asleep", State::Hibernated);
    assert!(
        hibernate_what_was_left_running(&store, |_| Some(()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(state_of(&store, &never), State::New);
    assert_eq!(state_of(&store, &ended), State::Ended);
    assert_eq!(state_of(&store, &asleep), State::Hibernated);
}

// `lithemux open` in a terminal is not the app's to put to sleep.
#[test]
fn a_session_open_in_another_terminal_is_left_alone() {
    let store = world();
    let elsewhere = in_state(&store, "elsewhere", State::Working);
    let ours = in_state(&store, "ours", State::Working);
    let hibernated =
        hibernate_what_was_left_running(&store, |id| (id != elsewhere.as_str()).then_some(()))
            .unwrap();
    assert_eq!(hibernated, [ours]);
    assert_eq!(state_of(&store, &elsewhere), State::Working);
}

// PLAN §2.5: shell sessions are relaunched; they have no state to settle.
#[test]
fn a_shell_session_is_not_hibernated() {
    let store = world();
    let new = NewSession {
        target: SessionRef::parse("demo/terminal").unwrap(),
        role: "terminal".to_string(),
        kind: Kind::Member,
        profile: Profile::Shell,
        cwd: None,
        model: None,
    };
    let shell = create_session(&store, &store, new, NOW).unwrap().1;
    apply_state(&store, &shell.id, State::Idle, None, None).unwrap();
    assert!(
        hibernate_what_was_left_running(&store, |_| Some(()))
            .unwrap()
            .is_empty()
    );
}
