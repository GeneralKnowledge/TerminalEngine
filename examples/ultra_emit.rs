//! One-shot Ultra Kitty emission (no interactive TUI).
use std::io::{self, Write};
use glam::{Mat4, Vec3};
use terminal_engine::{
    Camera, ImageProtocol, ImageProtocolEncoder, Mesh, Renderer, Rgb, Scene,
};

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    scene.add_object(Mesh::cube(Rgb::new(220, 170, 60)), Mat4::IDENTITY);
    let camera = Camera::new(Vec3::new(0.0, 1.2, 4.0), Vec3::ZERO);
    let mut renderer = Renderer::new(80, 60);
    renderer.render(&scene, &camera);
    let mut enc = ImageProtocolEncoder::new(ImageProtocol::Kitty);
    let mut out = Vec::new();
    enc.present(&renderer.framebuffer, 0, 0, 40, 20, &mut out)?;
    io::stdout().write_all(b"MARKER_START")?;
    io::stdout().write_all(&out)?;
    io::stdout().write_all(b"MARKER_END\n")?;
    eprintln!("EMIT_OK bytes={}", out.len());
    Ok(())
}
