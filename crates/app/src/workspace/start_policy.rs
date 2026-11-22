//! Whether showing a session must also start its program. No gpui.

use crate::session_host::Phase;

/// Showing a session from the sidebar asks the host for it unless its
/// program is here, or was and left its screen to be read. One that is
/// still closing is asked for too: the host then opens it by itself once
/// its last run is gone.
pub fn starts_when_shown(phase: Option<&Phase>) -> bool {
    !matches!(phase, Some(Phase::Opening | Phase::Running | Phase::Ended))
}
