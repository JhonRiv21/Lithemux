use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::state::State;
use crate::modules::session::error::SessionError;
use crate::modules::session::ports::session_repository::SessionRepository;

/// When the app starts, nothing of it runs: a Claude session the database
/// still has as working, idle or waiting was cut short by a crash, a
/// forced quit or a restart of the machine. It is hibernated, which is
/// what it is: its conversation is there to be resumed.
///
/// `claim` takes the session for the time it takes to write that, and
/// gives nothing for one that is somebody else's: open in a terminal with
/// `lithemux open`, or left running by an app that died. Those are left
/// alone. Returns the ids of the ones that were hibernated.
pub fn hibernate_what_was_left_running<Claim>(
    sessions: &dyn SessionRepository,
    claim: impl Fn(&str) -> Option<Claim>,
) -> Result<Vec<String>, SessionError> {
    let mut hibernated = Vec::new();
    for session in sessions.list_all()? {
        let was_running = matches!(session.state, State::Working | State::Idle | State::Waiting);
        if session.profile != Profile::Claude || !was_running {
            continue;
        }
        // Held until the state is written: nobody opens it in between.
        let Some(_claimed) = claim(&session.id) else {
            continue;
        };
        sessions.update_state(&session.id, State::Hibernated, None, None)?;
        sessions.update_notice(&session.id, None)?;
        hibernated.push(session.id);
    }
    Ok(hibernated)
}
