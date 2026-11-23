use crate::modules::import::domain::choices::{Choices, choose};
use crate::modules::import::domain::proposal::{Destination, Proposal, ProposedSession};
use crate::modules::import::error::ImportError;

fn proposed(number: usize, selected: bool, destination: Destination) -> ProposedSession {
    ProposedSession {
        number,
        name: format!("s{number}"),
        slug: format!("s{number}"),
        folder: "/work/a".to_string(),
        pinned: false,
        claude_session_id: format!("c{number}"),
        has_conversation: selected,
        selected,
        destination,
    }
}

fn existing() -> Destination {
    Destination::Existing {
        category_id: "a".to_string(),
    }
}

fn proposal() -> Proposal {
    Proposal {
        sessions: vec![
            proposed(1, true, existing()),
            proposed(2, false, existing()),
            proposed(4, true, Destination::Unassigned),
        ],
        ..Proposal::default()
    }
}

fn numbers(chosen: &[crate::modules::import::domain::proposal::Chosen]) -> Vec<usize> {
    chosen.iter().map(|c| c.number).collect()
}

// PLAN §3.4: the CLI chooses with --into, --skip and --include
#[test]
fn the_selected_ones_go_and_the_ones_without_a_category_go_into_the_chosen_one() {
    let into = Destination::Existing {
        category_id: "b".to_string(),
    };
    let chosen = choose(
        &proposal(),
        &Choices {
            into: Some(&into),
            ..Choices::default()
        },
    )
    .unwrap();

    assert_eq!(numbers(&chosen), [1, 4]);
    assert_eq!(chosen[1].destination, into);
}

// PLAN §3.4: skipping and including by number
#[test]
fn a_session_can_be_left_out_or_taken_in_by_its_number() {
    let chosen = choose(
        &proposal(),
        &Choices {
            skip: &[1, 4],
            include: &[2],
            into: None,
        },
    )
    .unwrap();

    assert_eq!(numbers(&chosen), [2]);
}

// PLAN §3.4: nothing is chosen while a chosen session has no category
#[test]
fn a_session_left_without_a_category_stops_the_import() {
    let refused = choose(&proposal(), &Choices::default());

    assert!(matches!(refused, Err(ImportError::WithoutCategory(ref n)) if n == &[4]));
    assert_eq!(
        refused.unwrap_err().to_string(),
        "session 4 without a category: choose one with --into <category>, or leave it out with --skip"
    );
}

// PLAN §3.4: a number the proposal does not have is a mistake, not ignored
#[test]
fn a_number_that_is_not_in_the_proposal_is_refused() {
    let refused = choose(
        &proposal(),
        &Choices {
            skip: &[3],
            ..Choices::default()
        },
    );

    assert!(matches!(refused, Err(ImportError::UnknownNumber(3))));
}
