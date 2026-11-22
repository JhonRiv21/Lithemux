//! When a running session is put to sleep. Its conversation is not at
//! stake: `claude --resume` brings it back.

use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::state::State;

/// The memory valve hibernates one session at most this often.
pub const VALVE_INTERVAL_SECONDS: i64 = 60;

/// At «warning», the valve only takes what has been quiet this long.
pub const VALVE_WARNING_QUIET_SECONDS: i64 = 30 * 60;

const SECONDS_PER_HOUR: i64 = 3600;

/// How short of memory the machine is, as macOS judges it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressure {
    Normal,
    Warning,
    Critical,
}

/// A session whose program runs in the app.
#[derive(Debug, Clone, Copy)]
pub struct Running<'a> {
    pub session: &'a Session,
    /// Whether it is the tab on show in a pane.
    pub is_visible: bool,
}

impl Running<'_> {
    /// Nobody is looking at it and the user did not ask to keep it.
    fn is_out_of_the_way(&self) -> bool {
        !self.is_visible && !self.session.keep_alive
    }

    fn last_active(&self) -> i64 {
        self.session
            .last_activity_at
            .unwrap_or(self.session.created_at)
    }

    fn quiet_for(&self, now: i64) -> i64 {
        now - self.last_active()
    }
}

/// Out of sight, not kept alive, and quiet for more than `after_hours`
/// while idle or waiting. A session that is working never is. One left
/// waiting is: a permission the user dismissed leaves it so for good,
/// since Claude Code reports nothing when that happens.
pub fn is_due_for_hibernation(running: &Running<'_>, now: i64, after_hours: i64) -> bool {
    let is_at_rest = matches!(running.session.state, State::Idle | State::Waiting);
    running.is_out_of_the_way()
        && is_at_rest
        && running.quiet_for(now) > after_hours * SECONDS_PER_HOUR
}

/// The memory valve: the session to hibernate now, if any. Only an idle
/// one, never one that waits for the user. The one that has been quiet
/// longest; at «warning», only if that is half an hour or more; and none
/// while the last one taken is less than a minute ago.
pub fn pick_for_memory_valve<'a>(
    running: &'a [Running<'a>],
    now: i64,
    last_taken_at: Option<i64>,
    pressure: Pressure,
) -> Option<&'a str> {
    let quiet_needed = match pressure {
        Pressure::Normal => return None,
        Pressure::Warning => VALVE_WARNING_QUIET_SECONDS,
        Pressure::Critical => 0,
    };
    if last_taken_at.is_some_and(|last| now - last < VALVE_INTERVAL_SECONDS) {
        return None;
    }
    running
        .iter()
        .filter(|running| running.is_out_of_the_way() && running.session.state == State::Idle)
        .filter(|running| running.quiet_for(now) >= quiet_needed)
        .min_by_key(|running| running.last_active())
        .map(|running| running.session.id.as_str())
}
