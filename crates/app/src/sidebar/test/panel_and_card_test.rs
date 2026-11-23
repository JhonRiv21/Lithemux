use std::collections::HashMap;

use lithemux_core::modules::category::Category;
use lithemux_core::modules::session::{Kind, Profile, Session, State};

use crate::app_shell::elsewhere::shown_phase;
use crate::session_host::Phase;
use crate::sidebar::status::{Status, session_status};
use crate::sidebar::tree::{Board, build};

fn hibernated_session() -> Session {
    Session {
        id: "s-api".to_string(),
        category_id: "c-demo".to_string(),
        slug: "api".to_string(),
        role: String::new(),
        kind: Kind::Member,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: Some("conversation".to_string()),
        model: None,
        keep_alive: false,
        state: State::Hibernated,
        claude_version: None,
        last_activity_at: None,
        created_at: 0,
        sort: 0,
        title: None,
        pin_sort: None,
        preview: None,
        unread: 0,
        notice: None,
    }
}

fn card_status(session: &Session, phases: &dyn Fn(&str) -> Option<Phase>) -> Status {
    let category = Category {
        id: "c-demo".to_string(),
        slug: "demo".to_string(),
        name: "Demo".to_string(),
        root_dir: "/work/demo".to_string(),
        color: None,
        sort: 1,
        created_at: 0,
        collapsed: false,
        notify: true,
    };
    let tree = build(&Board {
        categories: &[category],
        sessions: std::slice::from_ref(session),
        phases,
        ports: &HashMap::new(),
        memory: &HashMap::new(),
        home: None,
    });
    tree.groups[0].cards[0].status.clone()
}

// Carried over from 3B (ronda 3C): the tab of a panel says what the card
// says. Once the process that had the session open elsewhere is gone, both
// go back to what the database has, without pressing Open.
#[test]
fn the_panel_tab_and_the_card_agree_when_open_elsewhere_ends() {
    let session = hibernated_session();
    let told = Phase::OpenElsewhere { pid: Some(7) };
    for holder_runs in [true, false] {
        let phases = |_: &str| shown_phase(Some(&told), None, holder_runs);

        let card = card_status(&session, &phases);
        let tab = session_status(&phases, &session);

        assert_eq!(tab, card, "holder runs: {holder_runs}");
        let elsewhere = matches!(card, Status::OpenElsewhere(_));
        assert_eq!(elsewhere, holder_runs, "{card:?}");
    }
}
