# Terminal Specs

Game-style requirements for running TerminalEngine. Pick a terminal the way you’d pick a GPU tier.

Statuses:

| Mark | Meaning |
| --- | --- |
| **Verified** | Observed in our Phase 0 / demo environment, or called out from inspected reference projects |
| **Expected** | Common community/docs consensus; not re-tested here |
| **Future** | Planned encoder / backend, not in milestone 1 |

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

### Ultra — “Native pixels” *(future)*

Kitty / iTerm-style image protocols for near-framebuffer output (see terminal-doom research note). Not required for Phase 1.

| Feature | Need |
| --- | --- |
| Graphics protocol | Kitty graphics **or** iTerm2 inline images |
| Sync | Synchronized output (e.g. mode 2026) preferred |
| Bandwidth | Local or fast link; image frames are heavier than cell encoders |

---

## Mode card

What each encoder asks of the terminal:

| Mode | Vertical res | Uses FG | Uses BG | Glyph | Spec tier |
| --- | --- | --- | --- | --- | --- |
| `ascii` | 1× (1 pixel / cell) | yes | no | density ` .:-=+*#%@` | **Minimum** |
| `half-block` | 2× (`▀` packs 2 rows) | yes | yes | `▀` | **Recommended** |
| `bgcolor` | 1× | dim FG | yes | density on fill | **Recommended** |
| image protocol *(future)* | full framebuffer | n/a | n/a | bitmap chunks | **Ultra** |

---

## Terminal roster

Like a supported-platforms list. Prefer **Recommended** or better for half-block / bgcolor.

| Terminal | Platform | Min (`ascii`) | Rec (`half-block` / `bgcolor`) | Ultra *(future)* | Notes |
| --- | --- | --- | --- | --- | --- |
| **Ghostty** | macOS, Linux | Expected ✓ | Expected ✓ | Expected ✓ (Kitty protocol) | Strong modern default; used by terminal-doom authors |
| **Kitty** | macOS, Linux | Expected ✓ | Expected ✓ | Expected ✓ | Reference for image-protocol backend |
| **WezTerm** | macOS, Linux, Windows | Expected ✓ | Expected ✓ | Expected ✓ | Good all-rounder; GPU-accelerated |
| **iTerm2** | macOS | Expected ✓ | Expected ✓ | Expected ✓ (iTerm images) | Typical Mac “recommended” pick |
| **Terminal.app** | macOS | Expected ✓ | Expected ~ | — | True colour usually OK; half-block depends on font |
| **Alacritty** | macOS, Linux, Windows | Expected ✓ | Expected ✓ | — | Fast; no Kitty images |
| **Windows Terminal** | Windows | Expected ✓ | Expected ✓ | — | Prefer Cascadia Mono / similar |
| **xfce4-terminal (VTE)** | Linux | **Verified** ✓ | **Verified** ✗* | — | *`COLORTERM=truecolor` set; FG OK; `▀`/`▄` drew as black/white stripes in our demo host |
| **GNOME Terminal / Tilix / etc. (VTE)** | Linux | Expected ✓ | Expected ~ | — | Same VTE family; quality varies by build/font |
| **macOS/Linux `ssh` + remote tty** | any | Expected ✓ | Expected ~ | Expected ~ | Limited by **client** terminal, not the server |

\* On the Cursor agent VM (xfce4-terminal 1.1.3 + VTE 0.76), raw `printf` showed FG/BG true colour on spaces, but upper/lower half-block glyphs still rendered striped. Treat that host as **Minimum-tier only** for visuals.

---

## Quick chooser

```
Want it to Just Work?
  └─ ascii                          → Minimum tier

Want the sharpest cell look?
  └─ half-block on Ghostty/Kitty/
     WezTerm/iTerm2                 → Recommended tier

Want Doom-style bitmaps later?
  └─ Kitty / Ghostty / iTerm2       → Ultra tier (not built yet)
```

### Mac (your machine)

| Goal | Terminal | Command |
| --- | --- | --- |
| Safe | Terminal.app or iTerm2 | `te-demo --mode ascii` |
| Best cell graphics | iTerm2, Kitty, Ghostty, or WezTerm | `te-demo --mode half-block` |
| Also try filled cells | same as above | `te-demo --mode bgcolor` |

You should see a clear coloured mesh in `half-block` on those Recommended terminals — not the striped viewport we hit on the agent’s xfce4-terminal.

---

## Self-check (30 seconds)

Paste into the terminal before blaming the engine:

```bash
printf 'FG \033[38;2;255;200;0mYELLOW\033[0m  BG \033[48;2;220;20;20m RED \033[0m  half \033[38;2;220;170;60m\033[48;2;12;14;20m▀▀▀▀\033[0m  full \033[38;2;220;170;60m████\033[0m\n'
```

| Result | Tier |
| --- | --- |
| Yellow text + red patch + solid gold `▀` on dark | **Recommended** |
| Colours OK but `▀` striped / hollow | **Minimum** — use `ascii` (or change font/terminal) |
| No true colour | **Minimum** at best — `ascii`, expect muted/wrong colours |

---

## Related

- Research decision and encoder origins: [`research.md`](research.md)
- Run instructions: [`../README.md`](../README.md)
