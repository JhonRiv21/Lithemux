use lithemux_core::modules::context::{
    ContextError, ContextPorts, Lang, build_session_context, resolve_language,
};
use lithemux_core::modules::session::{Kind, State, apply_state};
use lithemux_core::modules::settings::{SettingKey, write_setting};

use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::sample_data::{NOW, add_category, add_session};

fn ports<'a>(store: &'a InMemoryStore, boards: &'a InMemoryBoards) -> ContextPorts<'a> {
    ContextPorts {
        categories: store,
        sessions: store,
        settings: store,
        boards,
    }
}

// PLAN §0.5: default language is English
#[test]
fn the_context_language_defaults_to_english() {
    assert_eq!(resolve_language(&InMemoryStore::new()).unwrap(), Lang::En);
}

// PLAN §0.5: context_language, else the interface language
#[test]
fn context_language_wins_over_the_interface_language() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::Language, "es").unwrap();
    assert_eq!(resolve_language(&store).unwrap(), Lang::Es);
    write_setting(&store, SettingKey::ContextLanguage, "en").unwrap();
    assert_eq!(resolve_language(&store).unwrap(), Lang::En);
}

// PLAN §0.5: rendered from stored state on every SessionStart
#[test]
fn the_context_reflects_the_category_as_it_is_now() {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    add_category(&store, &boards, "demo");
    add_session(&store, &store, "demo/planificadora", Kind::Mother);
    let executor = add_session(&store, &store, "demo/ejecutora", Kind::Child);

    let before = build_session_context(&ports(&store, &boards), &executor.id, NOW).unwrap();
    assert!(before.contains("demo-planificadora"));
    assert!(!before.contains("demo-documental"));

    let documenter = add_session(&store, &store, "demo/documental", Kind::Child);
    apply_state(
        &store,
        &documenter.id,
        State::Hibernated,
        Some(NOW - 7200),
        None,
    )
    .unwrap();

    let after = build_session_context(&ports(&store, &boards), &executor.id, NOW).unwrap();
    assert!(after.contains("demo-documental"));
    assert!(after.contains("state: hibernated, last activity 2 h ago"));
    assert!(after.contains("/lithemux/categories/demo/board.md"));
    assert!(
        !after.contains("demo-ejecutora ("),
        "a session is not its own sister"
    );
}

// PLAN §0.5: sessions of other categories are not part of the context
#[test]
fn the_context_only_knows_its_own_category() {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    add_category(&store, &boards, "demo");
    add_category(&store, &boards, "otra");
    let executor = add_session(&store, &store, "demo/ejecutora", Kind::Child);
    add_session(&store, &store, "otra/planificadora", Kind::Mother);
    let text = build_session_context(&ports(&store, &boards), &executor.id, NOW).unwrap();
    assert!(!text.contains("otra-planificadora"));
    assert!(text.contains("no mother"));
}

// PLAN §0.4: an id lithemux does not know produces no context
#[test]
fn an_unknown_session_has_no_context() {
    let (store, boards) = (InMemoryStore::new(), InMemoryBoards::default());
    assert!(matches!(
        build_session_context(&ports(&store, &boards), "missing", NOW),
        Err(ContextError::SessionNotFound(_))
    ));
}
