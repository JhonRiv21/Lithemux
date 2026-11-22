//! What the view paints: the visible grid, already resolved to colors and
//! grouped in runs of equal style. No gpui types here.

use alacritty_terminal::event::EventListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::color::Colors;
use alacritty_terminal::term::{Term, TermMode};
use alacritty_terminal::vte::ansi::{Color, CursorShape, NamedColor, Rgb};

use super::palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellStyle {
    pub foreground: Rgb,
    pub background: Option<Rgb>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikeout: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSnapshot {
    pub column: usize,
    /// Grid cells covered; a wide character covers two.
    pub cells: usize,
    pub text: String,
    /// Every character fills exactly one cell (no wide or combining ones).
    pub one_per_cell: bool,
    pub style: CellStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorSnapshot {
    pub line: usize,
    pub column: usize,
    pub wide: bool,
    pub shape: CursorShape,
    /// False when the program hid it; the position still matters, because
    /// text being composed is drawn there.
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSnapshot {
    pub columns: usize,
    pub rows: Vec<Vec<RunSnapshot>>,
    pub cursor: Option<CursorSnapshot>,
}

pub fn snapshot<T: EventListener>(term: &Term<T>) -> GridSnapshot {
    let content = term.renderable_content();
    let columns = term.columns();
    let offset = content.display_offset as i32;
    let mut rows = vec![Vec::new(); term.screen_lines()];
    let mut cursor_is_wide = false;

    for indexed in content.display_iter {
        let point = indexed.point;
        let Some(row) = usize::try_from(point.line.0 + offset)
            .ok()
            .and_then(|line| rows.get_mut(line))
        else {
            continue;
        };
        let cell = indexed.cell;
        if point == content.cursor.point {
            cursor_is_wide = cell.flags.contains(Flags::WIDE_CHAR);
        }
        if cell
            .flags
            .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
        {
            continue;
        }
        let selected = content.selection.is_some_and(|range| range.contains(point));
        let style = cell_style(cell, selected, content.colors);
        push_cell(row, point.column.0, cell, style);
    }

    let cursor = usize::try_from(content.cursor.point.line.0 + offset)
        .ok()
        .filter(|line| *line < rows.len())
        .map(|line| CursorSnapshot {
            line,
            column: content.cursor.point.column.0,
            wide: cursor_is_wide,
            shape: content.cursor.shape,
            visible: content.mode.contains(TermMode::SHOW_CURSOR)
                && content.cursor.shape != CursorShape::Hidden,
        });

    GridSnapshot {
        columns,
        rows,
        cursor,
    }
}

fn cell_style(cell: &Cell, selected: bool, colors: &Colors) -> CellStyle {
    let flags = cell.flags;
    let mut foreground = palette::resolve(cell.fg, colors);
    let mut background = palette::resolve(cell.bg, colors);
    let mut has_background = cell.bg != Color::Named(NamedColor::Background);
    if flags.contains(Flags::DIM) {
        foreground = palette::dim(foreground);
    }
    if flags.contains(Flags::INVERSE) {
        std::mem::swap(&mut foreground, &mut background);
        has_background = true;
    }
    if flags.contains(Flags::HIDDEN) {
        foreground = background;
    }
    if selected {
        background = palette::selection();
        has_background = true;
    }
    CellStyle {
        foreground,
        background: has_background.then_some(background),
        bold: flags.contains(Flags::BOLD),
        italic: flags.contains(Flags::ITALIC),
        underline: flags.intersects(Flags::ALL_UNDERLINES),
        strikeout: flags.contains(Flags::STRIKEOUT),
    }
}

fn push_cell(row: &mut Vec<RunSnapshot>, column: usize, cell: &Cell, style: CellStyle) {
    let blank = cell.c == ' ' && style.background.is_none() && !style.underline && !style.strikeout;
    if blank {
        return;
    }
    let wide = cell.flags.contains(Flags::WIDE_CHAR);
    let zerowidth = cell.zerowidth().unwrap_or_default();
    if !wide
        && zerowidth.is_empty()
        && let Some(last) = row.last_mut()
        && last.style == style
        && last.column + last.cells == column
        && last.one_per_cell
    {
        last.text.push(cell.c);
        last.cells += 1;
        return;
    }
    let mut text = String::from(cell.c);
    text.extend(zerowidth);
    row.push(RunSnapshot {
        column,
        cells: if wide { 2 } else { 1 },
        text,
        one_per_cell: !wide && zerowidth.is_empty(),
        style,
    });
}
