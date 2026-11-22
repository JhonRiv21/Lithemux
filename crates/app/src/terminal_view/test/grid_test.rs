use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{Processor, Rgb};

use crate::terminal_view::grid::{GridSnapshot, snapshot};
use crate::terminal_view::palette;

struct Screen;

impl Dimensions for Screen {
    fn total_lines(&self) -> usize {
        5
    }

    fn screen_lines(&self) -> usize {
        5
    }

    fn columns(&self) -> usize {
        20
    }
}

fn screen_after(output: &str) -> GridSnapshot {
    let mut term = Term::new(Config::default(), &Screen, VoidListener);
    let mut parser: Processor = Processor::new();
    parser.advance(&mut term, output.as_bytes());
    snapshot(&term)
}

// PLAN §1.2 test 1: truecolor
#[test]
fn a_24_bit_color_reaches_the_grid_exactly() {
    let grid = screen_after("\x1b[38;2;255;136;0mhola");
    let run = &grid.rows[0][0];
    assert_eq!(run.text, "hola");
    assert_eq!(
        run.style.foreground,
        Rgb {
            r: 255,
            g: 136,
            b: 0
        }
    );
    assert_eq!(run.style.background, None);
}

// PLAN §1.2 test 1: the 256-color cube, 16 + 36r + 6g + b.
#[test]
fn an_indexed_color_resolves_through_the_xterm_cube() {
    let grid = screen_after("\x1b[38;5;196mx\x1b[48;5;21my");
    assert_eq!(grid.rows[0][0].style.foreground, Rgb { r: 255, g: 0, b: 0 });
    assert_eq!(
        grid.rows[0][1].style.background,
        Some(Rgb { r: 0, g: 0, b: 255 })
    );
}

fn style_at(grid: &GridSnapshot, column: usize) -> crate::terminal_view::grid::CellStyle {
    grid.rows[0]
        .iter()
        .find(|run| (run.column..run.column + run.cells).contains(&column))
        .unwrap()
        .style
}

// PLAN §1.1: cells and colors
#[test]
fn bold_applies_only_to_the_cells_printed_in_bold() {
    let grid = screen_after("ab\x1b[1mcd\x1b[0mef");
    assert!(!style_at(&grid, 1).bold);
    assert!(style_at(&grid, 2).bold);
    assert!(style_at(&grid, 3).bold);
    assert!(!style_at(&grid, 4).bold);
}

// PLAN §1.1: cells
#[test]
fn reverse_video_swaps_foreground_and_background() {
    let grid = screen_after("\x1b[7mx");
    let style = grid.rows[0][0].style;
    assert_eq!(style.foreground, palette::background());
    assert_eq!(style.background, Some(palette::foreground()));
}

// PLAN §1.1: cells
#[test]
fn a_wide_character_takes_two_cells() {
    let grid = screen_after("a漢b");
    let runs = &grid.rows[0];
    let wide = runs.iter().find(|run| run.text == "漢").unwrap();
    assert_eq!((wide.column, wide.cells), (1, 2));
    let after = runs.iter().find(|run| run.text == "b").unwrap();
    assert_eq!(after.column, 3);
}

// PLAN §1.1: cursor
#[test]
fn the_cursor_sits_after_the_text() {
    let grid = screen_after("hola\r\nque");
    let cursor = grid.cursor.unwrap();
    assert_eq!((cursor.line, cursor.column), (1, 3));
    assert!(cursor.visible);
}

// Audit F1 #4: a hidden cursor keeps its position, because the accent menu
// and the text being composed are placed there.
#[test]
fn a_hidden_cursor_still_reports_where_it_is() {
    let cursor = screen_after("hola\x1b[?25l").cursor.unwrap();
    assert!(!cursor.visible);
    assert_eq!((cursor.line, cursor.column), (0, 4));
}
