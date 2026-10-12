//! Mesh containers and built-in primitives.

use glam::Vec3;

use crate::color::Rgb;
use crate::geometry::{Triangle, Vertex};

/// Opaque mesh handle for scene bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshId(pub u32);

/// Triangle mesh with per-vertex attributes.
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }

    pub fn cube(color: Rgb) -> Self {
        let positions = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
        ];
        let faces: [([usize; 4], Vec3); 6] = [
            ([0, 1, 2, 3], Vec3::NEG_Z),
            ([5, 4, 7, 6], Vec3::Z),
            ([4, 0, 3, 7], Vec3::NEG_X),
            ([1, 5, 6, 2], Vec3::X),
            ([3, 2, 6, 7], Vec3::Y),
            ([4, 5, 1, 0], Vec3::NEG_Y),
        ];

        let mut triangles = Vec::with_capacity(12);
        for (idx, normal) in faces {
            let v = |i: usize| Vertex::new(positions[i], normal, color);
            triangles.push(Triangle::new(v(idx[0]), v(idx[1]), v(idx[2])));
            triangles.push(Triangle::new(v(idx[0]), v(idx[2]), v(idx[3])));
        }
        Self { triangles }
    }

    pub fn pyramid(color: Rgb) -> Self {
        let apex = Vec3::new(0.0, 1.2, 0.0);
        let base = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
        ];
        let mut triangles = Vec::new();
        for i in 0..4 {
            let a = base[i];
            let b = base[(i + 1) % 4];
            let n = (b - a).cross(apex - a).normalize_or_zero();
            triangles.push(Triangle::new(
                Vertex::new(a, n, color),
                Vertex::new(b, n, color),
                Vertex::new(apex, n, color),
            ));
        }
        let n = Vec3::NEG_Y;
        triangles.push(Triangle::new(
            Vertex::new(base[0], n, color),
            Vertex::new(base[2], n, color),
            Vertex::new(base[1], n, color),
        ));
        triangles.push(Triangle::new(
            Vertex::new(base[0], n, color),
            Vertex::new(base[3], n, color),
            Vertex::new(base[2], n, color),
        ));
        Self { triangles }
    }
}
