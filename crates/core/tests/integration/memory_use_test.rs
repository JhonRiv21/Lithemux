//! PLAN §3.3: the memory of a session's process tree, measured on real
//! processes. The helpers are this test binary run again.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Stdio};
use std::time::Instant;

use lithemux_core::platform::process::memory_use::MemoryMeter;

use crate::sandbox::this_test_binary;

const ROLE: &str = "LITHEMUX_TEST_MEMORY_ROLE";
const HOLDS: &str = "holds";
const STARTS_ONE_THAT_HOLDS: &str = "starts-one-that-holds";
const HOLDS_NOTHING: &str = "holds-nothing";
/// What the helper that holds memory keeps, and touches.
const HELD_BYTES: usize = 64 * 1024 * 1024;

fn run_again_as(role: &str) -> Child {
    this_test_binary()
        .args(["--exact", "memory_use_test::holder", "--nocapture"])
        .env(ROLE, role)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

/// Not a test of its own: the body of the processes the tests start. It
/// holds memory, or starts another that does, says it is ready and stays
/// until its stdin closes.
#[test]
fn holder() {
    let Ok(role) = std::env::var("LITHEMUX_TEST_MEMORY_ROLE") else {
        return;
    };
    let mut held = Vec::new();
    let mut child = None;
    if role == HOLDS {
        held = vec![1_u8; HELD_BYTES];
    } else if role == STARTS_ONE_THAT_HOLDS {
        let mut started = run_again_as(HOLDS);
        wait_until_ready(&mut started);
        child = Some(started);
    }
    println!("ready {}", held.len());
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_to_end(&mut Vec::new());
    if let Some(mut child) = child {
        drop(child.stdin.take());
        let _ = child.wait();
    }
}

fn wait_until_ready(child: &mut Child) {
    let stdout = child.stdout.take().unwrap();
    for line in BufReader::new(stdout).lines() {
        if line.unwrap().starts_with("ready") {
            return;
        }
    }
    panic!("the helper ended without being ready");
}

fn finish(mut child: Child) {
    drop(child.stdin.take());
    child.wait().unwrap();
}

// PLAN §3.3: memory per session, from its process tree.
#[test]
fn the_memory_of_what_a_program_started_counts_as_its_own() {
    let mut child = run_again_as(STARTS_ONE_THAT_HOLDS);
    wait_until_ready(&mut child);
    let mut meter = MemoryMeter::new();
    let measured = meter.measure(&[child.id()], Instant::now());
    let used = measured[&child.id()];
    assert!(used >= HELD_BYTES as u64, "{used} bytes");
    finish(child);
}

// PLAN §3.3: and per session: one session's memory is not given to another.
#[test]
fn each_tree_counts_only_its_own_processes() {
    let mut heavy = run_again_as(STARTS_ONE_THAT_HOLDS);
    wait_until_ready(&mut heavy);
    let mut light = run_again_as(HOLDS_NOTHING);
    wait_until_ready(&mut light);
    let mut meter = MemoryMeter::new();
    let measured = meter.measure(&[heavy.id(), light.id()], Instant::now());
    assert!(measured[&heavy.id()] >= HELD_BYTES as u64);
    assert!(measured[&light.id()] < HELD_BYTES as u64 / 2);
    finish(heavy);
    finish(light);
}

// Correction round of 3A (10): a program that is gone is not measured at
// all: not «0 MB» on its card, and nothing in the total. Whether it was gone
// before the first measure or ended between two.
#[test]
fn a_program_that_ended_is_not_measured() {
    let mut gone = run_again_as(HOLDS);
    wait_until_ready(&mut gone);
    let gone_pid = gone.id();
    finish(gone);
    let mut meter = MemoryMeter::new();
    assert!(
        !meter
            .measure(&[gone_pid], Instant::now())
            .contains_key(&gone_pid)
    );

    let mut ending = run_again_as(HOLDS);
    wait_until_ready(&mut ending);
    let pid = ending.id();
    assert!(meter.measure(&[pid], Instant::now()).contains_key(&pid));
    finish(ending);
    assert!(!meter.measure(&[pid], Instant::now()).contains_key(&pid));
}
