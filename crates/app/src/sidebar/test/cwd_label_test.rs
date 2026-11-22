use std::path::Path;

use crate::sidebar::cwd_label::cwd_label;

const HOME: &str = "/Users/ana";

fn label(cwd: &str) -> String {
    cwd_label(cwd, Some(Path::new(HOME)))
}

// PLAN §2.2: the abbreviated cwd.
#[test]
fn the_home_is_written_as_a_tilde() {
    assert_eq!(label("/Users/ana"), "~");
    assert_eq!(label("/Users/ana/Personal/lithemux"), "~/Personal/lithemux");
}

// PLAN §2.2
#[test]
fn a_deep_directory_keeps_its_last_two_names() {
    assert_eq!(
        label("/Users/ana/Work/clients/acme/portal/web"),
        "~/…/portal/web"
    );
}

// PLAN §2.2
#[test]
fn a_directory_outside_the_home_keeps_its_root() {
    assert_eq!(label("/"), "/");
    assert_eq!(label("/opt/tools"), "/opt/tools");
    assert_eq!(label("/var/lib/data/service/logs"), "/…/service/logs");
    // Not the home, though it starts like it.
    assert_eq!(label("/Users/anabel/x"), "/Users/anabel/x");
}

// PLAN §2.2
#[test]
fn without_a_known_home_nothing_is_a_tilde() {
    assert_eq!(cwd_label("/Users/ana/x", None), "/Users/ana/x");
}
