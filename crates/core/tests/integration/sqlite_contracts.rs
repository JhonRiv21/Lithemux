use lithemux_core::modules::category::SqliteCategoryRepository;
use lithemux_core::modules::session::SqliteSessionRepository;
use lithemux_core::modules::settings::SqliteSettingsRepository;

use crate::support::contracts::category_repository_contract::category_repository_contract;
use crate::support::contracts::session_repository_contract::{
    one_session_per_conversation_contract, session_repository_contract,
};
use crate::support::contracts::settings_repository_contract::settings_repository_contract;
use crate::temp_home::TempHome;

// PLAN §0.2
#[test]
fn sqlite_category_repository_honors_the_contract() {
    let db = TempHome::new().db();
    category_repository_contract(&SqliteCategoryRepository::new(&db));
}

// PLAN §0.2 and §0.4
#[test]
fn sqlite_session_repository_honors_the_contract() {
    let db = TempHome::new().db();
    session_repository_contract(
        &SqliteCategoryRepository::new(&db),
        &SqliteSessionRepository::new(&db),
    );
}

// ronda 3B bug 5: one conversation, one session, enforced by the database
#[test]
fn sqlite_sessions_never_share_a_conversation() {
    let db = TempHome::new().db();
    one_session_per_conversation_contract(
        &SqliteCategoryRepository::new(&db),
        &SqliteSessionRepository::new(&db),
    );
}

// PLAN §0.2
#[test]
fn sqlite_settings_repository_honors_the_contract() {
    let db = TempHome::new().db();
    settings_repository_contract(&SqliteSettingsRepository::new(&db));
}
