# Phase 0 — Existing Project Research and Reuse Assessment

**Status:** Verified by cloning and inspecting source (2026-10-12).  
**Toolchain used for checks:** `rustc`/`cargo` 1.83 initially; environment also has stable 1.99.  
**Distinction:** Findings marked **Verified** were observed in repository files. Items marked **Assumption** are inferences not proven by inspection.

---

## Decision gate

**Selected: Option B — Reuse selected components (concepts and algorithms), build TerminalEngine independently.**

### Why not Option A (adopt and extend)

| Candidate | Why unsuitable as core |
| --- | --- |
| **rasterm** | Closest architecture, but **LICENSE is AGPL-3.0 + Commons Clause** (verified). `Cargo.toml` still declares `license = "MIT"` (verified mismatch). Commons Clause forbids selling products that include/depend on the software. Adopting it would force TerminalEngine into a restrictive licence and create compliance risk. Also tightly coupled to a demo reel in `main.rs`; terminal presentation is Crossterm-direct, not Ratatui. Current `image` dependency resolution failed on Rust 1.83 (`edition2024` transitive deps). |
| **rust-sloth** | ISC licence is friendly, but the rasterizer writes **characters into the framebuffer** (`Vec<(char, RGB)>`) rather than pixels. Encoding is not separable. No programmable shaders, near-plane clipping, or texture pipeline. Small, aging dependency set (`clap` 2, older Crossterm). Good tutorial, poor engine core. |

### Why not Option C alone

A from-scratch rasterizer is justified for the **pixel pipeline**, but several techniques are already proven in the references (edge-function triangles, density gradients, half-block cells, bgcolor SGR). Ignoring them would only recreate known work. Option B captures those ideas without taking unsuitable codebases as dependencies.

### Why Option B

1. **Architecture idea from rasterm (concept only):** RGB colour buffer + depth buffer → pluggable terminal presenters. That matches a renderer-independent framebuffer with multiple encoders.
2. **Encoding techniques from doom-ascii / bgcolor / Ratatui:** character-density ASCII, Unicode block gradients, true-colour FG, bgcolor fills, half-block double vertical resolution.
3. **TUI from Ratatui + Crossterm:** lifecycle, resize, input, flicker control — do not reinvent.
4. **Legal hygiene:** no AGPL/Commons Clause or GPL-2 game code in the engine; reimplement algorithms independently under TerminalEngine’s own licence (MIT/Apache-2.0 recommended).

---

## Assessment table

| Question | Conclusion |
| --- | --- |
| **Existing 3D renderer** | **rasterm** has the strongest *conceptual* foundation (framebuffer → encoders). **rust-sloth** is the strongest *legally simple* small reference. Neither should be the TerminalEngine core. |
| **Rasterization** | Reuse **ideas**: bounding-box edge functions, Z-buffer, optional near-plane clip, Lambertian shading. Implement ourselves with `glam`. Do not vendor rasterm or rust-sloth source. |
| **Terminal output** | Inform encoders from: doom-ascii (density / block / braille + FG truecolour + run-lengthish SGR reuse), doom-ascii-bgcolor (BG truecolour + dark FG), Ratatui `▀` half-block (FG+BG per cell), rasterm half-block `▄`. Image protocols deferred (terminal-doom / Kitty via libvaxis pattern). |
| **TUI** | Integrate **Ratatui** (`ratatui::run` / `DefaultTerminal`) with **Crossterm** events. Layout: control pane + graphics viewport widget that consumes an `FrameBuffer` via an encoder. Prefer Ratatui’s buffered draw over raw full-frame ANSI from the app loop. |
| **Architecture** | Shared across modes: `FrameBuffer` (RGBA8 or RGB f32), `DepthBuffer`, mesh/camera/math, `Encoder` trait, scene/raster API. Mode-specific: encoder implementations and optional future image-protocol backend. |
| **Licensing** | **Can reuse as dependency:** Ratatui, Crossterm, glam (MIT/Apache). **Can study, not copy GPL/AGPL code:** doom-ascii (GPL-2), doom-ascii-bgcolor (GPL-2 fork), terminal-doom (GPL-2 project + MIT Zig bits), rasterm (AGPL+Commons Clause). **Can reuse with notice if desired:** rust-sloth (ISC) — still prefer independent implementation. |
| **Maintenance** | rust-sloth: last meaningful push ~2024-01, builds and 3 geometry tests pass on 1.83 (**verified**). rasterm: active mid-2026, CI present, good unit tests; licence change 2026-04-10; fails to resolve on Rust 1.83 (`edition2024` transitive deps) but **builds and passes tests on Rust 1.99** (**verified**). doom-ascii: actively maintained, popular. bgcolor fork: stale (2024-01). terminal-doom: Zig 0.16, actively maintained; not a Rust dependency. |

---

## Project findings (verified)

### 1. rust-sloth (`ecumene/rust-sloth`)

- **Licence:** ISC (`LICENSE.md`).
- **Stack:** `nalgebra`, `crossterm`, `tobj`, `clap` 2, `stl_io`; edition 2021.
- **Pipeline:** OBJ/STL → `SimpleMesh` of coloured triangles → edge-function rasterizer → `frame_buffer: Vec<(char, (u8,u8,u8))>` + `z_buffer` → Crossterm `PrintStyledContent` or plain string / HTML span export.
- **Shading:** Face normal Z → discrete ASCII gradient (`.:-=+*#%@`).
- **Colour:** Per-triangle diffuse / vertex colour; fixed dark background RGB(25,25,25) in colour mode.
- **Camera:** Combined scale/translate matrix in `Context::update`; turntable rotation in `main`.
- **Tests:** Three geometry unit tests; CI rustfmt + `cargo test`.
- **Reuse:** Reference for minimal edge-function raster and density shading. **Not** a clean framebuffer/encoder split.

### 2. rasterm (`HoldMyBeer-gg/rasterm`, author jabberwock)

- **Licence file:** AGPL-3.0 **with Commons Clause** (software named oddly as “AI IPC” in the Commons Clause block). **Cargo.toml still says MIT** — treat as AGPL+Commons Clause until upstream clarifies.
- **Stack:** `glam`, `crossterm`, `image`, `anyhow`, optional `rayon`.
- **Architecture (important):**
  - `Texture` colour buffer + `DepthBuffer`
  - `VertexProgram` / `FragmentProgram` traits
  - Sutherland–Hodgman near-plane clip
  - Bounding-box edge-function raster with perspective-correct varyings
  - `Terminal` trait with `AsciiTerminal` and `ColorTerminal` presenters
- **Colour terminal:** Half-block `▄`; top pixel → background SGR `48;2`, bottom → foreground `38;2`; pre-cached `"0"…"255"` strings; optional rayon row parallelisation.
- **ASCII terminal:** Short gradient `" .:;+=xX$#&"` from average luminance.
- **Extensibility:** Encoder trait is the right shape; renderer + animation + demos are one application crate.
- **Tests:** Solid integration coverage for depth, texture, primitives, triangle smoke test.
- **Reuse:** **Patterns and algorithms only.** Do not add as a Cargo dependency or copy source.

### 3. doom-ascii (`wojciech-graj/doom-ascii`)

- **Licence:** GPL-2.0 (Doom port — not suitable to link).
- **Encoding (verified in `doomgeneric_ascii.c`):**
  - Luminance → ASCII density string (long Paul Bourke–style gradient), or Unicode shade blocks, or braille density sets
  - True-colour **foreground** via `\033[38;2;R;G;Bm`, with reuse when colour unchanged
  - Optional gamma LUT (`byte_sqrt`), bold, full clear vs cursor-home
  - Each logical pixel emits **two** characters (aspect correction)
- **Reuse:** Encoder design reference only.

### 4. doom-ascii-bgcolor (`Bossach/doom-ascii-bgcolor`)

- **Licence:** GPL-2.0 (fork).
- **Change (verified):** Uses `\033[48;2;…m` **background** colour, forces dark foreground (`\033[30m`), still maps luminance to ASCII glyph (gradient inverted vs upstream string). Drops many of upstream’s charset/gamma options.
- **Reuse:** Validates bgcolor / “filled cell” as a first-class encoder mode alongside FG-only ASCII and half-block.

### 5. Ratatui + Crossterm

- **Docs/example verified:** [colors_rgb](https://www.ratatui.rs/examples/style/colors_rgb/) uses `▀` with `Color::Rgb` FG/BG, `ratatui::run`, Crossterm `event::poll`/`read`, layout split for title/FPS vs colour field.
- **Integration plan:**
  - Lifecycle: `ratatui::run` (raw mode + alternate screen + restore on drop)
  - Input/resize: Crossterm events via Ratatui backend; resize → rebuild framebuffer dimensions (half-block: height × 2)
  - True colour: `Style` / `Color::Rgb` on cells
  - Half-block: widget writes `▀`/`▄` into `Buffer`
  - Flicker: rely on Ratatui’s double-buffer diff; avoid full `\033[2J` every frame in the app path
  - Separation: layout `controls | viewport` (or top bar + viewport); viewport is a widget; controls never touch the colour buffer

### 6. terminal-doom (`cryptocode/terminal-doom`) — optional future

- **Verified:** Zig + libvaxis; each `DG_DrawFrame` converts the Doom buffer to RGB, `transmitImage` (Kitty graphics), draw scaled to cells; sync output historically via mode 2026 (now largely in libvaxis).
- **Role:** Future `ImageProtocolEncoder` / backend sketch only. Not required for milestone 1. Zig/C Doom stack is not a Rust dependency.

---

## Shared architecture (from research)

```
Scene / Mesh / Camera
        │
        ▼
 SoftwareRasterizer ──► FrameBuffer (RGB[A]) + DepthBuffer
        │
        ▼
   Encoder trait
   ├── AsciiDensityEncoder
   ├── HalfBlockEncoder      (Ratatui Buffer or ANSI)
   ├── BgColorAsciiEncoder
   └── (later) Kitty/iTerm image protocol
        │
        ▼
   Ratatui viewport widget  +  control widgets
```

Components shared by all rendering modes: math (`glam`), geometry, camera, rasterizer, framebuffer, depth buffer, encoder trait. Everything below the encoder is mode-specific.

---

## Milestone adaptation

No prior milestone document exists in this repository (only `# TerminalEngine` README). Research therefore defines Phase 1 as:

1. Core library with framebuffer, depth buffer, triangle rasterizer, camera, and mesh primitives.
2. At least two encoders: ASCII density and half-block true colour; bgcolor ASCII as a third.
3. Ratatui demo: controls separated from a graphics viewport; quit/resize/mode switch.
4. Unit tests for raster/depth/encoders; MIT licence.
5. No dependency on the inspected reference repositories.

---

## Terminal specs

Game-style Minimum / Recommended / Ultra requirements and a per-terminal roster live in [`terminal-specs.md`](terminal-specs.md).

## Environment note (verified during demo)

On this agent’s **xfce4-terminal 1.1.3 / VTE** host: `COLORTERM=truecolor` is set, **foreground** `38;2` colours work, but **background** `48;2` colours are not applied (raw Crossterm probe: coloured spaces invisible; `▀` with FG+BG appears as black/white stripes). Consequently half-block and bgcolor encoders need a terminal with working BG true-colour for visual validation; ASCII density (FG-only) renders correctly. This is a host terminal limitation, not a framebuffer/encoder data bug (unit tests show correct `Color::Rgb` in the Ratatui `Buffer`).

## Assumptions (explicit)

- **Assumption:** TerminalEngine wants a general-purpose engine, not a Doom port or a single demo reel.
- **Assumption:** A permissive MIT (or MIT/Apache-2.0 dual) licence is desired; AGPL+Commons Clause is unacceptable as a core dependency.
- **Assumption (superseded):** First milestone targeted character/cell encoders; **Ultra** Kitty/iTerm presentation is now implemented in `encode/image_protocol.rs` (still optional at runtime).
- **Assumption:** “Original milestone requirements” referred to the architecture goals embedded in this Phase 0 brief, since none were checked into the empty repository.
