//! Interactive TerminalEngine demo.
//!
//! Controls are laid out separately from the graphics viewport (Ratatui layout).
//! Encoder modes can be cycled without changing the rasterizer or framebuffer producer.
//! Ultra mode paints via Kitty / iTerm2 image protocols after the Ratatui frame.

use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use glam::{Mat4, Quat, Vec3};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use terminal_engine::encode::Encoder;
use terminal_engine::{
    detect_image_protocol, ultra_framebuffer_size, AsciiDensityEncoder, BgColorAsciiEncoder,
    Camera, HalfBlockEncoder, ImageProtocolEncoder, Mesh, Renderer, Rgb, Scene,
};

#[derive(Parser, Debug)]
#[command(name = "te-demo", about = "TerminalEngine interactive demo")]
struct Args {
    /// Initial encoder: ascii | half-block | bgcolor | ultra
    ///
    /// Prefer `half-block` on terminals with working true-colour backgrounds
    /// (Kitty, Ghostty, WezTerm, …). Prefer `ultra` on Kitty / Ghostty / WezTerm /
    /// iTerm2 for native pixel graphics. `ascii` is the safe default.
    #[arg(long, default_value = "ascii")]
    mode: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Ascii,
    HalfBlock,
    BgColor,
    Ultra,
}

impl Mode {
    fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "ascii" => Self::Ascii,
            "bgcolor" | "bg" => Self::BgColor,
            "ultra" | "kitty" | "iterm" | "iterm2" | "image" => Self::Ultra,
            _ => Self::HalfBlock,
        }
    }

    fn next(self, ultra_available: bool) -> Self {
        match self {
            Self::Ascii => Self::HalfBlock,
            Self::HalfBlock => Self::BgColor,
            Self::BgColor => {
                if ultra_available {
                    Self::Ultra
                } else {
                    Self::Ascii
                }
            }
            Self::Ultra => Self::Ascii,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::HalfBlock => "half-block",
            Self::BgColor => "bgcolor",
            Self::Ultra => "ultra",
        }
    }
}

struct App {
    mode: Mode,
    scene: Scene,
    camera: Camera,
    renderer: Renderer,
    angle: f32,
    auto_rotate: bool,
    mesh_kind: usize,
    running: bool,
    fps: f32,
    frames: u32,
    last_fps: Instant,
    last_frame: Instant,
    image: Option<ImageProtocolEncoder>,
    /// Viewport content rect from the last draw (for Ultra placement).
    last_viewport: Rect,
    status_note: String,
}

impl App {
    fn new(requested: Mode) -> Self {
        let image = ImageProtocolEncoder::try_detect();
        let mut mode = requested;
        let mut status_note = String::new();
        if mode == Mode::Ultra {
            if let Some(ref enc) = image {
                status_note = format!("ultra via {}", enc.protocol().name());
            } else {
                mode = Mode::Ascii;
                status_note =
                    "ultra unavailable here — set TERMINAL_ENGINE_IMAGE_PROTOCOL or use Kitty/iTerm"
                        .into();
            }
        } else if let Some(ref enc) = image {
            status_note = format!("ultra ready ({})", enc.protocol().name());
        }

        let mut scene = Scene::new();
        scene.add_object(Mesh::cube(Rgb::new(220, 170, 60)), Mat4::IDENTITY);
        Self {
            mode,
            scene,
            camera: Camera::new(Vec3::new(0.0, 1.2, 4.0), Vec3::ZERO),
            renderer: Renderer::new(80, 40),
            angle: 0.0,
            auto_rotate: true,
            mesh_kind: 0,
            running: true,
            fps: 0.0,
            frames: 0,
            last_fps: Instant::now(),
            last_frame: Instant::now(),
            image,
            last_viewport: Rect::default(),
            status_note,
        }
    }

    fn ultra_available(&self) -> bool {
        self.image.is_some()
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            self.tick();
            terminal.draw(|frame| self.draw(frame))?;
            if self.mode == Mode::Ultra {
                self.present_ultra()?;
            }
            self.handle_input()?;
        }
        if let Some(ref image) = self.image {
            let mut out = io::stdout();
            image.cleanup(&mut out)?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        if self.auto_rotate {
            self.angle += dt * 0.9;
        }
        let rotation = Mat4::from_quat(Quat::from_euler(
            glam::EulerRot::YXZ,
            self.angle,
            self.angle * 0.35,
            0.0,
        ));
        if let Some(object) = self.scene.objects.first_mut() {
            object.transform = rotation;
        }

        self.frames += 1;
        if self.last_fps.elapsed() >= Duration::from_secs(1) {
            self.fps = self.frames as f32 / self.last_fps.elapsed().as_secs_f32();
            self.frames = 0;
            self.last_fps = Instant::now();
        }
    }

    fn ensure_framebuffer(&mut self, viewport: Rect) {
        let cols = viewport.width.max(1) as usize;
        let rows = viewport.height.max(1) as usize;
        let (w, h) = match self.mode {
            Mode::HalfBlock => (
                HalfBlockEncoder.framebuffer_width_for_cols(cols),
                HalfBlockEncoder.framebuffer_height_for_rows(rows),
            ),
            Mode::Ascii => (
                AsciiDensityEncoder.framebuffer_width_for_cols(cols),
                AsciiDensityEncoder.framebuffer_height_for_rows(rows),
            ),
            Mode::BgColor => (
                BgColorAsciiEncoder::new().framebuffer_width_for_cols(cols),
                BgColorAsciiEncoder::new().framebuffer_height_for_rows(rows),
            ),
            Mode::Ultra => ultra_framebuffer_size(cols, rows),
        };
        if self.renderer.dimensions() != (w, h) {
            self.renderer.resize(w, h);
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let [controls, viewport] =
            Layout::vertical([Constraint::Length(5), Constraint::Min(3)]).areas(area);

        let panel = Style::default().fg(Color::Gray).bg(Color::Rgb(12, 14, 20));
        let mode_extra = if self.mode == Mode::Ultra {
            self.image
                .as_ref()
                .map(|e| format!("/{}", e.protocol().name()))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let help = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    " TerminalEngine ",
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Rgb(12, 14, 20))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        " mode={}{}  mesh={}  {:.0} fps",
                        self.mode.label(),
                        mode_extra,
                        if self.mesh_kind == 0 {
                            "cube"
                        } else {
                            "pyramid"
                        },
                        self.fps
                    ),
                    panel,
                ),
            ]),
            Line::from(Span::styled(
                "q quit · m next mode · 1/2 mesh · space toggle spin · arrows orbit · +/- zoom",
                panel,
            )),
            Line::from(Span::styled(
                if self.status_note.is_empty() {
                    if self.ultra_available() {
                        "ultra available — press m to cycle (ascii→half-block→bgcolor→ultra)"
                            .to_string()
                    } else {
                        "ultra unavailable on this terminal (cell modes only)".to_string()
                    }
                } else {
                    self.status_note.clone()
                },
                panel,
            )),
        ])
        .style(panel)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(panel)
                .title("controls")
                .style(panel),
        );
        frame.render_widget(help, controls);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(panel)
            .title("viewport")
            .style(panel);
        let inner = block.inner(viewport);
        frame.render_widget(block, viewport);
        self.last_viewport = inner;

        self.ensure_framebuffer(inner);
        self.renderer.render(&self.scene, &self.camera);

        let fb = &self.renderer.framebuffer;
        match self.mode {
            Mode::Ascii => AsciiDensityEncoder.render_to_buffer(fb, inner, frame.buffer_mut()),
            Mode::HalfBlock => HalfBlockEncoder.render_to_buffer(fb, inner, frame.buffer_mut()),
            Mode::BgColor => {
                BgColorAsciiEncoder::new().render_to_buffer(fb, inner, frame.buffer_mut())
            }
            Mode::Ultra => {
                // Leave the Ratatui cells as the panel background; pixels are
                // painted afterward via the image protocol.
            }
        }
    }

    fn present_ultra(&mut self) -> Result<()> {
        let Some(image) = self.image.as_mut() else {
            return Ok(());
        };
        let area = self.last_viewport;
        if area.width == 0 || area.height == 0 {
            return Ok(());
        }
        let mut out = io::stdout();
        image.present(
            &self.renderer.framebuffer,
            area.x,
            area.y,
            area.width,
            area.height,
            &mut out,
        )?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f32(1.0 / 60.0);
        if !event::poll(timeout)? {
            return Ok(());
        }
        let Event::Key(key) = event::read()? else {
            return Ok(());
        };
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Char('m') => {
                self.mode = self.mode.next(self.ultra_available());
                if self.mode == Mode::Ultra {
                    if let Some(ref enc) = self.image {
                        self.status_note = format!("ultra via {}", enc.protocol().name());
                    }
                } else {
                    self.status_note.clear();
                }
            }
            KeyCode::Char(' ') => self.auto_rotate = !self.auto_rotate,
            KeyCode::Char('1') => self.set_mesh(0),
            KeyCode::Char('2') => self.set_mesh(1),
            KeyCode::Left => self.camera.orbit(-0.08, 0.0, 1.0),
            KeyCode::Right => self.camera.orbit(0.08, 0.0, 1.0),
            KeyCode::Up => self.camera.orbit(0.0, 0.06, 1.0),
            KeyCode::Down => self.camera.orbit(0.0, -0.06, 1.0),
            KeyCode::Char('+') | KeyCode::Char('=') => self.camera.orbit(0.0, 0.0, 0.9),
            KeyCode::Char('-') | KeyCode::Char('_') => self.camera.orbit(0.0, 0.0, 1.1),
            _ => {}
        }
        Ok(())
    }

    fn set_mesh(&mut self, kind: usize) {
        self.mesh_kind = kind;
        let mesh = if kind == 0 {
            Mesh::cube(Rgb::new(220, 170, 60))
        } else {
            Mesh::pyramid(Rgb::new(80, 180, 220))
        };
        if let Some(object) = self.scene.objects.first_mut() {
            object.mesh = mesh;
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mode = Mode::from_str(&args.mode);
    // Force-detect path for debugging: printed only when ultra was requested.
    if matches!(mode, Mode::Ultra) && detect_image_protocol().is_none() {
        eprintln!(
            "warning: no image protocol detected; falling back to ascii. \
             On Kitty/Ghostty/WezTerm/iTerm2 this should not happen. \
             You can force with TERMINAL_ENGINE_IMAGE_PROTOCOL=kitty|iterm2"
        );
    }
    let mut terminal = ratatui::init();
    let result = App::new(mode).run(&mut terminal);
    ratatui::restore();
    result
}
