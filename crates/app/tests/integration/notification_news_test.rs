//! PLAN §3.1 through the real path: the hook writes the state and tells the
//! app's socket, the host hears it, and the policy decides on the news and
//! on the session as the hook left it.

use std::time::{Duration, Instant};

use lithemux_app::notifications::policy::{Seen, news_rings};
use lithemux_app::session_host::{Phase, SessionHost};
use lithemux_core::modules::session::Profile;

use crate::hosted_world::{HostedWorld, run};

const PATIENCE: Duration = Duration::from_secs(5);

/// Fires one event and returns whether each piece of news it brought rings,
/// for a session out of view in a category that wants the sound.
fn hear(
    world: &HostedWorld,
    host: &mut SessionHost,
    id: &str,
    event: &str,
    stdin: &str,
) -> Vec<bool> {
    world.fire_hook(id, event, stdin);
    let deadline = Instant::now() + PATIENCE;
    let mut news = Vec::new();
    while news.is_empty() {
        assert!(
            Instant::now() < deadline,
            "the news of {event} never arrived"
        );
        host.process_pending();
        news = host.take_news();
        std::thread::sleep(Duration::from_millis(2));
    }
    let category = world.category();
    let seen = Seen {
        category: Some(&category),
        is_in_view: false,
        runs_here: host.phase(id) == Some(&Phase::Running),
    };
    news.iter().map(|news| news_rings(news, &seen)).collect()
}

fn a_running_session(world: &HostedWorld, host: &mut SessionHost) -> String {
    world.add_session("ejecutora", Profile::Claude);
    let id = run(host, world, "ejecutora");
    hear(world, host, &id, "session-start", r#"{"source":"startup"}"#);
    id
}

// PLAN §3.1: a turn that ends rings; waiting for the user does not, and
// neither does a second `Stop` that finds the session already idle.
#[test]
fn only_the_end_of_a_turn_rings() {
    let world = HostedWorld::new();
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();
    let id = a_running_session(&world, &mut host);

    assert_eq!(
        hear(
            &world,
            &mut host,
            &id,
            "prompt-submit",
            r#"{"prompt":"go"}"#
        ),
        [false]
    );
    let permission = r#"{"notification_type":"permission_prompt","message":"May I?"}"#;
    assert_eq!(
        hear(&world, &mut host, &id, "notification", permission),
        [false]
    );
    assert_eq!(hear(&world, &mut host, &id, "post-tool-use", "{}"), [false]);
    let done = r#"{"last_assistant_message":"Done."}"#;
    assert_eq!(hear(&world, &mut host, &id, "stop", done), [true]);
    assert_eq!(hear(&world, &mut host, &id, "stop", done), [false]);
}

// PLAN §3.1: a session whose program runs in a terminal of its own is not
// the app's to ring for.
#[test]
fn a_session_running_outside_the_app_does_not_ring() {
    let world = HostedWorld::new();
    let (mut host, _changes) = world.start();
    let outside = world.add_session("afuera", Profile::Claude).id;

    hear(
        &world,
        &mut host,
        &outside,
        "session-start",
        r#"{"source":"startup"}"#,
    );
    hear(
        &world,
        &mut host,
        &outside,
        "prompt-submit",
        r#"{"prompt":"go"}"#,
    );
    let done = r#"{"last_assistant_message":"Done."}"#;
    assert_eq!(hear(&world, &mut host, &outside, "stop", done), [false]);
}

// B6: news nobody reads (the app without a window reads none) do not pile
// up for ever; the newest are the ones kept.
#[test]
fn news_nobody_takes_are_kept_only_up_to_a_limit() {
    let world = HostedWorld::new();
    let (mut host, _changes) = world.start();
    let id = world.add_session("callada", Profile::Claude).id;
    let fired = 300;
    for turn in 0..fired {
        let said = format!(r#"{{"last_assistant_message":"turn {turn}"}}"#);
        world.fire_hook(&id, "stop", &said);
    }
    // The socket hands each message over on a thread of its own; a second
    // of attending is plenty for all of them to be in.
    let until = Instant::now() + Duration::from_secs(1);
    while Instant::now() < until {
        host.process_pending();
        std::thread::sleep(Duration::from_millis(5));
    }
    let kept = host.take_news();
    assert!(!kept.is_empty());
    assert!(kept.len() <= 100, "{} news kept", kept.len());
}
