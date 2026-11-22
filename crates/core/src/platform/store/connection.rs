use std::fs;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};

use crate::platform::paths::fs_error::FsError;
use crate::platform::paths::home::Home;
use crate::platform::store::error::StoreError;
use crate::platform::store::migrations;

const BUSY_TIMEOUT: Duration = Duration::from_millis(250);
/// Creating and migrating happens once and may queue behind other processes
/// doing the same, so it waits longer than everyday queries.
const SETUP_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Opens the database, creating and migrating it when needed.
    pub fn open(home: &Home) -> Result<Self, StoreError> {
        fs::create_dir_all(home.path())
            .map_err(|e| FsError::new(format!("creating {}", home.path().display()), e))?;
        let mut conn = Connection::open(home.db_path())?;
        conn.busy_timeout(SETUP_BUSY_TIMEOUT)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrations::migrate(&mut conn)?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        Ok(Db { conn })
    }

    /// Opens the database only if it already exists; never creates or
    /// migrates. This is the path the hook binary takes.
    pub fn open_existing(home: &Home) -> Result<Option<Self>, StoreError> {
        let path = home.db_path();
        if !path.is_file() {
            return Ok(None);
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Some(Db { conn }))
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn schema_version(&self) -> Result<i64, StoreError> {
        migrations::user_version(&self.conn)
    }
}
