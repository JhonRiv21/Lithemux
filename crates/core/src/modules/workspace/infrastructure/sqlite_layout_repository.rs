use rusqlite::OptionalExtension;

use crate::modules::workspace::error::WorkspaceError;
use crate::modules::workspace::ports::layout_repository::LayoutRepository;
use crate::platform::store::connection::Db;
use crate::platform::store::error::StoreError;

pub struct SqliteLayoutRepository<'a> {
    db: &'a Db,
}

impl<'a> SqliteLayoutRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        SqliteLayoutRepository { db }
    }
}

impl LayoutRepository for SqliteLayoutRepository<'_> {
    fn load(&self) -> Result<Option<String>, WorkspaceError> {
        Ok(self
            .db
            .conn()
            .query_row(
                "SELECT layout FROM workspace_layout WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::from)?)
    }

    fn save(&self, layout: &str) -> Result<(), WorkspaceError> {
        self.db
            .conn()
            .execute(
                "INSERT INTO workspace_layout(id, layout) VALUES (1, ?1) \
                 ON CONFLICT(id) DO UPDATE SET layout = excluded.layout",
                [layout],
            )
            .map_err(StoreError::from)?;
        Ok(())
    }
}
