//! Draws a grid snapshot with gpui primitives.

use std::sync::atomic::{AtomicBool, Ordering};

use alacritty_terminal::vte::ansi::{CursorShape, Rgb};
use gpui::{
    App, Bounds, Font, FontStyle, FontWeight, Hsla, Pixels, Rgba, SharedString, Size,
    StrikethroughStyle, TextAlign, TextRun, UnderlineStyle, Window, fill, font, outline, point, px,
    size,
};

use super::grid::{CellStyle, CursorSnapshot, GridSnapshot, RunSnapshot};
use super::palette;

pub(super) const FONT_FAMILY: &str = "Menlo";
pub(super) const FONT_SIZE: f32 = 13.0;
const LINE_HEIGHT_FACTOR: f32 = 1.2;
const FOCUSED_CURSOR_ALPHA: f32 = 0.55;

pub(super) struct FontMetrics {
    pub font: Font,
    pub font_size: Pixels,
    pub cell: Size<Pixels>,
}

pub(super) struct Frame {
    pub snapshot: GridSnapshot,
    /// Text the input method is still composing, shown at the cursor.
    pub marked: String,
    pub focused: bool,
}

pub(super) fn measure(window: &Window) -> FontMetrics {
    let font = font(FONT_FAMILY);
    let font_size = px(FONT_SIZE);
    let text_system = window.text_system();
    let font_id = text_system.resolve_font(&font);
    let width = text_system
        .advance(font_id, font_size, 'm')
        .map(|advance| advance.width)
        .unwrap_or(font_size * 0.6);
    FontMetrics {
        font,
        font_size,
        cell: size(width, px((FONT_SIZE * LINE_HEIGHT_FACTOR).ceil())),
    }
}

fn color(rgb: Rgb, alpha: f32) -> Hsla {
    Rgba {
        r: f32::from(rgb.r) / 255.0,
        g: f32::from(rgb.g) / 255.0,
        b: f32::from(rgb.b) / 255.0,
        a: alpha,
    }
    .into()
}

fn cell_bounds(
    bounds: Bounds<Pixels>,
    metrics: &FontMetrics,
    line: usize,
    column: usize,
    cells: usize,
) -> Bounds<Pixels> {
    Bounds::new(
        point(
            bounds.origin.x + metrics.cell.width * column as f32,
            bounds.origin.y + metrics.cell.height * line as f32,
        ),
        size(metrics.cell.width * cells as f32, metrics.cell.height),
    )
}

pub(super) fn cursor_bounds(
    bounds: Bounds<Pixels>,
    metrics: &FontMetrics,
    cursor: &CursorSnapshot,
) -> Bounds<Pixels> {
    let cells = if cursor.wide { 2 } else { 1 };
    cell_bounds(bounds, metrics, cursor.line, cursor.column, cells)
}

pub(super) fn paint_frame(
    bounds: Bounds<Pixels>,
    metrics: &FontMetrics,
    frame: &Frame,
    window: &mut Window,
    cx: &mut App,
) {
    window.paint_quad(fill(bounds, color(palette::BACKGROUND, 1.0)));
    for (line, runs) in frame.snapshot.rows.iter().enumerate() {
        for run in runs {
            if let Some(background) = run.style.background {
                let area = cell_bounds(bounds, metrics, line, run.column, run.cells);
                window.paint_quad(fill(area, color(background, 1.0)));
            }
        }
        for run in runs {
            paint_run(bounds, metrics, line, run, window, cx);
        }
    }
    if let Some(cursor) = &frame.snapshot.cursor {
        if !frame.marked.is_empty() {
            paint_marked(bounds, metrics, cursor, &frame.marked, window, cx);
        } else if cursor.visible {
            paint_cursor(bounds, metrics, cursor, frame.focused, window);
        }
    }
}

fn text_run(metrics: &FontMetrics, style: &CellStyle, len: usize) -> TextRun {
    let mut font = metrics.font.clone();
    if style.bold {
        font.weight = FontWeight::BOLD;
    }
    if style.italic {
        font.style = FontStyle::Italic;
    }
    let foreground = color(style.foreground, 1.0);
    TextRun {
        len,
        font,
        color: foreground,
        background_color: None,
        underline: style.underline.then_some(UnderlineStyle {
            thickness: px(1.0),
            color: Some(foreground),
            wavy: false,
        }),
        strikethrough: style.strikeout.then_some(StrikethroughStyle {
            thickness: px(1.0),
            color: Some(foreground),
        }),
    }
}

fn paint_run(
    bounds: Bounds<Pixels>,
    metrics: &FontMetrics,
    line: usize,
    run: &RunSnapshot,
    window: &mut Window,
    cx: &mut App,
) {
    if run.text.trim().is_empty() && !run.style.underline && !run.style.strikeout {
        return;
    }
    // One glyph per cell keeps columns aligned whatever the font's advance
    // is; wide and combined characters take their natural width.
    let shaped = window.text_system().shape_line(
        SharedString::new(run.text.as_str()),
        metrics.font_size,
        &[text_run(metrics, &run.style, run.text.len())],
        run.one_per_cell.then_some(metrics.cell.width),
    );
    let origin = cell_bounds(bounds, metrics, line, run.column, run.cells).origin;
    // A glyph that fails to rasterize leaves a gap; the frame still paints.
    let painted = shaped.paint(
        origin,
        metrics.cell.height,
        TextAlign::Left,
        None,
        window,
        cx,
    );
    report_once(painted);
}

/// Says on stderr, once per process, that text failed to paint.
fn report_once(painted: gpui::Result<()>) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if let Err(error) = painted
        && !REPORTED.swap(true, Ordering::Relaxed)
    {
        eprintln!("lithemux: text failed to paint: {error:#}");
    }
}

fn paint_cursor(
    bounds: Bounds<Pixels>,
    metrics: &FontMetrics,
    cursor: &CursorSnapshot,
    focused: bool,
    window: &mut Window,
) {
    let cell = cursor_bounds(bounds, metrics, cursor);
    let solid = color(palette::CURSOR, 1.0);
    if !focused {
        window.paint_quad(outline(cell, solid, gpui::BorderStyle::Solid));
        return;
    }
    let thin = px(2.0);
    let area = match cursor.shape {
        CursorShape::Beam => Bounds::new(cell.origin, size(thin, cell.size.height)),
        CursorShape::Underline => Bounds::new(
            point(cell.origin.x, cell.origin.y + cell.size.height - thin),
            size(cell.size.width, thin),
        ),
        _ => {
            let translucent = color(palette::CURSOR, FOCUSED_CURSOR_ALPHA);
            window.paint_quad(fill(cell, translucent));
            return;
        }
    };
    window.paint_quad(fill(area, solid));
}

fn paint_marked(
    bounds: Bounds<Pixels>,
    metrics: &FontMetrics,
    cursor: &CursorSnapshot,
    marked: &str,
    window: &mut Window,
    cx: &mut App,
) {
    let style = CellStyle {
        foreground: palette::FOREGROUND,
        background: None,
        bold: false,
        italic: false,
        underline: true,
        strikeout: false,
    };
    let shaped = window.text_system().shape_line(
        SharedString::new(marked),
        metrics.font_size,
        &[text_run(metrics, &style, marked.len())],
        None,
    );
    let origin = cursor_bounds(bounds, metrics, cursor).origin;
    let area = Bounds::new(origin, size(shaped.width, metrics.cell.height));
    window.paint_quad(fill(area, color(palette::BACKGROUND, 1.0)));
    let painted = shaped.paint(
        origin,
        metrics.cell.height,
        TextAlign::Left,
        None,
        window,
        cx,
    );
    report_once(painted);
}
