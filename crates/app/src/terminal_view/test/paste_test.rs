use crate::terminal_view::paste::paste_bytes;

// PLAN §1.2 test 4
#[test]
fn a_multiline_paste_is_bracketed_and_keeps_its_lines() {
    let pasted = paste_bytes("uno\ndos\ntres", true);
    assert_eq!(pasted, b"\x1b[200~uno\ndos\ntres\x1b[201~");
}

// PLAN §1.2 test 4: accents and ñ survive a paste byte for byte.
#[test]
fn pasted_text_keeps_its_utf8() {
    let pasted = paste_bytes("año ¿qué?", true);
    assert_eq!(pasted, "\x1b[200~año ¿qué?\x1b[201~".as_bytes());
}

// PLAN §1.2 test 4, audit F1 #6: nothing in the text may end the paste (ESC
// or the one-byte CSI, U+009B) or interrupt the program (ctrl+c).
#[test]
fn pasted_text_cannot_close_the_bracket_or_interrupt() {
    let pasted = paste_bytes("a\x1b[201~rm -rf x\n\u{9b}201~\x03b", true);
    let text = String::from_utf8(pasted).unwrap();
    let inside = text
        .strip_prefix("\x1b[200~")
        .and_then(|rest| rest.strip_suffix("\x1b[201~"))
        .unwrap();
    assert!(!inside.contains('\x1b'));
    assert!(!inside.contains('\u{9b}'));
    assert!(!inside.contains('\x03'));
    assert_eq!(inside, "a[201~rm -rf x\n201~b");
}

// PLAN §1.2 test 4: a program that did not ask for bracketed paste gets the
// text as typed, where a line ends with a carriage return.
#[test]
fn without_bracketed_paste_lines_end_like_the_enter_key() {
    assert_eq!(paste_bytes("uno\ndos", false), b"uno\rdos");
    assert_eq!(paste_bytes("uno\r\ndos", false), b"uno\rdos");
}
