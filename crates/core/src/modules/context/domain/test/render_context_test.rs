use std::path::Path;

use crate::modules::context::domain::context_input::{ContextInput, MAX_CONTEXT_CHARS};
use crate::modules::context::domain::lang::Lang;
use crate::modules::context::domain::render_context::render_context;
use crate::modules::context::domain::test::fixtures::{
    BOARD, NOW, category, documenter, executor, planner, session,
};
use crate::modules::session::{Kind, Profile, Session};

fn render(me: &Session, others: &[Session], lang: Lang) -> String {
    render_context(&ContextInput {
        category: &category(),
        me,
        others,
        board: Path::new(BOARD),
        now: NOW,
        lang,
    })
}

// PLAN §0.5: mother
#[test]
fn mother_context_in_english() {
    insta::assert_snapshot!(render(&planner(), &[executor(), documenter()], Lang::En));
}

// PLAN §0.5: mother
#[test]
fn mother_context_in_spanish() {
    insta::assert_snapshot!(render(&planner(), &[executor(), documenter()], Lang::Es));
}

// PLAN §0.5: child
#[test]
fn child_context_in_english() {
    insta::assert_snapshot!(render(&executor(), &[planner(), documenter()], Lang::En));
}

// PLAN §0.5: child (the plan's reference example)
#[test]
fn child_context_in_spanish() {
    insta::assert_snapshot!(render(&executor(), &[planner(), documenter()], Lang::Es));
}

// PLAN §0.5: without mother
#[test]
fn motherless_context_in_english() {
    insta::assert_snapshot!(render(&executor(), &[documenter()], Lang::En));
}

// PLAN §0.5: without mother
#[test]
fn motherless_context_in_spanish() {
    insta::assert_snapshot!(render(&executor(), &[documenter()], Lang::Es));
}

// PLAN §0.5: what a mother is told
#[test]
fn the_mother_learns_her_children_her_duty_and_the_board() {
    let text = render(&planner(), &[executor(), documenter()], Lang::Es);
    assert!(text.contains("«Selektal»"));
    assert!(text.contains("/Users/x/Linktic/RecruitLine"));
    assert!(text.contains("selektal-planificadora"));
    assert!(text.contains("planifica, revisa y redacta prompts"));
    assert!(text.contains("selektal-ejecutora"));
    assert!(text.contains("selektal-documental"));
    assert!(text.contains("hibernada"));
    assert!(text.contains("hace 3 días"));
    assert!(text.contains("coordina la categoría"));
    assert!(text.contains("única que escribe el tablero"));
    assert!(text.contains(BOARD));
    assert!(text.contains("`lithemux status selektal`"));
}

// PLAN §0.5: what a child is told
#[test]
fn the_child_learns_her_mother_her_sisters_and_to_report() {
    let text = render(&executor(), &[planner(), documenter()], Lang::Es);
    assert!(text.contains("Nombre de esta sesión: selektal-ejecutora"));
    assert!(text.contains("Madre de la categoría: selektal-planificadora"));
    assert!(text.contains("planifica, revisa y redacta prompts"));
    assert!(text.contains("Hermanas:"));
    assert!(text.contains("selektal-documental"));
    assert!(text.contains("reporta a la madre"));
    assert!(text.contains("fuera de su rol pasa por la madre"));
    assert!(text.contains("solo lectura"));
    assert!(text.contains(BOARD));
    assert!(text.contains("`lithemux status selektal`"));
    // The mother is named as mother, not listed again among the sisters.
    assert_eq!(text.matches("selektal-planificadora (").count(), 1);
}

// PLAN §0.5: without mother the sessions are sisters that coordinate directly
#[test]
fn without_mother_the_sessions_are_sisters() {
    let text = render(&executor(), &[documenter()], Lang::Es);
    assert!(text.contains("no tiene madre"));
    assert!(text.contains("coordinan directamente entre sí"));
    assert!(text.contains("selektal-documental"));
    assert!(!text.contains("reporta a la madre"));
}

// PLAN §0.5: short, under 2,000 characters, even in a crowded category
#[test]
fn the_context_stays_under_two_thousand_characters() {
    let long_role = "hace muchas cosas distintas ".repeat(20);
    let sisters: Vec<Session> = (0..40)
        .map(|n| session(&format!("hija-{n}"), &long_role, Kind::Child))
        .chain([planner()])
        .collect();
    for lang in [Lang::En, Lang::Es] {
        let text = render(&executor(), &sisters, lang);
        assert!(text.chars().count() < MAX_CONTEXT_CHARS);
        assert!(text.contains("selektal-planificadora"));
        assert!(text.contains("`lithemux status selektal`"));
    }
}

// PLAN §0.3: a shell session has no hooks, so it cannot be messaged
#[test]
fn a_shell_sister_is_marked_as_not_reachable() {
    let mut terminal = session("term", "terminal libre", Kind::Child);
    terminal.profile = Profile::Shell;
    let text = render(&executor(), &[planner(), terminal], Lang::En);
    assert!(text.contains("selektal-term (shell terminal, not reachable with SendMessage"));
}
