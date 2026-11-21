use crate::terminal_view::keys::{KeyModes, KeyModifiers, KeyPress, key_bytes};

const NONE: KeyModifiers = KeyModifiers {
    shift: false,
    control: false,
    alt: false,
    platform: false,
};
const SHIFT: KeyModifiers = KeyModifiers {
    shift: true,
    ..NONE
};
const CONTROL: KeyModifiers = KeyModifiers {
    control: true,
    ..NONE
};
const ALT: KeyModifiers = KeyModifiers { alt: true, ..NONE };
const COMMAND: KeyModifiers = KeyModifiers {
    platform: true,
    ..NONE
};

fn bytes(key: &str, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    key_bytes(KeyPress { key, modifiers }, KeyModes::default())
}

// PLAN §1.2 test 3
#[test]
fn shift_enter_is_a_new_line_and_not_a_submit() {
    let enter = bytes("enter", NONE).unwrap();
    let shift_enter = bytes("enter", SHIFT).unwrap();
    assert_eq!(enter, b"\r");
    assert_eq!(shift_enter, b"\x1b\r");
}

// PLAN §1.2 test 2: text never goes through the key table, so the layout and
// the input method decide what a key types.
#[test]
fn letters_and_spanish_characters_are_left_to_the_input_method() {
    for key in ["a", "ñ", "¿", "¡", "´", "space", "1"] {
        assert_eq!(bytes(key, NONE), None, "{key}");
        assert_eq!(bytes(key, SHIFT), None, "shift {key}");
    }
}

// PLAN §1.2 test 2: on a Spanish layout option types @, #, [, ], {, } and ~.
#[test]
fn option_with_a_character_key_is_text_and_not_a_meta_prefix() {
    for key in ["2", "3", "ñ", "+", "q", "e"] {
        assert_eq!(bytes(key, ALT), None, "{key}");
    }
}

// PLAN §1.2 test 4: cmd+c and cmd+v belong to copy and paste.
#[test]
fn command_shortcuts_never_reach_the_program_as_letters() {
    for key in ["c", "v", "q", "a"] {
        assert_eq!(bytes(key, COMMAND), None, "{key}");
    }
}

// PLAN §1.2 test 1: what Claude's TUI needs to be driven at all.
#[test]
fn control_letters_are_the_ascii_control_characters() {
    assert_eq!(bytes("c", CONTROL).unwrap(), [0x03]);
    assert_eq!(bytes("d", CONTROL).unwrap(), [0x04]);
    assert_eq!(bytes("r", CONTROL).unwrap(), [0x12]);
    assert_eq!(bytes("z", CONTROL).unwrap(), [0x1a]);
}

// PLAN §1.2 test 1
#[test]
fn editing_keys_use_the_xterm_sequences() {
    assert_eq!(bytes("escape", NONE).unwrap(), b"\x1b");
    assert_eq!(bytes("tab", NONE).unwrap(), b"\t");
    assert_eq!(bytes("tab", SHIFT).unwrap(), b"\x1b[Z");
    assert_eq!(bytes("backspace", NONE).unwrap(), [0x7f]);
    assert_eq!(bytes("delete", NONE).unwrap(), b"\x1b[3~");
    assert_eq!(bytes("pageup", NONE).unwrap(), b"\x1b[5~");
    assert_eq!(bytes("pagedown", NONE).unwrap(), b"\x1b[6~");
}

// PLAN §1.2 test 1
#[test]
fn arrows_follow_the_cursor_key_mode_the_program_asked_for() {
    assert_eq!(bytes("up", NONE).unwrap(), b"\x1b[A");
    assert_eq!(bytes("down", NONE).unwrap(), b"\x1b[B");
    assert_eq!(bytes("right", NONE).unwrap(), b"\x1b[C");
    assert_eq!(bytes("left", NONE).unwrap(), b"\x1b[D");

    let application = KeyModes { app_cursor: true };
    let press = KeyPress {
        key: "up",
        modifiers: NONE,
    };
    assert_eq!(key_bytes(press, application).unwrap(), b"\x1bOA");
}

// PLAN §1.2 test 1: xterm encodes modifiers as 1 + shift(1) + alt(2) + ctrl(4).
#[test]
fn modified_arrows_carry_the_xterm_modifier_parameter() {
    assert_eq!(bytes("up", SHIFT).unwrap(), b"\x1b[1;2A");
    assert_eq!(bytes("right", CONTROL).unwrap(), b"\x1b[1;5C");
    let shift_control = KeyModifiers {
        shift: true,
        control: true,
        ..NONE
    };
    assert_eq!(bytes("left", shift_control).unwrap(), b"\x1b[1;6D");
}

// PLAN §1.2 test 1
#[test]
fn function_keys_use_the_xterm_sequences() {
    assert_eq!(bytes("f1", NONE).unwrap(), b"\x1bOP");
    assert_eq!(bytes("f4", NONE).unwrap(), b"\x1bOS");
    assert_eq!(bytes("f5", NONE).unwrap(), b"\x1b[15~");
    assert_eq!(bytes("f12", NONE).unwrap(), b"\x1b[24~");
}
