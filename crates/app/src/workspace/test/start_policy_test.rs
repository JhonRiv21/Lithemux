use crate::session_host::{Failure, Phase};
use crate::workspace::start_policy::starts_when_shown;

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
