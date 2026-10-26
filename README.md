# radioterm

A minimal, lightning-fast terminal radio player and world map explorer inspired by Radio.garden. Powered by `mpv`, Rust, and `terminalmap`.

---

## Features

- **Interactive World Map**: Explore global radio stations with smooth panning (`hjkl`, WASD, arrows) and zooming (`i`/`o`, `+`/`-`).
- **Clean Minimalist UI**:
  - Pinned station info bar at the top of the status block.
  - Minimalist controls bar with Nerd Font icons and keypress feedback.
  - Tab-toggleable compact view.
- **Favorites System**: Bookmark stations with `f` (marked with a red heart `♥`).
- **Event-Driven Rendering**: Low CPU usage and zero terminal flicker when idle.

---

## Controls

| Key | Action |
| :--- | :--- |
| `Space` | Play / Pause |
| `Enter` | Play Selected Station |
| `f` | Toggle Favorite |
| `h j k l` / `WASD` / `Arrows` | Pan Map |
| `i` / `+` | Zoom In |
| `o` / `-` | Zoom Out |
| `n` / `p` | Next / Previous Station |
| `Tab` | Toggle Full / Compact Controls |
| `q` | Quit |

---

## Requirements

- **Rust** (Edition 2021+)
- **`mpv`** audio player backend (`sudo apt install mpv` on Debian/Ubuntu or `brew install mpv` on macOS)
- A **Nerd Font** installed in your terminal for icons.

---

## Installation & Running

```bash
git clone https://github.com/yourusername/radioterm.git
cd radioterm
cargo run --release
```
