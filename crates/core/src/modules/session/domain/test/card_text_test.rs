use crate::modules::session::domain::card_text::{
    MAX_PREVIEW_CHARS, MAX_TITLE_CHARS, preview_from, title_from,
};

// PLAN §2.2: the card shows one line of preview of the last message.
#[test]
fn a_preview_is_one_line_however_many_the_message_has() {
    assert_eq!(
        preview_from("first line\n\n  second\tline  ").as_deref(),
        Some("first line second line")
    );
}

// PLAN §2.2
#[test]
fn a_long_message_is_cut_and_the_cut_is_marked() {
    let preview = preview_from(&"word ".repeat(200)).unwrap();
    assert_eq!(preview.chars().count(), MAX_PREVIEW_CHARS);
    assert!(preview.ends_with('…'));
}

// PLAN §2.2
#[test]
fn a_message_without_text_is_no_preview() {
    assert_eq!(preview_from(" \n\t "), None);
}

// PLAN §2.2: the title is the session name or a custom title.
#[test]
fn an_empty_title_means_no_custom_title() {
    assert_eq!(title_from("   "), None);
    assert_eq!(title_from("  my api  ").as_deref(), Some("my api"));
}

// PLAN §2.2
#[test]
fn a_title_is_bounded() {
    let title = title_from(&"x".repeat(500)).unwrap();
    assert_eq!(title.chars().count(), MAX_TITLE_CHARS);
}

// Block 4, part 0 (6): the card shows text, not markdown source.
#[test]
fn a_preview_drops_the_basic_markdown_markers() {
    assert_eq!(
        preview_from("Un **PTY** es un `pseudo terminal`, __no__ un tubo").as_deref(),
        Some("Un PTY es un pseudo terminal, no un tubo")
    );
    assert_eq!(
        preview_from("## Resumen\n\n# Cambios\nTodo listo").as_deref(),
        Some("Resumen Cambios Todo listo")
    );
}

// Block 4, part 0 (6): only the markers go; what looks like one inside a
// word or a path stays.
#[test]
fn text_that_only_resembles_markdown_is_left_alone() {
    assert_eq!(
        preview_from("issue #12 in snake_case_name, 2 * 3 = 6").as_deref(),
        Some("issue #12 in snake_case_name, 2 * 3 = 6")
    );
}
