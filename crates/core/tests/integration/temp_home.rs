use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;
use tempfile::TempDir;

/// A throwaway LITHEMUX_HOME with a real SQLite database.
pub struct TempHome {
    pub dir: TempDir,
    pub home: Home,
}

impl TempHome {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::new(dir.path().join("lithemux"));
        TempHome { dir, home }
    }

    pub fn db(&self) -> Db {
        Db::open(&self.home).unwrap()
    }
}
