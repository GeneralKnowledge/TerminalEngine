//! Vertex and triangle primitives.

use glam::Vec3;

use crate::color::Rgb;

/// A vertex in model space with colour and normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Rgb,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, color: Rgb) -> Self {
        Self {
            position,
            normal: normal.normalize_or_zero(),
            color,
        }
    }
}

/// A triangle referencing three vertices (by value for simplicity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
}

impl Triangle {
    pub fn new(v0: Vertex, v1: Vertex, v2: Vertex) -> Self {
        Self { v0, v1, v2 }
    }

    /// Face normal from positions (not necessarily unit length).
    pub fn face_normal(self) -> Vec3 {
        let e1 = self.v1.position - self.v0.position;
        let e2 = self.v2.position - self.v0.position;
        e1.cross(e2)
    }
}
