use rusqlite::Connection;

use crate::platform::store::error::StoreError;

pub const SCHEMA_VERSION: i64 = 1;

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

pub fn user_version(conn: &Connection) -> Result<i64, StoreError> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

pub fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let found = user_version(conn)?;
    if found > SCHEMA_VERSION {
        return Err(StoreError::SchemaTooNew {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    if found < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_1)?;
        tx.pragma_update(None, "user_version", 1)?;
        tx.commit()?;
    }
    Ok(())
}
