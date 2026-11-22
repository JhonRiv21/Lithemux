//! Terminal colors to RGB. Programs may override entries at runtime (OSC 4,
//! 10, 11); anything not overridden comes from this palette.

use alacritty_terminal::term::color::Colors;
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};

use crate::theme;

/// The colors of a terminal that the program did not set, for one theme.
struct Defaults {
    foreground: Rgb,
    background: Rgb,
    cursor: Rgb,
    selection: Rgb,
    ansi: [Rgb; 16],
}

const DARK: Defaults = Defaults {
    foreground: rgb(0xd8, 0xd8, 0xd8),
    background: rgb(0x16, 0x16, 0x18),
    cursor: rgb(0xe6, 0xe6, 0xe6),
    selection: rgb(0x2f, 0x4f, 0x7a),
    ansi: [
        rgb(0x1d, 0x1f, 0x21),
        rgb(0xcc, 0x66, 0x66),
        rgb(0xb5, 0xbd, 0x68),
        rgb(0xf0, 0xc6, 0x74),
        rgb(0x81, 0xa2, 0xbe),
        rgb(0xb2, 0x94, 0xbb),
        rgb(0x8a, 0xbe, 0xb7),
        rgb(0xc5, 0xc8, 0xc6),
        rgb(0x66, 0x66, 0x66),
        rgb(0xd5, 0x4e, 0x53),
        rgb(0xb9, 0xca, 0x4a),
        rgb(0xe7, 0xc5, 0x47),
        rgb(0x7a, 0xa6, 0xda),
        rgb(0xc3, 0x97, 0xd8),
        rgb(0x70, 0xc0, 0xb1),
        rgb(0xea, 0xea, 0xea),
    ],
};

// On a white background the «white» entries are grays, so that text a
// program paints with them can still be read.
const LIGHT: Defaults = Defaults {
    foreground: rgb(0x1c, 0x1c, 0x1f),
    background: rgb(0xff, 0xff, 0xff),
    cursor: rgb(0x33, 0x33, 0x38),
    selection: rgb(0xb5, 0xd0, 0xf5),
    ansi: [
        rgb(0x1d, 0x1f, 0x21),
        rgb(0xc8, 0x28, 0x29),
        rgb(0x5a, 0x7a, 0x00),
        rgb(0x9a, 0x76, 0x00),
        rgb(0x2f, 0x66, 0xc8),
        rgb(0x89, 0x59, 0xa8),
        rgb(0x2a, 0x80, 0x86),
        rgb(0x6e, 0x70, 0x6c),
        rgb(0x66, 0x66, 0x66),
        rgb(0xc8, 0x28, 0x29),
        rgb(0x5a, 0x7a, 0x00),
        rgb(0x9a, 0x76, 0x00),
        rgb(0x2f, 0x66, 0xc8),
        rgb(0x89, 0x59, 0xa8),
        rgb(0x2a, 0x80, 0x86),
        rgb(0x4d, 0x4d, 0x4c),
    ],
};

fn defaults() -> &'static Defaults {
    if theme::is_light() { &LIGHT } else { &DARK }
}

pub fn foreground() -> Rgb {
    defaults().foreground
}

pub fn background() -> Rgb {
    defaults().background
}

pub fn cursor() -> Rgb {
    defaults().cursor
}

pub fn selection() -> Rgb {
    defaults().selection
}

const DIM_FACTOR: f32 = 0.66;
const CUBE_STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

const fn rgb(r: u8, g: u8, b: u8) -> Rgb {
    Rgb { r, g, b }
}

pub fn dim(color: Rgb) -> Rgb {
    let scale = |channel: u8| (f32::from(channel) * DIM_FACTOR) as u8;
    rgb(scale(color.r), scale(color.g), scale(color.b))
}

/// The 256 indexed colors followed by the named entries alacritty keeps at
/// 256 and above (foreground, background, cursor, dim variants).
pub fn default_color(index: usize) -> Rgb {
    match index {
        0..=15 => defaults().ansi[index],
        16..=231 => {
            let cube = index - 16;
            rgb(
                CUBE_STEPS[cube / 36],
                CUBE_STEPS[(cube / 6) % 6],
                CUBE_STEPS[cube % 6],
            )
        }
        232..=255 => {
            let level = 8 + 10 * (index - 232) as u8;
            rgb(level, level, level)
        }
        index if index == NamedColor::Background as usize => background(),
        index if index == NamedColor::Cursor as usize => cursor(),
        index if index == NamedColor::DimForeground as usize => dim(foreground()),
        index if index == NamedColor::BrightForeground as usize => defaults().ansi[15],
        index
            if (NamedColor::DimBlack as usize..=NamedColor::DimWhite as usize).contains(&index) =>
        {
            dim(defaults().ansi[index - NamedColor::DimBlack as usize])
        }
        _ => foreground(),
    }
}

pub fn resolve(color: Color, overrides: &Colors) -> Rgb {
    match color {
        Color::Spec(rgb) => rgb,
        Color::Named(named) => indexed(named as usize, overrides),
        Color::Indexed(index) => indexed(usize::from(index), overrides),
    }
}

fn indexed(index: usize, overrides: &Colors) -> Rgb {
    overrides[index].unwrap_or_else(|| default_color(index))
}
