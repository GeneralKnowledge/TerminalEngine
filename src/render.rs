//! High-level scene and renderer orchestration.

use glam::Mat4;

use crate::camera::Camera;
use crate::color::Rgb;
use crate::depth::DepthBuffer;
use crate::framebuffer::FrameBuffer;
use crate::mesh::Mesh;
use crate::raster::Rasterizer;

/// A mesh instance with a model transform.
#[derive(Clone, Debug)]
pub struct Object {
    pub mesh: Mesh,
    pub transform: Mat4,
}

/// Collection of objects drawn each frame.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub objects: Vec<Object>,
    pub clear_color: Rgb,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            clear_color: Rgb::new(12, 14, 20),
        }
    }

    pub fn add_object(&mut self, mesh: Mesh, transform: Mat4) {
        self.objects.push(Object { mesh, transform });
    }
}

/// Owns the colour/depth targets and drives the rasterizer.
#[derive(Debug)]
pub struct Renderer {
    pub framebuffer: FrameBuffer,
    pub depth: DepthBuffer,
    pub rasterizer: Rasterizer,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            framebuffer: FrameBuffer::new(width, height),
            depth: DepthBuffer::new(width, height),
            rasterizer: Rasterizer::new(),
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.framebuffer.resize(width, height);
        self.depth.resize(width, height);
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.framebuffer.width(), self.framebuffer.height())
    }

    pub fn render(&mut self, scene: &Scene, camera: &Camera) {
        self.framebuffer.clear(scene.clear_color);
        self.depth.clear();
        let (w, h) = self.dimensions();
        let aspect = w as f32 / h as f32;
        let view_proj = camera.view_projection(aspect);
        for object in &scene.objects {
            for triangle in &object.mesh.triangles {
                self.rasterizer.draw_triangle(
                    &mut self.framebuffer,
                    &mut self.depth,
                    view_proj,
                    object.transform,
                    triangle,
                );
            }
        }
    }
}
