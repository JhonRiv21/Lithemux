//! Owns the terminals of the sessions the app has open. No gpui here: a
//! view shows a session's terminal, and the host keeps it alive, hidden or
//! not, and knows what state it is in.
//!
//! Nothing here waits: opening a session is prepared in the background and
//! closing one ends its program without watching it go. Only `shut_down`,
//! on the way out of the app, stays until the programs are gone.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc;

use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::session::{SessionRef, SqliteSessionRepository, State, get_session};
use lithemux_core::platform::log::app_log::AppLog;
use lithemux_core::platform::paths::home::current_user;
use lithemux_core::platform::store::connection::Db;

use super::config::HostConfig;
use super::error::HostError;
use super::event_socket::{EventSocket, HookNews};
use super::hosted::Hosted;
use super::instance_lock::InstanceLock;
use super::leaving::Leaving;
use super::mailbox::{Mailbox, Signal};
use super::opened::Opened;
use super::opening::prepare_in_background;
use super::phase::Phase;
use super::startup::{open_the_database, settle_what_was_left_running};
use crate::terminal_view::GridSize;

/// News of the hooks that nobody took yet: the app without a window takes
/// none, and the newest matter most.
const MAX_NEWS: usize = 64;

/// A session waiting for its launch to be prepared.
pub(super) struct Pending {
    pub(super) run: u64,
    pub(super) size: GridSize,
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
    /// `<home>/logs/app.log` (PLAN §3.6).
    pub(super) log: Arc<AppLog>,
    /// What the hooks said since it was last asked.
    pub(super) news: Vec<HookNews>,
    /// When the memory valve last hibernated a session.
    pub(super) valve_last_used_at: Option<i64>,
    next_run: u64,
    inbox: mpsc::Receiver<Signal>,
    pub(super) mailbox: Mailbox,
    /// `None` once the host has shut down: it no longer listens.
    pub(super) socket: Option<EventSocket>,
    // Declared last: released after everything above is gone.
    _instance: InstanceLock,
}

impl SessionHost {
    /// Fails when another app already runs on this home. The receiver gets a
    /// message whenever `process_pending` has something to do.
    pub fn start(config: HostConfig) -> Result<(Self, async_channel::Receiver<()>), HostError> {
        let log = AppLog::shared(config.home.app_log_path());
        let db = open_the_database(&config, &log)?;
        let instance = InstanceLock::acquire(&config.home.app_lock_path())?;
        settle_what_was_left_running(&config, &db)?;
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
            log,
            news: Vec::new(),
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
                Signal::Hook(news) => {
                    self.refresh(&news.session_id);
                    self.keep_the_news(news);
                }
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

    fn keep_the_news(&mut self, news: HookNews) {
        if self.news.len() >= MAX_NEWS {
            self.news.remove(0);
        }
        self.news.push(news);
    }
}
