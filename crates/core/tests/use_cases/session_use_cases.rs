use lithemux_core::modules::session::{
    Kind, Profile, SessionError, SessionRef, State, create_session, find_session_by_id,
    follow_claude_session, get_session, list_sessions, remove_session, set_keep_alive, set_kind,
    set_model, set_role,
};

use crate::support::in_memory_store::{InMemoryBoards, InMemoryStore};
use crate::support::sample_data::{NOW, add_category, add_session, new_session};

fn store_with(categories: &[&str]) -> InMemoryStore {
    let store = InMemoryStore::new();
    for slug in categories {
        add_category(&store, &InMemoryBoards::default(), slug);
    }
    store
}

// PLAN §0.2: a new session
#[test]
fn a_new_claude_session_starts_new_with_its_own_conversation_id() {
    let store = store_with(&["demo"]);
    let (category, session) = create_session(
        &store,
        &store,
        new_session("demo/ejecutora", Kind::Member),
        NOW,
    )
    .unwrap();
    assert_eq!(session.state, State::New);
    assert_eq!(session.profile, Profile::Claude);
    assert!(!session.keep_alive);
    assert_eq!(session.cwd, None);
    assert_eq!(session.messaging_name(&category), "demo-ejecutora");
    let claude_id = session.claude_session_id.clone().unwrap();
    assert_eq!(claude_id.len(), 36);
    assert_ne!(claude_id, session.id);
}

// PLAN §0.2: claude_session_id only for profile=claude
#[test]
fn a_shell_session_has_no_conversation_id() {
    let store = store_with(&["demo"]);
    let mut new = new_session("demo/term", Kind::Member);
    new.profile = Profile::Shell;
    let (_, session) = create_session(&store, &store, new, NOW).unwrap();
    assert_eq!(session.claude_session_id, None);
}

// PLAN §0.2: one_coordinator_per_category
#[test]
fn only_one_coordinator_per_category() {
    let store = store_with(&["demo", "otra"]);
    add_session(&store, &store, "demo/planificadora", Kind::Coordinator);
    let second = create_session(
        &store,
        &store,
        new_session("demo/jefa", Kind::Coordinator),
        NOW,
    );
    assert!(matches!(
        second,
        Err(SessionError::CoordinatorExists { category, coordinator })
            if category == "demo" && coordinator == "planificadora"
    ));
    // Another category can have its own coordinator.
    add_session(&store, &store, "otra/planificadora", Kind::Coordinator);
}

// PLAN §0.2: a category may have no coordinator
#[test]
fn a_category_can_live_without_a_coordinator() {
    let store = store_with(&["demo"]);
    add_session(&store, &store, "demo/uno", Kind::Member);
    add_session(&store, &store, "demo/dos", Kind::Member);
    let category = lithemux_core::modules::category::get_category(&store, "demo").unwrap();
    let all = list_sessions(&store, &category).unwrap();
    assert!(all.iter().all(|s| s.kind == Kind::Member));
}

// PLAN §0.2: UNIQUE(category_id, slug)
#[test]
fn a_session_slug_is_unique_inside_its_category() {
    let store = store_with(&["demo", "otra"]);
    add_session(&store, &store, "demo/ejecutora", Kind::Member);
    let again = create_session(
        &store,
        &store,
        new_session("demo/ejecutora", Kind::Member),
        NOW,
    );
    assert!(matches!(again, Err(SessionError::AlreadyExists(_))));
    add_session(&store, &store, "otra/ejecutora", Kind::Member);
}

// PLAN §0.2: the messaging name is unique in the whole system
#[test]
fn a_messaging_name_cannot_be_taken_twice_across_categories() {
    let store = store_with(&["a-b", "a"]);
    add_session(&store, &store, "a-b/c", Kind::Member);
    let clash = create_session(&store, &store, new_session("a/b-c", Kind::Member), NOW);
    assert!(matches!(
        clash,
        Err(SessionError::MessagingNameTaken { name, owner })
            if name == "a-b-c" && owner == "a-b/c"
    ));
}

// PLAN §0.6
#[test]
fn a_session_needs_an_existing_category() {
    let store = store_with(&[]);
    let result = create_session(
        &store,
        &store,
        new_session("nada/ejecutora", Kind::Member),
        NOW,
    );
    assert!(matches!(result, Err(SessionError::Category(_))));
}

// PLAN §0.6: `set --coordinator` respects one coordinator per category
#[test]
fn promoting_a_member_fails_while_another_coordinator_exists() {
    let store = store_with(&["demo"]);
    let coordinator = add_session(&store, &store, "demo/planificadora", Kind::Coordinator);
    let child = add_session(&store, &store, "demo/ejecutora", Kind::Member);
    let category = lithemux_core::modules::category::get_category(&store, "demo").unwrap();

    assert!(matches!(
        set_kind(&store, &category, &child, Kind::Coordinator),
        Err(SessionError::CoordinatorExists { .. })
    ));
    set_kind(&store, &category, &coordinator, Kind::Member).unwrap();
    set_kind(&store, &category, &child, Kind::Coordinator).unwrap();

    let target = SessionRef::parse("demo/ejecutora").unwrap();
    assert_eq!(
        get_session(&store, &store, &target).unwrap().1.kind,
        Kind::Coordinator
    );
}

// PLAN §0.6: `set --role|--keep-alive|--model`
#[test]
fn role_keep_alive_and_model_can_be_changed() {
    let store = store_with(&["demo"]);
    let session = add_session(&store, &store, "demo/ejecutora", Kind::Member);
    set_role(&store, &session, "otro rol").unwrap();
    set_keep_alive(&store, &session, true).unwrap();
    set_model(&store, &session, Some("opus")).unwrap();
    let (_, changed) = find_session_by_id(&store, &store, &session.id)
        .unwrap()
        .unwrap();
    assert_eq!(changed.role, "otro rol");
    assert!(changed.keep_alive);
    assert_eq!(changed.model.as_deref(), Some("opus"));
    set_model(&store, &session, None).unwrap();
    let (_, cleared) = find_session_by_id(&store, &store, &session.id)
        .unwrap()
        .unwrap();
    assert_eq!(cleared.model, None);
}

// PLAN §0.6: `rm`
#[test]
fn a_removed_session_is_gone() {
    let store = store_with(&["demo"]);
    let session = add_session(&store, &store, "demo/ejecutora", Kind::Member);
    remove_session(&store, &session).unwrap();
    let target = SessionRef::parse("demo/ejecutora").unwrap();
    assert!(matches!(
        get_session(&store, &store, &target),
        Err(SessionError::NotFound(_))
    ));
}

// docs/decisiones.md: lithemux follows the conversation the tab has open
#[test]
fn a_new_conversation_id_replaces_the_stored_one_and_reports_the_old_one() {
    let store = store_with(&["demo"]);
    let session = add_session(&store, &store, "demo/ejecutora", Kind::Member);
    let old = session.claude_session_id.clone().unwrap();
    let new = "11111111-2222-4333-8444-555555555555";

    assert_eq!(
        follow_claude_session(&store, &session.id, new).unwrap(),
        Some(old)
    );
    let (_, stored) = find_session_by_id(&store, &store, &session.id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.claude_session_id.as_deref(), Some(new));
    // Reporting the same id again changes nothing.
    assert_eq!(
        follow_claude_session(&store, &session.id, new).unwrap(),
        None
    );
}

// docs/decisiones.md: only a real conversation id is followed
#[test]
fn a_malformed_conversation_id_is_ignored() {
    let store = store_with(&["demo"]);
    let session = add_session(&store, &store, "demo/ejecutora", Kind::Member);
    assert_eq!(
        follow_claude_session(&store, &session.id, "../../etc/passwd").unwrap(),
        None
    );
    let (_, stored) = find_session_by_id(&store, &store, &session.id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.claude_session_id, session.claude_session_id);
}
