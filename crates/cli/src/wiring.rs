use lithemux_core::modules::category::{FsBoardStore, SqliteCategoryRepository};
use lithemux_core::modules::launch::FlockSessionLocks;
use lithemux_core::modules::session::SqliteSessionRepository;
use lithemux_core::modules::settings::SqliteSettingsRepository;
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;

/// Composition root: the real adapters behind every port.
pub struct Wiring {
    pub home: Home,
    pub db: Db,
}

impl Wiring {
    pub fn open() -> anyhow::Result<Self> {
        let home = Home::resolve()?;
        let db = Db::open(&home)?;
        Ok(Wiring { home, db })
    }

    pub fn categories(&self) -> SqliteCategoryRepository<'_> {
        SqliteCategoryRepository::new(&self.db)
    }

    pub fn sessions(&self) -> SqliteSessionRepository<'_> {
        SqliteSessionRepository::new(&self.db)
    }

    pub fn settings(&self) -> SqliteSettingsRepository<'_> {
        SqliteSettingsRepository::new(&self.db)
    }

    pub fn boards(&self) -> FsBoardStore<'_> {
        FsBoardStore::new(&self.home)
    }

    pub fn locks(&self) -> FlockSessionLocks<'_> {
        FlockSessionLocks::new(&self.home)
    }
}
