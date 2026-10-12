//! Terminal image encoders — convert a shared framebuffer into cell output.
//!
//! Encoders are intentionally independent of the rasterizer. Any producer that
//! can fill a [`FrameBuffer`] can use them.

mod ascii;
mod bgcolor;
mod half_block;

pub use ascii::AsciiDensityEncoder;
pub use bgcolor::BgColorAsciiEncoder;
pub use half_block::{HalfBlockCell, HalfBlockEncoder};

use crate::framebuffer::FrameBuffer;

/// Converts a framebuffer into a terminal representation.
pub trait Encoder {
    /// Human-readable mode name for UI labels.
    fn name(&self) -> &'static str;

    /// Recommended framebuffer height for a viewport of `rows` terminal rows.
    fn framebuffer_height_for_rows(&self, rows: usize) -> usize {
        rows.max(1)
    }

    /// Recommended framebuffer width for a viewport of `cols` terminal columns.
    fn framebuffer_width_for_cols(&self, cols: usize) -> usize {
        cols.max(1)
    }
}

/// Shared density gradient (dark → light), similar in spirit to classic ASCII art
/// tables. Reimplemented independently (not copied from GPL sources).
pub(crate) const ASCII_GRADIENT: &[u8] = b" .:-=+*#%@";

#[inline]
pub(crate) fn density_char(luminance: f32) -> char {
    let idx = (luminance.clamp(0.0, 0.999) * ASCII_GRADIENT.len() as f32) as usize;
    ASCII_GRADIENT[idx] as char
}

/// Helper used by tests and ANSI fallbacks: encode the whole buffer as a
/// newline-separated string of density characters (no colour escapes).
pub fn encode_ascii_plain(fb: &FrameBuffer) -> String {
    let mut out = String::with_capacity(fb.width() * fb.height() + fb.height());
    for y in 0..fb.height() {
        for x in 0..fb.width() {
            out.push(density_char(fb.get(x, y).luminance()));
        }
        out.push('\n');
    }
    out
}
