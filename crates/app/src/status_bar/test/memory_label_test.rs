use std::collections::HashMap;

use crate::status_bar::memory_label::{memory_label, total};

// PLAN §3.3: a memory meter, readable at a glance: megabytes below a
// gigabyte, gigabytes with one decimal above.
#[test]
fn memory_reads_in_megabytes_or_gigabytes() {
    assert_eq!(memory_label(0), "0 MB");
    assert_eq!(memory_label(312 * 1024 * 1024), "312 MB");
    assert_eq!(memory_label(1023 * 1024 * 1024), "1023 MB");
    assert_eq!(memory_label(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(memory_label(1536 * 1024 * 1024), "1.5 GB");
    assert_eq!(memory_label(12 * 1024 * 1024 * 1024 + 1), "12.0 GB");
}

// Less than a megabyte is not shown as nothing.
#[test]
fn a_little_memory_is_still_a_megabyte() {
    assert_eq!(memory_label(1), "1 MB");
    assert_eq!(memory_label(1024 * 1024 + 1), "2 MB");
}

// PLAN §3.3 «y total»: the sum of the sessions shown, and how many.
#[test]
fn the_total_is_the_sum_of_every_session_and_their_number() {
    let mb = 1024 * 1024;
    let memory = HashMap::from([("a".to_string(), 95 * mb), ("b".to_string(), 132 * mb)]);
    assert_eq!(total(&memory), Some((227 * mb, 2)));
    assert_eq!(total(&HashMap::new()), None);
}
