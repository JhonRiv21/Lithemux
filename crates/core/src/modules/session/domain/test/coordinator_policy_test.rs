use crate::modules::session::domain::coordinator_policy::check_can_be_coordinator;
use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::state::State;

fn coordinator(id: &str, slug: &str) -> Session {
    Session {
        id: id.to_string(),
        category_id: "cat".to_string(),
        slug: slug.to_string(),
        role: "coordinates".to_string(),
        kind: Kind::Coordinator,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: None,
        model: None,
        keep_alive: false,
        state: State::New,
        claude_version: None,
        last_activity_at: None,
        created_at: 0,
        sort: 1,
        title: None,
        pin_sort: None,
        preview: None,
        unread: 0,
        notice: None,
    }
}

// PLAN §0.2: a category may have no coordinator
#[test]
fn a_category_without_coordinator_accepts_one() {
    assert_eq!(check_can_be_coordinator(None, None), Ok(()));
    assert_eq!(check_can_be_coordinator(None, Some("s1")), Ok(()));
}

// PLAN §0.2: one_coordinator_per_category
#[test]
fn only_one_coordinator_per_category() {
    let current = coordinator("m1", "planificadora");
    assert_eq!(
        check_can_be_coordinator(Some(&current), None),
        Err("planificadora".to_string())
    );
    assert_eq!(
        check_can_be_coordinator(Some(&current), Some("other")),
        Err("planificadora".to_string())
    );
}

// PLAN §0.6: `set --coordinator` on the session that already is the coordinator
#[test]
fn the_current_coordinator_can_be_confirmed_as_coordinator() {
    let current = coordinator("m1", "planificadora");
    assert_eq!(check_can_be_coordinator(Some(&current), Some("m1")), Ok(()));
}
