use std::path::{Path, PathBuf};

use crate::platform::paths::error::PathsError;

pub const HOME_ENV: &str = "LITHEMUX_HOME";
pub const SESSION_ENV: &str = "LITHEMUX_SESSION_ID";

/// Root of everything lithemux stores on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home(PathBuf);

impl Home {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Home(path.into())
    }

    /// `LITHEMUX_HOME` when set, otherwise the platform data directory.
    pub fn resolve() -> Result<Self, PathsError> {
        if let Some(path) = Self::env_override() {
            return Ok(Home(path));
        }
        directories::ProjectDirs::from("", "", "lithemux")
            .map(|dirs| Home(dirs.data_dir().to_path_buf()))
            .ok_or(PathsError::NoDataDir)
    }

    pub fn env_override() -> Option<PathBuf> {
        std::env::var_os(HOME_ENV)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn db_path(&self) -> PathBuf {
        self.0.join("lithemux.db")
    }

    pub fn locks_dir(&self) -> PathBuf {
        self.0.join("locks")
    }

    pub fn lock_path(&self, session_id: &str) -> PathBuf {
        self.locks_dir().join(format!("{session_id}.lock"))
    }

    pub fn hook_log_path(&self) -> PathBuf {
        self.0.join("logs").join("hook.log")
    }

    pub fn app_socket_path(&self) -> PathBuf {
        self.0.join("app.sock")
    }

    pub fn board_path(&self, category_slug: &str) -> PathBuf {
        self.0
            .join("categories")
            .join(category_slug)
            .join("board.md")
    }
}
