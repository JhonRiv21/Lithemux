use rusqlite::{Transaction, TransactionBehavior};

use crate::modules::import::error::ImportError;
use crate::modules::import::ports::import_store::ImportStore;
use crate::platform::store::connection::Db;
use crate::platform::store::error::StoreError;

/// The repositories of the same `Db` write inside the transaction.
pub struct SqliteImportStore<'a> {
    db: &'a Db,
}

impl<'a> SqliteImportStore<'a> {
    pub fn new(db: &'a Db) -> Self {
        SqliteImportStore { db }
    }
}

impl ImportStore for SqliteImportStore<'_> {
    /// `IMMEDIATE`: the write lock is taken at the start, so another
    /// import waits instead of reading what this one is about to change.
    fn atomically(
        &self,
        work: &mut dyn FnMut() -> Result<(), ImportError>,
    ) -> Result<(), ImportError> {
        let transaction =
            Transaction::new_unchecked(self.db.conn(), TransactionBehavior::Immediate)
                .map_err(StoreError::from)?;
        // Dropped without a commit, it rolls back.
        work()?;
        transaction.commit().map_err(StoreError::from)?;
        Ok(())
    }
}
