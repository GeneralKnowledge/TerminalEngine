//! Depth (Z) buffer for visibility testing.

/// Per-pixel depth buffer. Smaller depth values are closer to the camera.
#[derive(Clone, Debug)]
pub struct DepthBuffer {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

impl DepthBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0, "depth buffer dimensions must be > 0");
        Self {
            width,
            height,
            data: vec![f32::INFINITY; width * height],
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
    pub fn get(&self, x: usize, y: usize) -> f32 {
        debug_assert!(x < self.width && y < self.height);
        self.data[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, depth: f32) {
        debug_assert!(x < self.width && y < self.height);
        self.data[y * self.width + x] = depth;
    }

    /// Returns true if `depth` is closer than the stored value and updates it.
    #[inline]
    pub fn test_and_set(&mut self, x: usize, y: usize, depth: f32) -> bool {
        let slot = &mut self.data[y * self.width + x];
        if depth < *slot {
            *slot = depth;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(f32::INFINITY);
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        assert!(width > 0 && height > 0, "depth buffer dimensions must be > 0");
        if self.width == width && self.height == height {
            self.clear();
            return;
        }
        self.width = width;
        self.height = height;
        self.data.resize(width * height, f32::INFINITY);
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and_set_orders_by_nearer() {
        let mut depth = DepthBuffer::new(8, 8);
        assert!(depth.test_and_set(3, 3, 0.5));
        assert!(!depth.test_and_set(3, 3, 0.9));
        assert!(depth.test_and_set(3, 3, 0.2));
        assert!((depth.get(3, 3) - 0.2).abs() < f32::EPSILON);
        depth.clear();
        assert!(depth.get(3, 3).is_infinite());
    }
}
