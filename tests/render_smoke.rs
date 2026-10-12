//! Headless smoke test: rasterize a scene and run every encoder path.

use glam::{Mat4, Vec3};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use terminal_engine::encode::{framebuffer_to_rgb24, ultra_framebuffer_size, Encoder};
use terminal_engine::{
    AsciiDensityEncoder, BgColorAsciiEncoder, Camera, HalfBlockEncoder, ImageProtocol,
    ImageProtocolEncoder, Mesh, Renderer, Rgb, Scene,
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

    // Ultra: higher-res framebuffer + RGB24 packing + Kitty encode into a sink.
    let (uw, uh) = ultra_framebuffer_size(48, 18);
    assert!(uw >= 48 && uh >= 18);
    let mut ultra = Renderer::new(uw, uh);
    ultra.render(&scene, &camera);
    let rgb = framebuffer_to_rgb24(&ultra.framebuffer);
    assert_eq!(rgb.len(), uw * uh * 3);

    let mut enc = ImageProtocolEncoder::new(ImageProtocol::Kitty);
    let mut sink = Vec::new();
    enc.present(&ultra.framebuffer, 0, 4, 48, 18, &mut sink)
        .expect("kitty present");
    let text = String::from_utf8_lossy(&sink);
    assert!(text.contains("_Ga=T,f=24,"));
    assert!(text.contains("\u{1b}[?2026h"));
    assert!(text.contains("\u{1b}[?2026l"));

    let mut iterm = ImageProtocolEncoder::new(ImageProtocol::ITerm2);
    let mut sink = Vec::new();
    iterm
        .present(&ultra.framebuffer, 0, 4, 48, 18, &mut sink)
        .expect("iterm present");
    let text = String::from_utf8_lossy(&sink);
    assert!(text.contains("\u{1b}]1337;File=inline=1;"));
    assert!(text.contains("width=48;height=18;"));
}
