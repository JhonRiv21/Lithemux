//! PLAN §2.2: the listening ports of a session's process tree, found
//! without the network. The listeners are this test binary run again.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Child, Stdio};

use lithemux_core::platform::process::listening_ports::listening_ports;
use lithemux_core::platform::process::local_page::local_page_url;

use crate::sandbox::this_test_binary;

const ROLE: &str = "LITHEMUX_TEST_LISTENER_ROLE";
const LISTENS: &str = "listens";
const STARTS_ONE_THAT_LISTENS: &str = "starts-one-that-listens";

fn run_again_as(role: &str) -> Child {
    this_test_binary()
        .args(["--exact", "listening_ports_test::listener", "--nocapture"])
        .env(ROLE, role)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

/// Not a test of its own: the body of the processes the tests start. It
/// listens, or starts another that does, says the port and stays until its
/// stdin closes.
#[test]
fn listener() {
    let Ok(role) = std::env::var("LITHEMUX_TEST_LISTENER_ROLE") else {
        return;
    };
    let mut kept: (Option<TcpListener>, Option<Child>) = (None, None);
    let port = if role == LISTENS {
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        kept.0 = Some(socket);
        port
    } else {
        let mut child = run_again_as(LISTENS);
        let port = port_announced_by(&mut child);
        kept.1 = Some(child);
        port
    };
    println!("port {port}");
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_to_end(&mut Vec::new());
    if let Some(mut child) = kept.1 {
        drop(child.stdin.take());
        let _ = child.wait();
    }
}

fn port_announced_by(child: &mut Child) -> u16 {
    let stdout = child.stdout.take().unwrap();
    for line in BufReader::new(stdout).lines() {
        if let Some(port) = line.unwrap().strip_prefix("port ") {
            return port.trim().parse().unwrap();
        }
    }
    panic!("the listener ended without saying its port");
}

fn finish(mut child: Child) {
    drop(child.stdin.take());
    child.wait().unwrap();
}

// Block 4, part 0 (1): Claude Code itself listens on a local port, one per
// session, that is of no use to the user. A card shows what the session
// started, never the port of its own program.
#[test]
fn the_port_of_the_program_itself_is_not_one_of_its_ports() {
    let mut child = run_again_as(LISTENS);
    port_announced_by(&mut child);
    let found = listening_ports(&[child.id()]);
    assert_eq!(found.get(&child.id()), Some(&Vec::new()));
    finish(child);
}

// PLAN §2.2: the ports of the session's process tree.
#[test]
fn the_port_of_a_program_started_by_the_program_is_found() {
    let mut child = run_again_as(STARTS_ONE_THAT_LISTENS);
    let port = port_announced_by(&mut child);
    let found = listening_ports(&[child.id()]);
    assert_eq!(found.get(&child.id()), Some(&vec![port]));
    finish(child);
}

// PLAN §2.2: a card shows the ports of its own session only.
#[test]
fn the_ports_of_other_programs_are_not_given_to_it() {
    let ours = TcpListener::bind("127.0.0.1:0").unwrap();
    let our_port = ours.local_addr().unwrap().port();
    let mut first = run_again_as(STARTS_ONE_THAT_LISTENS);
    let mut second = run_again_as(STARTS_ONE_THAT_LISTENS);
    let (first_port, second_port) = (
        port_announced_by(&mut first),
        port_announced_by(&mut second),
    );
    let found = listening_ports(&[first.id(), second.id()]);
    assert_eq!(found[&first.id()], [first_port]);
    assert_eq!(found[&second.id()], [second_port]);
    assert!(!found[&first.id()].contains(&our_port));
    finish(first);
    finish(second);
}

// PLAN §2.2
#[test]
fn a_program_that_ended_has_no_ports() {
    let mut child = run_again_as(STARTS_ONE_THAT_LISTENS);
    port_announced_by(&mut child);
    let pid = child.id();
    finish(child);
    assert_eq!(listening_ports(&[pid]).get(&pid), Some(&Vec::new()));
}

// PLAN §2.2: a click opens http://localhost:<port>.
#[test]
fn the_page_of_a_port_is_on_localhost() {
    assert_eq!(local_page_url(52844), "http://localhost:52844");
}

// PLAN §2.4: on this machine the kernel answers with a level.
#[test]
fn the_memory_pressure_of_this_machine_can_be_read() {
    use lithemux_core::platform::process::memory_pressure::memory_pressure;
    assert!(memory_pressure().is_some());
}
