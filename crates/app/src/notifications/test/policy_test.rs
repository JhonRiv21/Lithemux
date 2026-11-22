//! PLAN §3.1 (Jhon, 22-nov): «un sonido simple al terminar una tarea»,
//! only for a session that runs in the app and is not in view («a la
//! vista»: the tab on show in a pane, with the window active). The switch
//! is per category. At most one sound every 2 s.

use std::time::{Duration, Instant};

use lithemux_core::modules::hook_event::HookEvent;
use lithemux_core::modules::session::State;

use crate::notifications::policy::{QUIET_AFTER_A_SOUND, Situation, SoundGate, rings};

/// A turn that just ended, out of view, in a category that rings, here.
fn ended() -> Situation {
    Situation {
        entered: Some(State::Idle),
        is_in_view: false,
        category_rings: true,
        runs_here: true,
    }
}

#[test]
fn a_turn_that_ends_out_of_view_rings() {
    assert!(rings(Some(HookEvent::Stop), &ended()));
}

#[test]
fn in_view_a_silent_category_or_running_elsewhere_never_rings() {
    let cases = [
        Situation {
            is_in_view: true,
            ..ended()
        },
        Situation {
            category_rings: false,
            ..ended()
        },
        Situation {
            runs_here: false,
            ..ended()
        },
    ];
    for (case, situation) in cases.iter().enumerate() {
        assert!(!rings(Some(HookEvent::Stop), situation), "case {case}");
    }
}

// A `Stop` that changed nothing (the session was already idle, or was
// hibernated on purpose) is not a turn that ends.
#[test]
fn a_stop_that_moved_nothing_does_not_ring() {
    assert!(!rings(
        Some(HookEvent::Stop),
        &Situation {
            entered: None,
            ..ended()
        }
    ));
}

// «waiting no da aviso. El contador de la tarjeta se queda.»
#[test]
fn nothing_but_a_finished_turn_rings() {
    let waiting = Situation {
        entered: Some(State::Waiting),
        ..ended()
    };
    assert!(!rings(Some(HookEvent::Notification), &waiting));
    for event in [
        HookEvent::SessionStart,
        HookEvent::PromptSubmit,
        HookEvent::Notification,
        HookEvent::SessionEnd,
        HookEvent::PostToolUse,
        HookEvent::PostToolUseFailure,
    ] {
        assert!(!rings(Some(event), &ended()), "{event:?}");
    }
    assert!(!rings(None, &ended()));
}

#[test]
fn sessions_that_end_together_ring_once_every_two_seconds() {
    let mut gate = SoundGate::default();
    let start = Instant::now();
    assert!(gate.may_ring(start));
    assert!(!gate.may_ring(start));
    assert!(!gate.may_ring(start + Duration::from_millis(1999)));
    assert!(gate.may_ring(start + QUIET_AFTER_A_SOUND));
    assert!(!gate.may_ring(start + QUIET_AFTER_A_SOUND + Duration::from_millis(10)));
}
