//! How much memory, written for a person.

use std::collections::HashMap;

const MEGABYTE: u64 = 1024 * 1024;
const GIGABYTE: u64 = 1024 * MEGABYTE;

/// Megabytes, rounded up, below a gigabyte; gigabytes with one decimal at
/// and above it.
pub fn memory_label(bytes: u64) -> String {
    if bytes >= GIGABYTE {
        return format!("{:.1} GB", bytes as f64 / GIGABYTE as f64);
    }
    format!("{} MB", bytes.div_ceil(MEGABYTE))
}

/// The memory of all the sessions measured, and how many they are; nothing
/// while none is.
pub fn total(memory: &HashMap<String, u64>) -> Option<(u64, usize)> {
    (!memory.is_empty()).then(|| (memory.values().sum(), memory.len()))
}
