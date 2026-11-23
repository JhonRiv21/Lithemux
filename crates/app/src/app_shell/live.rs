//! What the sidebar and the workspace share: the session host, the board
//! of categories and sessions as the database has it, and the listening
//! ports of what runs.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui::{App, Context, Task};
use lithemux_core::modules::category::Category;
use lithemux_core::modules::launch::Update;
use lithemux_core::modules::session::{Session, SessionRef, SqliteSessionRepository, mark_read};
use lithemux_core::modules::settings::Preferences;
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::process::liveness::is_alive;
use lithemux_core::platform::store::connection::Db;

use super::elsewhere::shown_phase;
use crate::i18n::{Language, Texts};
use crate::notifications::policy::SoundGate;
use crate::session_host::{HostError, Phase, SessionHost};
use crate::sidebar::actions::SidebarActions;
use crate::sidebar::error::SidebarError;
use crate::sidebar::tree::{Board, Tree, build};
use crate::terminal_view::GridSize;

/// How often the board is read again and the ports looked for. Changes
/// made from the CLI, in another terminal, show up at this pace.
pub(super) const REFRESH_EVERY: Duration = Duration::from_secs(3);

pub struct Live {
    pub host: SessionHost,
    pub texts: Rc<Texts>,
    /// The settings, as last read.
    pub(super) preferences: Preferences,
    /// The font last asked for and the one in use for it.
    pub(super) checked_font: Option<(String, String)>,
    /// The sessions the user is looking at right now.
    pub(crate) in_view: Vec<String>,
    /// What `claude --version` said last, for the one a launch would use.
    pub(super) installed_claude: Option<String>,
    pub(super) update: Update,
    pub(crate) sound_gate: SoundGate,
    _version_checks: Task<()>,
    pub(super) db: Db,
    home: Home,
    user_home: Option<PathBuf>,
    categories: Vec<Category>,
    sessions: Vec<Session>,
    pub(super) ports: HashMap<String, Vec<u16>>,
    /// The memory of each session that runs here (PLAN §3.3).
    pub(crate) memory: HashMap<String, u64>,
    /// The sessions whose conversation another process runs, with it.
    pub(super) elsewhere: HashMap<String, u32>,
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
            texts: Rc::new(Texts::of(Language::English)),
            preferences: Preferences::default(),
            checked_font: None,
            in_view: Vec::new(),
            installed_claude: None,
            update: Update::default(),
            sound_gate: SoundGate::default(),
            _version_checks: Self::keep_checking_the_version(cx),
            db,
            user_home: std::env::var_os("HOME").map(PathBuf::from),
            home,
            categories: Vec::new(),
            sessions: Vec::new(),
            ports: HashMap::new(),
            memory: HashMap::new(),
            elsewhere: HashMap::new(),
            notice: None,
            _refresh: refresh,
        };
        live.reload(cx);
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
                    live.reload(cx);
                    live.announce_the_news(cx);
                    cx.notify();
                });
                if attended.is_err() {
                    return;
                }
            }
        })
        .detach();
    }

    /// Where lithemux keeps its files.
    pub fn home_folder(&self) -> PathBuf {
        self.home.path().to_path_buf()
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn home(&self) -> &Home {
        &self.home
    }

    fn actions(&self) -> SidebarActions<'_> {
        SidebarActions {
            db: &self.db,
            home: &self.home,
        }
    }

    /// Reads the board and the settings again. Returns whether anything
    /// changed.
    pub fn reload(&mut self, cx: &App) -> bool {
        let settings_changed = self.follow_the_settings(cx);
        // What was removed from another terminal stops running here.
        let closed = self.host.close_removed() || settings_changed;
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
        self.reload(cx);
        cx.notify();
        outcome.ok()
    }

    pub fn tree(&self) -> Tree {
        build(&Board {
            categories: &self.categories,
            sessions: &self.sessions,
            phases: &|id| self.shown_phase(id),
            ports: &self.ports,
            memory: &self.memory,
            home: self.user_home.as_deref(),
        })
    }

    /// Where a session stands for the user: what the host has of it, or
    /// that another process runs it.
    pub fn shown_phase(&self, session_id: &str) -> Option<Phase> {
        let host = self.host.phase(session_id);
        let holder_runs = match host {
            Some(Phase::OpenElsewhere { pid: Some(pid) }) => is_alive(*pid),
            _ => false,
        };
        shown_phase(host, self.elsewhere.get(session_id).copied(), holder_runs)
    }

    pub fn sessions(&self) -> &[Session] {
        &self.sessions
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
        self.reload(cx);
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
        self.reload(cx);
        cx.notify();
    }
}
