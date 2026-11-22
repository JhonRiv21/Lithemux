use lithemux_core::modules::category::{Category, CategoryRepository};
use lithemux_core::modules::session::{Kind, Profile, Session, SessionRepository, State};

fn category(id: &str, slug: &str) -> Category {
    Category {
        id: id.to_string(),
        slug: slug.to_string(),
        name: slug.to_uppercase(),
        root_dir: format!("/work/{slug}"),
        color: None,
        sort: 1,
        created_at: 10,
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

    let child = session("s2", "c1", "ejecutora", Kind::Child, 2);
    let mother = session("s1", "c1", "planificadora", Kind::Mother, 1);
    let other = session("s3", "c2", "ejecutora", Kind::Child, 1);
    for s in [&child, &mother, &other] {
        sessions.insert(s).unwrap();
    }

    // Round trip, ordering and lookups.
    assert_eq!(
        sessions.list_in("c1").unwrap(),
        vec![mother.clone(), child.clone()]
    );
    assert_eq!(sessions.next_sort("c1").unwrap(), 3);
    assert_eq!(
        sessions.find("c1", "ejecutora").unwrap(),
        Some(child.clone())
    );
    assert_eq!(
        sessions.find("c2", "ejecutora").unwrap(),
        Some(other.clone())
    );
    assert_eq!(sessions.find("c2", "planificadora").unwrap(), None);
    assert_eq!(sessions.find_by_id("s1").unwrap(), Some(mother.clone()));
    assert_eq!(sessions.find_mother("c1").unwrap(), Some(mother));
    assert_eq!(sessions.find_mother("c2").unwrap(), None);

    // Field updates.
    sessions.update_role("s2", "new role").unwrap();
    sessions.update_kind("s3", Kind::Mother).unwrap();
    sessions.update_keep_alive("s2", true).unwrap();
    sessions.update_model("s2", None).unwrap();
    sessions.update_claude_version("s2", "2.1.330").unwrap();
    sessions
        .update_claude_session_id("s2", "claude-new")
        .unwrap();
    let updated = sessions.find_by_id("s2").unwrap().unwrap();
    assert_eq!(updated.role, "new role");
    assert!(updated.keep_alive);
    assert_eq!(updated.model, None);
    assert_eq!(updated.claude_version.as_deref(), Some("2.1.330"));
    assert_eq!(updated.claude_session_id.as_deref(), Some("claude-new"));
    assert_eq!(sessions.find_mother("c2").unwrap().unwrap().id, "s3");

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

    // Deleting a session, then a category with its sessions.
    sessions.delete("s1").unwrap();
    assert_eq!(sessions.list_in("c1").unwrap().len(), 1);
    categories.delete("c1").unwrap();
    assert!(sessions.list_in("c1").unwrap().is_empty());
    assert_eq!(sessions.find_by_id("s2").unwrap(), None);
    assert_eq!(sessions.list_in("c2").unwrap().len(), 1);
}
