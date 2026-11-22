//! A database of any previous version is brought up to date with its data.
//! The old schemas are fixtures: the SQL each version really ran.

use lithemux_core::modules::category::{SqliteCategoryRepository, list_categories};
use lithemux_core::modules::session::{Kind, SessionRepository, SqliteSessionRepository, State};
use lithemux_core::platform::store::connection::Db;
use rusqlite::Connection;

use crate::temp_home::TempHome;

const SCHEMA_V1: &str = include_str!("../fixtures/schema_v1.sql");
const MIGRATION_2: &str = include_str!("../fixtures/migration_2.sql");
const MIGRATION_3: &str = "ALTER TABLE sessions ADD COLUMN notice TEXT;";
const SCHEMA_V3_AS_STORED: &str = include_str!("../fixtures/schema_v3_as_stored.sql");

/// A category with the two kinds of session of the old schemas, one of
/// them with everything a session can carry at version 1.
const OLD_DATA: &str = "
INSERT INTO categories(id, slug, name, root_dir, sort, created_at)
  VALUES ('c1', 'demo', 'Demo', '/work/demo', 1, 10);
INSERT INTO sessions(id, category_id, slug, role, kind, profile, cwd, claude_session_id, model,
                     keep_alive, state, claude_version, last_activity_at, created_at, sort)
  VALUES ('s1', 'c1', 'planner', 'plans', 'mother', 'claude', '/work/other', 'conv-1', 'opus',
          1, 'idle', '2.1.330', 77, 20, 1);
INSERT INTO sessions(id, category_id, slug, role, kind, profile, state, created_at, sort)
  VALUES ('s2', 'c1', 'api', 'the api', 'child', 'claude', 'working', 21, 2);
INSERT INTO sessions(id, category_id, slug, role, kind, profile, state, created_at, sort)
  VALUES ('s3', 'c1', 'term', 'shell', 'child', 'shell', 'new', 22, 3);
";

fn old_database(temp: &TempHome, version: i64, schema: &[&str], data: &[&str]) {
    std::fs::create_dir_all(temp.home.db_path().parent().unwrap()).unwrap();
    let old = Connection::open(temp.home.db_path()).unwrap();
    for sql in schema.iter().chain(data) {
        old.execute_batch(sql).unwrap();
    }
    old.pragma_update(None, "user_version", version).unwrap();
}

fn assert_up_to_date_with_its_data(db: &Db) {
    assert_eq!(db.schema_version().unwrap(), 5);
    let sessions = SqliteSessionRepository::new(db);
    let planner = sessions.find_by_id("s1").unwrap().unwrap();
    assert_eq!(planner.kind, Kind::Coordinator);
    assert_eq!(
        (
            planner.slug.as_str(),
            planner.role.as_str(),
            planner.keep_alive
        ),
        ("planner", "plans", true)
    );
    assert_eq!(planner.cwd.as_deref(), Some("/work/other"));
    assert_eq!(planner.claude_session_id.as_deref(), Some("conv-1"));
    assert_eq!(planner.model.as_deref(), Some("opus"));
    assert_eq!(planner.claude_version.as_deref(), Some("2.1.330"));
    assert_eq!(
        (planner.state, planner.last_activity_at),
        (State::Idle, Some(77))
    );
    for id in ["s2", "s3"] {
        assert_eq!(sessions.find_by_id(id).unwrap().unwrap().kind, Kind::Member);
    }
    assert_eq!(sessions.find_coordinator("c1").unwrap().unwrap().id, "s1");
    let order: Vec<String> = sessions
        .list_in("c1")
        .unwrap()
        .into_iter()
        .map(|s| s.slug)
        .collect();
    assert_eq!(order, ["planner", "api", "term"]);
    assert_eq!(
        list_categories(&SqliteCategoryRepository::new(db))
            .unwrap()
            .len(),
        1
    );
    // The rule of one coordinator per category is still the database's own.
    let second = db.conn().execute(
        "UPDATE sessions SET kind = 'coordinator' WHERE id = 's2'",
        [],
    );
    assert!(second.is_err());
    // And the old words are no longer values of the column.
    assert!(
        db.conn()
            .execute("UPDATE sessions SET kind = 'mother' WHERE id = 's2'", [])
            .is_err()
    );
}

// PLAN §0.2: migrations by user_version. From the Phase 0 schema.
#[test]
fn a_version_1_database_is_migrated_with_its_data() {
    let temp = TempHome::new();
    old_database(&temp, 1, &[SCHEMA_V1], &[OLD_DATA]);
    let db = Db::open(&temp.home).unwrap();
    assert_up_to_date_with_its_data(&db);
    let planner = SqliteSessionRepository::new(&db)
        .find_by_id("s1")
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            planner.title,
            planner.pin_sort,
            planner.preview,
            planner.unread,
            planner.notice
        ),
        (None, None, None, 0, None)
    );
}

const CARD_DATA: &str = "
UPDATE sessions SET title = 'The planner', pin_sort = 3, preview = 'done', unread = 2
  WHERE id = 's1';
UPDATE categories SET collapsed = 1 WHERE id = 'c1';
INSERT INTO workspace_layout(id, layout) VALUES (1, 'kept as it is');
";

fn assert_the_card_data_is_kept(db: &Db) {
    let planner = SqliteSessionRepository::new(db)
        .find_by_id("s1")
        .unwrap()
        .unwrap();
    assert_eq!(planner.title.as_deref(), Some("The planner"));
    assert_eq!((planner.pin_sort, planner.unread), (Some(3), 2));
    assert_eq!(planner.preview.as_deref(), Some("done"));
    assert!(list_categories(&SqliteCategoryRepository::new(db)).unwrap()[0].collapsed);
    let layout: String = db
        .conn()
        .query_row("SELECT layout FROM workspace_layout", [], |row| row.get(0))
        .unwrap();
    assert_eq!(layout, "kept as it is");
}

// Audit of block 4 (9): from version 2 (block 3), with what the sidebar
// and the workspace had stored.
#[test]
fn a_version_2_database_is_migrated_with_its_data() {
    let temp = TempHome::new();
    old_database(&temp, 2, &[SCHEMA_V1, MIGRATION_2], &[OLD_DATA, CARD_DATA]);
    let db = Db::open(&temp.home).unwrap();
    assert_up_to_date_with_its_data(&db);
    assert_the_card_data_is_kept(&db);
}

const NOTICE_DATA: &str = "UPDATE sessions SET notice = 'Claude needs your permission', state = 'waiting' WHERE id = 's2';";

// Correction round of block 4 (4): from version 3, built the way the
// migrations built it.
#[test]
fn a_version_3_database_is_migrated_with_its_data() {
    let temp = TempHome::new();
    let schema = [SCHEMA_V1, MIGRATION_2, MIGRATION_3];
    old_database(&temp, 3, &schema, &[OLD_DATA, CARD_DATA, NOTICE_DATA]);
    let db = Db::open(&temp.home).unwrap();
    assert_up_to_date_with_its_data(&db);
    assert_the_card_data_is_kept(&db);
    let api = SqliteSessionRepository::new(&db)
        .find_by_id("s2")
        .unwrap()
        .unwrap();
    assert_eq!(api.notice.as_deref(), Some("Claude needs your permission"));
    assert_eq!(api.state, State::Waiting);
}

// The same, from the schema exactly as SQLite stores it in a version 3
// database that the app itself made and migrated.
#[test]
fn a_version_3_database_as_the_app_left_it_is_migrated_with_its_data() {
    let temp = TempHome::new();
    old_database(
        &temp,
        3,
        &[SCHEMA_V3_AS_STORED],
        &[OLD_DATA, CARD_DATA, NOTICE_DATA],
    );
    let db = Db::open(&temp.home).unwrap();
    assert_up_to_date_with_its_data(&db);
    assert_the_card_data_is_kept(&db);
}

// Opening an up-to-date database again changes nothing.
#[test]
fn migrating_twice_is_migrating_once() {
    let temp = TempHome::new();
    old_database(&temp, 1, &[SCHEMA_V1], &[OLD_DATA]);
    drop(Db::open(&temp.home).unwrap());
    let db = Db::open(&temp.home).unwrap();
    assert_up_to_date_with_its_data(&db);
}
