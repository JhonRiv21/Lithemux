use std::time::{Duration, Instant};

use rustix::process::{Pid, test_kill_process};

pub fn is_alive(pid: i32) -> bool {
    Pid::from_raw(pid).is_some_and(|pid| test_kill_process(pid).is_ok())
}

pub fn wait_until_gone(pid: i32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while is_alive(pid) {
        assert!(Instant::now() < deadline, "process {pid} is still alive");
        std::thread::sleep(Duration::from_millis(10));
    }
}
