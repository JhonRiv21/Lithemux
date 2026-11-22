//! How short of memory the machine is, as macOS itself judges it. Read
//! from the kernel with `sysctl`; the same level its memory pressure
//! notifications carry.

use std::path::Path;
use std::time::Duration;

use crate::platform::process::capture::stdout_within;

const SYSCTL: &str = "/usr/sbin/sysctl";
const LEVEL: &str = "kern.memorystatus_vm_pressure_level";
/// It answers in milliseconds; one that hangs must not hold the cycle that
/// asks.
const PATIENCE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    Normal,
    Warning,
    Critical,
}

/// The kernel's levels: 1 normal, 2 warning, 4 critical.
pub fn parse_pressure_level(output: &str) -> Option<MemoryPressure> {
    match output.trim() {
        "1" => Some(MemoryPressure::Normal),
        "2" => Some(MemoryPressure::Warning),
        "4" => Some(MemoryPressure::Critical),
        _ => None,
    }
}

/// `None` when it cannot be known, or `sysctl` does not answer in time;
/// whoever asks treats that as no pressure.
pub fn memory_pressure() -> Option<MemoryPressure> {
    let output = stdout_within(Path::new(SYSCTL), &["-n", LEVEL], PATIENCE)?;
    parse_pressure_level(&output)
}
