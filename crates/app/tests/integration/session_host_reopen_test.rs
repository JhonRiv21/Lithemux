use lithemux_app::session_host::{Opened, Phase};
use lithemux_core::modules::session::Profile;

use crate::hosted_world::{HostedWorld, attend_until, run, target};
use crate::processes::wait_until_gone;
use crate::support::SIZE;

/// Lets the host work until the session runs or cannot, and returns every
/// phase it went through on the way.
fn phases_until_settled(
    host: &mut lithemux_app::session_host::SessionHost,
    id: &str,
) -> Vec<Option<Phase>> {
    let mut seen: Vec<Option<Phase>> = Vec::new();
    attend_until(host, "the session to run or fail", |host| {
        let phase = host.phase(id).cloned();
        if seen.last() != Some(&phase) {
            seen.push(phase.clone());
        }
        matches!(
            phase,
            Some(Phase::Running | Phase::OpenElsewhere { .. } | Phase::Failed(_))
        )
    });
    seen
}

// PLAN §2.2 (carried from 2.1): reopening a session while its program is
// still leaving shows it as closing and reopens it by itself; it is never
// reported as open in another terminal, with the pid of our own child.
#[test]
fn a_session_reopened_while_its_program_is_leaving_waits_and_opens_by_itself() {
    let world = HostedWorld::new();
    world.add_session("lenta", Profile::Claude);
    // It ignores the hangup: it is gone only after the grace and the kill.
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "lenta");
    let (first_pid, _) = world.wait_for_run(&id, 1);

    host.close(&id);
    assert_eq!(host.phase(&id), Some(&Phase::Closing));
    let asked = host.open(&target("lenta"), SIZE).unwrap();
    assert_eq!(
        asked,
        Opened::WhenClosed {
            session_id: id.clone()
        }
    );
    let seen = phases_until_settled(&mut host, &id);

    assert_eq!(
        seen,
        [
            Some(Phase::Closing),
            Some(Phase::Opening),
            Some(Phase::Running)
        ]
    );
    let (second_pid, _) = world.wait_for_run(&id, 2);
    assert_ne!(first_pid, second_pid);
    wait_until_gone(first_pid);
}

// PLAN §2.2 (carried from 2.1): the same while its launch was still being
// prepared, which holds the lock too.
#[test]
fn a_session_closed_and_reopened_while_it_was_opening_opens_once() {
    let world = HostedWorld::new();
    world.add_session("rapida", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    let (mut host, _changes) = world.start();

    let asked = host.open(&target("rapida"), SIZE).unwrap();
    let id = asked.session_id().to_string();
    host.close(&id);
    host.open(&target("rapida"), SIZE).unwrap();
    let seen = phases_until_settled(&mut host, &id);

    assert_eq!(seen.last(), Some(&Some(Phase::Running)), "{seen:?}");
    assert!(
        !seen.contains(&Some(Phase::OpenElsewhere { pid: None }))
            && seen
                .iter()
                .all(|phase| !matches!(phase, Some(Phase::OpenElsewhere { .. }))),
        "{seen:?}"
    );
    world.wait_for_run(&id, 1);
    // A second launch, from the one that was dropped, would come late:
    // the host is given time to make that mistake.
    let until = std::time::Instant::now() + std::time::Duration::from_millis(600);
    while std::time::Instant::now() < until {
        host.process_pending();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(world.fake_claude_runs().len(), 1);
    assert_eq!(host.phase(&id), Some(&Phase::Running));
}

// PLAN §2.2 (carried from 2.1): closing shows as closing until the program
// is gone, and then the session is simply closed.
#[test]
fn a_closed_session_is_closing_until_its_program_is_gone() {
    let world = HostedWorld::new();
    world.add_session("lenta", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "lenta");
    let (pid, _) = world.wait_for_run(&id, 1);

    host.close(&id);
    assert_eq!(host.phase(&id), Some(&Phase::Closing));
    attend_until(&mut host, "the session to be closed", |host| {
        host.phase(&id).is_none()
    });
    wait_until_gone(pid);
}

// PLAN §2.2 (carried from 2.1): changing one's mind about reopening.
#[test]
fn closing_again_a_session_that_waited_to_reopen_leaves_it_closed() {
    let world = HostedWorld::new();
    world.add_session("lenta", Profile::Claude);
    world.steer_fake_claude("fake-claude.sleep", "30");
    world.steer_fake_claude("fake-claude.ignore-hup", "");
    let (mut host, _changes) = world.start();
    let id = run(&mut host, &world, "lenta");
    world.wait_for_run(&id, 1);

    host.close(&id);
    host.open(&target("lenta"), SIZE).unwrap();
    host.close(&id);
    attend_until(&mut host, "the session to be closed", |host| {
        host.phase(&id).is_none()
    });
    assert_eq!(world.fake_claude_runs().len(), 1);
}
