//! Ultra-tier image protocol encoders (Kitty graphics + iTerm2 inline images).
//!
//! These write protocol bytes directly to a [`Write`] sink (typically stdout),
//! not into a Ratatui cell buffer. The rasterizer still fills the shared
//! [`FrameBuffer`]; only presentation changes.

use std::env;
use std::io::{self, Write};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;

use super::Encoder;
use crate::framebuffer::FrameBuffer;

/// Chunk size for Kitty base64 payloads (bytes of encoded text), per the spec examples.
const KITTY_B64_CHUNK: usize = 4096;

/// Stable image id used for in-place frame replacement.
const DEFAULT_IMAGE_ID: u32 = 42;

/// Which inline graphics protocol to speak.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageProtocol {
    /// Kitty graphics protocol (`APC` `\033_G…\033\\`). Also used by Ghostty / WezTerm.
    Kitty,
    /// iTerm2 inline images (`OSC 1337`).
    ITerm2,
}

impl ImageProtocol {
    pub fn name(self) -> &'static str {
        match self {
            Self::Kitty => "kitty",
            Self::ITerm2 => "iterm2",
        }
    }
}

/// Detect a usable Ultra-tier protocol from the environment.
///
/// Order of preference is conservative: explicit override, then well-known
/// env markers. Returns [`None`] when nothing looks capable — callers should
/// fall back to a cell encoder.
pub fn detect_image_protocol() -> Option<ImageProtocol> {
    if let Ok(v) = env::var("TERMINAL_ENGINE_IMAGE_PROTOCOL") {
        match v.to_ascii_lowercase().as_str() {
            "kitty" | "ghostty" => return Some(ImageProtocol::Kitty),
            "iterm" | "iterm2" => return Some(ImageProtocol::ITerm2),
            "off" | "none" | "0" => return None,
            _ => {}
        }
    }

    if env::var_os("KITTY_WINDOW_ID").is_some() {
        return Some(ImageProtocol::Kitty);
    }

    if let Ok(term) = env::var("TERM") {
        let t = term.to_ascii_lowercase();
        if t.contains("kitty") || t.contains("ghostty") {
            return Some(ImageProtocol::Kitty);
        }
    }

    if let Ok(program) = env::var("TERM_PROGRAM") {
        match program.as_str() {
            "iTerm.app" => return Some(ImageProtocol::ITerm2),
            "WezTerm" | "WarpTerminal" | "ghostty" | "Ghostty" => {
                return Some(ImageProtocol::Kitty);
            }
            _ => {}
        }
    }

    None
}

/// Estimate pixels per terminal cell. Falls back to 8×16 when the tty reports 0.
pub fn cell_pixel_size() -> (u32, u32) {
    if let Ok(ws) = crossterm::terminal::window_size() {
        let cols = u32::from(ws.columns.max(1));
        let rows = u32::from(ws.rows.max(1));
        let cw = u32::from(ws.width) / cols;
        let ch = u32::from(ws.height) / rows;
        if cw > 0 && ch > 0 {
            return (cw.clamp(4, 32), ch.clamp(8, 64));
        }
    }
    (8, 16)
}

/// Recommended framebuffer size for an Ultra viewport, capped for interactive FPS.
pub fn ultra_framebuffer_size(cols: usize, rows: usize) -> (usize, usize) {
    let (cw, ch) = cell_pixel_size();
    let mut w = cols.max(1) * cw as usize;
    let mut h = rows.max(1) * ch as usize;
    // Soft cap so a huge window does not explode CPU cost.
    const MAX_W: usize = 960;
    const MAX_H: usize = 540;
    if w > MAX_W {
        let scale = MAX_W as f32 / w as f32;
        w = MAX_W;
        h = ((h as f32) * scale).round().max(1.0) as usize;
    }
    if h > MAX_H {
        let scale = MAX_H as f32 / h as f32;
        h = MAX_H;
        w = ((w as f32) * scale).round().max(1.0) as usize;
    }
    (w.max(1), h.max(1))
}

/// Pack the framebuffer into tightly packed RGB24 bytes (row-major).
pub fn framebuffer_to_rgb24(fb: &FrameBuffer) -> Vec<u8> {
    let mut out = Vec::with_capacity(fb.width() * fb.height() * 3);
    for px in fb.pixels() {
        out.push(px.r);
        out.push(px.g);
        out.push(px.b);
    }
    out
}

/// Encoder metadata for Ultra mode (presentation is via [`ImageProtocolEncoder::present`]).
#[derive(Clone, Debug)]
pub struct ImageProtocolEncoder {
    protocol: ImageProtocol,
    image_id: u32,
}

impl Encoder for ImageProtocolEncoder {
    fn name(&self) -> &'static str {
        "ultra"
    }

    fn framebuffer_height_for_rows(&self, rows: usize) -> usize {
        ultra_framebuffer_size(1, rows).1
    }

    fn framebuffer_width_for_cols(&self, cols: usize) -> usize {
        ultra_framebuffer_size(cols, 1).0
    }
}

impl ImageProtocolEncoder {
    pub fn new(protocol: ImageProtocol) -> Self {
        Self {
            protocol,
            image_id: DEFAULT_IMAGE_ID,
        }
    }

    pub fn try_detect() -> Option<Self> {
        detect_image_protocol().map(Self::new)
    }

    pub fn protocol(&self) -> ImageProtocol {
        self.protocol
    }

    /// Present `fb` in a viewport measured in terminal cells at `(col, row)`.
    pub fn present(
        &mut self,
        fb: &FrameBuffer,
        col: u16,
        row: u16,
        cols: u16,
        rows: u16,
        out: &mut impl Write,
    ) -> io::Result<()> {
        // Synchronized update reduces flicker between Ratatui refresh and image paint.
        write!(out, "\x1b[?2026h")?;
        // Position cursor at the top-left of the viewport content area.
        write!(out, "\x1b[{};{}H", row + 1, col + 1)?;

        match self.protocol {
            ImageProtocol::Kitty => self.present_kitty(fb, cols, rows, out)?,
            ImageProtocol::ITerm2 => present_iterm2(fb, cols, rows, out)?,
        }

        write!(out, "\x1b[?2026l")?;
        out.flush()
    }

    fn present_kitty(
        &self,
        fb: &FrameBuffer,
        cols: u16,
        rows: u16,
        out: &mut impl Write,
    ) -> io::Result<()> {
        let rgb = framebuffer_to_rgb24(fb);
        let payload =
            encode_kitty_rgb24(&rgb, fb.width(), fb.height(), cols, rows, self.image_id);
        out.write_all(&payload)
    }

    /// Delete transmitted images (Kitty). No-op for iTerm2.
    pub fn cleanup(&self, out: &mut impl Write) -> io::Result<()> {
        if self.protocol == ImageProtocol::Kitty {
            write!(out, "\x1b_Ga=d,d=A,q=2\x1b\\")?;
            out.flush()?;
        }
        Ok(())
    }
}

fn present_iterm2(
    fb: &FrameBuffer,
    cols: u16,
    rows: u16,
    out: &mut impl Write,
) -> io::Result<()> {
    let rgb = framebuffer_to_rgb24(fb);
    // iTerm2 inline images expect encoded image *file* bytes for many clients.
    // We embed a minimal uncompressed PPM so receivers that decode via image
    // libraries still work; iTerm2 itself also accepts raw with name hints, but
    // PPM is widely tolerated and trivial to build without extra crates.
    let ppm = rgb24_to_ppm(fb.width(), fb.height(), &rgb);
    let b64 = B64.encode(&ppm);
    let size = ppm.len();
    write!(
        out,
        "\x1b]1337;File=inline=1;width={cols};height={rows};preserveAspectRatio=0;size={size}:{b64}\x07"
    )?;
    Ok(())
}

/// Build a binary P6 PPM for iTerm2 / WezTerm inline display.
pub fn rgb24_to_ppm(width: usize, height: usize, rgb: &[u8]) -> Vec<u8> {
    let header = format!("P6\n{width} {height}\n255\n");
    let mut out = Vec::with_capacity(header.len() + rgb.len());
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(rgb);
    out
}

/// Encode a Kitty transmit+display sequence for 24-bit RGB.
pub fn encode_kitty_rgb24(
    rgb: &[u8],
    width: usize,
    height: usize,
    cols: u16,
    rows: u16,
    image_id: u32,
) -> Vec<u8> {
    let mut out = Vec::new();
    let b64 = B64.encode(rgb);
    if b64.is_empty() {
        return out;
    }
    let mut offset = 0;
    let mut first = true;
    while offset < b64.len() {
        let end = (offset + KITTY_B64_CHUNK).min(b64.len());
        let more = if end < b64.len() { 1 } else { 0 };
        let chunk = &b64[offset..end];
        if first {
            // a=T transmit+display, f=24 RGB, q=2 quiet (no success response).
            write!(
                &mut out,
                "\x1b_Ga=T,f=24,s={width},v={height},c={cols},r={rows},i={image_id},q=2,m={more};{chunk}\x1b\\"
            )
            .expect("vec write");
            first = false;
        } else {
            write!(&mut out, "\x1b_Ga=T,i={image_id},m={more};{chunk}\x1b\\").expect("vec write");
        }
        offset = end;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgb;

    #[test]
    fn rgb24_roundtrip_size() {
        let mut fb = FrameBuffer::new(4, 3);
        fb.set(0, 0, Rgb::new(1, 2, 3));
        let bytes = framebuffer_to_rgb24(&fb);
        assert_eq!(bytes.len(), 4 * 3 * 3);
        assert_eq!(&bytes[0..3], &[1, 2, 3]);
    }

    #[test]
    fn kitty_payload_contains_header_and_chunk_markers() {
        let rgb = vec![255u8, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        let encoded = encode_kitty_rgb24(&rgb, 2, 2, 10, 5, 7);
        let s = String::from_utf8(encoded).unwrap();
        assert!(s.contains("\u{1b}_Ga=T,f=24,s=2,v=2,c=10,r=5,i=7,q=2,m=0;"));
        assert!(s.ends_with("\u{1b}\\"));
        // RGB24 12 bytes → 16 base64 chars, well under one chunk.
        assert!(!s.contains(",m=1;"));
    }

    #[test]
    fn kitty_chunks_large_payload() {
        let rgb = vec![128u8; 3000 * 3]; // enough to force multiple 4K base64 chunks
        let encoded = encode_kitty_rgb24(&rgb, 3000, 1, 80, 1, 1);
        let s = String::from_utf8(encoded).unwrap();
        assert!(s.contains(",m=1;"));
        assert!(s.contains(",m=0;"));
    }

    #[test]
    fn ppm_header_is_p6() {
        let rgb = vec![10u8, 20, 30, 40, 50, 60];
        let ppm = rgb24_to_ppm(2, 1, &rgb);
        assert!(ppm.starts_with(b"P6\n2 1\n255\n"));
        assert_eq!(&ppm[ppm.len() - 6..], &rgb[..]);
    }

    #[test]
    fn ultra_size_caps() {
        let (w, h) = ultra_framebuffer_size(400, 200);
        assert!(w <= 960);
        assert!(h <= 540);
    }

    #[test]
    fn detect_honours_explicit_override() {
        // SAFETY: single-threaded unit test; restore after.
        unsafe {
            env::set_var("TERMINAL_ENGINE_IMAGE_PROTOCOL", "kitty");
        }
        assert_eq!(detect_image_protocol(), Some(ImageProtocol::Kitty));
        unsafe {
            env::set_var("TERMINAL_ENGINE_IMAGE_PROTOCOL", "iterm2");
        }
        assert_eq!(detect_image_protocol(), Some(ImageProtocol::ITerm2));
        unsafe {
            env::set_var("TERMINAL_ENGINE_IMAGE_PROTOCOL", "off");
        }
        assert_eq!(detect_image_protocol(), None);
        unsafe {
            env::remove_var("TERMINAL_ENGINE_IMAGE_PROTOCOL");
        }
    }
}
