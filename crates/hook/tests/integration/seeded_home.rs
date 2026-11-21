use std::path::Path;

use assert_cmd::Command;
use lithemux_core::modules::category::{
    FsBoardStore, NewCategory, Slug, SqliteCategoryRepository, create_category,
};
use lithemux_core::modules::session::{
    Kind, NewSession, Profile, Session, SessionRef, SqliteSessionRepository, create_session,
    find_session_by_id,
};
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;
use tempfile::TempDir;

/// A real LITHEMUX_HOME with the category `demo`: a mother and one child.
pub struct SeededHome {
    pub dir: TempDir,
    pub home: Home,
    pub executor: Session,
}

impl SeededHome {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::new(dir.path());
        let db = Db::open(&home).unwrap();
        let (categories, sessions) = (
            SqliteCategoryRepository::new(&db),
            SqliteSessionRepository::new(&db),
        );
        create_category(
            &categories,
            &FsBoardStore::new(&home),
            NewCategory {
                slug: Slug::parse("demo").unwrap(),
                name: "Demo".to_string(),
                root_dir: "/work/demo".to_string(),
                color: None,
            },
            1_800_000_000,
        )
        .unwrap();
        let add = |target: &str, kind: Kind| {
            let new = NewSession {
                target: SessionRef::parse(target).unwrap(),
                role: format!("role of {target}"),
                kind,
                profile: Profile::Claude,
                cwd: None,
                model: None,
            };
            create_session(&categories, &sessions, new, 1_800_000_000)
                .unwrap()
                .1
        };
        add("demo/planificadora", Kind::Mother);
        let executor = add("demo/ejecutora", Kind::Child);
        drop(db);
        SeededHome {
            dir,
            home,
            executor,
        }
    }

    pub fn db(&self) -> Db {
        Db::open(&self.home).unwrap()
    }

    pub fn executor_now(&self) -> Session {
        let db = self.db();
        find_session_by_id(
            &SqliteCategoryRepository::new(&db),
            &SqliteSessionRepository::new(&db),
            &self.executor.id,
        )
        .unwrap()
        .unwrap()
        .1
    }

    /// The hook as Claude Code runs it for the executor session.
    pub fn hook(&self, subcommand: &str) -> Command {
        let mut command = hook();
        command
            .arg(subcommand)
            .env("LITHEMUX_HOME", self.dir.path())
            .env("LITHEMUX_SESSION_ID", &self.executor.id);
        command
    }
}

/// The hook binary with a clean environment.
pub fn hook() -> Command {
    let mut command = Command::cargo_bin("lithemux-hook").unwrap();
    command.env_clear();
    command
}

pub fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}
