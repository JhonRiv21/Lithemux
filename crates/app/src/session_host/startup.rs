//! What the host does once, before it listens: the database, and the
//! sessions an earlier app left as running.

use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::{SqliteSessionRepository, hibernate_what_was_left_running};
use lithemux_core::platform::clock::system_clock::now_unix;
use lithemux_core::platform::log::app_log::{AppLog, Level};
use lithemux_core::platform::store::connection::Db;

use super::config::HostConfig;
use super::error::HostError;

/// Opens, and migrates, the database; what it found and what it made of it
/// go to the log.
pub(super) fn open_the_database(config: &HostConfig, log: &AppLog) -> Result<Db, HostError> {
    let found = match Db::open_existing(&config.home) {
        Ok(None) => Found::Nothing,
        Ok(Some(db)) => db.schema_version().map_or(Found::Unreadable, Found::Schema),
        Err(_) => Found::Unreadable,
    };
    let db = Db::open(&config.home).inspect_err(|error| {
        let line = format!("could not open the database: {error}");
        log.write(Level::Error, &line, now_unix());
    })?;
    if let Some(note) = migration_note(found, db.schema_version().ok()) {
        log.write(Level::Info, &note, now_unix());
    }
    Ok(db)
}

/// What there was before the app opened the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    Nothing,
    Schema(i64),
    /// A file that could not be read: it was there, so it was not created.
    Unreadable,
}

/// A database that was made, or moved to a newer schema, says so in the log.
pub fn migration_note(found: Found, now: Option<i64>) -> Option<String> {
    match (found, now?) {
        (Found::Schema(before), now) if before != now => {
            Some(format!("database migrated from schema v{before} to v{now}"))
        }
        (Found::Nothing, now) => Some(format!("database created at schema v{now}")),
        _ => None,
    }
}

/// Nothing of this app runs yet: what the database still has as running
/// was cut short, and is hibernated (PLAN §2.5).
pub(super) fn settle_what_was_left_running(config: &HostConfig, db: &Db) -> Result<(), HostError> {
    let locks = FlockSessionLocks::new(&config.home);
    hibernate_what_was_left_running(&SqliteSessionRepository::new(db), |id| {
        // The pid is read first: taking the lock writes ours over it.
        // One that is alive is an orphan of an app that died, or a
        // terminal: either way the session is not ours to settle. And
        // it is the lock, not the pid, that says whether it is free.
        let somebody_runs_it = locks.holder(id).is_some();
        let lock = locks.acquire(id).ok()?;
        (!somebody_runs_it).then_some(lock)
    })?;
    Ok(())
}
