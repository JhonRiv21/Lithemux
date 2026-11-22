use rusqlite::{Connection, TransactionBehavior};

use crate::platform::store::error::StoreError;

pub const SCHEMA_VERSION: i64 = 5;

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

// What Claude is waiting for, apart from the last thing said (block 4).
const MIGRATION_3: &str = "
ALTER TABLE sessions ADD COLUMN notice TEXT;
";

// «mother» and «child» became «coordinator» and «member» (22-nov-2026). The
// old values are in a CHECK of the table, so the table is made again. This
// is the one place where the old words may still be written.
const MIGRATION_4: &str = "
CREATE TABLE sessions_renamed(
  id TEXT PRIMARY KEY,
  category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  slug TEXT NOT NULL,
  role TEXT NOT NULL,
  kind TEXT NOT NULL CHECK(kind IN ('coordinator','member')),
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
  title TEXT,
  pin_sort INTEGER,
  preview TEXT,
  unread INTEGER NOT NULL DEFAULT 0,
  notice TEXT,
  UNIQUE(category_id, slug));

INSERT INTO sessions_renamed
SELECT id, category_id, slug, role,
       CASE kind WHEN 'mother' THEN 'coordinator' ELSE 'member' END,
       profile, cwd, claude_session_id, model, keep_alive, state, claude_version,
       last_activity_at, created_at, sort, title, pin_sort, preview, unread, notice
FROM sessions;

DROP TABLE sessions;
ALTER TABLE sessions_renamed RENAME TO sessions;

CREATE UNIQUE INDEX one_coordinator_per_category ON sessions(category_id)
  WHERE kind='coordinator';
";

// Desktop notifications, on or off per category (PLAN §3.1).
const MIGRATION_5: &str = "
ALTER TABLE categories ADD COLUMN notify INTEGER NOT NULL DEFAULT 1;
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
    if checked_version(&tx)? < 3 {
        tx.execute_batch(MIGRATION_3)?;
        tx.pragma_update(None, "user_version", 3)?;
    }
    if checked_version(&tx)? < 4 {
        tx.execute_batch(MIGRATION_4)?;
        tx.pragma_update(None, "user_version", 4)?;
    }
    if checked_version(&tx)? < 5 {
        tx.execute_batch(MIGRATION_5)?;
        tx.pragma_update(None, "user_version", 5)?;
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
