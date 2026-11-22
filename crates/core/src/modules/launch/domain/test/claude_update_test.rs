//! PLAN §3.2: «comparar `claude --version` con el `claude_version` de cada
//! sesión viva. Si hay una versión nueva, mostrar un aviso discreto…
//! reanudar las sesiones idle, en lote y solo con las idle.»

use crate::modules::launch::domain::claude_update::{
    LiveSession, is_newer_version, sessions_to_update,
};
use crate::modules::session::State;

#[test]
fn a_higher_number_in_any_place_is_a_newer_version() {
    assert!(is_newer_version("2.1.331", "2.1.330"));
    assert!(is_newer_version("2.2.0", "2.1.330"));
    assert!(is_newer_version("3.0.0", "2.99.999"));
    // Compared as numbers, not as text: 330 is more than 99.
    assert!(is_newer_version("2.1.330", "2.1.99"));
}

#[test]
fn the_same_or_an_older_version_is_not_newer() {
    assert!(!is_newer_version("2.1.330", "2.1.330"));
    assert!(!is_newer_version("2.1.329", "2.1.330"));
    assert!(!is_newer_version("1.9.9", "2.0.0"));
}

// `2.1` and `2.1.0` are the same version.
#[test]
fn missing_places_count_as_zero() {
    assert!(!is_newer_version("2.1", "2.1.0"));
    assert!(is_newer_version("2.1.1", "2.1"));
}

// What cannot be read is never newer: a notice must not be made up.
#[test]
fn a_version_that_cannot_be_read_is_never_newer() {
    assert!(!is_newer_version("", "2.1.330"));
    assert!(!is_newer_version("latest", "2.1.330"));
    assert!(!is_newer_version("2.1.330", "unknown"));
    assert!(!is_newer_version("2.x.1", "2.1.0"));
}

// A pre-release suffix does not make a number unreadable.
#[test]
fn a_suffix_after_the_numbers_is_ignored() {
    assert!(is_newer_version("2.1.331-beta.1", "2.1.330"));
}

fn live(id: &str, state: State, version: Option<&str>) -> LiveSession {
    LiveSession {
        id: id.to_string(),
        state,
        claude_version: version.map(str::to_string),
        keep_alive: false,
    }
}

#[test]
fn a_new_version_is_noticed_when_any_live_session_runs_an_older_one() {
    let sessions = [
        live("a", State::Working, Some("2.1.329")),
        live("b", State::Idle, Some("2.1.330")),
    ];
    let update = sessions_to_update("2.1.330", &sessions);
    assert!(update.is_available());
}

#[test]
fn nothing_is_noticed_when_every_live_session_is_up_to_date() {
    let sessions = [
        live("a", State::Idle, Some("2.1.330")),
        live("b", State::Waiting, Some("2.1.330")),
    ];
    let update = sessions_to_update("2.1.330", &sessions);
    assert!(!update.is_available());
    assert!(update.idle.is_empty());
}

// «en lote y solo con las idle»: one that works or waits for the user is
// never cut short.
#[test]
fn only_the_idle_sessions_on_an_older_version_are_resumed() {
    let sessions = [
        live("old-idle", State::Idle, Some("2.1.300")),
        live("old-working", State::Working, Some("2.1.300")),
        live("old-waiting", State::Waiting, Some("2.1.300")),
        live("new-idle", State::Idle, Some("2.1.330")),
        live("unknown-idle", State::Idle, None),
    ];
    let update = sessions_to_update("2.1.330", &sessions);
    assert!(update.is_available());
    assert_eq!(update.idle, ["old-idle"]);
    assert_eq!(update.busy, ["old-working", "old-waiting"]);
}

// B4: a session kept alive (a development server) is never cut short by
// the batch, nor counted by the notice.
#[test]
fn a_session_kept_alive_is_never_resumed_nor_counted() {
    let kept = |id, state| LiveSession {
        keep_alive: true,
        ..live(id, state, Some("2.1.300"))
    };
    let update = sessions_to_update(
        "2.1.330",
        &[
            kept("server", State::Idle),
            kept("busy-server", State::Working),
        ],
    );
    assert!(update.idle.is_empty());
    assert!(update.busy.is_empty());
    assert!(!update.is_available());
}

// A session whose version was never recorded is not judged.
#[test]
fn a_session_without_a_recorded_version_is_left_alone() {
    let update = sessions_to_update("9.9.9", &[live("a", State::Idle, None)]);
    assert!(!update.is_available());
}
