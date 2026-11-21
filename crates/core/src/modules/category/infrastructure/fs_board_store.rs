use std::fs;
use std::path::PathBuf;

use crate::modules::category::domain::category::Category;
use crate::modules::category::error::CategoryError;
use crate::modules::category::ports::board_store::BoardStore;
use crate::platform::paths::fs_error::FsError;
use crate::platform::paths::home::Home;

pub struct FsBoardStore<'a> {
    home: &'a Home,
}

impl<'a> FsBoardStore<'a> {
    pub fn new(home: &'a Home) -> Self {
        FsBoardStore { home }
    }
}

impl BoardStore for FsBoardStore<'_> {
    fn path(&self, category_slug: &str) -> PathBuf {
        self.home.board_path(category_slug)
    }

    fn ensure(&self, category: &Category) -> Result<(), CategoryError> {
        let path = self.path(&category.slug);
        if path.exists() {
            return Ok(());
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| FsError::new(format!("creating {}", dir.display()), e))?;
        }
        fs::write(&path, format!("# {} board\n\n", category.name))
            .map_err(|e| FsError::new(format!("writing {}", path.display()), e))?;
        Ok(())
    }
}
