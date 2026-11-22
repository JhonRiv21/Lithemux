use crate::session_host::{Failure, Phase};
use lithemux_core::modules::session::{Profile, State};

use crate::workspace::start_policy::{relaunches_at_startup, starts_when_shown, wakes_when_seen};

// PLAN §2.2 (carried from 2.1): reopening a session while its program is
// still leaving shows it as closing and reopens it by itself. For the host
// to do that, the interface has to ask.
#[test]
fn a_session_that_is_closing_is_asked_for_so_that_it_reopens_by_itself() {
    assert!(starts_when_shown(Some(&Phase::Closing)));
}

// PLAN §2.3: opening from the sidebar starts a session the app does not have.
#[test]
fn a_session_the_app_does_not_have_is_started() {
    assert!(starts_when_shown(None));
}

// PLAN §2.2: a click on the card of a session that could not start, or
// that another terminal had, tries again.
#[test]
fn a_session_that_could_not_start_is_tried_again() {
    assert!(starts_when_shown(Some(&Phase::Failed(
        Failure::ClaudeNotFound("no claude".into())
    ))));
    assert!(starts_when_shown(Some(&Phase::OpenElsewhere {
        pid: Some(7)
    })));
}

// PLAN §2.3: one session, one terminal.
#[test]
fn a_session_that_runs_or_is_opening_is_not_started_again() {
    assert!(!starts_when_shown(Some(&Phase::Running)));
    assert!(!starts_when_shown(Some(&Phase::Opening)));
}

// PLAN §2.4: a session that ended stays ended, with a way to resume it.
#[test]
fn a_session_that_ended_keeps_its_screen_until_the_user_resumes_it() {
    assert!(!starts_when_shown(Some(&Phase::Ended)));
}

// PLAN §2.4: «Despertar: al enfocar una sesión hibernada se relanza».
#[test]
fn a_hibernated_claude_session_wakes_when_its_tab_comes_on_show() {
    assert!(wakes_when_seen(None, State::Hibernated, Profile::Claude));
}

// PLAN §2.4: «Si el proceso termina solo, la sesión queda en `ended` con un
// botón para reanudar»: it does not start again by being looked at.
#[test]
fn only_a_hibernated_session_wakes_by_being_seen() {
    for state in [
        State::Ended,
        State::New,
        State::Idle,
        State::Working,
        State::Waiting,
    ] {
        assert!(!wakes_when_seen(None, state, Profile::Claude), "{state:?}");
    }
}

// One session, one program: what the app already runs, or is starting, is
// not started again; and one that ended here waits for the user.
#[test]
fn a_session_the_app_already_has_is_not_woken() {
    for phase in [Phase::Opening, Phase::Running, Phase::Ended] {
        assert!(
            !wakes_when_seen(Some(&phase), State::Hibernated, Profile::Claude),
            "{phase:?}"
        );
    }
}

// Audit of block 4 (7): a session hibernated a moment ago is still leaving
// (up to 5 s). Looked at then, it must wake: the host reopens it by itself
// when the last run is gone. (This test asserted the opposite.)
#[test]
fn a_hibernated_session_that_is_still_leaving_wakes_when_it_is_seen() {
    assert!(wakes_when_seen(
        Some(&Phase::Closing),
        State::Hibernated,
        Profile::Claude
    ));
    // A session the user closed is closing too, and stays closed.
    assert!(!wakes_when_seen(
        Some(&Phase::Closing),
        State::Ended,
        Profile::Claude
    ));
}

// PLAN §2.5: «las de perfil shell se relanzan en su cwd». A shell has no
// conversation to resume and no state: it is relaunched at startup, and
// only then, so that closing one keeps it closed.
#[test]
fn a_shell_session_is_relaunched_at_startup_and_not_by_being_seen() {
    assert!(relaunches_at_startup(None, Profile::Shell));
    assert!(!relaunches_at_startup(
        Some(&Phase::Running),
        Profile::Shell
    ));
    assert!(!relaunches_at_startup(None, Profile::Claude));
    assert!(!wakes_when_seen(None, State::New, Profile::Shell));
    assert!(!wakes_when_seen(None, State::Hibernated, Profile::Shell));
}
