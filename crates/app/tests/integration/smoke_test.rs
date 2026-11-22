//! PLAN §3.6: «Smoke test automatizado del ciclo: crear sesión → abrir →
//! hibernar → reanudar». End to end on the host, with the fake `claude`,
//! real PTYs, a real database and a clock that is given, not waited for.

use std::fs;

use lithemux_app::session_host::Phase;
use lithemux_app::sidebar::actions::SidebarActions;
use lithemux_core::modules::launch::{FlockSessionLocks, SessionLocks};
use lithemux_core::modules::session::{Profile, State};

use crate::hosted_world::{HostedWorld, attend_until, open};
use crate::processes::{is_alive, wait_until_gone};

const NOW: i64 = 1_900_000_000;
const HOUR: i64 = 3600;

#[test]
fn a_session_is_created_opened_hibernated_and_resumed_with_its_conversation() {
    let world = HostedWorld::new();
    world.steer_fake_claude("fake-claude.sleep", "60");

    // Create: as the sidebar does.
    let db = world.db();
    let actions = SidebarActions {
        db: &db,
        home: &world.home,
    };
    let (categories, _) = actions.load().unwrap();
    let (_, session) = actions
        .create_session(&categories[0].id, "humo", Profile::Claude)
        .unwrap();
    let id = session.id.clone();
    let conversation = session.claude_session_id.clone().unwrap();
    assert_eq!(world.stored(&id).state, State::New);

    // Open: a new conversation, with its id.
    let (mut host, _changes) = world.start();
    assert_eq!(open(&mut host, "humo"), Phase::Running);
    let (first, flag) = world.wait_for_run(&id, 1);
    assert_eq!(flag, "--session-id");
    // What Claude Code does on its first turn: the transcript appears.
    let project = world.claude_config().join("projects/-work");
    fs::create_dir_all(&project).unwrap();
    let transcript = project.join(format!("{conversation}.jsonl"));
    fs::write(&transcript, "{\"said\":\"the keyword is lighthouse\"}\n").unwrap();

    // Hibernate: two days idle and out of sight, by the given clock.
    world.set_idle_since(&id, NOW - 49 * HOUR);
    let nothing_visible: [String; 0] = [];
    assert_eq!(
        host.hibernate_what_is_due(&nothing_visible, NOW, 48),
        std::slice::from_ref(&id)
    );
    assert_eq!(world.stored(&id).state, State::Hibernated);
    wait_until_gone(first);
    attend_until(&mut host, "the session to be free", |host| {
        host.phase(&id).is_none()
    });
    assert_eq!(FlockSessionLocks::new(&world.home).holder(&id), None);

    // Resume: the same conversation, never a new one.
    assert_eq!(open(&mut host, "humo"), Phase::Running);
    let (second, flag) = world.wait_for_run(&id, 2);
    assert_ne!(first, second);
    assert_eq!(flag, "--resume");
    // The arguments of the last run, one per line: it resumed that very
    // conversation.
    let log = fs::read_to_string(world.home.path().join("fake-claude.log")).unwrap();
    let args: Vec<&str> = log
        .lines()
        .filter_map(|line| line.strip_prefix("arg="))
        .collect();
    let resumed = args
        .windows(2)
        .any(|pair| pair == ["--resume", conversation.as_str()]);
    assert!(resumed, "{log}");
    assert_eq!(
        fs::read_to_string(&transcript).unwrap(),
        "{\"said\":\"the keyword is lighthouse\"}\n"
    );
    assert!(is_alive(second));
    drop(host);
    assert!(!is_alive(second));
    assert_eq!(world.stored(&id).state, State::Hibernated);
}
