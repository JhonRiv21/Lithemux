//! Reading cmux's sessions and importing them, through the use cases of
//! `core` with the real adapters. cmux's file is only read.
//!
//! Each one opens its own connection to the database: they run away from
//! the interface thread, which keeps its own.

use std::path::Path;

use lithemux_core::modules::category::{FsBoardStore, SqliteCategoryRepository};
use lithemux_core::modules::import::{
    Chosen, CmuxSessionFile, ImportPorts, Imported, Proposal, SqliteImportStore, apply_import,
    propose_import,
};
use lithemux_core::modules::launch::ClaudeProjectsTranscriptLocator;
use lithemux_core::modules::session::SqliteSessionRepository;
use lithemux_core::platform::clock::system_clock::now_unix;
use lithemux_core::platform::log::app_log::{AppLog, Level};
use lithemux_core::platform::paths::home::Home;
use lithemux_core::platform::store::connection::Db;

/// What went wrong, in words: the window shows it as it is.
pub type Failed = String;

fn with_ports<T>(
    home: &Home,
    run: impl FnOnce(&ImportPorts<'_>, Option<&str>) -> Result<T, Failed>,
) -> Result<T, Failed> {
    let db = Db::open(home).map_err(|error| error.to_string())?;
    let user_home = std::env::home_dir();
    let cmux = CmuxSessionFile::of_user(user_home.as_deref().unwrap_or(Path::new("/")));
    let ports = ImportPorts {
        cmux: &cmux,
        categories: &SqliteCategoryRepository::new(&db),
        sessions: &SqliteSessionRepository::new(&db),
        transcripts: &ClaudeProjectsTranscriptLocator::from_env(),
        boards: &FsBoardStore::new(home),
        store: &SqliteImportStore::new(&db),
    };
    let user_home = user_home.map(|home| home.to_string_lossy().into_owned());
    run(&ports, user_home.as_deref())
}

pub fn propose(home: &Home) -> Result<Proposal, Failed> {
    with_ports(home, |ports, user_home| {
        propose_import(ports, user_home).map_err(|error| error.to_string())
    })
}

/// A board that could not be made does not undo the import: it goes to
/// the app's log.
pub fn import(home: &Home, chosen: &[Chosen]) -> Result<Imported, Failed> {
    let imported = with_ports(home, |ports, _| {
        apply_import(ports, chosen, now_unix()).map_err(|error| error.to_string())
    })?;
    if !imported.boards_failed.is_empty() {
        let log = AppLog::shared(home.app_log_path());
        for failed in &imported.boards_failed {
            log.write(
                Level::Warn,
                &format!("import: the board of {failed}"),
                now_unix(),
            );
        }
    }
    Ok(imported)
}
