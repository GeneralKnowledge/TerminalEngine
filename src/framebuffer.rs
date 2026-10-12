//! Renderer-independent colour framebuffer.

use crate::color::Rgb;

/// Dense RGB colour buffer written by the rasterizer and read by encoders.
#[derive(Clone, Debug)]
pub struct FrameBuffer {
    width: usize,
    height: usize,
    pixels: Vec<Rgb>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0, "framebuffer dimensions must be > 0");
        Self {
            width,
            height,
            pixels: vec![Rgb::BLACK; width * height],
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn pixels(&self) -> &[Rgb] {
        &self.pixels
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Rgb {
        debug_assert!(x < self.width && y < self.height);
        self.pixels[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, color: Rgb) {
        debug_assert!(x < self.width && y < self.height);
        self.pixels[y * self.width + x] = color;
    }

    pub fn clear(&mut self, color: Rgb) {
        self.pixels.fill(color);
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        assert!(width > 0 && height > 0, "framebuffer dimensions must be > 0");
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        self.pixels.resize(width * height, Rgb::BLACK);
        self.pixels.fill(Rgb::BLACK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_clear() {
        let mut fb = FrameBuffer::new(4, 3);
        fb.set(1, 2, Rgb::new(10, 20, 30));
        assert_eq!(fb.get(1, 2), Rgb::new(10, 20, 30));
        fb.clear(Rgb::WHITE);
        assert_eq!(fb.get(1, 2), Rgb::WHITE);
    }

    #[test]
    fn resize_preserves_capacity_semantics() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set(0, 0, Rgb::new(1, 2, 3));
        fb.resize(8, 4);
        assert_eq!(fb.width(), 8);
        assert_eq!(fb.height(), 4);
        assert_eq!(fb.get(0, 0), Rgb::BLACK);
    }
}
