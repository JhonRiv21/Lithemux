#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "../support/mod.rs"]
mod support;

mod board_store_test;
mod directories_test;
mod fake_claude_test;
mod hook_log_test;
mod login_shell_probe_test;
mod session_locks_test;
mod sqlite_contracts;
mod store_test;
mod temp_home;
mod transcript_locator_test;
