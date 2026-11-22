use lithemux_core::modules::session::{Profile, State};

use crate::session_host::{Failure, Phase};
use crate::sidebar::status::{Status, status_of};

const EVERY_STATE: [State; 6] = [
    State::New,
    State::Working,
    State::Idle,
    State::Waiting,
    State::Hibernated,
    State::Ended,
];

// PLAN §2.2 (carried from 2.1): while a session is opening the label says
// so, not its last saved state.
#[test]
fn a_session_that_is_opening_says_opening_whatever_was_saved() {
    for state in EVERY_STATE {
        assert_eq!(
            status_of(Some(&Phase::Opening), state, Profile::Claude),
            Status::Opening,
            "{state:?}"
        );
    }
}

// PLAN §2.2 (carried from 2.1): the saved state is from the run before
// until Claude reports the new one.
#[test]
fn a_relaunched_session_does_not_show_ended_before_claude_reports() {
    for stale in [State::Ended, State::Hibernated, State::New] {
        assert_eq!(
            status_of(Some(&Phase::Running), stale, Profile::Claude),
            Status::Opening,
            "{stale:?}"
        );
    }
}

// PLAN §2.2 (carried from 2.1)
#[test]
fn a_session_whose_program_is_leaving_says_closing() {
    for state in EVERY_STATE {
        assert_eq!(
            status_of(Some(&Phase::Closing), state, Profile::Claude),
            Status::Closing
        );
    }
}

// PLAN §2.2: working, idle, waiting for permission (highlighted).
#[test]
fn a_running_claude_shows_what_it_reported() {
    let running = Some(&Phase::Running);
    assert_eq!(
        status_of(running, State::Working, Profile::Claude),
        Status::Working
    );
    assert_eq!(
        status_of(running, State::Idle, Profile::Claude),
        Status::Idle
    );
    assert_eq!(
        status_of(running, State::Waiting, Profile::Claude),
        Status::Waiting
    );
}

// PLAN §2.2
#[test]
fn only_waiting_for_the_user_is_highlighted() {
    let all = [
        Status::Opening,
        Status::Closing,
        Status::Working,
        Status::Idle,
        Status::Waiting,
        Status::Hibernated,
        Status::Ended,
        Status::NotStarted,
        Status::Running,
        Status::NotRunning,
        Status::OpenElsewhere(None),
        Status::Failed(Failure::Other(String::new())),
    ];
    let highlighted: Vec<&Status> = all.iter().filter(|s| s.is_highlighted()).collect();
    assert_eq!(highlighted, [&Status::Waiting]);
}

// PLAN §2.2: hibernated and ended.
#[test]
fn a_session_that_ended_here_shows_ended_or_hibernated() {
    let ended = Some(&Phase::Ended);
    assert_eq!(
        status_of(ended, State::Ended, Profile::Claude),
        Status::Ended
    );
    assert_eq!(
        status_of(ended, State::Hibernated, Profile::Claude),
        Status::Hibernated
    );
}

// PLAN §2.2: a session the app does not run shows what the database says;
// `lithemux open` in another terminal keeps it true.
#[test]
fn a_session_the_app_does_not_run_shows_its_saved_state() {
    assert_eq!(
        status_of(None, State::New, Profile::Claude),
        Status::NotStarted
    );
    assert_eq!(
        status_of(None, State::Working, Profile::Claude),
        Status::Working
    );
    assert_eq!(
        status_of(None, State::Hibernated, Profile::Claude),
        Status::Hibernated
    );
    assert_eq!(
        status_of(None, State::Ended, Profile::Claude),
        Status::Ended
    );
}

// PLAN §2.2: shell sessions have a card too.
#[test]
fn a_shell_runs_or_does_not() {
    assert_eq!(
        status_of(Some(&Phase::Running), State::New, Profile::Shell),
        Status::Running
    );
    assert_eq!(
        status_of(None, State::New, Profile::Shell),
        Status::NotRunning
    );
    assert_eq!(
        status_of(Some(&Phase::Ended), State::New, Profile::Shell),
        Status::NotRunning
    );
    assert_eq!(
        status_of(Some(&Phase::Opening), State::New, Profile::Shell),
        Status::Opening
    );
}

// PLAN §2.1: a session another terminal has is said to be there.
#[test]
fn a_session_open_elsewhere_or_that_failed_says_so() {
    assert_eq!(
        status_of(
            Some(&Phase::OpenElsewhere { pid: Some(7) }),
            State::Idle,
            Profile::Claude
        ),
        Status::OpenElsewhere(Some(7))
    );
    assert_eq!(
        status_of(
            Some(&Phase::Failed(Failure::ClaudeNotFound("no claude".into()))),
            State::New,
            Profile::Claude
        ),
        Status::Failed(Failure::ClaudeNotFound("no claude".into()))
    );
}
