//! Starting a session: the slow part (asking the shell for `claude`, its
//! version, taking the lock) on a thread of its own, and the terminal once
//! that is done.

use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::launch::{
    ClaudeProjectsTranscriptLocator, CommandLineConversations, FlockSessionLocks, HookBinary,
    LaunchEnvironment, LaunchError, LaunchNotices, LaunchPorts, LoginShellProbe, PreparedLaunch,
    installed_claude_version, prepare_launch,
};
use lithemux_core::modules::session::{SessionRef, SqliteSessionRepository};
use lithemux_core::modules::settings::SqliteSettingsRepository;
use lithemux_core::platform::process::session_markers::without_session_markers;
use lithemux_core::platform::store::connection::Db;

use super::config::HostConfig;
use super::error::HostError;
use super::hosted::Hosted;
use super::mailbox::{Mailbox, Signal};
use crate::terminal_view::{GridSize, ShellCommand, TerminalSession};

struct FixedHookBinary(PathBuf);

impl HookBinary for FixedHookBinary {
    fn path(&self) -> Result<PathBuf, LaunchError> {
        Ok(self.0.clone())
    }
}

#[derive(Default)]
struct CollectedNotices(Mutex<Vec<String>>);

impl LaunchNotices for CollectedNotices {
    fn warn(&self, message: &str) {
        if let Ok(mut warnings) = self.0.lock() {
            warnings.push(message.to_string());
        }
    }
}

/// The lock of a running session. The thread that waits for the program
/// empties it, so the session is free the moment nothing of it is left,
/// whether or not the host is paying attention.
type HeldLaunch = Arc<Mutex<Option<PreparedLaunch>>>;

/// Prepares the launch away from the interface thread and posts the result.
/// It works on a database connection of its own.
pub(super) fn prepare_in_background(
    config: HostConfig,
    target: SessionRef,
    session_id: String,
    run: u64,
    mailbox: Mailbox,
) {
    thread::spawn(move || {
        let notices = CollectedNotices::default();
        let outcome = Box::new(prepare(&config, &target, &notices));
        mailbox.post(Signal::Prepared {
            session_id,
            run,
            outcome,
            warnings: notices.0.into_inner().unwrap_or_default(),
        });
    });
}

fn prepare(
    config: &HostConfig,
    target: &SessionRef,
    notices: &CollectedNotices,
) -> Result<PreparedLaunch, LaunchError> {
    let db = Db::open(&config.home).map_err(lithemux_core::modules::session::SessionError::from)?;
    let transcripts = match &config.claude_config_dir {
        Some(dir) => ClaudeProjectsTranscriptLocator::in_config_dir(dir),
        None => ClaudeProjectsTranscriptLocator::from_env(),
    };
    let ports = LaunchPorts {
        categories: &SqliteCategoryRepository::new(&db),
        sessions: &SqliteSessionRepository::new(&db),
        settings: &SqliteSettingsRepository::new(&db),
        shell: &LoginShellProbe::new(&config.shell),
        notices,
        hook_binary: &FixedHookBinary(config.hook_binary.clone()),
        transcripts: &transcripts,
        locks: &FlockSessionLocks::new(&config.home),
        conversations: &CommandLineConversations,
    };
    let environment = LaunchEnvironment {
        shell: config.shell.clone(),
        lithemux_home: config.lithemux_home.clone(),
    };
    prepare_launch(&ports, &environment, target)
}

/// The version of the `claude` a launch would use now, asked the way a
/// launch asks, on a database connection of its own.
pub(super) fn installed_version(config: &HostConfig) -> Option<String> {
    let db = Db::open(&config.home).ok()?;
    let settings = SqliteSettingsRepository::new(&db);
    installed_claude_version(&settings, &LoginShellProbe::new(&config.shell))
}

/// What the program gets from the environment the app was started in:
/// everything but the marks of the session the app itself may run in.
fn inherited(environment: &[(OsString, OsString)]) -> Vec<(String, String)> {
    without_session_markers(environment.iter().cloned())
        .into_iter()
        .filter_map(|(name, value)| Some((name.into_string().ok()?, value.into_string().ok()?)))
        .collect()
}

/// Runs the prepared launch in a terminal. The lock goes to the thread that
/// waits for the program, which releases it and tells the host.
pub(super) fn start_terminal(
    config: &HostConfig,
    launch: PreparedLaunch,
    run: u64,
    size: GridSize,
    mailbox: Mailbox,
) -> Result<Hosted, HostError> {
    let plan = launch.plan().clone();
    if !plan.cwd.is_dir() {
        return Err(LaunchError::MissingCwd(plan.cwd).into());
    }
    let session_id = launch.session().id.clone();
    let profile = launch.session().profile;
    let mut command = ShellCommand::new(&plan.program);
    command.args = plan.args;
    command.inherit_env = false;
    command.env = inherited(&config.environment);
    command.env.extend(plan.env);
    command.cwd = Some(plan.cwd);

    let held: HeldLaunch = Arc::new(Mutex::new(Some(launch)));
    let on_exit = {
        let (held, session_id) = (Arc::clone(&held), session_id.clone());
        Box::new(move || {
            if let Ok(mut launch) = held.lock() {
                launch.take();
            }
            mailbox.post(Signal::Exited { session_id, run });
        })
    };
    let (terminal, wakes) = TerminalSession::spawn_observed(&command, size, on_exit)?;
    if let (Some(pid), Ok(mut launch)) = (terminal.process_id(), held.lock())
        && let Some(launch) = launch.as_mut()
    {
        launch.started(pid);
    }
    Ok(Hosted {
        session_id,
        run,
        profile,
        terminal: Rc::new(terminal),
        wakes,
    })
}
