//! Interactive TerminalEngine demo.
//!
//! Controls are laid out separately from the graphics viewport (Ratatui layout).
//! Encoder modes can be cycled without changing the rasterizer or framebuffer producer.

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
    AsciiDensityEncoder, BgColorAsciiEncoder, Camera, HalfBlockEncoder, Mesh, Renderer, Rgb, Scene,
};

#[derive(Parser, Debug)]
#[command(name = "te-demo", about = "TerminalEngine interactive demo")]
struct Args {
    /// Initial encoder: ascii | half-block | bgcolor
    #[arg(long, default_value = "half-block")]
    mode: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Ascii,
    HalfBlock,
    BgColor,
}

impl Mode {
    fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "ascii" => Self::Ascii,
            "bgcolor" | "bg" => Self::BgColor,
            _ => Self::HalfBlock,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Ascii => Self::HalfBlock,
            Self::HalfBlock => Self::BgColor,
            Self::BgColor => Self::Ascii,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::HalfBlock => "half-block",
            Self::BgColor => "bgcolor",
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
}

impl App {
    fn new(mode: Mode) -> Self {
        let mut scene = Scene::new();
        scene.add_object(
            Mesh::cube(Rgb::new(220, 170, 60)),
            Mat4::IDENTITY,
        );
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
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            self.tick();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_input()?;
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
        };
        if self.renderer.dimensions() != (w, h) {
            self.renderer.resize(w, h);
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let [controls, viewport] =
            Layout::vertical([Constraint::Length(4), Constraint::Min(3)]).areas(area);

        let help = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    " TerminalEngine ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    " mode={}  mesh={}  {:.0} fps",
                    self.mode.label(),
                    if self.mesh_kind == 0 { "cube" } else { "pyramid" },
                    self.fps
                )),
            ]),
            Line::from(
                "q quit · m next mode · 1/2 mesh · space toggle spin · arrows orbit · +/- zoom",
            ),
        ])
        .block(Block::default().borders(Borders::ALL).title("controls"));
        frame.render_widget(help, controls);

        let block = Block::default().borders(Borders::ALL).title("viewport");
        let inner = block.inner(viewport);
        frame.render_widget(block, viewport);

        self.ensure_framebuffer(inner);
        self.renderer.render(&self.scene, &self.camera);

        let fb = &self.renderer.framebuffer;
        match self.mode {
            Mode::Ascii => AsciiDensityEncoder.render_to_buffer(fb, inner, frame.buffer_mut()),
            Mode::HalfBlock => HalfBlockEncoder.render_to_buffer(fb, inner, frame.buffer_mut()),
            Mode::BgColor => {
                BgColorAsciiEncoder::new().render_to_buffer(fb, inner, frame.buffer_mut())
            }
        }
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
            KeyCode::Char('m') => self.mode = self.mode.next(),
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
    let mut terminal = ratatui::init();
    let result = App::new(mode).run(&mut terminal);
    ratatui::restore();
    result
}
