mod application;
mod domain;
mod error;
mod infrastructure;
mod ports;

pub use application::arrange_categories::{
    rename_category, reorder_categories, set_collapsed, set_notify,
};
pub use application::create_category::create_category;
pub use application::find_category::{find_category_by_id, get_category, list_categories};
pub use application::locate_directory::locate_directory;
pub use application::remove_category::remove_category;
pub use domain::category::{Category, NewCategory};
pub use domain::slug::Slug;
pub use domain::slug_policy::{free_slug, slug_from_name};
pub use error::{CategoryError, InvalidSlug};
pub use infrastructure::fs_board_store::FsBoardStore;
pub use infrastructure::fs_directories::FsDirectories;
pub use infrastructure::sqlite_category_repository::SqliteCategoryRepository;
pub use ports::board_store::BoardStore;
pub use ports::category_repository::CategoryRepository;
pub use ports::directories::Directories;
