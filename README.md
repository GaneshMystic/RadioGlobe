# RadioTerm

An interactive terminal radio exploration map. Explore global radio stations on an interactive world map, tune in to streams directly from your terminal, and discover music worldwide.

## Features

- **Interactive World Map**: Pan and zoom smoothly around the globe.
- **500+ Curated Stations**: Sourced from Radio Browser API with geo-coordinates.
- **Live Tuning**: Listen to live audio streams using `mpv` under the hood.
- **Clean Map Markers**: Minimal dot pins showing station positions without cluttering labels or animations.
- **Fast Station Hopping**: Jump between stations directly using `n` (next) and `p` (prev).
- **Status Bar Explorer**: Displays station name, region/city, and playback controls.

## Requirements

- Rust (2021 edition)
- `mpv` installed for audio playback:
  ```bash
  sudo apt install mpv
  ```

## Running

Launch the application:
```bash
cargo run
```

## Key Controls

| Key | Action |
|---|---|
| `n` / `p` | Jump to next / previous station |
| `W` `A` `S` `D` / Arrows | Pan map around the globe |
| `+` / `-` | Zoom in / out |
| `Enter` | Play highlighted station |
| `Space` | Stop audio playback |
| `q` | Quit application |

