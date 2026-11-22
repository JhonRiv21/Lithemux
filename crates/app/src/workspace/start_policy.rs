//! Whether showing a session must also start its program. No gpui.

use lithemux_core::modules::session::{Profile, State};

use crate::session_host::Phase;

/// Showing a session from the sidebar asks the host for it unless its
/// program is here, or was and left its screen to be read. One that is
/// still closing is asked for too: the host then opens it by itself once
/// its last run is gone.
pub fn starts_when_shown(phase: Option<&Phase>) -> bool {
    !matches!(phase, Some(Phase::Opening | Phase::Running | Phase::Ended))
}

/// A tab that comes on show: its session wakes if it is a hibernated
/// Claude session whose program the app does not run. That includes one
/// that is still leaving, within the grace of its hibernation: asked for
/// then, the host opens it once the last run is gone. One that ended waits
/// for the user to resume it.
pub fn wakes_when_seen(phase: Option<&Phase>, state: State, profile: Profile) -> bool {
    let is_not_running = matches!(phase, None | Some(Phase::Closing));
    is_not_running && profile == Profile::Claude && state == State::Hibernated
}

/// When the app starts, the shell sessions that have a tab are started
/// again in their directory.
pub fn relaunches_at_startup(phase: Option<&Phase>, profile: Profile) -> bool {
    phase.is_none() && profile == Profile::Shell
}
