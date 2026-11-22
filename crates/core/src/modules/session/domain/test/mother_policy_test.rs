use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::mother_policy::check_can_be_mother;
use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::state::State;

fn mother(id: &str, slug: &str) -> Session {
    Session {
        id: id.to_string(),
        category_id: "cat".to_string(),
        slug: slug.to_string(),
        role: "coordinates".to_string(),
        kind: Kind::Mother,
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
    }
}

// PLAN §0.2: a category may have no mother
#[test]
fn a_category_without_mother_accepts_one() {
    assert_eq!(check_can_be_mother(None, None), Ok(()));
    assert_eq!(check_can_be_mother(None, Some("s1")), Ok(()));
}

// PLAN §0.2: one_mother_per_category
#[test]
fn only_one_mother_per_category() {
    let current = mother("m1", "planificadora");
    assert_eq!(
        check_can_be_mother(Some(&current), None),
        Err("planificadora".to_string())
    );
    assert_eq!(
        check_can_be_mother(Some(&current), Some("other")),
        Err("planificadora".to_string())
    );
}

// PLAN §0.6: `set --mother` on the session that already is the mother
#[test]
fn the_current_mother_can_be_confirmed_as_mother() {
    let current = mother("m1", "planificadora");
    assert_eq!(check_can_be_mother(Some(&current), Some("m1")), Ok(()));
}
