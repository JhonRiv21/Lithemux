use crate::support::contracts::category_repository_contract::category_repository_contract;
use crate::support::contracts::session_repository_contract::{
    one_session_per_conversation_contract, session_repository_contract,
};
use crate::support::contracts::settings_repository_contract::settings_repository_contract;
use crate::support::in_memory_store::InMemoryStore;

// CLAUDE.md «Pruebas» 2: the in-memory ports honor the SQLite contract
#[test]
fn in_memory_category_repository_honors_the_contract() {
    category_repository_contract(&InMemoryStore::new());
}

#[test]
fn in_memory_session_repository_honors_the_contract() {
    let store = InMemoryStore::new();
    session_repository_contract(&store, &store);
}

#[test]
fn in_memory_sessions_never_share_a_conversation() {
    let store = InMemoryStore::new();
    one_session_per_conversation_contract(&store, &store);
}

#[test]
fn in_memory_settings_repository_honors_the_contract() {
    settings_repository_contract(&InMemoryStore::new());
}
