# Terminal Specs

Game-style requirements for running TerminalEngine. Pick a terminal the way you’d pick a GPU tier.

Statuses:

| Mark | Meaning |
| --- | --- |
| **Verified** | Observed in our Phase 0 / demo environment, or called out from inspected reference projects |
| **Expected** | Common community/docs consensus; not re-tested here |
| **Implemented** | Encoder exists in-tree and is covered by unit/smoke tests |

---

## Spec tiers

### Minimum — “It boots”

Enough to run `te-demo` and see a recognisable mesh.

| Feature | Need |
| --- | --- |
| Unicode | Basic ASCII + box-drawing for the Ratatui chrome |
| Colour | 256-colour **or** true-colour **foreground** |
| Mode | `--mode ascii` |
| Input | Keyboard events (Crossterm) |
| Resize | Optional; demo adapts when supported |

**Playable on:** almost any modern terminal, including constrained VTE hosts and many SSH sessions.

```bash
cargo run --release --bin te-demo -- --mode ascii
```

### Recommended — “Full cell graphics”

Double vertical resolution and filled cells look correct.

| Feature | Need |
| --- | --- |
| True colour | `38;2` **and** `48;2` (FG **and** BG) |
| Block glyphs | `▀` / `▄` / `█` render as solid halves, not striped bars |
| Font | A monospace font with proper block elements (e.g. JetBrains Mono, Cascadia Mono, Menlo) |
| Modes | `ascii`, `half-block`, `bgcolor` |
| Sync | Fast redraw without forced full clears every frame (Ratatui diff is enough) |

```bash
cargo run --release --bin te-demo -- --mode half-block
```

### Ultra — “Native pixels” **(implemented)**

Kitty / iTerm-style image protocols paint the shared RGB framebuffer as real pixels inside the viewport.

| Feature | Need |
| --- | --- |
| Graphics protocol | **Kitty graphics** (Kitty, Ghostty, WezTerm, …) **or** **iTerm2 inline images** |
| Sync | Synchronized output (mode 2026) used around each paint |
| Bandwidth | Local or fast link; image frames are heavier than cell encoders |

```bash
cargo run --release --bin te-demo -- --mode ultra
# force a protocol if auto-detect fails:
TERMINAL_ENGINE_IMAGE_PROTOCOL=kitty cargo run --release --bin te-demo -- --mode ultra
TERMINAL_ENGINE_IMAGE_PROTOCOL=iterm2 cargo run --release --bin te-demo -- --mode ultra
```

Detection (first match wins): `TERMINAL_ENGINE_IMAGE_PROTOCOL` → `KITTY_WINDOW_ID` / `TERM` → `TERM_PROGRAM` (`iTerm.app`, `WezTerm`, `ghostty`, …).

---

## Mode card

What each encoder asks of the terminal:

| Mode | Vertical res | Uses FG | Uses BG | Glyph / payload | Spec tier |
| --- | --- | --- | --- | --- | --- |
| `ascii` | 1× (1 pixel / cell) | yes | no | density ` .:-=+*#%@` | **Minimum** |
| `half-block` | 2× (`▀` packs 2 rows) | yes | yes | `▀` | **Recommended** |
| `bgcolor` | 1× | dim FG | yes | density on fill | **Recommended** |
| `ultra` | ~cell-pixel sized FB (capped) | n/a | n/a | Kitty RGB24 / iTerm PPM | **Ultra** |

---

## Terminal roster

Prefer **Recommended** or better for half-block / bgcolor; **Ultra** where listed.

| Terminal | Platform | Min (`ascii`) | Rec (`half-block` / `bgcolor`) | Ultra (`kitty` / `iterm2`) | Notes |
| --- | --- | --- | --- | --- | --- |
| **Ghostty** | macOS, Linux | Expected ✓ | Expected ✓ | Expected ✓ (Kitty) | Strong modern default |
| **Kitty** | macOS, Linux | Expected ✓ | Expected ✓ | Expected ✓ (Kitty) | Reference for the graphics protocol |
| **WezTerm** | macOS, Linux, Windows | Expected ✓ | Expected ✓ | Expected ✓ (Kitty) | Also understands iTerm images |
| **iTerm2** | macOS | Expected ✓ | Expected ✓ | Expected ✓ (iTerm2) | Typical Mac Ultra pick |
| **Terminal.app** | macOS | Expected ✓ | Expected ~ | — | No Ultra; use `ascii` / try `half-block` |
| **Alacritty** | macOS, Linux, Windows | Expected ✓ | Expected ✓ | — | No stock image protocol |
| **Windows Terminal** | Windows | Expected ✓ | Expected ✓ | — | Prefer Cascadia Mono / similar |
| **xfce4-terminal (VTE)** | Linux | **Verified** ✓ | **Verified** ✗* | — | *Half-block glyphs striped on our demo host |
| **GNOME Terminal / Tilix / etc. (VTE)** | Linux | Expected ✓ | Expected ~ | — | Same VTE family; quality varies |
| **macOS/Linux `ssh` + remote tty** | any | Expected ✓ | Expected ~ | Expected ~ | Limited by **client** terminal |

\* On the Cursor agent VM (xfce4-terminal 1.1.3 + VTE 0.76), upper/lower half-block glyphs rendered striped. Treat that host as **Minimum-tier** for visuals.

---

## Quick chooser

```
Want it to Just Work?
  └─ ascii                          → Minimum tier

Want the sharpest cell look?
  └─ half-block on Ghostty/Kitty/
     WezTerm/iTerm2                 → Recommended tier

Want native pixels?
  └─ ultra on Kitty / Ghostty /
     WezTerm / iTerm2               → Ultra tier
```

### Mac (your machine)

| Goal | Terminal | Command |
| --- | --- | --- |
| Safe | Terminal.app or iTerm2 | `te-demo --mode ascii` |
| Best cell graphics | iTerm2, Kitty, Ghostty, or WezTerm | `te-demo --mode half-block` |
| Native pixels | Kitty, Ghostty, WezTerm, or iTerm2 | `te-demo --mode ultra` |

---

## Self-check (30 seconds)

Paste into the terminal before blaming the engine:

```bash
printf 'FG \033[38;2;255;200;0mYELLOW\033[0m  BG \033[48;2;220;20;20m RED \033[0m  half \033[38;2;220;170;60m\033[48;2;12;14;20m▀▀▀▀\033[0m  full \033[38;2;220;170;60m████\033[0m\n'
```

| Result | Tier |
| --- | --- |
| Yellow text + red patch + solid gold `▀` on dark | **Recommended** (try `half-block`) |
| Colours OK but `▀` striped / hollow | **Minimum** — use `ascii`; try `ultra` if Kitty/iTerm |
| No true colour | **Minimum** at best — `ascii`, expect muted/wrong colours |

Ultra self-check: if `echo $KITTY_WINDOW_ID` is non-empty, or `echo $TERM_PROGRAM` is `iTerm.app` / `WezTerm` / `ghostty`, `--mode ultra` should engage.

---

## Related

- Research decision and encoder origins: [`research.md`](research.md)
- Run instructions: [`../README.md`](../README.md)
