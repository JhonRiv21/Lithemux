use gpui::NoopTextSystem;

use crate::terminal_view::{TerminalError, probe_text_rendering};

// PLAN §1.2 test 1: a build whose text system draws nothing must not pass
// for one that shows a prompt.
#[test]
fn a_text_system_that_draws_nothing_fails_the_probe() {
    let outcome = probe_text_rendering(&NoopTextSystem::new());

    assert!(matches!(outcome, Err(TerminalError::TextRendering(_))));
}
