//! What the sidebar and the workspace share: the session host, the board
//! of categories and sessions as the database has it, and the listening
//! ports of what runs.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui::{AppContext, Context, Task};
use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::{Session, SessionRef, SqliteSessionRepository, mark_read};
use lithemux_core::modules::settings::{SettingKey, SqliteSettingsRepository, read_setting};
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::process::listening_ports::listening_ports;
use lithemux_core::platform::store::connection::Db;

use crate::i18n::{Language, Texts};
use crate::session_host::{HostError, Phase, SessionHost};
use crate::sidebar::actions::SidebarActions;
use crate::sidebar::error::SidebarError;
use crate::sidebar::tree::{Board, Tree, build};
use crate::terminal_view::GridSize;

/// How often the board is read again and the ports looked for. Changes
/// made from the CLI, in another terminal, show up at this pace.
const REFRESH_EVERY: Duration = Duration::from_secs(3);

/// The ports found for each pid, by the session the pid belongs to.
fn ports_by_session(
    running: Vec<(String, u32)>,
    found: &HashMap<u32, Vec<u16>>,
) -> HashMap<String, Vec<u16>> {
    running
        .into_iter()
        .filter_map(|(id, pid)| Some((id, found.get(&pid)?.clone())))
        .filter(|(_, ports)| !ports.is_empty())
        .collect()
}

pub struct Live {
    pub host: SessionHost,
    pub texts: Rc<Texts>,
    db: Db,
    home: Home,
    user_home: Option<PathBuf>,
    categories: Vec<Category>,
    sessions: Vec<Session>,
    ports: HashMap<String, Vec<u16>>,
    /// The last thing that went wrong, for the sidebar to show.
    pub notice: Option<String>,
    _refresh: Task<()>,
}

impl Live {
    pub fn new(
        host: SessionHost,
        db: Db,
        home: Home,
        changes: async_channel::Receiver<()>,
        cx: &mut Context<Self>,
    ) -> Self {
        let language = read_setting(&SqliteSettingsRepository::new(&db), SettingKey::Language)
            .ok()
            .flatten();
        Self::attend_the_host(changes, cx);
        // cmd+q and the quit event of macOS end here: the programs are
        // ended, and waited for, before the app is gone. Signals do the
        // same on their own (`quit_signals`).
        cx.on_app_quit(|live, _| {
            live.host.shut_down();
            async {}
        })
        .detach();
        let refresh = Self::keep_refreshing(cx);
        let mut live = Live {
            host,
            texts: Rc::new(Texts::of(Language::from_setting(language.as_deref()))),
            db,
            user_home: std::env::var_os("HOME").map(PathBuf::from),
            home,
            categories: Vec::new(),
            sessions: Vec::new(),
            ports: HashMap::new(),
            notice: None,
            _refresh: refresh,
        };
        live.reload();
        live
    }

    /// The host's mail: launches that are ready, hooks, programs that ended.
    fn attend_the_host(changes: async_channel::Receiver<()>, cx: &mut Context<Self>) {
        cx.spawn(async move |live, cx| {
            while changes.recv().await.is_ok() {
                let attended = live.update(cx, |live, cx| {
                    live.host.process_pending();
                    for warning in live.host.take_warnings() {
                        eprintln!("lithemux-app: {warning}");
                    }
                    live.reload();
                    cx.notify();
                });
                if attended.is_err() {
                    return;
                }
            }
        })
        .detach();
    }

    /// Every while: the board again, for what changed outside the app, and
    /// the ports of what runs, looked for away from the interface thread.
    fn keep_refreshing(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |live, cx| {
            loop {
                cx.background_executor().timer(REFRESH_EVERY).await;
                let Ok(running) = live.update(cx, |live, cx| {
                    if live.reload() {
                        cx.notify();
                    }
                    live.host.running_programs()
                }) else {
                    return;
                };
                let pids: Vec<u32> = running.iter().map(|(_, pid)| *pid).collect();
                let found = cx
                    .background_spawn(async move { listening_ports(&pids) })
                    .await;
                let ports = ports_by_session(running, &found);
                let updated = live.update(cx, |live, cx| {
                    if live.ports != ports {
                        live.ports = ports;
                        cx.notify();
                    }
                });
                if updated.is_err() {
                    return;
                }
            }
        })
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    fn actions(&self) -> SidebarActions<'_> {
        SidebarActions {
            db: &self.db,
            home: &self.home,
        }
    }

    /// Reads the board again. Returns whether it changed.
    pub fn reload(&mut self) -> bool {
        // What was removed from another terminal stops running here.
        let closed = self.host.close_removed();
        match self.actions().load() {
            Ok((categories, sessions)) => {
                let changed = closed || categories != self.categories || sessions != self.sessions;
                self.categories = categories;
                self.sessions = sessions;
                changed
            }
            Err(error) => {
                self.notice = Some(self.texts.say(&(&error).into()));
                true
            }
        }
    }

    /// Runs a change of the sidebar; what goes wrong becomes the notice.
    pub fn perform<T>(
        &mut self,
        change: impl FnOnce(&SidebarActions<'_>) -> Result<T, SidebarError>,
        cx: &mut Context<Self>,
    ) -> Option<T> {
        let outcome = change(&self.actions());
        self.notice = outcome
            .as_ref()
            .err()
            .map(|error| self.texts.say(&error.into()));
        self.reload();
        cx.notify();
        outcome.ok()
    }

    pub fn tree(&self) -> Tree {
        build(&Board {
            categories: &self.categories,
            sessions: &self.sessions,
            phases: &|id| self.host.phase(id).cloned(),
            ports: &self.ports,
            home: self.user_home.as_deref(),
        })
    }

    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    pub fn session(&self, id: &str) -> Option<&Session> {
        self.sessions.iter().find(|session| session.id == id)
    }

    pub fn session_exists(&self, id: &str) -> bool {
        self.session(id).is_some()
    }

    /// `<category>/<session>` of a session of the board.
    pub fn address_of(&self, id: &str) -> Option<String> {
        let session = self.session(id)?;
        let category = self
            .categories
            .iter()
            .find(|category| category.id == session.category_id)?;
        Some(format!("{}/{}", category.slug, session.slug))
    }

    /// Starts the session's program unless the app already has it.
    pub fn open(&mut self, session_id: &str, cx: &mut Context<Self>) {
        let Some(address) = self.address_of(session_id) else {
            return;
        };
        let opened = SessionRef::parse(&address)
            .map_err(HostError::from)
            .and_then(|target| self.host.open(&target, GridSize::INITIAL));
        self.notice = opened.err().map(|error| self.texts.say(&(&error).into()));
        cx.notify();
    }

    /// Whether the app has a program of the session, or is starting one.
    pub fn is_live(&self, session_id: &str) -> bool {
        matches!(
            self.host.phase(session_id),
            Some(Phase::Opening | Phase::Running | Phase::Closing)
        )
    }

    pub fn close(&mut self, session_id: &str, cx: &mut Context<Self>) {
        self.host.close(session_id);
        self.reload();
        cx.notify();
    }

    /// The user is looking at the session.
    pub fn mark_read(&mut self, session_id: &str, cx: &mut Context<Self>) {
        if self.session(session_id).is_none_or(|s| s.unread == 0) {
            return;
        }
        if let Err(error) = mark_read(&SqliteSessionRepository::new(&self.db), session_id) {
            self.notice = Some(self.texts.say(&(&error).into()));
        }
        self.reload();
        cx.notify();
    }
}
