//! Background-colour ASCII encoder.
//!
//! Inspired by the doom-ascii-bgcolor approach (GPL reference — not copied):
//! fill the cell with true-colour background and use a dark density glyph.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::{density_char, Encoder};
use crate::color::Rgb;
use crate::framebuffer::FrameBuffer;

/// Each pixel becomes a cell with `48;2`-style background and a dim glyph.
#[derive(Clone, Debug, Default)]
pub struct BgColorAsciiEncoder {
    pub foreground: Rgb,
}

impl Encoder for BgColorAsciiEncoder {
    fn name(&self) -> &'static str {
        "bgcolor"
    }
}

impl BgColorAsciiEncoder {
    pub fn new() -> Self {
        Self {
            foreground: Rgb::new(20, 20, 20),
        }
    }

    pub fn render_to_buffer(&self, fb: &FrameBuffer, area: Rect, buf: &mut Buffer) {
        let w = (area.width as usize).min(fb.width());
        let h = (area.height as usize).min(fb.height());
        for y in 0..h {
            for x in 0..w {
                let px = fb.get(x, y);
                if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
                    // Invert luminance for glyph so bright areas stay readable on bright BG.
                    cell.set_char(density_char(1.0 - px.luminance()));
                    cell.set_fg(Color::Rgb(
                        self.foreground.r,
                        self.foreground.g,
                        self.foreground.b,
                    ));
                    cell.set_bg(Color::Rgb(px.r, px.g, px.b));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_and_defaults() {
        let enc = BgColorAsciiEncoder::new();
        assert_eq!(enc.name(), "bgcolor");
        assert_eq!(enc.foreground, Rgb::new(20, 20, 20));
    }
}
