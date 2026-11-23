use lithemux_core::modules::category::{Category, CategoryRepository};
use lithemux_core::modules::session::{
    Kind, Profile, Session, SessionError, SessionRepository, State,
};

fn category(id: &str, slug: &str) -> Category {
    Category {
        id: id.to_string(),
        slug: slug.to_string(),
        name: slug.to_uppercase(),
        root_dir: format!("/work/{slug}"),
        color: None,
        sort: 1,
        created_at: 10,
        collapsed: false,
        notify: true,
    }
}

fn session(id: &str, category_id: &str, slug: &str, kind: Kind, sort: i64) -> Session {
    Session {
        id: id.to_string(),
        category_id: category_id.to_string(),
        slug: slug.to_string(),
        role: format!("role of {slug}"),
        kind,
        profile: Profile::Claude,
        cwd: Some("/work/elsewhere".to_string()),
        claude_session_id: Some(format!("claude-{id}")),
        model: Some("opus".to_string()),
        keep_alive: false,
        state: State::New,
        claude_version: None,
        last_activity_at: None,
        created_at: 20,
        sort,
        title: None,
        pin_sort: None,
        preview: None,
        unread: 0,
        notice: None,
    }
}

/// What every SessionRepository must do. PLAN §0.2 and §0.4.
pub fn session_repository_contract(
    categories: &dyn CategoryRepository,
    sessions: &dyn SessionRepository,
) {
    categories.insert(&category("c1", "demo")).unwrap();
    categories.insert(&category("c2", "selektal")).unwrap();
    assert_eq!(sessions.next_sort("c1").unwrap(), 1);

    let child = session("s2", "c1", "ejecutora", Kind::Member, 2);
    let coordinator = session("s1", "c1", "planificadora", Kind::Coordinator, 1);
    let other = session("s3", "c2", "ejecutora", Kind::Member, 1);
    for s in [&child, &coordinator, &other] {
        sessions.insert(s).unwrap();
    }

    // Round trip, ordering and lookups.
    assert_eq!(
        sessions.list_in("c1").unwrap(),
        vec![coordinator.clone(), child.clone()]
    );
    assert_eq!(sessions.next_sort("c1").unwrap(), 3);
    // All of them at once: each category's together, in its own order.
    assert_eq!(
        sessions.list_all().unwrap(),
        vec![coordinator.clone(), child.clone(), other.clone()]
    );
    assert_eq!(
        sessions.find("c1", "ejecutora").unwrap(),
        Some(child.clone())
    );
    assert_eq!(
        sessions.find("c2", "ejecutora").unwrap(),
        Some(other.clone())
    );
    assert_eq!(sessions.find("c2", "planificadora").unwrap(), None);
    assert_eq!(
        sessions.find_by_id("s1").unwrap(),
        Some(coordinator.clone())
    );
    assert_eq!(sessions.find_coordinator("c1").unwrap(), Some(coordinator));
    assert_eq!(sessions.find_coordinator("c2").unwrap(), None);

    // Field updates.
    sessions.update_role("s2", "new role").unwrap();
    sessions.update_kind("s3", Kind::Coordinator).unwrap();
    sessions.update_keep_alive("s2", true).unwrap();
    sessions.update_model("s2", None).unwrap();
    sessions
        .update_claude_version("s2", Some("2.1.330"))
        .unwrap();
    sessions
        .update_claude_session_id("s2", "claude-new")
        .unwrap();
    let updated = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!(updated.role, "new role");
    assert!(updated.keep_alive);
    assert_eq!(updated.model, None);
    assert_eq!(updated.claude_version.as_deref(), Some("2.1.330"));
    assert_eq!(updated.claude_session_id.as_deref(), Some("claude-new"));
    sessions.update_claude_version("s2", None).unwrap();
    assert_eq!(
        sessions.find_by_id("s2").unwrap().unwrap().claude_version,
        None
    );
    assert_eq!(sessions.find_coordinator("c2").unwrap().unwrap().id, "s3");

    // State: activity is kept when not given, and `unless` protects a state.
    sessions
        .update_state("s2", State::Working, Some(100), None)
        .unwrap();
    sessions
        .update_state("s2", State::Waiting, None, None)
        .unwrap();
    let waiting = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!(
        (waiting.state, waiting.last_activity_at),
        (State::Waiting, Some(100))
    );
    sessions
        .update_state("s2", State::Hibernated, None, None)
        .unwrap();
    sessions
        .update_state("s2", State::Ended, Some(999), Some(State::Hibernated))
        .unwrap();
    let kept = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!(
        (kept.state, kept.last_activity_at),
        (State::Hibernated, Some(100))
    );
    sessions
        .update_state("s1", State::Ended, None, Some(State::Hibernated))
        .unwrap();
    assert_eq!(
        sessions.find_by_id("s1").unwrap().unwrap().state,
        State::Ended
    );
    sessions
        .update_state("missing", State::Idle, Some(1), None)
        .unwrap();

    // Activity alone leaves the state where it was.
    sessions.touch_activity("s2", 500).unwrap();
    let s2 = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!(
        (s2.state, s2.last_activity_at),
        (State::Hibernated, Some(500))
    );
    sessions.touch_activity("missing", 1).unwrap();

    // A conditional move applies only from the named state.
    sessions
        .move_state("s2", State::Waiting, State::Working)
        .unwrap();
    assert_eq!(
        sessions.find_by_id("s2").unwrap().unwrap().state,
        State::Hibernated
    );
    sessions
        .update_state("s2", State::Waiting, None, None)
        .unwrap();
    sessions
        .move_state("s2", State::Waiting, State::Working)
        .unwrap();
    let s2 = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!((s2.state, s2.last_activity_at), (State::Working, Some(500)));
    sessions
        .move_state("missing", State::Waiting, State::Working)
        .unwrap();

    // The card: title, place, pins, preview and unread count (PLAN §2.2).
    sessions.update_title("s2", Some("the api")).unwrap();
    assert_eq!(
        sessions.find_by_id("s2").unwrap().unwrap().title.as_deref(),
        Some("the api")
    );
    sessions.update_title("s2", None).unwrap();
    assert_eq!(sessions.find_by_id("s2").unwrap().unwrap().title, None);

    sessions.update_sort("s2", 1).unwrap();
    sessions.update_sort("s1", 2).unwrap();
    let ids = |list: Vec<Session>| list.into_iter().map(|s| s.id).collect::<Vec<_>>();
    assert_eq!(ids(sessions.list_in("c1").unwrap()), ["s2", "s1"]);

    assert!(sessions.list_pinned().unwrap().is_empty());
    sessions.update_pin_sort("s3", Some(2)).unwrap();
    sessions.update_pin_sort("s1", Some(1)).unwrap();
    assert_eq!(ids(sessions.list_pinned().unwrap()), ["s1", "s3"]);
    sessions.update_pin_sort("s1", None).unwrap();
    assert_eq!(ids(sessions.list_pinned().unwrap()), ["s3"]);

    sessions.note_message("s2", Some("first"), true).unwrap();
    sessions.note_message("s2", None, true).unwrap();
    sessions.note_message("s2", Some("second"), false).unwrap();
    sessions.note_message("missing", Some("x"), true).unwrap();
    let noted = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!(
        (noted.preview.as_deref(), noted.unread),
        (Some("second"), 2)
    );
    sessions.clear_unread("s2").unwrap();
    let read = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!((read.preview.as_deref(), read.unread), (Some("second"), 0));

    // Deleting a session, then a category with its sessions.
    sessions.delete("s1").unwrap();
    assert_eq!(sessions.list_in("c1").unwrap().len(), 1);
    categories.delete("c1").unwrap();
    assert!(sessions.list_in("c1").unwrap().is_empty());
    assert_eq!(sessions.find_by_id("s2").unwrap(), None);
    assert_eq!(sessions.list_in("c2").unwrap().len(), 1);
}

/// One conversation, one session: two sessions resuming the same
/// conversation lose messages (migration 6).
pub fn one_session_per_conversation_contract(
    categories: &dyn CategoryRepository,
    sessions: &dyn SessionRepository,
) {
    categories.insert(&category("k1", "kept")).unwrap();
    let first = session("t1", "k1", "first", Kind::Member, 1);
    sessions.insert(&first).unwrap();
    let mut twin = session("t2", "k1", "twin", Kind::Member, 2);
    twin.claude_session_id = first.claude_session_id.clone();

    let refused = sessions.insert(&twin);

    assert!(
        matches!(refused, Err(SessionError::ConversationTaken(ref id)) if Some(id) == first.claude_session_id.as_ref()),
        "{refused:?}"
    );
    assert_eq!(sessions.list_in("k1").unwrap(), vec![first]);
    // Shells have no conversation: any number of them.
    for (id, slug) in [("t3", "shell-a"), ("t4", "shell-b")] {
        let mut shell = session(id, "k1", slug, Kind::Member, 3);
        shell.profile = Profile::Shell;
        shell.claude_session_id = None;
        sessions.insert(&shell).unwrap();
    }
}
