//! The keystrokes are built the way gpui's macOS platform builds them
//! (`gpui_macos/src/events.rs`, `parse_keystroke`): the key is the
//! character the layout gives, and with shift on a key that is not a
//! letter the shift is dropped and the key is the shifted character.

use gpui::{Keystroke, Modifiers};

use crate::workspace::shortcuts::bindings;

fn pressed(key: &str, modifiers: Modifiers) -> Keystroke {
    Keystroke {
        modifiers,
        key: key.to_string(),
        key_char: None,
    }
}

fn command() -> Modifiers {
    Modifiers {
        platform: true,
        ..Modifiers::default()
    }
}

fn command_shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..command()
    }
}

fn command_option() -> Modifiers {
    Modifiers {
        alt: true,
        ..command()
    }
}

/// The action a keystroke runs, by its name.
fn action_of(keystroke: &Keystroke) -> Option<String> {
    bindings()
        .iter()
        .find(|binding| binding.match_keystrokes(std::slice::from_ref(keystroke)) == Some(false))
        .map(|binding| binding.action().name().to_string())
}

fn runs(keystroke: &Keystroke, action: &str) -> bool {
    action_of(keystroke).is_some_and(|name| name.ends_with(action))
}

// Block 4, part 0 (2): arrows and digits are the same key on every layout.
// On a Latin American keyboard these are what changes the tab.
#[test]
fn tabs_change_with_keys_that_do_not_depend_on_the_layout() {
    assert!(runs(&pressed("right", command_shift()), "NextTab"));
    assert!(runs(&pressed("left", command_shift()), "PreviousTab"));
    assert!(runs(&pressed("1", command()), "ShowTab1"));
    assert!(runs(&pressed("9", command()), "ShowTab9"));
}

// Block 4, part 0 (2)
#[test]
fn panes_change_with_keys_that_do_not_depend_on_the_layout() {
    assert!(runs(&pressed("right", command_option()), "NextPane"));
    assert!(runs(&pressed("left", command_option()), "PreviousPane"));
}

// Block 4, part 0 (2): on a US keyboard cmd+shift+] is reported as `}`
// without shift. Bound as «cmd-shift-]» it never matched.
#[test]
fn the_bracket_shortcuts_match_what_a_us_keyboard_sends() {
    assert!(runs(&pressed("}", command()), "NextTab"));
    assert!(runs(&pressed("{", command()), "PreviousTab"));
    assert!(runs(&pressed("]", command()), "NextPane"));
    assert!(runs(&pressed("[", command()), "PreviousPane"));
}

// Block 4, part 0 (2): on a Latin American keyboard the key right of «P»
// gives `´` and, with shift, `¨`; the brackets need Option. Nothing is
// bound to those, and nothing must be: the layout-free keys are the way.
#[test]
fn what_a_latin_american_keyboard_sends_for_those_keys_runs_nothing() {
    assert_eq!(action_of(&pressed("¨", command())), None);
    assert_eq!(action_of(&pressed("´", command())), None);
    assert_eq!(action_of(&pressed("]", command_shift())), None);
}

// PLAN §2.3: shortcuts to split, change focus, change tab and close.
#[test]
fn every_action_of_the_plan_has_a_shortcut_that_needs_no_layout() {
    let layout_free = |key: &str| key.len() > 1 || key.chars().all(|c| c.is_ascii_alphanumeric());
    for action in [
        "NewTerminal",
        "SplitRight",
        "SplitDown",
        "CloseTab",
        "ClosePane",
        "NextPane",
        "PreviousPane",
        "NextTab",
        "PreviousTab",
    ] {
        let has_one = bindings().iter().any(|binding| {
            binding.action().name().ends_with(action)
                && binding
                    .keystrokes()
                    .iter()
                    .all(|keystroke| layout_free(keystroke.key()))
        });
        assert!(has_one, "{action}");
    }
}

// The terminal must keep every key a shell uses: all the shortcuts carry cmd.
#[test]
fn every_shortcut_carries_the_command_key() {
    for binding in bindings() {
        for keystroke in binding.keystrokes() {
            assert!(keystroke.modifiers().platform, "{keystroke:?}");
        }
    }
}

// Audit of block 4 (11): however a binding is written, with shift it must
// name a letter or a named key. gpui drops the shift of any other key, so
// such a binding would never match. Checked on the parsed keystrokes, where
// the order the modifiers were written in no longer matters.
#[test]
fn no_shortcut_combines_shift_with_a_key_that_is_not_a_letter() {
    for binding in bindings() {
        for keystroke in binding.keystrokes() {
            let key = keystroke.key();
            let is_a_single_symbol =
                key.chars().count() == 1 && !key.chars().all(|c| c.is_ascii_alphabetic());
            assert!(
                !(keystroke.modifiers().shift && is_a_single_symbol),
                "{keystroke:?}"
            );
        }
    }
}
