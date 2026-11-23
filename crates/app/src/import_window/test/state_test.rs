use lithemux_core::modules::category::Category;
use lithemux_core::modules::import::{Destination, NewCategoryProposal, Proposal, ProposedSession};

use crate::import_window::state::ImportForm;

fn session(
    number: usize,
    folder: &str,
    selected: bool,
    destination: Destination,
) -> ProposedSession {
    ProposedSession {
        number,
        name: format!("s{number}"),
        slug: format!("s{number}"),
        folder: folder.to_string(),
        pinned: false,
        claude_session_id: format!("c{number}"),
        has_conversation: selected,
        selected,
        destination,
    }
}

fn category() -> Category {
    Category {
        id: "id-demo".to_string(),
        slug: "demo".to_string(),
        name: "Demo".to_string(),
        root_dir: "/work/demo".to_string(),
        color: None,
        sort: 0,
        created_at: 0,
        collapsed: false,
        notify: true,
    }
}

fn demo() -> Destination {
    Destination::Existing {
        category_id: "id-demo".to_string(),
    }
}

fn beta() -> Destination {
    Destination::New {
        root_dir: "/work/beta".to_string(),
    }
}

fn form() -> ImportForm {
    let proposal = Proposal {
        sessions: vec![
            session(1, "/work/demo/api", true, demo()),
            session(2, "/Users/someone", true, Destination::Unassigned),
            session(3, "/work/beta", false, beta()),
        ],
        new_categories: vec![NewCategoryProposal {
            slug: "beta".to_string(),
            name: "beta".to_string(),
            root_dir: "/work/beta".to_string(),
        }],
        ..Proposal::default()
    };
    ImportForm::new(&proposal, &[category()])
}

fn numbers(form: &ImportForm) -> Vec<usize> {
    form.chosen()
        .unwrap()
        .iter()
        .map(|chosen| chosen.number)
        .collect()
}

// PLAN §3.4: casillas por sesión, marcadas como lo propone
#[test]
fn the_sessions_start_marked_as_proposed_and_each_box_toggles_one() {
    let mut form = form();
    assert_eq!(form.marked(), 2);

    form.toggle(3);
    form.toggle(1);

    let marked: Vec<_> = form.rows.iter().map(|row| row.marked).collect();
    assert_eq!(marked, [false, true, true]);
}

// PLAN §3.4: lo de la carpeta home no se importa hasta que el usuario elige
#[test]
fn a_marked_session_without_a_category_holds_the_import_back() {
    let mut form = form();
    assert_eq!(form.chosen(), Err(vec![2]));

    form.set_destination(2, demo());

    assert_eq!(numbers(&form), [1, 2]);
    assert_eq!(form.chosen().unwrap()[1].destination, demo());
}

// PLAN §3.4: un selector de categoría por sesión (las existentes o una nueva)
#[test]
fn a_session_can_go_to_an_existing_category_or_a_new_one() {
    let form = form();

    let choices: Vec<_> = form
        .choices_for(2)
        .into_iter()
        .map(|choice| (choice.name, choice.is_new))
        .collect();

    assert_eq!(
        choices,
        [
            ("Demo".to_string(), false),
            ("beta".to_string(), true),
            ("someone".to_string(), true),
        ]
    );
    // The folder of session 3 already has its new category: no second one.
    assert_eq!(form.choices_for(3).len(), 2);
}
