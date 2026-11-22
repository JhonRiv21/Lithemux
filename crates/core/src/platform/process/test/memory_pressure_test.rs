use crate::platform::process::memory_pressure::{MemoryPressure, parse_pressure_level};

// PLAN §2.4: the valve acts on the critical state of macOS's memory
// pressure. The kernel reports it as 1, 2 or 4.
#[test]
fn the_levels_of_the_kernel_are_read() {
    assert_eq!(parse_pressure_level("1\n"), Some(MemoryPressure::Normal));
    assert_eq!(parse_pressure_level("2\n"), Some(MemoryPressure::Warning));
    assert_eq!(parse_pressure_level(" 4 "), Some(MemoryPressure::Critical));
}

// PLAN §2.4: only a clear «critical» hibernates anything.
#[test]
fn anything_else_is_not_a_level() {
    for output in ["", "3", "0", "critical", "4 4", "-4"] {
        assert_eq!(parse_pressure_level(output), None, "{output:?}");
    }
}
