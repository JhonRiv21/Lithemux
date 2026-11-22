//! Owns the terminals of the sessions the app has open. No gpui here: a
//! view shows a session's terminal, and the host keeps it alive, hidden or
//! not, and knows what state it is in.
//!
//! Nothing here waits: opening a session is prepared in the background and
//! closing one ends its program without watching it go. Only `shut_down`,
//! on the way out of the app, stays until the programs are gone.

use std::collections::HashMap;
use std::sync::mpsc;

use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::launch::{FlockSessionLocks, LaunchError, SessionLocks};
use lithemux_core::modules::session::{
    Profile, SessionRef, SqliteSessionRepository, State, apply_state, find_session_by_id,
    get_session, hibernate_what_was_left_running,
};
use lithemux_core::platform::paths::home::current_user;
use lithemux_core::platform::store::connection::Db;

use super::config::HostConfig;
use super::error::HostError;
use super::event_socket::EventSocket;
use super::failure::Failure;
use super::hosted::Hosted;
use super::instance_lock::InstanceLock;
use super::leaving::Leaving;
use super::mailbox::{Mailbox, Signal};
use super::opened::Opened;
use super::opening::{prepare_in_background, start_terminal};
use super::phase::Phase;
use crate::terminal_view::GridSize;

/// A session waiting for its launch to be prepared.
pub(super) struct Pending {
    pub(super) run: u64,
    size: GridSize,
}

pub struct SessionHost {
    pub(super) config: HostConfig,
    pub(super) db: Db,
    pub(super) hosted: Vec<Hosted>,
    pub(super) pending: HashMap<String, Pending>,
    /// What still holds the lock of sessions that were closed.
    pub(super) leaving: Leaving,
    pub(super) phases: HashMap<String, Phase>,
    /// The last state read from the database for every session heard of.
    pub(super) states: HashMap<String, State>,
    pub(super) warnings: Vec<String>,
    /// When the memory valve last hibernated a session.
    pub(super) valve_last_used_at: Option<i64>,
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
        // Nothing of this app runs yet: what the database still has as
        // running was cut short, and is hibernated (PLAN §2.5).
        let locks = FlockSessionLocks::new(&config.home);
        hibernate_what_was_left_running(&SqliteSessionRepository::new(&db), |id| {
            // The pid is read first: taking the lock writes ours over it.
            // One that is alive is an orphan of an app that died, or a
            // terminal: either way the session is not ours to settle. And
            // it is the lock, not the pid, that says whether it is free.
            let somebody_runs_it = locks.holder(id).is_some();
            let lock = locks.acquire(id).ok()?;
            (!somebody_runs_it).then_some(lock)
        })?;
        let (mailbox, inbox, changes) = Mailbox::new();
        let address = config.home.app_socket();
        address
            .prepare(current_user())
            .map_err(HostError::io("preparing the directory of", address.path()))?;
        let socket = EventSocket::bind(address.path(), mailbox.clone())?;
        let host = SessionHost {
            config,
            db,
            hosted: Vec::new(),
            pending: HashMap::new(),
            leaving: Leaving::default(),
            phases: HashMap::new(),
            states: HashMap::new(),
            warnings: Vec::new(),
            valve_last_used_at: None,
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
        // Our own last run still holds the session's lock: asking for it
        // now would find it taken, by us.
        if self.leaving.holds(&session_id) {
            self.leaving.reopen(&session_id, target.clone(), size);
            self.phases.insert(session_id.clone(), Phase::Closing);
            return Ok(Opened::WhenClosed { session_id });
        }
        self.start_opening(target, &session_id, size);
        Ok(Opened::Opening { session_id })
    }

    fn start_opening(&mut self, target: &SessionRef, session_id: &str, size: GridSize) {
        let session_id = session_id.to_string();
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
    }

    /// Something of a closed session finished leaving. Once nothing of it
    /// is left, the session opens again if it was asked for, or is simply
    /// closed.
    pub(super) fn settle(&mut self, session_id: &str) {
        if self.leaving.holds(session_id) {
            return;
        }
        match self.leaving.take_reopen(session_id) {
            Some((target, size)) => self.start_opening(&target, session_id, size),
            None if self.phases.get(session_id) == Some(&Phase::Closing) => {
                self.phases.remove(session_id);
            }
            None => {}
        }
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
        if self.pending.get(session_id).is_none_or(|p| p.run != run) {
            // The launch, and its lock, end here.
            drop(outcome);
            self.leaving.launch_dropped(session_id);
            self.settle(session_id);
            return;
        }
        let Some(pending) = self.pending.remove(session_id) else {
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
                Phase::Failed(Failure::from(&error))
            }
        };
        self.phases.insert(session_id.to_string(), phase);
        self.refresh(session_id);
    }

    /// The program is gone and the session is already free for anyone to
    /// open again. Its screen stays.
    pub(super) fn program_ended(&mut self, session_id: &str, run: u64) {
        if self.leaving.program_left(session_id, run) {
            self.settle(session_id);
            return;
        }
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
}
