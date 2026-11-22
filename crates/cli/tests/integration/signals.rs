use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal, kill_process, test_kill_process};

fn pid(raw: impl TryInto<i32>) -> Pid {
    Pid::from_raw(raw.try_into().ok().unwrap()).unwrap()
}

pub fn send(to: u32, signal: Signal) {
    kill_process(pid(to), signal).unwrap();
}

pub fn is_alive(raw: i32) -> bool {
    test_kill_process(pid(raw)).is_ok()
}

pub fn wait_until_gone(raw: i32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while is_alive(raw) {
        assert!(Instant::now() < deadline, "process {raw} is still alive");
        std::thread::sleep(Duration::from_millis(10));
    }
}
