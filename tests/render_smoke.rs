//! Headless smoke test: rasterize a scene and run every encoder path.

use glam::{Mat4, Vec3};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use terminal_engine::encode::Encoder;
use terminal_engine::{
    AsciiDensityEncoder, BgColorAsciiEncoder, Camera, HalfBlockEncoder, Mesh, Renderer, Rgb, Scene,
};

#[test]
fn render_and_encode_all_modes() {
    let mut scene = Scene::new();
    scene.add_object(
        Mesh::cube(Rgb::new(200, 120, 40)),
        Mat4::from_rotation_y(0.4) * Mat4::from_rotation_x(0.3),
    );

    let camera = Camera::new(Vec3::new(0.0, 1.0, 3.5), Vec3::ZERO);
    let mut renderer = Renderer::new(48, 36);
    renderer.render(&scene, &camera);

    let lit = renderer
        .framebuffer
        .pixels()
        .iter()
        .filter(|c| c.channel_sum() > 0)
        .count();
    assert!(lit > 50, "cube should cover a non-trivial area, got {lit}");

    let area = Rect::new(0, 0, 48, 18);
    let mut buf = Buffer::empty(area);

    AsciiDensityEncoder.render_to_buffer(&renderer.framebuffer, area, &mut buf);
    assert_ne!(buf[(0, 0)].symbol(), "");

    // Half-block needs doubled framebuffer height relative to rows.
    assert_eq!(HalfBlockEncoder.framebuffer_height_for_rows(18), 36);
    HalfBlockEncoder.render_to_buffer(&renderer.framebuffer, area, &mut buf);
    assert_eq!(buf[(24, 9)].symbol(), "▀");

    BgColorAsciiEncoder::new().render_to_buffer(&renderer.framebuffer, area, &mut buf);
}
