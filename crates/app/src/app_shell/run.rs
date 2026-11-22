//! Starts the app: always with the session host, which keeps it to one
//! instance per home, and then its window or, for tests, none.

use std::process::ExitCode;

use gpui::{App, AppContext};

use super::live::Live;
use super::{headless, window};
use crate::launch_arguments::LaunchRequest;
use crate::session_host::{HostConfig, SessionHost};
use lithemux_core::platform::log::app_log::AppLog;
use lithemux_core::platform::log::panic_hook::log_panics;
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;

/// Debug builds only: lets a test make the app panic the moment it starts,
/// to see that the panic reaches the log.
fn panic_if_a_test_asks() {
    if cfg!(debug_assertions) && std::env::var_os("LITHEMUX_PANIC_ON_START").is_some() {
        panic!("induced at start for a test");
    }
}

pub fn run(request: LaunchRequest) -> ExitCode {
    // First of all: whatever goes wrong from here on leaves its trace.
    if let Ok(home) = Home::resolve() {
        log_panics(AppLog::shared(home.app_log_path()));
    }
    panic_if_a_test_asks();
    // Before anything shows: a second instance says why it leaves and leaves.
    let started = HostConfig::from_environment().and_then(|config| {
        let home = config.home.clone();
        let (host, changes) = SessionHost::start(config)?;
        // The interface reads the board on a connection of its own.
        let db = Db::open(&home)?;
        Ok((host, changes, home, db))
    });
    let (host, changes, home, db) = match started {
        Ok(started) => started,
        Err(error) => {
            eprintln!("lithemux-app: {error}");
            return ExitCode::FAILURE;
        }
    };
    if !request.with_window {
        gpui_platform::headless().run(move |cx: &mut App| {
            headless::start(host, changes, &request.targets, cx);
        });
        return ExitCode::SUCCESS;
    }
    gpui_platform::application().run(move |cx: &mut App| {
        let live = cx.new(|cx| Live::new(host, db, home, changes, cx));
        window::open(live, cx);
    });
    ExitCode::SUCCESS
}
