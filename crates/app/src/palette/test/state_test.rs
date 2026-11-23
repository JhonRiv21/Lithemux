use crate::palette::ranking::Entry;
use crate::palette::state::PaletteState;

fn entry(id: &str, title: &str, activity: i64) -> Entry {
    Entry {
        session_id: id.to_string(),
        title: title.to_string(),
        slug: id.to_string(),
        category: "Demo".to_string(),
        category_slug: "demo".to_string(),
        role: String::new(),
        last_activity_at: Some(activity),
    }
}

fn state() -> PaletteState {
    PaletteState::new(vec![
        entry("a", "Alpha", 3),
        entry("b", "Beta", 2),
        entry("c", "Gamma", 1),
    ])
}

// PLAN §3.5: ↑/↓ mueven la selección; Enter salta a la seleccionada
#[test]
fn the_arrows_move_the_selection_and_go_round_at_the_ends() {
    let mut palette = state();
    assert_eq!(palette.chosen(), Some("a"));

    palette.move_by(1);
    assert_eq!(palette.chosen(), Some("b"));
    palette.move_by(-2);
    assert_eq!(palette.chosen(), Some("c"));
    palette.move_by(1);
    assert_eq!(palette.chosen(), Some("a"));
}

// PLAN §3.5: typing searches again and selects the best result
#[test]
fn a_new_search_selects_its_best_result() {
    let mut palette = state();
    palette.move_by(2);

    palette.search("beta");

    assert_eq!(palette.selected(), 0);
    assert_eq!(palette.chosen(), Some("b"));
}

// PLAN §3.5: a search that finds nothing has nothing to jump to
#[test]
fn a_search_that_finds_nothing_has_nothing_to_jump_to() {
    let mut palette = state();

    palette.search("zzz");
    palette.move_by(1);

    assert!(palette.found().is_empty());
    assert_eq!(palette.chosen(), None);
}
