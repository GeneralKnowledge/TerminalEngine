//! Foreground-coloured ASCII density encoder.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::{density_char, Encoder};
use crate::framebuffer::FrameBuffer;

/// Maps each framebuffer pixel to one terminal cell: density glyph + FG colour.
#[derive(Clone, Debug, Default)]
pub struct AsciiDensityEncoder;

impl Encoder for AsciiDensityEncoder {
    fn name(&self) -> &'static str {
        "ascii"
    }
}

impl AsciiDensityEncoder {
    pub fn new() -> Self {
        Self
    }

    /// Render into a Ratatui buffer rectangle (1:1 pixel → cell).
    pub fn render_to_buffer(&self, fb: &FrameBuffer, area: Rect, buf: &mut Buffer) {
        let w = (area.width as usize).min(fb.width());
        let h = (area.height as usize).min(fb.height());
        for y in 0..h {
            for x in 0..w {
                let px = fb.get(x, y);
                if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
                    cell.set_char(density_char(px.luminance()));
                    cell.set_fg(Color::Rgb(px.r, px.g, px.b));
                    cell.set_bg(Color::Reset);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgb;

    #[test]
    fn encodes_luminance_extremes() {
        let mut fb = FrameBuffer::new(2, 1);
        fb.set(0, 0, Rgb::BLACK);
        fb.set(1, 0, Rgb::WHITE);
        let plain = crate::encode::encode_ascii_plain(&fb);
        assert!(plain.starts_with(' '));
        assert!(plain.contains('@'));
    }
}
