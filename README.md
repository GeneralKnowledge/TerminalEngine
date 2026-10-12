# TerminalEngine

A small Rust software rasterizer that draws into a **shared RGB framebuffer**, then presents through **pluggable terminal encoders** (ASCII density, half-block true colour, background-colour ASCII). The interactive shell uses **Ratatui + Crossterm**.

Phase 0 research and the Option B reuse decision are recorded in [`docs/research.md`](docs/research.md).

**Terminal requirements** (Minimum / Recommended / Ultra, plus a per-app roster): [`docs/terminal-specs.md`](docs/terminal-specs.md).

## Quick start

```bash
cargo test
cargo run --release --bin te-demo
```

Encoder modes (`--mode ascii|half-block|bgcolor|ultra`) — full matrix in the [terminal specs](docs/terminal-specs.md):

- `ascii` (default) — **Minimum** tier; FG true-colour + density glyphs.
- `half-block` — **Recommended** tier; `▀` with FG/BG true-colour.
- `bgcolor` — **Recommended** tier; BG true-colour fill + dim glyph.
- `ultra` — **Ultra** tier; Kitty graphics or iTerm2 inline images (auto-detected).

### Controls

| Key | Action |
| --- | --- |
| `q` / Esc | Quit |
| `m` | Cycle encoder (`ascii` → `half-block` → `bgcolor` → `ultra` when available) |
| `1` / `2` | Cube / pyramid |
| Space | Toggle auto-rotation |
| Arrow keys | Orbit camera |
| `+` / `-` | Zoom |

## Layout

| Module | Role |
| --- | --- |
| `framebuffer`, `depth` | Shared targets for every render mode |
| `raster` | CPU triangle rasterizer (edge functions + Z-buffer) |
| `encode::*` | Terminal presenters; independent of the rasterizer |
| `render`, `mesh`, `camera` | Scene orchestration |
| `te-demo` | Ratatui UI: **controls** pane separate from **viewport** |

## Licence

MIT. Reference projects inspected during Phase 0 are **not** dependencies; see `docs/research.md` for licensing notes.
