//! CPU triangle rasterization into a shared framebuffer.

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::color::Rgb;
use crate::depth::DepthBuffer;
use crate::framebuffer::FrameBuffer;
use crate::geometry::Triangle;

/// Configuration for the software rasterizer.
#[derive(Clone, Debug)]
pub struct Rasterizer {
    pub light_dir: Vec3,
    pub ambient: f32,
}

impl Default for Rasterizer {
    fn default() -> Self {
        Self {
            light_dir: Vec3::new(0.35, 0.8, 0.45).normalize(),
            ambient: 0.18,
        }
    }
}

impl Rasterizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rasterize a model-space triangle using `model` then `view_proj`.
    pub fn draw_triangle(
        &self,
        framebuffer: &mut FrameBuffer,
        depth: &mut DepthBuffer,
        view_proj: Mat4,
        model: Mat4,
        triangle: &Triangle,
    ) {
        let mvp = view_proj * model;
        let normal_matrix = model.inverse().transpose();

        let clip = |pos: Vec3| mvp * Vec4::new(pos.x, pos.y, pos.z, 1.0);
        let p0 = clip(triangle.v0.position);
        let p1 = clip(triangle.v1.position);
        let p2 = clip(triangle.v2.position);

        // Reject triangles entirely behind the near plane (w <= 0).
        if p0.w <= 0.0 && p1.w <= 0.0 && p2.w <= 0.0 {
            return;
        }

        let width = framebuffer.width();
        let height = framebuffer.height();
        let (s0, iz0) = to_screen(p0, width, height);
        let (s1, iz1) = to_screen(p1, width, height);
        let (s2, iz2) = to_screen(p2, width, height);

        let area = edge(s0, s1, s2);
        if area.abs() < 1e-4 {
            return;
        }
        let inv_area = 1.0 / area;

        // Transform normals for lighting.
        let n0 = transform_normal(normal_matrix, triangle.v0.normal);
        let n1 = transform_normal(normal_matrix, triangle.v1.normal);
        let n2 = transform_normal(normal_matrix, triangle.v2.normal);

        let min_x = s0.x.min(s1.x).min(s2.x).floor().max(0.0) as usize;
        let max_x = s0.x.max(s1.x).max(s2.x).ceil().min(width as f32) as usize;
        let min_y = s0.y.min(s1.y).min(s2.y).floor().max(0.0) as usize;
        let max_y = s0.y.max(s1.y).max(s2.y).ceil().min(height as f32) as usize;

        for y in min_y..max_y {
            for x in min_x..max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge(s1, s2, p) * inv_area;
                let w1 = edge(s2, s0, p) * inv_area;
                let w2 = edge(s0, s1, p) * inv_area;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }

                let frag_depth = w0 * iz0 + w1 * iz1 + w2 * iz2;
                if !depth.test_and_set(x, y, frag_depth) {
                    continue;
                }

                let normal = (n0 * w0 + n1 * w1 + n2 * w2).normalize_or_zero();
                let base = lerp_color(triangle.v0.color, triangle.v1.color, triangle.v2.color, w0, w1, w2);
                let ndotl = normal.dot(self.light_dir).abs();
                let shade = (self.ambient + (1.0 - self.ambient) * ndotl).clamp(0.0, 1.0);
                framebuffer.set(x, y, scale_rgb(base, shade));
            }
        }
    }
}

#[inline]
fn edge(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x)
}

#[inline]
fn to_screen(clip: Vec4, width: usize, height: usize) -> (Vec2, f32) {
    let inv_w = 1.0 / clip.w;
    let ndc_x = clip.x * inv_w;
    let ndc_y = clip.y * inv_w;
    let x = (ndc_x * 0.5 + 0.5) * width as f32;
    let y = (-ndc_y * 0.5 + 0.5) * height as f32;
    (Vec2::new(x, y), inv_w)
}

#[inline]
fn transform_normal(m: Mat4, n: Vec3) -> Vec3 {
    (m * Vec4::new(n.x, n.y, n.z, 0.0)).truncate().normalize_or_zero()
}

#[inline]
fn lerp_color(c0: Rgb, c1: Rgb, c2: Rgb, w0: f32, w1: f32, w2: f32) -> Rgb {
    Rgb::new(
        (c0.r as f32 * w0 + c1.r as f32 * w1 + c2.r as f32 * w2) as u8,
        (c0.g as f32 * w0 + c1.g as f32 * w1 + c2.g as f32 * w2) as u8,
        (c0.b as f32 * w0 + c1.b as f32 * w1 + c2.b as f32 * w2) as u8,
    )
}

#[inline]
fn scale_rgb(color: Rgb, shade: f32) -> Rgb {
    Rgb::new(
        (color.r as f32 * shade) as u8,
        (color.g as f32 * shade) as u8,
        (color.b as f32 * shade) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Vertex;
    use glam::Vec3;

    #[test]
    fn draws_visible_triangle() {
        let mut fb = FrameBuffer::new(64, 64);
        let mut depth = DepthBuffer::new(64, 64);
        let raster = Rasterizer::new();
        let view_proj = Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0)
            * Mat4::look_at_rh(Vec3::new(0.0, 0.0, 3.0), Vec3::ZERO, Vec3::Y);
        let tri = Triangle::new(
            Vertex::new(Vec3::new(0.0, 0.7, 0.0), Vec3::Z, Rgb::new(255, 0, 0)),
            Vertex::new(Vec3::new(-0.7, -0.7, 0.0), Vec3::Z, Rgb::new(0, 255, 0)),
            Vertex::new(Vec3::new(0.7, -0.7, 0.0), Vec3::Z, Rgb::new(0, 0, 255)),
        );
        raster.draw_triangle(&mut fb, &mut depth, view_proj, Mat4::IDENTITY, &tri);

        let lit = fb.pixels().iter().filter(|c| c.channel_sum() > 0).count();
        assert!(lit > 100, "expected filled triangle pixels, got {lit}");
    }
}
