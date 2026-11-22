//! PLAN §3.1 (22-nov): a session of a category with a coordinator reports
//! to it with Claude Code's own messaging; the coordinator hands out tasks
//! asking to be told when a session is free again.

use std::path::Path;

use crate::modules::context::domain::context_input::ContextInput;
use crate::modules::context::domain::lang::Lang;
use crate::modules::context::domain::render_context::render_context;
use crate::modules::context::domain::test::fixtures::{
    BOARD, NOW, category, documenter, executor, planner,
};
use crate::modules::session::Session;

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

// b) its coordinator's address; report with SendMessage what it did, what is
// left and where the details are, and stop; if the message cannot be
// delivered, say so to the user.
#[test]
fn a_session_reports_to_its_coordinator_by_message_and_stops() {
    let english = render(&executor(), &[planner(), documenter()], Lang::En);
    for said in [
        r#"reports to the coordinator with SendMessage (to: "selektal-planificadora")"#,
        "what it did, what is left and where the details are",
        "then it stops",
        "If the message cannot be delivered",
        "tells the user",
    ] {
        assert!(english.contains(said), "{said}\n{english}");
    }
    let spanish = render(&executor(), &[planner(), documenter()], Lang::Es);
    for said in [
        r#"reporta a la coordinadora con SendMessage (a: "selektal-planificadora")"#,
        "qué hizo, qué quedó pendiente y dónde está el detalle",
        "y se detiene",
        "Si el envío falla",
        "se lo dice al usuario",
    ] {
        assert!(spanish.contains(said), "{said}\n{spanish}");
    }
    assert!(!english.contains("notify_when_idle"));
}

// c) the addresses of its sessions, and how to hand out a task.
#[test]
fn the_coordinator_hands_out_tasks_asking_to_hear_when_a_session_is_free() {
    for lang in [Lang::En, Lang::Es] {
        let text = render(&planner(), &[executor(), documenter()], lang);
        assert!(text.contains("selektal-ejecutora"), "{text}");
        assert!(text.contains("selektal-documental"), "{text}");
        assert!(text.contains("SendMessage"), "{text}");
        assert!(text.contains("notify_when_idle: true"), "{text}");
        assert!(
            !text.contains("(to: \"") && !text.contains("(a: \""),
            "{text}"
        );
    }
}

// d) without a coordinator, none of it.
#[test]
fn without_a_coordinator_nobody_reports_nor_hands_out_tasks() {
    for lang in [Lang::En, Lang::Es] {
        let text = render(&executor(), &[documenter()], lang);
        assert!(!text.contains("notify_when_idle"), "{text}");
        assert!(
            !text.contains("(to: \"") && !text.contains("(a: \""),
            "{text}"
        );
    }
}
