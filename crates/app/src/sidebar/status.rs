//! What a card says about its session: where the host has it and, when it
//! runs, what Claude last reported.

use lithemux_core::modules::session::{Profile, Session, State};

use crate::session_host::{Failure, Phase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Opening,
    /// Waking from hibernation: its conversation is being resumed.
    Resuming,
    Closing,
    Working,
    Idle,
    /// Blocked on the user: the one that is highlighted.
    Waiting,
    Hibernated,
    Ended,
    NotStarted,
    /// A shell that runs; it reports nothing else.
    Running,
    NotRunning,
    OpenElsewhere(Option<u32>),
    Failed(Failure),
}

impl Status {
    pub fn is_highlighted(&self) -> bool {
        *self == Status::Waiting
    }

    /// Whether there is a program of the session in the app right now.
    pub fn is_live(&self) -> bool {
        matches!(
            self,
            Status::Opening
                | Status::Resuming
                | Status::Working
                | Status::Idle
                | Status::Waiting
                | Status::Running
        )
    }
}

/// What a session shows, on its card and on its tab in a panel. One
/// function for both, fed with the same phases (`Live::shown_phase`), so
/// that they always say the same (ronda 3C).
pub fn session_status(phases: &dyn Fn(&str) -> Option<Phase>, session: &Session) -> Status {
    status_of(phases(&session.id).as_ref(), session.state, session.profile)
}

pub fn status_of(phase: Option<&Phase>, state: State, profile: Profile) -> Status {
    match phase {
        Some(Phase::Opening) if state == State::Hibernated && profile == Profile::Claude => {
            Status::Resuming
        }
        Some(Phase::Opening) => Status::Opening,
        Some(Phase::Closing) => Status::Closing,
        Some(Phase::OpenElsewhere { pid }) => Status::OpenElsewhere(*pid),
        Some(Phase::Failed(reason)) => Status::Failed(reason.clone()),
        Some(Phase::Running) if profile == Profile::Shell => Status::Running,
        Some(Phase::Running) => match state {
            State::Working => Status::Working,
            State::Idle => Status::Idle,
            State::Waiting => Status::Waiting,
            // The program started and Claude has not said so yet: what the
            // database has is from the run before.
            State::Hibernated => Status::Resuming,
            State::New | State::Ended => Status::Opening,
        },
        Some(Phase::Ended) | None if profile == Profile::Shell => Status::NotRunning,
        Some(Phase::Ended) => match state {
            State::Hibernated => Status::Hibernated,
            _ => Status::Ended,
        },
        // Not in the app. It may run in another terminal, which keeps the
        // database up to date just the same.
        None => match state {
            State::New => Status::NotStarted,
            State::Working => Status::Working,
            State::Idle => Status::Idle,
            State::Waiting => Status::Waiting,
            State::Hibernated => Status::Hibernated,
            State::Ended => Status::Ended,
        },
    }
}
