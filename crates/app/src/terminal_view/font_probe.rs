//! Checks, without a window, that the platform can turn text into pixels.

use gpui::{PlatformTextSystem, Point, RenderGlyphParams, font};

use super::error::TerminalError;
use super::look::{DEFAULT_FONT_FAMILY as FONT_FAMILY, font_size};

const PROBE_CHARACTER: char = 'm';

/// Fails when the text system cannot find the terminal font or rasterizes
/// its glyphs to nothing, which is what a build without font support does.
pub fn probe_text_rendering(text_system: &dyn PlatformTextSystem) -> Result<(), TerminalError> {
    let installed = text_system.all_font_names();
    if !installed.iter().any(|name| name == FONT_FAMILY) {
        return Err(failure(format!(
            "font {FONT_FAMILY} not found among {} fonts",
            installed.len()
        )));
    }
    let font_id = text_system
        .font_id(&font(FONT_FAMILY))
        .map_err(|error| failure(format!("font {FONT_FAMILY} did not load: {error}")))?;
    let glyph_id = text_system
        .glyph_for_char(font_id, PROBE_CHARACTER)
        .ok_or_else(|| failure(format!("no glyph for {PROBE_CHARACTER:?}")))?;
    let params = RenderGlyphParams {
        font_id,
        glyph_id,
        font_size: gpui::px(font_size()),
        subpixel_variant: Point::default(),
        scale_factor: 2.0,
        is_emoji: false,
        subpixel_rendering: false,
        dilation: 0,
    };
    let bounds = text_system
        .glyph_raster_bounds(&params)
        .map_err(|error| failure(format!("glyph has no bounds: {error}")))?;
    let (_, pixels) = text_system
        .rasterize_glyph(&params, bounds)
        .map_err(|error| failure(format!("glyph did not rasterize: {error}")))?;
    if pixels.iter().all(|pixel| *pixel == 0) {
        return Err(failure(format!(
            "glyph for {PROBE_CHARACTER:?} rasterized to {} blank bytes",
            pixels.len()
        )));
    }
    Ok(())
}

fn failure(message: String) -> TerminalError {
    TerminalError::TextRendering(message)
}
