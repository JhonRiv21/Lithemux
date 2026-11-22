use std::process::Stdio;

use crate::sandbox::Sandbox;

const PROCESSES: usize = 8;
const ROUNDS: usize = 20;

// PLAN §2.0 bug 2: several processes that find an empty home at the same
// time all end up with a working database.
#[test]
fn processes_that_start_together_on_an_empty_home_all_succeed() {
    for round in 0..ROUNDS {
        let sandbox = Sandbox::new();
        let children: Vec<_> = (0..PROCESSES)
            .map(|_| {
                sandbox
                    .std_command(&["cat", "list"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap()
            })
            .collect();
        for child in children {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "round {round}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
