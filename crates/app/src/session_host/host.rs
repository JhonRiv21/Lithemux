//! Owns the terminals of the sessions the app has open. No gpui here: a
//! view shows a session's terminal, and the host keeps it alive, hidden or
//! not, and knows what state it is in.
//!
//! Nothing here waits: opening a session is prepared in the background and
//! closing one ends its program without watching it go. Only `shut_down`,
//! on the way out of the app, stays until the programs are gone.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;

use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::launch::LaunchError;
use lithemux_core::modules::session::{
    Profile, SessionRef, SqliteSessionRepository, State, apply_state, find_session_by_id,
    get_session,
};
use lithemux_core::platform::store::connection::Db;

use super::config::HostConfig;
use super::error::HostError;
use super::event_socket::EventSocket;
use super::hosted::Hosted;
use super::instance_lock::InstanceLock;
use super::mailbox::{Mailbox, Signal};
use super::opening::{prepare_in_background, start_terminal};
use super::phase::Phase;
use crate::terminal_view::{GridSize, TerminalSession};

/// What became of a request to open a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// It is being prepared; `phase` will say how it went.
    Opening { session_id: String },
    /// It was already running, or being opened, here.
    AlreadyHere { session_id: String },
}

/// A session waiting for its launch to be prepared.
pub(super) struct Pending {
    run: u64,
    size: GridSize,
}

pub struct SessionHost {
    config: HostConfig,
    db: Db,
    pub(super) hosted: Vec<Hosted>,
    pub(super) pending: HashMap<String, Pending>,
    pub(super) phases: HashMap<String, Phase>,
    /// The last state read from the database for every session heard of.
    states: HashMap<String, State>,
    warnings: Vec<String>,
    next_run: u64,
    inbox: mpsc::Receiver<Signal>,
    mailbox: Mailbox,
    /// `None` once the host has shut down: it no longer listens.
    pub(super) socket: Option<EventSocket>,
    // Declared last: released after everything above is gone.
    _instance: InstanceLock,
}

/// A problem that does not stop the host. The rotated log comes later
/// (PLAN §3.6); until then it goes to stderr.
fn report(what: &str, session_id: &str, error: &dyn std::fmt::Display) {
    eprintln!("lithemux-app: {what} {session_id}: {error}");
}

impl SessionHost {
    /// Fails when another app already runs on this home. The receiver gets a
    /// message whenever `process_pending` has something to do.
    pub fn start(config: HostConfig) -> Result<(Self, async_channel::Receiver<()>), HostError> {
        let db = Db::open(&config.home)?;
        let instance = InstanceLock::acquire(&config.home.app_lock_path())?;
        let (mailbox, inbox, changes) = Mailbox::new();
        let socket = EventSocket::bind(&config.home.app_socket_path(), mailbox.clone())?;
        let host = SessionHost {
            config,
            db,
            hosted: Vec::new(),
            pending: HashMap::new(),
            phases: HashMap::new(),
            states: HashMap::new(),
            warnings: Vec::new(),
            next_run: 0,
            inbox,
            mailbox,
            socket: Some(socket),
            _instance: instance,
        };
        Ok((host, changes))
    }

    /// Asks for the session to run in a terminal of its own. It returns at
    /// once: the session is `Opening` until its launch is prepared, and
    /// `phase` tells what it turned into.
    pub fn open(&mut self, target: &SessionRef, size: GridSize) -> Result<Opened, HostError> {
        let categories = SqliteCategoryRepository::new(&self.db);
        let sessions = SqliteSessionRepository::new(&self.db);
        let (_, session) = get_session(&categories, &sessions, target)?;
        let session_id = session.id;
        if matches!(
            self.phases.get(&session_id),
            Some(Phase::Opening | Phase::Running)
        ) {
            return Ok(Opened::AlreadyHere { session_id });
        }
        self.next_run += 1;
        let run = self.next_run;
        self.pending
            .insert(session_id.clone(), Pending { run, size });
        self.phases.insert(session_id.clone(), Phase::Opening);
        self.refresh(&session_id);
        prepare_in_background(
            self.config.clone(),
            target.clone(),
            session_id.clone(),
            run,
            self.mailbox.clone(),
        );
        Ok(Opened::Opening { session_id })
    }

    /// Reads the mail of the background threads: launches that are ready,
    /// hooks that reported a change and programs that ended. Returns whether
    /// anything was there.
    pub fn process_pending(&mut self) -> bool {
        let signals: Vec<Signal> = self.inbox.try_iter().collect();
        let any = !signals.is_empty();
        for signal in signals {
            match signal {
                Signal::Hook { session_id } => self.refresh(&session_id),
                Signal::Prepared {
                    session_id,
                    run,
                    outcome,
                    warnings,
                } => {
                    self.warnings.extend(warnings);
                    self.launch_prepared(&session_id, run, *outcome);
                }
                Signal::Exited { session_id, run } => self.program_ended(&session_id, run),
            }
        }
        any
    }

    fn launch_prepared(
        &mut self,
        session_id: &str,
        run: u64,
        outcome: Result<lithemux_core::modules::launch::PreparedLaunch, LaunchError>,
    ) {
        // Closed, or asked for again, while it was being prepared: the
        // launch is dropped here, and its lock with it.
        let Some(pending) = self.pending.remove(session_id).filter(|p| p.run == run) else {
            return;
        };
        let started = outcome.map_err(HostError::from).and_then(|launch| {
            start_terminal(
                &self.config,
                launch,
                run,
                pending.size,
                self.mailbox.clone(),
            )
        });
        let phase = match started {
            Ok(hosted) => {
                // An earlier run that ended leaves its place to this one.
                self.hosted.retain(|hosted| hosted.session_id != session_id);
                self.hosted.push(hosted);
                Phase::Running
            }
            Err(HostError::Launch(LaunchError::AlreadyOpen { pid })) => {
                Phase::OpenElsewhere { pid }
            }
            Err(error) => {
                report("could not start", session_id, &error);
                Phase::Failed(error.to_string())
            }
        };
        self.phases.insert(session_id.to_string(), phase);
        self.refresh(session_id);
    }

    /// The program is gone and the session is already free for anyone to
    /// open again. Its screen stays.
    pub(super) fn program_ended(&mut self, session_id: &str, run: u64) {
        // An earlier run of a session that was opened again is old news.
        let Some(profile) = self
            .hosted
            .iter()
            .find(|hosted| hosted.session_id == session_id && hosted.run == run)
            .map(|hosted| hosted.profile)
        else {
            return;
        };
        self.phases.insert(session_id.to_string(), Phase::Ended);
        self.mark_ended(session_id, profile);
        self.refresh(session_id);
    }

    /// Normally the SessionEnd hook already said so; this covers a `claude`
    /// that was killed. A deliberate hibernation stays.
    pub(super) fn mark_ended(&self, session_id: &str, profile: Profile) {
        if profile != Profile::Claude {
            return;
        }
        let sessions = SqliteSessionRepository::new(&self.db);
        let ended = apply_state(
            &sessions,
            session_id,
            State::Ended,
            None,
            Some(State::Hibernated),
        );
        if let Err(error) = ended {
            report("could not record the end of", session_id, &error);
        }
    }

    /// The database is the source of truth; a session it does not know is
    /// forgotten here too.
    pub(super) fn refresh(&mut self, session_id: &str) {
        let categories = SqliteCategoryRepository::new(&self.db);
        let sessions = SqliteSessionRepository::new(&self.db);
        match find_session_by_id(&categories, &sessions, session_id) {
            Ok(Some((_, session))) => {
                self.states.insert(session_id.to_string(), session.state);
            }
            Ok(None) => {
                self.states.remove(session_id);
            }
            Err(error) => report("could not read the state of", session_id, &error),
        }
    }

    /// The state the database had for the session the last time the host
    /// heard about it.
    pub fn state(&self, session_id: &str) -> Option<State> {
        self.states.get(session_id).copied()
    }

    /// Where a session the host was asked to open stands; `None` for one it
    /// was never asked for, or that was closed.
    pub fn phase(&self, session_id: &str) -> Option<&Phase> {
        self.phases.get(session_id)
    }

    pub fn is_running(&self, session_id: &str) -> bool {
        self.phase(session_id) == Some(&Phase::Running)
    }

    /// The terminal of a session opened here, running or ended, and the
    /// channel that says when it has something new to paint.
    pub fn terminal(
        &self,
        session_id: &str,
    ) -> Option<(Rc<TerminalSession>, async_channel::Receiver<()>)> {
        self.hosted
            .iter()
            .find(|hosted| hosted.session_id == session_id)
            .map(|hosted| (Rc::clone(&hosted.terminal), hosted.wakes.clone()))
    }

    /// What the launcher had to say while opening sessions.
    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }
}
