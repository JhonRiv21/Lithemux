//! PLAN §2.4, from its text: «Hibernar: la sesión no está visible,
//! `keep_alive=0`, `state=idle` y lleva más de `hibernate_after_hours` sin
//! actividad. Ese valor es 48 h por defecto.»
//! Adjustment of 22-nov: «una sesión en `waiting` también se hiberna cuando
//! lleva más de `hibernate_after_hours` sin ningún evento. `working` nunca
//! se hiberna. La válvula de memoria sigue sin tocar las `waiting`.»
//! «Válvula de memoria: en estado crítico, se hiberna la sesión idle no
//! visible más antigua (que no sea `keep_alive`). Como máximo una cada 60 s.»
//! Adjustment of 22-nov: «la válvula también actúa en warning, pero solo con
//! sesiones idle no visibles que lleven al menos 30 min sin actividad. En
//! crítico, sin ese mínimo.»

use crate::modules::session::domain::hibernation_policy::{
    Pressure, Running, VALVE_INTERVAL_SECONDS, VALVE_WARNING_QUIET_SECONDS, is_due_for_hibernation,
    pick_for_memory_valve,
};
use crate::modules::session::domain::kind::Kind;
use crate::modules::session::domain::profile::Profile;
use crate::modules::session::domain::session::Session;
use crate::modules::session::domain::state::State;

const NOW: i64 = 1_800_000_000;
const HOUR: i64 = 3600;
/// The default of the setting (`settings::Preferences`).
const DEFAULT_HIBERNATE_AFTER_HOURS: i64 = 48;

fn session(id: &str, state: State, idle_for_hours: i64) -> Session {
    Session {
        id: id.to_string(),
        category_id: "c1".to_string(),
        slug: id.to_string(),
        role: String::new(),
        kind: Kind::Member,
        profile: Profile::Claude,
        cwd: None,
        claude_session_id: None,
        model: None,
        keep_alive: false,
        state,
        claude_version: None,
        last_activity_at: Some(NOW - idle_for_hours * HOUR),
        created_at: 0,
        sort: 0,
        title: None,
        pin_sort: None,
        preview: None,
        unread: 0,
        notice: None,
    }
}

fn due(session: &Session, is_visible: bool) -> bool {
    let running = Running {
        session,
        is_visible,
    };
    is_due_for_hibernation(&running, NOW, DEFAULT_HIBERNATE_AFTER_HOURS)
}

#[test]
fn an_idle_hidden_session_hibernates_after_the_hours_without_activity() {
    assert!(due(&session("a", State::Idle, 49), false));
    assert!(!due(&session("a", State::Idle, 47), false));
}

// «más de»: at exactly the limit it is not yet due.
#[test]
fn exactly_the_limit_is_not_more_than_the_limit() {
    assert!(!due(&session("a", State::Idle, 48), false));
    let mut just_over = session("a", State::Idle, 48);
    just_over.last_activity_at = Some(NOW - 48 * HOUR - 1);
    assert!(due(&just_over, false));
}

#[test]
fn a_visible_session_never_hibernates() {
    assert!(!due(&session("a", State::Idle, 500), true));
}

#[test]
fn a_session_kept_alive_never_hibernates() {
    let mut kept = session("a", State::Idle, 500);
    kept.keep_alive = true;
    assert!(!due(&kept, false));
}

// «`working` nunca se hiberna»; nor what is not running at all.
#[test]
fn a_working_session_never_hibernates() {
    for state in [State::Working, State::New, State::Ended, State::Hibernated] {
        assert!(!due(&session("a", state, 500), false), "{state:?}");
    }
}

// Adjustment of 22-nov: a permission rejected with Esc leaves the session
// in `waiting` for good; without this it would never hibernate.
#[test]
fn a_session_left_waiting_hibernates_after_the_hours_too() {
    assert!(due(&session("a", State::Waiting, 49), false));
    assert!(!due(&session("a", State::Waiting, 47), false));
    assert!(!due(&session("a", State::Waiting, 500), true));
    let mut kept = session("a", State::Waiting, 500);
    kept.keep_alive = true;
    assert!(!due(&kept, false));
}

#[test]
fn the_hours_are_configurable() {
    let idle = session("a", State::Idle, 3);
    let running = Running {
        session: &idle,
        is_visible: false,
    };
    assert!(is_due_for_hibernation(&running, NOW, 2));
    assert!(!is_due_for_hibernation(&running, NOW, 4));
}

// A session that never reported activity counts from when it was made.
#[test]
fn without_recorded_activity_the_age_of_the_session_counts() {
    let mut old = session("a", State::Idle, 0);
    old.last_activity_at = None;
    old.created_at = NOW - 100 * HOUR;
    assert!(due(&old, false));
    old.created_at = NOW - HOUR;
    assert!(!due(&old, false));
}

fn valve<'a>(running: &'a [Running<'a>], last: Option<i64>) -> Option<&'a str> {
    pick_for_memory_valve(running, NOW, last, Pressure::Critical)
}

fn hidden(session: &Session) -> Running<'_> {
    Running {
        session,
        is_visible: false,
    }
}

#[test]
fn the_valve_takes_the_idle_hidden_session_that_has_been_quiet_longest() {
    let (recent, oldest, middle) = (
        session("recent", State::Idle, 1),
        session("oldest", State::Idle, 9),
        session("middle", State::Idle, 4),
    );
    let running = [&recent, &oldest, &middle].map(hidden);
    assert_eq!(valve(&running, None), Some("oldest"));
}

// «En crítico, sin ese mínimo»: memory is short now.
#[test]
fn at_critical_the_valve_does_not_wait_for_any_quiet_time() {
    let fresh = session("fresh", State::Idle, 0);
    assert_eq!(valve(&[hidden(&fresh)], None), Some("fresh"));
}

// «La válvula de memoria sigue sin tocar las `waiting`», nor anything
// visible, busy or kept alive.
#[test]
fn the_valve_never_takes_a_visible_busy_waiting_or_kept_alive_session() {
    let visible = session("visible", State::Idle, 9);
    let working = session("working", State::Working, 9);
    let waiting = session("waiting", State::Waiting, 900);
    let mut kept = session("kept", State::Idle, 9);
    kept.keep_alive = true;
    let running = [
        Running {
            session: &visible,
            is_visible: true,
        },
        hidden(&working),
        hidden(&waiting),
        hidden(&kept),
    ];
    for pressure in [Pressure::Warning, Pressure::Critical] {
        assert_eq!(pick_for_memory_valve(&running, NOW, None, pressure), None);
    }
}

// «Como máximo una cada 60 s.»
#[test]
fn the_valve_takes_at_most_one_session_every_sixty_seconds() {
    assert_eq!(VALVE_INTERVAL_SECONDS, 60);
    let idle = session("idle", State::Idle, 9);
    let running = [hidden(&idle)];
    for pressure in [Pressure::Warning, Pressure::Critical] {
        let pick = |last| pick_for_memory_valve(&running, NOW, last, pressure);
        assert_eq!(pick(Some(NOW - 59)), None);
        assert_eq!(pick(Some(NOW - 60)), Some("idle"));
        assert_eq!(pick(Some(NOW - 600)), Some("idle"));
    }
}

#[test]
fn without_pressure_the_valve_does_nothing() {
    let idle = session("idle", State::Idle, 900);
    assert_eq!(
        pick_for_memory_valve(&[hidden(&idle)], NOW, None, Pressure::Normal),
        None
    );
}

// «en warning… al menos 30 min sin actividad»
#[test]
fn at_warning_the_valve_takes_only_what_has_been_quiet_for_half_an_hour() {
    assert_eq!(VALVE_WARNING_QUIET_SECONDS, 30 * 60);
    let mut quiet = session("quiet", State::Idle, 0);
    quiet.last_activity_at = Some(NOW - 30 * 60);
    let mut recent = session("recent", State::Idle, 0);
    recent.last_activity_at = Some(NOW - 30 * 60 + 1);
    let at_warning = |running: &[Running<'_>]| {
        pick_for_memory_valve(running, NOW, None, Pressure::Warning).map(str::to_string)
    };
    assert_eq!(at_warning(&[hidden(&recent)]), None);
    assert_eq!(at_warning(&[hidden(&quiet)]).as_deref(), Some("quiet"));
    // Among the ones quiet enough, still the quietest.
    let older = session("older", State::Idle, 5);
    assert_eq!(
        at_warning(&[hidden(&quiet), hidden(&recent), hidden(&older)]).as_deref(),
        Some("older")
    );
}
