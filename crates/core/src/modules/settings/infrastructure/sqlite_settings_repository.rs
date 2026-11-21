use rusqlite::{OptionalExtension, params};

use crate::modules::settings::error::SettingsError;
use crate::modules::settings::ports::settings_repository::SettingsRepository;
use crate::platform::store::connection::Db;
use crate::platform::store::error::StoreError;

pub struct SqliteSettingsRepository<'a> {
    db: &'a Db,
}

impl<'a> SqliteSettingsRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        SqliteSettingsRepository { db }
    }
}

impl SettingsRepository for SqliteSettingsRepository<'_> {
    fn get(&self, key: &str) -> Result<Option<String>, SettingsError> {
        let value: Option<Option<String>> = self
            .db
            .conn()
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()
            .map_err(StoreError::from)?;
        Ok(value.flatten())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), SettingsError> {
        self.db
            .conn()
            .execute(
                "INSERT INTO settings(key, value) VALUES (?1, ?2) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(StoreError::from)?;
        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), SettingsError> {
        self.db
            .conn()
            .execute("DELETE FROM settings WHERE key = ?1", [key])
            .map_err(StoreError::from)?;
        Ok(())
    }
}
