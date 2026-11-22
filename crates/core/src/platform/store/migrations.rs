use rusqlite::{Connection, TransactionBehavior};

use crate::platform::store::error::StoreError;

pub const SCHEMA_VERSION: i64 = 2;

const MIGRATION_1: &str = "
CREATE TABLE categories(
  id TEXT PRIMARY KEY,
  slug TEXT UNIQUE NOT NULL,
  name TEXT NOT NULL,
  root_dir TEXT NOT NULL,
  color TEXT,
  sort INTEGER,
  created_at INTEGER);

CREATE TABLE sessions(
  id TEXT PRIMARY KEY,
  category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  slug TEXT NOT NULL,
  role TEXT NOT NULL,
  kind TEXT NOT NULL CHECK(kind IN ('mother','child')),
  profile TEXT NOT NULL CHECK(profile IN ('claude','shell')),
  cwd TEXT,
  claude_session_id TEXT,
  model TEXT,
  keep_alive INTEGER NOT NULL DEFAULT 0,
  state TEXT NOT NULL DEFAULT 'new',
  claude_version TEXT,
  last_activity_at INTEGER,
  created_at INTEGER,
  sort INTEGER,
  UNIQUE(category_id, slug));

CREATE UNIQUE INDEX one_mother_per_category ON sessions(category_id) WHERE kind='mother';

CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT);
";

// What the sidebar and the workspace keep (PLAN §2.2, §2.3).
const MIGRATION_2: &str = "
ALTER TABLE sessions ADD COLUMN title TEXT;
ALTER TABLE sessions ADD COLUMN pin_sort INTEGER;
ALTER TABLE sessions ADD COLUMN preview TEXT;
ALTER TABLE sessions ADD COLUMN unread INTEGER NOT NULL DEFAULT 0;
ALTER TABLE categories ADD COLUMN collapsed INTEGER NOT NULL DEFAULT 0;

CREATE TABLE workspace_layout(
  id INTEGER PRIMARY KEY CHECK(id = 1),
  layout TEXT NOT NULL);
";

pub fn user_version(conn: &Connection) -> Result<i64, StoreError> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Safe to run from several processes at once: the write lock is taken
/// before the version is trusted, so only the first one migrates.
pub fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    if checked_version(conn)? >= SCHEMA_VERSION {
        return Ok(());
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if checked_version(&tx)? < 1 {
        tx.execute_batch(MIGRATION_1)?;
        tx.pragma_update(None, "user_version", 1)?;
    }
    if checked_version(&tx)? < 2 {
        tx.execute_batch(MIGRATION_2)?;
        tx.pragma_update(None, "user_version", 2)?;
    }
    tx.commit()?;
    Ok(())
}

fn checked_version(conn: &Connection) -> Result<i64, StoreError> {
    let found = user_version(conn)?;
    if found > SCHEMA_VERSION {
        return Err(StoreError::SchemaTooNew {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(found)
}
