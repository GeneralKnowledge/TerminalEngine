//! Half-block true-colour encoder (two vertical pixels per terminal cell).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::Encoder;
use crate::color::Rgb;
use crate::framebuffer::FrameBuffer;

/// One terminal cell produced by the half-block encoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HalfBlockCell {
    pub top: Rgb,
    pub bottom: Rgb,
    pub glyph: char,
}

/// Packs two framebuffer rows into each terminal row using `▀` (FG=top, BG=bottom).
///
/// This matches the technique demonstrated by Ratatui's `colors_rgb` example and
/// is related to rasterm's `▄` convention (inverted glyph, same idea).
#[derive(Clone, Debug, Default)]
pub struct HalfBlockEncoder;

impl Encoder for HalfBlockEncoder {
    fn name(&self) -> &'static str {
        "half-block"
    }

    fn framebuffer_height_for_rows(&self, rows: usize) -> usize {
        rows.max(1) * 2
    }
}

impl HalfBlockEncoder {
    pub fn new() -> Self {
        Self
    }

    pub fn cell_at(&self, fb: &FrameBuffer, x: usize, cell_y: usize) -> HalfBlockCell {
        let y_top = cell_y * 2;
        let y_bottom = y_top + 1;
        let top = if y_top < fb.height() {
            fb.get(x, y_top)
        } else {
            Rgb::BLACK
        };
        let bottom = if y_bottom < fb.height() {
            fb.get(x, y_bottom)
        } else {
            top
        };
        HalfBlockCell {
            top,
            bottom,
            glyph: '▀',
        }
    }

    pub fn render_to_buffer(&self, fb: &FrameBuffer, area: Rect, buf: &mut Buffer) {
        let w = (area.width as usize).min(fb.width());
        let h = (area.height as usize).min(fb.height().div_ceil(2));
        for y in 0..h {
            for x in 0..w {
                let cell_data = self.cell_at(fb, x, y);
                if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
                    cell.set_char(cell_data.glyph);
                    cell.set_fg(Color::Rgb(
                        cell_data.top.r,
                        cell_data.top.g,
                        cell_data.top.b,
                    ));
                    cell.set_bg(Color::Rgb(
                        cell_data.bottom.r,
                        cell_data.bottom.g,
                        cell_data.bottom.b,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_two_rows() {
        let mut fb = FrameBuffer::new(1, 2);
        fb.set(0, 0, Rgb::new(255, 0, 0));
        fb.set(0, 1, Rgb::new(0, 0, 255));
        let enc = HalfBlockEncoder::new();
        let cell = enc.cell_at(&fb, 0, 0);
        assert_eq!(cell.glyph, '▀');
        assert_eq!(cell.top, Rgb::new(255, 0, 0));
        assert_eq!(cell.bottom, Rgb::new(0, 0, 255));
        assert_eq!(enc.framebuffer_height_for_rows(10), 20);
    }
}
