use lithemux_core::modules::settings::ThemeChoice;

use crate::theme::{DARK, LIGHT, Palette, is_light_for};

// PLAN §2.6: theme light, dark or system.
#[test]
fn light_and_dark_are_what_the_user_chose_whatever_the_system_says() {
    for system_is_dark in [true, false] {
        assert!(is_light_for(ThemeChoice::Light, system_is_dark));
        assert!(!is_light_for(ThemeChoice::Dark, system_is_dark));
    }
}

// PLAN §2.6
#[test]
fn system_follows_the_appearance_of_macos() {
    assert!(!is_light_for(ThemeChoice::System, true));
    assert!(is_light_for(ThemeChoice::System, false));
}

fn luminance(color: u32) -> f64 {
    let channel = |shift: u32| {
        let value = f64::from((color >> shift) & 0xff) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
}

fn contrast(a: u32, b: u32) -> f64 {
    let (lighter, darker) = (
        luminance(a).max(luminance(b)),
        luminance(a).min(luminance(b)),
    );
    (lighter + 0.05) / (darker + 0.05)
}

/// Every background a text can be drawn on.
fn backgrounds(palette: &Palette) -> [(&'static str, u32); 5] {
    [
        ("window", palette.window),
        ("sidebar", palette.sidebar),
        ("surface", palette.surface),
        ("surface_hover", palette.surface_hover),
        ("surface_active", palette.surface_active),
    ]
}

// PLAN §2.6: both themes must be usable: every color used for text can be
// read on every background it is drawn on. WCAG AA for normal text is
// 4.5:1. (Audit of block 4, 10: the first version of this test checked two
// colors only, and the light «attention» was at 3.4.)
#[test]
fn text_can_be_read_on_its_backgrounds_in_both_themes() {
    for (name, palette) in [("dark", &DARK), ("light", &LIGHT)] {
        let texts = [
            ("text", palette.text),
            ("text_muted", palette.text_muted),
            ("text_faint", palette.text_faint),
            ("attention", palette.attention),
            ("link", palette.link),
            ("danger", palette.danger),
        ];
        for (text, color) in texts {
            for (background, behind) in backgrounds(palette) {
                let ratio = contrast(color, behind);
                assert!(ratio >= 4.5, "{name}: {text} on {background} is {ratio:.1}");
            }
        }
        assert!(contrast(palette.text, palette.window) >= 7.0, "{name}");
    }
}

// The same for what is written on the accent color: the option in use in
// the settings, a menu entry under the pointer, the unread badge.
#[test]
fn text_on_the_accent_colors_can_be_read_in_both_themes() {
    for (name, palette) in [("dark", &DARK), ("light", &LIGHT)] {
        for (background, behind) in [("accent", palette.accent), ("badge", palette.badge)] {
            let ratio = contrast(palette.on_accent, behind);
            assert!(
                ratio >= 4.5,
                "{name}: on_accent on {background} is {ratio:.1}"
            );
        }
    }
}
