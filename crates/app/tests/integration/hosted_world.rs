use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lithemux_app::session_host::{HostConfig, Phase, SessionHost};
use lithemux_core::modules::category::{
    FsBoardStore, NewCategory, Slug, SqliteCategoryRepository, create_category,
};
use lithemux_core::modules::session::{
    Kind, NewSession, Profile, Session, SessionRef, SqliteSessionRepository, State, apply_state,
    create_session, find_session_by_id, get_session, remove_session, set_keep_alive,
};
use lithemux_core::modules::settings::{SettingKey, SqliteSettingsRepository, write_setting};
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;
use tempfile::TempDir;

use crate::support::SIZE;

const PATIENCE: Duration = Duration::from_secs(10);
const NOW: i64 = 1_800_000_000;

/// The fakes of `core`: a `claude`, a login shell and a hook that only exist.
pub fn core_fake(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/fakes")
        .join(name)
        .canonicalize()
        .unwrap()
}

/// A machine for the host: its own home with the category `demo`, a fake
/// `claude` fixed with `claude_path`, a fake shell and a fake hook.
pub struct HostedWorld {
    pub dir: TempDir,
    pub home: Home,
}

impl HostedWorld {
    pub fn new() -> Self {
        Self::with_home_named("home")
    }

    /// A home whose path is too long for the address of a Unix socket.
    pub fn with_a_long_home() -> Self {
        Self::with_home_named(&"a-long-folder-name/".repeat(6))
    }

    fn with_home_named(name: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::new(dir.path().canonicalize().unwrap().join(name));
        let world = HostedWorld { dir, home };
        fs::create_dir_all(world.work()).unwrap();
        let db = world.db();
        create_category(
            &SqliteCategoryRepository::new(&db),
            &FsBoardStore::new(&world.home),
            NewCategory {
                slug: Slug::parse("demo").unwrap(),
                name: "Demo".to_string(),
                root_dir: world.work().display().to_string(),
                color: None,
            },
            NOW,
        )
        .unwrap();
        let fake_claude = core_fake("fake-claude").display().to_string();
        write_setting(
            &SqliteSettingsRepository::new(&db),
            SettingKey::ClaudePath,
            &fake_claude,
        )
        .unwrap();
        world
    }

    pub fn db(&self) -> Db {
        Db::open(&self.home).unwrap()
    }

    pub fn work(&self) -> PathBuf {
        self.dir.path().canonicalize().unwrap().join("work")
    }

    pub fn claude_config(&self) -> PathBuf {
        self.dir.path().join("claude-config")
    }

    pub fn config(&self) -> HostConfig {
        HostConfig {
            home: self.home.clone(),
            lithemux_home: Some(self.home.path().to_path_buf()),
            hook_binary: core_fake("fake-hook"),
            shell: core_fake("fake-shell"),
            claude_config_dir: Some(self.claude_config()),
            // Nothing of the machine: the sessions start from this alone.
            environment: vec![
                ("PATH".into(), "/usr/bin:/bin".into()),
                ("HOME".into(), self.dir.path().into()),
            ],
            hibernate_grace: std::time::Duration::from_millis(300),
        }
    }

    pub fn start(&self) -> (SessionHost, async_channel::Receiver<()>) {
        SessionHost::start(self.config()).unwrap()
    }

    pub fn add_session(&self, slug: &str, profile: Profile) -> Session {
        let db = self.db();
        create_session(
            &SqliteCategoryRepository::new(&db),
            &SqliteSessionRepository::new(&db),
            NewSession {
                target: target(slug),
                role: format!("role of {slug}"),
                kind: Kind::Member,
                profile,
                cwd: None,
                model: None,
            },
            NOW,
        )
        .unwrap()
        .1
    }

    pub fn id_of(&self, slug: &str) -> String {
        let db = self.db();
        get_session(
            &SqliteCategoryRepository::new(&db),
            &SqliteSessionRepository::new(&db),
            &target(slug),
        )
        .unwrap()
        .1
        .id
    }

    pub fn stored(&self, session_id: &str) -> Session {
        let db = self.db();
        find_session_by_id(
            &SqliteCategoryRepository::new(&db),
            &SqliteSessionRepository::new(&db),
            session_id,
        )
        .unwrap()
        .unwrap()
        .1
    }

    /// What `lithemux rm` does to the database.
    pub fn remove(&self, session_id: &str) {
        let db = self.db();
        let sessions = SqliteSessionRepository::new(&db);
        remove_session(&sessions, &self.stored(session_id)).unwrap();
    }

    pub fn set_state(&self, session_id: &str, state: State) {
        let db = self.db();
        apply_state(
            &SqliteSessionRepository::new(&db),
            session_id,
            state,
            None,
            None,
        )
        .unwrap();
    }

    /// As if its last activity had been at `at` and it were idle since.
    pub fn set_idle_since(&self, session_id: &str, at: i64) {
        let db = self.db();
        apply_state(
            &SqliteSessionRepository::new(&db),
            session_id,
            State::Idle,
            Some(at),
            None,
        )
        .unwrap();
    }

    pub fn keep_alive(&self, session_id: &str) {
        let db = self.db();
        let sessions = SqliteSessionRepository::new(&db);
        set_keep_alive(&sessions, &self.stored(session_id), true).unwrap();
    }

    /// A file the fake claude reads to know how to behave.
    pub fn steer_fake_claude(&self, file: &str, contents: &str) {
        fs::create_dir_all(self.home.path()).unwrap();
        fs::write(self.home.path().join(file), contents).unwrap();
    }

    /// `(session id, pid, first argument)` of every run of the fake claude.
    pub fn fake_claude_runs(&self) -> Vec<(String, i32, String)> {
        fs::read_to_string(self.home.path().join("fake-claude.runs"))
            .unwrap_or_default()
            .lines()
            .map(|line| {
                let mut parts = line.split(' ');
                let id = parts.next().unwrap().to_string();
                let pid = parts.next().unwrap().parse().unwrap();
                (id, pid, parts.next().unwrap_or_default().to_string())
            })
            .collect()
    }

    /// Waits for the run of the fake claude number `count` of that session.
    pub fn wait_for_run(&self, session_id: &str, count: usize) -> (i32, String) {
        let deadline = Instant::now() + PATIENCE;
        loop {
            let runs: Vec<_> = self
                .fake_claude_runs()
                .into_iter()
                .filter(|(id, _, _)| id == session_id)
                .collect();
            if let Some((_, pid, flag)) = runs.get(count - 1) {
                return (*pid, flag.clone());
            }
            assert!(Instant::now() < deadline, "the fake claude never started");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

pub fn target(slug: &str) -> SessionRef {
    SessionRef::parse(&format!("demo/{slug}")).unwrap()
}

/// Asks for the session and waits until the host knows how it went.
pub fn open(host: &mut SessionHost, slug: &str) -> Phase {
    let asked = host.open(&target(slug), SIZE).unwrap();
    let session_id = asked.session_id().to_string();
    attend_until(host, "the session to stop opening", |host| {
        host.phase(&session_id) != Some(&Phase::Opening)
    });
    host.phase(&session_id).cloned().unwrap()
}

/// Opens a session that is expected to run, and returns its id.
pub fn run(host: &mut SessionHost, world: &HostedWorld, slug: &str) -> String {
    let phase = open(host, slug);
    assert_eq!(phase, Phase::Running, "{slug} did not start");
    world.id_of(slug)
}

/// Lets the host read its mail until `done` holds.
pub fn attend_until(
    host: &mut SessionHost,
    what: &str,
    mut done: impl FnMut(&SessionHost) -> bool,
) {
    let deadline = Instant::now() + PATIENCE;
    loop {
        host.process_pending();
        if done(host) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}
