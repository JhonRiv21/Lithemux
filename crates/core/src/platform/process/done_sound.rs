//! The sound of a finished turn (docs/decisiones.md D-58): one of the
//! system's sounds, played by `afplay`. Nothing is shown and nothing leaves
//! this Mac.

use std::path::Path;
use std::time::Duration;

use crate::platform::process::capture::stdout_within;

const AFPLAY: &str = "/usr/bin/afplay";
pub const DONE_SOUND: &str = "/System/Library/Sounds/Glass.aiff";
/// The sound lasts about a second; a player that hangs is given up.
const PATIENCE: Duration = Duration::from_secs(3);

/// Plays it and returns once it was played, so whoever calls does it away
/// from the interface thread. The player is reaped either way.
pub fn play_sound(sound: &Path) {
    let _ = stdout_within(Path::new(AFPLAY), &[sound], PATIENCE);
}
