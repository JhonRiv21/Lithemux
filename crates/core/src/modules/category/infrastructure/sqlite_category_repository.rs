use rusqlite::{OptionalExtension, Row, params};

use crate::modules::category::domain::category::Category;
use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::category_repository::CategoryRepository;
use crate::platform::store::connection::Db;
use crate::platform::store::error::StoreError;

const COLUMNS: &str = "id, slug, name, root_dir, color, sort, created_at";

pub struct SqliteCategoryRepository<'a> {
    db: &'a Db,
}

impl<'a> SqliteCategoryRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        SqliteCategoryRepository { db }
    }

    fn find_where(&self, column: &str, value: &str) -> Result<Option<Category>, CategoryError> {
        Ok(self
            .db
            .conn()
            .query_row(
                &format!("SELECT {COLUMNS} FROM categories WHERE {column} = ?1"),
                [value],
                from_row,
            )
            .optional()
            .map_err(StoreError::from)?)
    }
}

impl CategoryRepository for SqliteCategoryRepository<'_> {
    fn insert(&self, category: &Category) -> Result<(), CategoryError> {
        self.db
            .conn()
            .execute(
                "INSERT INTO categories(id, slug, name, root_dir, color, sort, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    category.id,
                    category.slug,
                    category.name,
                    category.root_dir,
                    category.color,
                    category.sort,
                    category.created_at
                ],
            )
            .map_err(StoreError::from)?;
        Ok(())
    }

    fn next_sort(&self) -> Result<i64, CategoryError> {
        Ok(self
            .db
            .conn()
            .query_row(
                "SELECT COALESCE(MAX(sort), 0) + 1 FROM categories",
                [],
                |row| row.get(0),
            )
            .map_err(StoreError::from)?)
    }

    fn list(&self) -> Result<Vec<Category>, CategoryError> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM categories ORDER BY sort, created_at, slug"
            ))
            .map_err(StoreError::from)?;
        let rows = stmt.query_map([], from_row).map_err(StoreError::from)?;
        Ok(rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(StoreError::from)?)
    }

    fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, CategoryError> {
        self.find_where("slug", slug)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Category>, CategoryError> {
        self.find_where("id", id)
    }

    // Sessions go with it through ON DELETE CASCADE.
    fn delete(&self, id: &str) -> Result<(), CategoryError> {
        self.db
            .conn()
            .execute("DELETE FROM categories WHERE id = ?1", [id])
            .map_err(StoreError::from)?;
        Ok(())
    }
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        root_dir: row.get(3)?,
        color: row.get(4)?,
        sort: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
        created_at: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
    })
}
