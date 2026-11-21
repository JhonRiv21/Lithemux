use std::time::{Duration, Instant};

use rustix::process::{Pid, test_kill_process};

use crate::support::{start, start_with_wakes, text, wait_for, wait_until};

// Audit F1 #1: a program that opens a synchronized update (BSU) and stalls
// must not freeze the screen: the terminal gives up waiting after 150 ms.
#[test]
fn a_synchronized_update_left_open_is_shown_after_its_timeout() {
    let started = Instant::now();
    let session = start("sync-stall");
    let screen = wait_for(&session, "frame");
    assert!(
        !screen.contains("prompt$"),
        "the frame only showed once more output arrived"
    );
    // The program stays silent for a full second after the frame.
    assert!(started.elapsed() < Duration::from_millis(900));
}

fn pid_on_screen(screen: &str) -> Pid {
    let number = screen
        .split("pid:")
        .nth(1)
        .and_then(|rest| rest.split(':').next())
        .unwrap();
    Pid::from_raw(number.parse().unwrap()).unwrap()
}

// Audit F1 #2: closing the view ends the program even when it ignores SIGHUP,
// and the session's threads end with it.
#[test]
fn a_program_that_ignores_hangup_dies_with_its_session() {
    let (session, wakes) = start_with_wakes("ignore-hup");
    let pid = pid_on_screen(&wait_for(&session, "pid:"));
    assert!(test_kill_process(pid).is_ok(), "the program is running");

    drop(session);

    wait_until("the program to die", || test_kill_process(pid).is_err());
    // Every thread that feeds the screen holds a sender of this channel.
    wait_until("the session threads to end", || {
        wakes.try_recv().is_err_and(|error| error.is_closed())
    });
}

// Audit F1: nothing repaints while nothing happens.
#[test]
fn an_idle_terminal_asks_for_no_repaints() {
    let (session, wakes) = start_with_wakes("greet");
    wait_for(&session, "ready");
    std::thread::sleep(Duration::from_millis(200));
    while wakes.try_recv().is_ok() {}

    std::thread::sleep(Duration::from_secs(2));
    assert!(wakes.try_recv().is_err(), "a repaint was requested at rest");
    assert!(text(&session.snapshot()).contains("ready"));
}
