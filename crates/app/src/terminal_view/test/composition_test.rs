use crate::terminal_view::composition::Composition;

// PLAN §1.2 test 2: ´ + a = á
#[test]
fn a_dead_key_sends_nothing_until_the_letter_completes_it() {
    let mut composition = Composition::default();
    composition.mark("´");
    assert_eq!(composition.marked(), "´");

    let sent = composition.commit(None, "á");
    assert_eq!(sent, "á".as_bytes());
    assert_eq!(composition.marked(), "");
}

// PLAN §1.2 test 2
#[test]
fn spanish_characters_are_sent_as_utf8() {
    let mut composition = Composition::default();
    for text in ["ñ", "¿", "¡", "Ñ", "ü"] {
        assert_eq!(composition.commit(None, text), text.as_bytes());
    }
}

// PLAN §1.2 test 2: holding a vowel types it, and choosing an accent from the
// menu replaces the letter already sent.
#[test]
fn the_accent_menu_replaces_the_letter_it_was_opened_on() {
    let mut composition = Composition::default();
    assert_eq!(composition.commit(None, "a"), b"a");

    let sent = composition.commit(Some(0..1), "á");
    let mut expected = vec![0x7f];
    expected.extend_from_slice("á".as_bytes());
    assert_eq!(sent, expected);
}

// PLAN §1.2 test 2
#[test]
fn an_abandoned_composition_sends_nothing() {
    let mut composition = Composition::default();
    composition.mark("´");
    composition.unmark();
    assert_eq!(composition.marked(), "");
    assert_eq!(composition.marked_range_utf16(), None);
    assert_eq!(composition.commit(None, ""), b"");
}

// PLAN §1.2 test 2: once another key went to the program, the letter typed
// before it is no longer the menu's to delete.
#[test]
fn nothing_is_erased_after_a_key_that_is_not_text() {
    let mut composition = Composition::default();
    composition.commit(None, "a");
    composition.reset();
    assert_eq!(composition.commit(Some(0..1), "á"), "á".as_bytes());
}

// PLAN §1.2 test 2: the system addresses text in UTF-16 units.
#[test]
fn the_text_being_composed_is_reported_in_utf16_units() {
    let mut composition = Composition::default();
    composition.commit(None, "😀");
    composition.mark("´");
    assert_eq!(composition.document(), "😀´");
    assert_eq!(composition.marked_range_utf16(), Some(2..3));
    assert_eq!(composition.cursor_utf16(), 3);
}
