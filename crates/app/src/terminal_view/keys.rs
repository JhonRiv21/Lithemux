//! Keystrokes to the bytes a terminal program expects (xterm legacy encoding).
//! Printable text is not handled here: it arrives through the input method.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub platform: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyPress<'a> {
    pub key: &'a str,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyModes {
    pub app_cursor: bool,
}

const ESC: u8 = 0x1b;
const DEL: u8 = 0x7f;

/// `None` means the key is text (or nothing) and the input method decides.
pub fn key_bytes(press: KeyPress<'_>, modes: KeyModes) -> Option<Vec<u8>> {
    let mods = press.modifiers;
    if mods.platform {
        return platform_bytes(press.key, mods);
    }
    match press.key {
        "enter" if mods.shift || mods.alt => Some(vec![ESC, b'\r']),
        "enter" => Some(vec![b'\r']),
        "tab" if mods.shift => Some(b"\x1b[Z".to_vec()),
        "tab" => Some(vec![b'\t']),
        "escape" => Some(vec![ESC]),
        "backspace" if mods.alt => Some(vec![ESC, DEL]),
        "backspace" if mods.control => Some(vec![0x08]),
        "backspace" => Some(vec![DEL]),
        "space" if mods.control => Some(vec![0x00]),
        "left" if alt_only(mods) => Some(vec![ESC, b'b']),
        "right" if alt_only(mods) => Some(vec![ESC, b'f']),
        "up" => Some(cursor_key(b'A', mods, modes)),
        "down" => Some(cursor_key(b'B', mods, modes)),
        "right" => Some(cursor_key(b'C', mods, modes)),
        "left" => Some(cursor_key(b'D', mods, modes)),
        "home" => Some(cursor_key(b'H', mods, modes)),
        "end" => Some(cursor_key(b'F', mods, modes)),
        "insert" => Some(tilde_key(2, mods)),
        "delete" => Some(tilde_key(3, mods)),
        "pageup" => Some(tilde_key(5, mods)),
        "pagedown" => Some(tilde_key(6, mods)),
        key if mods.control => control_bytes(key, mods.alt),
        key => function_key(key, mods),
    }
}

fn alt_only(mods: KeyModifiers) -> bool {
    mods.alt && !mods.shift && !mods.control
}

// macOS line editing habits, as the readline control characters.
fn platform_bytes(key: &str, mods: KeyModifiers) -> Option<Vec<u8>> {
    if mods.control || mods.alt || mods.shift {
        return None;
    }
    match key {
        "left" => Some(vec![0x01]),
        "right" => Some(vec![0x05]),
        "backspace" => Some(vec![0x15]),
        _ => None,
    }
}

/// xterm modifier parameter: 1 + shift(1) + alt(2) + control(4); 1 means none.
fn modifier_param(mods: KeyModifiers) -> u8 {
    1 + u8::from(mods.shift) + 2 * u8::from(mods.alt) + 4 * u8::from(mods.control)
}

fn cursor_key(letter: u8, mods: KeyModifiers, modes: KeyModes) -> Vec<u8> {
    match modifier_param(mods) {
        1 if modes.app_cursor => vec![ESC, b'O', letter],
        1 => vec![ESC, b'[', letter],
        param => format!("\x1b[1;{param}{}", char::from(letter)).into_bytes(),
    }
}

fn tilde_key(number: u8, mods: KeyModifiers) -> Vec<u8> {
    match modifier_param(mods) {
        1 => format!("\x1b[{number}~").into_bytes(),
        param => format!("\x1b[{number};{param}~").into_bytes(),
    }
}

fn function_key(key: &str, mods: KeyModifiers) -> Option<Vec<u8>> {
    let number: u8 = key.strip_prefix('f')?.parse().ok()?;
    let param = modifier_param(mods);
    if let Some(letter) = b"PQRS".get(usize::from(number).wrapping_sub(1)) {
        return Some(match param {
            1 => vec![ESC, b'O', *letter],
            param => format!("\x1b[1;{param}{}", char::from(*letter)).into_bytes(),
        });
    }
    let code = match number {
        5 => 15,
        6 => 17,
        7 => 18,
        8 => 19,
        9 => 20,
        10 => 21,
        11 => 23,
        12 => 24,
        _ => return None,
    };
    Some(tilde_key(code, mods))
}

fn control_bytes(key: &str, alt: bool) -> Option<Vec<u8>> {
    let mut chars = key.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return None;
    };
    let byte = match c.to_ascii_lowercase() {
        c @ 'a'..='z' => c as u8 - b'a' + 1,
        '@' | '2' => 0x00,
        '[' | '3' => ESC,
        '\\' | '4' => 0x1c,
        ']' | '5' => 0x1d,
        '^' | '6' => 0x1e,
        '_' | '-' | '7' => 0x1f,
        '?' | '8' => DEL,
        _ => return None,
    };
    Some(if alt { vec![ESC, byte] } else { vec![byte] })
}
