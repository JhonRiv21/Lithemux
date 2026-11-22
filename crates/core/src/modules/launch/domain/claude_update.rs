//! Whether the `claude` that would be launched now is newer than the one
//! a running session started with. Sessions keep running the version they
//! started with; only a new start picks up an update.

use crate::modules::session::State;

/// A session whose program runs in the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveSession {
    pub id: String,
    pub state: State,
    /// What `claude --version` said when it was launched.
    pub claude_version: Option<String>,
    /// Kept alive on purpose (a development server, say).
    pub keep_alive: bool,
}

/// The live sessions on an older version than the installed one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Update {
    /// The ones that can be resumed on the new version now.
    pub idle: Vec<String>,
    /// The ones that work or wait for the user: they are left alone.
    pub busy: Vec<String>,
}

impl Update {
    pub fn is_available(&self) -> bool {
        !self.idle.is_empty() || !self.busy.is_empty()
    }
}

/// `2.1.330` → `[2, 1, 330]`. What follows the numbers (`-beta.1`) is
/// ignored; anything else that is not a number makes it unreadable.
fn numbers_of(version: &str) -> Option<Vec<u64>> {
    let numbers = version.split(['-', '+']).next()?;
    let parts: Option<Vec<u64>> = numbers.split('.').map(|part| part.parse().ok()).collect();
    parts.filter(|parts| !parts.is_empty())
}

/// Compared place by place, as numbers; a missing place is a zero. A
/// version that cannot be read is never newer, nor older.
pub fn is_newer_version(installed: &str, running: &str) -> bool {
    let (Some(mut installed), Some(mut running)) = (numbers_of(installed), numbers_of(running))
    else {
        return false;
    };
    let places = installed.len().max(running.len());
    installed.resize(places, 0);
    running.resize(places, 0);
    installed > running
}

/// The notice and the batch of PLAN §3.2. A session whose version was
/// never recorded is not judged, and one kept alive (a development server)
/// is neither resumed nor counted: the batch must not cut it short.
pub fn sessions_to_update(installed: &str, sessions: &[LiveSession]) -> Update {
    let is_older = |session: &&LiveSession| {
        session
            .claude_version
            .as_deref()
            .is_some_and(|running| is_newer_version(installed, running))
    };
    let (idle, busy): (Vec<&LiveSession>, Vec<&LiveSession>) = sessions
        .iter()
        .filter(is_older)
        .filter(|session| !session.keep_alive)
        .partition(|session| session.state == State::Idle);
    let ids = |sessions: Vec<&LiveSession>| sessions.iter().map(|s| s.id.clone()).collect();
    Update {
        idle: ids(idle),
        busy: ids(busy),
    }
}
