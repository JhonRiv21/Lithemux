use lithemux_core::modules::category::{Category, CategoryRepository, SqliteCategoryRepository};
use lithemux_core::modules::session::{
    Kind, Profile, Session, SessionRepository, SqliteSessionRepository, State,
};
use lithemux_core::platform::store::connection::Db;

use crate::temp_home::TempHome;

fn pragma(db: &Db, name: &str) -> String {
    db.conn()
        .pragma_query_value(None, name, |row| {
            row.get::<_, rusqlite::types::Value>(0)
                .map(|value| match value {
                    rusqlite::types::Value::Integer(n) => n.to_string(),
                    rusqlite::types::Value::Text(text) => text,
                    other => format!("{other:?}"),
                })
        })
        .unwrap()
}

fn category(id: &str, slug: &str) -> Category {
    Category {
        id: id.to_string(),
        slug: slug.to_string(),
        name: slug.to_string(),
        root_dir: "/work".to_string(),
        color: None,
        sort: 1,
        created_at: 0,
        collapsed: false,
        notify: true,
    }
}

fn session(id: &str, slug: &str, kind: Kind) -> Session {
    Session {
        id: id.to_string(),
        category_id: "c1".to_string(),
        slug: slug.to_string(),
        role: "role".to_string(),
        kind,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: None,
        model: None,
        keep_alive: false,
        state: State::New,
        claude_version: None,
        last_activity_at: None,
        created_at: 0,
        sort: 1,
        title: None,
        pin_sort: None,
        preview: None,
        unread: 0,
        notice: None,
    }
}

// PLAN §0.2: database file under LITHEMUX_HOME
#[test]
fn the_database_lives_in_the_lithemux_home() {
    let temp = TempHome::new();
    temp.db();
    assert!(temp.dir.path().join("lithemux/lithemux.db").is_file());
}

// PLAN §0.2: WAL, busy_timeout of 250 ms, migrations by user_version
#[test]
fn the_database_uses_wal_a_250_ms_busy_timeout_and_user_version() {
    let db = TempHome::new().db();
    assert_eq!(pragma(&db, "journal_mode"), "wal");
    assert_eq!(pragma(&db, "busy_timeout"), "250");
    assert_eq!(pragma(&db, "foreign_keys"), "1");
    assert_eq!(db.schema_version().unwrap(), 6);
}

// PLAN §0.2: migrations run once
#[test]
fn reopening_keeps_the_data_and_the_schema_version() {
    let temp = TempHome::new();
    SqliteCategoryRepository::new(&temp.db())
        .insert(&category("c1", "demo"))
        .unwrap();
    let reopened = temp.db();
    assert_eq!(reopened.schema_version().unwrap(), 6);
    assert_eq!(
        SqliteCategoryRepository::new(&reopened)
            .list()
            .unwrap()
            .len(),
        1
    );
}

// PLAN §0.4: the hook never creates the database
#[test]
fn open_existing_does_not_create_a_database() {
    let temp = TempHome::new();
    assert!(Db::open_existing(&temp.home).unwrap().is_none());
    assert!(!temp.home.db_path().exists());
    temp.db();
    assert!(Db::open_existing(&temp.home).unwrap().is_some());
}

// PLAN §0.2: a database from a newer build is not touched
#[test]
fn a_newer_schema_is_refused() {
    let temp = TempHome::new();
    temp.db()
        .conn()
        .pragma_update(None, "user_version", 99)
        .unwrap();
    assert!(Db::open(&temp.home).is_err());
}

// PLAN §0.2: CREATE UNIQUE INDEX one_coordinator_per_category
#[test]
fn the_database_itself_refuses_a_second_coordinator() {
    let db = TempHome::new().db();
    SqliteCategoryRepository::new(&db)
        .insert(&category("c1", "demo"))
        .unwrap();
    let sessions = SqliteSessionRepository::new(&db);
    sessions
        .insert(&session("s1", "una", Kind::Coordinator))
        .unwrap();
    assert!(
        sessions
            .insert(&session("s2", "otra", Kind::Coordinator))
            .is_err()
    );
    sessions
        .insert(&session("s3", "otra", Kind::Member))
        .unwrap();
}

// PLAN §0.2: slug UNIQUE, UNIQUE(category_id, slug), CHECK constraints
#[test]
fn the_database_enforces_uniqueness_and_allowed_values() {
    let db = TempHome::new().db();
    let categories = SqliteCategoryRepository::new(&db);
    categories.insert(&category("c1", "demo")).unwrap();
    assert!(categories.insert(&category("c2", "demo")).is_err());
    let sessions = SqliteSessionRepository::new(&db);
    sessions
        .insert(&session("s1", "otra", Kind::Member))
        .unwrap();
    assert!(
        sessions
            .insert(&session("s2", "otra", Kind::Member))
            .is_err()
    );
    let bad_kind = db.conn().execute(
        "INSERT INTO sessions(id, category_id, slug, role, kind, profile) \
         VALUES ('s9', 'c1', 'x', 'r', 'aunt', 'claude')",
        [],
    );
    assert!(bad_kind.is_err());
    let bad_profile = db.conn().execute(
        "INSERT INTO sessions(id, category_id, slug, role, kind, profile) \
         VALUES ('s9', 'c1', 'x', 'r', 'member', 'codex')",
        [],
    );
    assert!(bad_profile.is_err());
}

// PLAN §2.0 bug 2 (audit M9): a start that finds the database busy for a
// moment waits instead of failing
#[test]
fn opening_waits_for_a_database_that_is_busy_for_a_moment() {
    let temp = TempHome::new();
    std::fs::create_dir_all(temp.home.path()).unwrap();
    let path = temp.home.db_path();
    let (locked, wait_for_lock) = std::sync::mpsc::channel();
    let holder = std::thread::spawn(move || {
        let mut conn = rusqlite::Connection::open(path).unwrap();
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)
            .unwrap();
        locked.send(()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(700));
        tx.commit().unwrap();
    });
    wait_for_lock.recv().unwrap();

    let opened = Db::open(&temp.home);

    holder.join().unwrap();
    assert!(opened.is_ok(), "{:?}", opened.err());
}
