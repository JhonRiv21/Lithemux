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
