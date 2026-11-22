//! Temporary, for the §2.1 block: the sessions asked for with `--open`,
//! kept by the session host. With or without a window, they are closed in
//! order when the app quits.

use std::process::ExitCode;

use gpui::{App, AppContext, Context, Entity};
use lithemux_app::launch_arguments::OpenRequest;
use lithemux_app::session_host::{HostConfig, Opened, Phase, SessionHost};
use lithemux_app::terminal_view::GridSize;
use lithemux_core::modules::session::{SessionRef, State};

use crate::hosted_window;
use crate::quit_signals::quit_on_signals;

/// One `--open`: the session it turned out to be, or why it is none.
pub struct Asked {
    pub target: String,
    pub session: Result<String, String>,
}

pub struct HostedSessions {
    pub host: SessionHost,
    pub asked: Vec<Asked>,
}

impl HostedSessions {
    fn new(
        mut host: SessionHost,
        changes: async_channel::Receiver<()>,
        targets: &[String],
        cx: &mut Context<Self>,
    ) -> Self {
        let asked = targets
            .iter()
            .map(|target| Asked {
                target: target.clone(),
                session: ask(&mut host, target),
            })
            .collect();
        // The host's mail: launches that are ready, hooks, programs that ended.
        cx.spawn(async move |sessions, cx| {
            while changes.recv().await.is_ok() {
                let attended = sessions.update(cx, |sessions, cx| {
                    sessions.host.process_pending();
                    for warning in sessions.host.take_warnings() {
                        eprintln!("lithemux-app: {warning}");
                    }
                    cx.notify();
                });
                if attended.is_err() {
                    return;
                }
            }
        })
        .detach();
        // cmd+q and the quit event of macOS end here: the programs are ended,
        // and waited for, before the app is gone. Signals do the same on
        // their own (`quit_signals`).
        cx.on_app_quit(|sessions, _| {
            sessions.host.shut_down();
            async {}
        })
        .detach();
        Self { host, asked }
    }

    /// What to show next to a session: where it stands and its stored state.
    pub fn status(&self, asked: &Asked) -> String {
        let target = &asked.target;
        let session_id = match &asked.session {
            Ok(session_id) => session_id,
            Err(problem) => return format!("{target} · {problem}"),
        };
        let state = self.host.state(session_id).map_or("unknown", State::as_str);
        match self.host.phase(session_id) {
            Some(Phase::Opening) => format!("{target} · opening…"),
            Some(Phase::Running) => format!("{target} · {state} · running"),
            Some(Phase::Ended) => format!("{target} · {state} · not running"),
            Some(Phase::OpenElsewhere { pid: Some(pid) }) => {
                format!("{target} · open in another terminal (pid {pid})")
            }
            Some(Phase::OpenElsewhere { pid: None }) => {
                format!("{target} · open in another terminal")
            }
            Some(Phase::Failed(problem)) => format!("{target} · {problem}"),
            None => format!("{target} · closed"),
        }
    }
}

fn ask(host: &mut SessionHost, target: &str) -> Result<String, String> {
    let session = SessionRef::parse(target).map_err(|error| error.to_string())?;
    match host.open(&session, GridSize::INITIAL) {
        Ok(Opened::Opening { session_id } | Opened::AlreadyHere { session_id }) => Ok(session_id),
        Err(error) => Err(error.to_string()),
    }
}

pub fn run(request: OpenRequest) -> ExitCode {
    // Before anything shows: a second instance says why it leaves and leaves.
    let started = HostConfig::from_environment().and_then(SessionHost::start);
    let (host, changes) = match started {
        Ok(started) => started,
        Err(error) => {
            eprintln!("lithemux-app: {error}");
            return ExitCode::FAILURE;
        }
    };
    let application = if request.with_window {
        gpui_platform::application()
    } else {
        gpui_platform::headless()
    };
    application.run(move |cx: &mut App| {
        let sessions: Entity<HostedSessions> =
            cx.new(|cx| HostedSessions::new(host, changes, &request.targets, cx));
        quit_on_signals(&sessions, cx);
        if request.with_window {
            hosted_window::open(sessions, cx);
        } else {
            // Nothing else holds it: the app keeps the sessions for its life.
            std::mem::forget(sessions);
        }
    });
    ExitCode::SUCCESS
}
