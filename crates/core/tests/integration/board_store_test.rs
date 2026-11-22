use std::fs;

use lithemux_core::modules::category::{BoardStore, Category, FsBoardStore};

use crate::temp_home::TempHome;

fn category() -> Category {
    Category {
        id: "c1".to_string(),
        slug: "selektal".to_string(),
        name: "Selektal".to_string(),
        root_dir: "/work".to_string(),
        color: None,
        sort: 1,
        created_at: 0,
        collapsed: false,
        notify: true,
    }
}

// PLAN §0.5: <home>/categories/<slug>/board.md, created empty with a heading
#[test]
fn the_board_is_created_with_a_heading_under_the_home() {
    let temp = TempHome::new();
    let boards = FsBoardStore::new(&temp.home);
    boards.ensure(&category()).unwrap();
    let path = temp.home.path().join("categories/selektal/board.md");
    assert_eq!(boards.path("selektal"), path);
    let content = fs::read_to_string(path).unwrap();
    assert!(content.starts_with("# "));
    assert!(content.contains("Selektal"));
    assert_eq!(content.trim().lines().count(), 1);
}

// docs/decisiones.md: `cat rm` keeps the board, so re-creating must not wipe it
#[test]
fn an_existing_board_is_never_overwritten() {
    let temp = TempHome::new();
    let boards = FsBoardStore::new(&temp.home);
    boards.ensure(&category()).unwrap();
    fs::write(
        boards.path("selektal"),
        "# Selektal\n\nnotas de la coordinadora\n",
    )
    .unwrap();
    boards.ensure(&category()).unwrap();
    let content = fs::read_to_string(boards.path("selektal")).unwrap();
    assert!(content.contains("notas de la coordinadora"));
}
