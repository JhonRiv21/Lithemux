use crate::text_field::buffer::Buffer;

fn typed(text: &str) -> Buffer {
    let mut buffer = Buffer::default();
    buffer.replace(None, text);
    buffer
}

// PLAN §2.2: names and titles are typed in the sidebar.
#[test]
fn typing_goes_where_the_cursor_is() {
    let mut buffer = typed("ac");
    buffer.left();
    buffer.replace(None, "b");
    assert_eq!(buffer.text(), "abc");
    assert_eq!(buffer.around_cursor(), ("ab", "c"));
}

// PLAN §2.2
#[test]
fn deleting_takes_whole_characters() {
    let mut buffer = typed("añ🙂");
    buffer.delete_before();
    assert_eq!(buffer.text(), "añ");
    buffer.start();
    buffer.right();
    buffer.delete_after();
    assert_eq!(buffer.text(), "a");
    buffer.delete_after();
    buffer.start();
    buffer.delete_before();
    assert_eq!(buffer.text(), "a");
}

// PLAN §2.2
#[test]
fn the_cursor_stops_at_the_ends() {
    let mut buffer = typed("ñu");
    buffer.right();
    assert_eq!(buffer.around_cursor(), ("ñu", ""));
    buffer.start();
    buffer.left();
    assert_eq!(buffer.around_cursor(), ("", "ñu"));
    buffer.end();
    assert_eq!(buffer.around_cursor(), ("ñu", ""));
}

// PLAN §2.2: a field is one line.
#[test]
fn a_paste_with_line_breaks_stays_on_one_line() {
    assert_eq!(typed("one\ntwo\r\n\tthree").text(), "onetwothree");
    assert_eq!(Buffer::with_text("a\nb").text(), "ab");
}

// PLAN §1.2: accents typed with dead keys (the composition replaces itself).
#[test]
fn a_composition_is_replaced_by_what_it_becomes() {
    let mut buffer = typed("caf");
    buffer.compose(None, "´");
    assert_eq!(buffer.text(), "caf´");
    assert_eq!(buffer.marked_utf16(), Some(3..4));
    buffer.replace(None, "é");
    assert_eq!(buffer.text(), "café");
    assert_eq!(buffer.marked_utf16(), None);
}

// PLAN §1.2
#[test]
fn positions_are_translated_to_and_from_utf16() {
    let buffer = typed("a🙂ñ");
    assert_eq!(buffer.cursor_utf16(), 4);
    assert_eq!(buffer.bytes_of_utf16(3), 5);
    assert_eq!(buffer.range_from_utf16(&(1..3)), 1..5);
    assert_eq!(buffer.bytes_of_utf16(99), buffer.text().len());
}

// PLAN §2.2: a range from the system that is out of the text is not trusted.
#[test]
fn a_range_outside_the_text_is_brought_inside() {
    let mut buffer = typed("añ");
    buffer.replace(Some(2..50), "b");
    assert_eq!(buffer.text(), "ab");
}
