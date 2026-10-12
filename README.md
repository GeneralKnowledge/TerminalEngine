# TerminalEngine

A small Rust software rasterizer that draws into a **shared RGB framebuffer**, then presents through **pluggable terminal encoders** (ASCII density, half-block true colour, background-colour ASCII). The interactive shell uses **Ratatui + Crossterm**.

Phase 0 research and the Option B reuse decision are recorded in [`docs/research.md`](docs/research.md).

## Quick start

```bash
cargo test
cargo run --release --bin te-demo
```

Encoder modes (`--mode ascii|half-block|bgcolor`):

- `ascii` (default) — FG true-colour + density glyphs; works widely.
- `half-block` — `▀` with FG/BG true-colour (needs a terminal that applies `48;2` backgrounds: Kitty, Ghostty, WezTerm, …).
- `bgcolor` — BG true-colour fill + dim glyph (same BG requirement).

### Controls

| Key | Action |
| --- | --- |
| `q` / Esc | Quit |
| `m` | Cycle encoder (`ascii` → `half-block` → `bgcolor`) |
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
