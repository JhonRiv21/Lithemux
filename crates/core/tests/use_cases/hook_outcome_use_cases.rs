use lithemux_core::modules::hook_event::HookOutcome;
use lithemux_core::modules::session::State;

use crate::support::hook_world::World;
use crate::support::sample_data::NOW;

fn told(world: &World) -> HookOutcome {
    world.notifier.outcomes.borrow().last().copied().unwrap()
}

const PERMISSION: &str =
    r#"{"notification_type":"permission_prompt","message":"Claude needs your permission"}"#;

// PLAN §3.1: the app hears the state an event moved the session into, and only when it moved it
#[test]
fn the_app_is_told_the_state_an_event_moved_the_session_into() {
    let world = World::new();
    world.fire("session-start", r#"{"source":"startup"}"#, NOW);
    assert_eq!(told(&world).entered, Some(State::Idle));
    world.fire("prompt-submit", r#"{"prompt":"go"}"#, NOW);
    assert_eq!(told(&world).entered, Some(State::Working));

    world.fire("notification", PERMISSION, NOW);
    assert_eq!(told(&world).entered, Some(State::Waiting));
    for kind in ["permission_prompt", "agent_completed", "auth_success"] {
        let still = format!(r#"{{"notification_type":"{kind}","message":"x"}}"#);
        world.fire("notification", &still, NOW);
        assert_eq!(told(&world).entered, None, "{kind} while waiting");
    }
    world.fire("post-tool-use", "{}", NOW);
    assert_eq!(told(&world).entered, Some(State::Working));
    world.fire("post-tool-use", "{}", NOW);
    assert_eq!(told(&world).entered, None);
}
