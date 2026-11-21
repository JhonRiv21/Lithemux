//! Clipboard text to the bytes written to the PTY.

const PASTE_START: &str = "\x1b[200~";
const PASTE_END: &str = "\x1b[201~";

pub fn paste_bytes(text: &str, bracketed: bool) -> Vec<u8> {
    if bracketed {
        // ESC or the one-byte CSI inside the text could close the paste early
        // and let the rest run as typed input; ctrl+c would interrupt the
        // program in the middle of it.
        let safe: String = text
            .chars()
            .filter(|c| !matches!(c, '\x1b' | '\u{9b}' | '\x03'))
            .collect();
        format!("{PASTE_START}{safe}{PASTE_END}").into_bytes()
    } else {
        // Without bracketed paste the program reads keystrokes, and the
        // enter key is a carriage return.
        text.replace("\r\n", "\r").replace('\n', "\r").into_bytes()
    }
}
