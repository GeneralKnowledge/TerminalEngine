//! TerminalEngine — software rasterization to a shared framebuffer, with
//! pluggable terminal encoders.
//!
//! Architecture (see `docs/research.md`):
//!
//! ```text
//! Scene / Mesh / Camera
//!         │
//!         ▼
//!  SoftwareRasterizer ──► FrameBuffer + DepthBuffer
//!         │
//!         ▼
//!    Encoder trait  (ASCII / half-block / bgcolor / ultra …)
//!         │
//!         ▼
//!    Ratatui viewport or Kitty / iTerm image protocol
//! ```

pub mod camera;
pub mod color;
pub mod depth;
pub mod encode;
pub mod framebuffer;
pub mod geometry;
pub mod mesh;
pub mod raster;
pub mod render;

pub use camera::Camera;
pub use color::Rgb;
pub use depth::DepthBuffer;
pub use encode::{
    detect_image_protocol, ultra_framebuffer_size, AsciiDensityEncoder, BgColorAsciiEncoder,
    Encoder, HalfBlockCell, HalfBlockEncoder, ImageProtocol, ImageProtocolEncoder,
};
pub use framebuffer::FrameBuffer;
pub use geometry::{Triangle, Vertex};
pub use mesh::{Mesh, MeshId};
pub use raster::Rasterizer;
pub use render::{Renderer, Scene};
