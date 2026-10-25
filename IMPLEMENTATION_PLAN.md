# RadioTerm — Terminal Radio Garden

> **For the AI:** Read the current phase only. Do one task, run it, verify it works, then move to the next. Never skip ahead. If stuck, check the **Rescue Rules** section at the bottom.

---

## Project Snapshot

| Item | Value |
|---|---|
| Language | Rust (2021 edition) |
| Map library | `terminalmap` crate (from `https://github.com/psmux/TerminalMap`) |
| Audio | Spawn `mpv --no-video <url>` as a child process |
| Station data | Bundled JSON file (`stations.json`) — top ~500 global stations |
| Interaction | Center-crosshair pattern: pan map → Enter to play nearest station |
| TUI framework | `ratatui` + `crossterm` (already used by terminalmap) |

---

## Directory Layout (target)

```
radio/
├── Cargo.toml
├── stations.json          ← bundled station list (lat, lon, name, stream_url)
├── src/
│   ├── main.rs            ← event loop, terminal setup/teardown
│   ├── app.rs             ← App state struct, update logic
│   ├── map_view.rs        ← MapState wrapper, marker sync
│   ├── player.rs          ← mpv child process controller
│   └── stations.rs        ← Station struct, load/parse stations.json
```

---

## Phases (work one phase at a time)

### ✅ PHASE 0 — Repo & Scaffolding
**Status:** `DONE` (git repo already initialized)

---

### 🔲 PHASE 1 — Cargo Setup

**Goal:** A `cargo run` that compiles a blank terminal app and exits cleanly.

**Tasks:**
1. Create `Cargo.toml` with these dependencies:
   ```toml
   [package]
   name = "radioterm"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   terminalmap = { git = "https://github.com/psmux/TerminalMap" }
   ratatui = "0.29"
   crossterm = "0.28"
   tokio = { version = "1", features = ["full"] }
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   anyhow = "1"
   ```
2. Create `src/main.rs` with a bare `#[tokio::main] async fn main() { println!("ok"); }`.
3. Run `cargo build`. Fix compile errors only — do not add features yet.

**Done when:** `cargo build` succeeds with no errors.

---

### 🔲 PHASE 2 — Station Data

**Goal:** Load and parse a list of stations from a bundled JSON file.

**Tasks:**
1. Create `stations.json` in project root with this exact structure (start with 5 real stations only):
   ```json
   [
     {"id":"bbc1","name":"BBC Radio 1","lat":51.5,"lon":-0.1,"stream_url":"http://stream.live.vc.bbcmedia.co.uk/bbc_radio_one"},
     {"id":"kexp","name":"KEXP 90.3","lat":47.6,"lon":-122.3,"stream_url":"https://kexp-mp3-128.streamguys1.com/kexp128.mp3"},
     {"id":"fip","name":"FIP Radio","lat":48.8,"lon":2.3,"stream_url":"https://icecast.radiofrance.fr/fip-midfi.mp3"},
     {"id":"nprdfw","name":"NPR KERA","lat":32.7,"lon":-96.8,"stream_url":"https://nprdfw.streamguys1.com/nprdfw.mp3"},
     {"id":"abc","name":"ABC Radio National","lat":-33.8,"lon":151.2,"stream_url":"https://live-radio01.mediahubaustralia.com/2RNW/mp3/"}
   ]
   ```
2. Create `src/stations.rs`:
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Clone, Deserialize)]
   pub struct Station {
       pub id: String,
       pub name: String,
       pub lat: f64,
       pub lon: f64,
       pub stream_url: String,
   }

   pub fn load() -> anyhow::Result<Vec<Station>> {
       let data = include_str!("../stations.json");
       Ok(serde_json::from_str(data)?)
   }
   ```
3. In `main.rs`, call `stations::load()` and print the count: `println!("{} stations loaded", stations.len())`.

**Done when:** `cargo run` prints `5 stations loaded`.

---

### 🔲 PHASE 3 — Player Module

**Goal:** Play and stop audio streams via `mpv` child process.

**Tasks:**
1. Create `src/player.rs`:
   ```rust
   use std::process::{Child, Command, Stdio};

   pub struct Player {
       child: Option<Child>,
   }

   impl Player {
       pub fn new() -> Self { Self { child: None } }

       pub fn play(&mut self, url: &str) {
           self.stop();
           let child = Command::new("mpv")
               .args(["--no-video", "--really-quiet", url])
               .stdin(Stdio::null())
               .stdout(Stdio::null())
               .stderr(Stdio::null())
               .spawn();
           self.child = child.ok();
       }

       pub fn stop(&mut self) {
           if let Some(mut c) = self.child.take() {
               let _ = c.kill();
           }
       }

       pub fn is_playing(&self) -> bool {
           self.child.is_some()
       }
   }

   impl Drop for Player {
       fn drop(&mut self) { self.stop(); }
   }
   ```
2. In `main.rs`, construct a `Player`, call `player.play("https://kexp-mp3-128.streamguys1.com/kexp128.mp3")`, sleep 5 seconds, call `player.stop()`.

**Done when:** Running `cargo run` plays 5 seconds of audio and exits cleanly. Verify with speakers on.

---

### 🔲 PHASE 4 — TUI Shell + Map

**Goal:** Full-screen terminal with the map rendered. `q` quits. Map pans with arrow keys.

**Tasks:**
1. Set up terminal raw mode + alternate screen in `main.rs` using crossterm. Always restore terminal on exit via a cleanup guard or RAII wrapper.
2. Initialize `MapState` from terminalmap:
   ```rust
   use terminalmap::{widget::MapState, config::MapConfig};
   let mut map = MapState::new(MapConfig::default()).await?;
   map.set_size(terminal_width, terminal_height);
   ```
3. Event loop: read crossterm events with `event::poll(Duration::from_millis(50))`.
   - Arrow keys / WASD → call `map.pan(...)` or update `center_lat/center_lon` directly.
   - `+` / `-` → `map.zoom_in()` / `map.zoom_out()`.
   - `q` → break loop, restore terminal.
4. Render loop: call `map.render().await?` and write the result string to stdout with crossterm.

**Do not add stations or player in this phase.**

**Done when:** Map fills the terminal, pans smoothly, zooms, and `q` exits without leaving the terminal broken.

---

### 🔲 PHASE 5 — Station Markers on Map

**Goal:** Every station from `stations.json` is rendered as a pulsing dot on the map.

**Tasks:**
1. After loading stations, iterate and call `map.add_marker(...)` for each:
   ```rust
   use terminalmap::marker::{MapMarker, MarkerShape, MarkerAnimation};
   for s in &stations {
       map.add_marker(MapMarker {
           lat: s.lat,
           lon: s.lon,
           label: Some(s.name.clone()),
           color: 82,           // bright green
           shape: MarkerShape::Dot,
           animation: MarkerAnimation::Pulse,
           id: s.id.clone(),
       });
   }
   ```
2. Call `map.advance_tick()` once per render frame so animations work.

**Done when:** You can see all 5 station dots pulsing on the correct geographic positions on the map.

---

### 🔲 PHASE 6 — Crosshair & Station Selection

**Goal:** A center crosshair `⌖` drawn over the map. The nearest station to the map center is highlighted in yellow.

**Tasks:**
1. Create `src/app.rs` with `App` struct:
   ```rust
   pub struct App {
       pub selected_station_idx: Option<usize>,
       pub is_playing: bool,
       pub status_msg: String,
   }
   ```
2. After each map pan, compute the nearest station:
   ```rust
   fn nearest_station(stations: &[Station], lat: f64, lon: f64) -> Option<usize> {
       stations.iter().enumerate()
           .min_by_key(|(_, s)| {
               let d = (s.lat - lat).powi(2) + (s.lon - lon).powi(2);
               (d * 1e6) as i64
           })
           .map(|(i, _)| i)
   }
   ```
3. Re-color the selected marker to yellow (`226`) and all others to green (`82`) by clearing markers and re-adding them each frame.
4. Draw crosshair: after writing map output, write `⌖` at terminal center using crossterm cursor positioning.
5. Draw a status bar at the bottom row: `[station name] | Enter: Play | q: Quit`.

**Done when:** Panning highlights the nearest station, and the status bar updates with the correct station name.

---

### 🔲 PHASE 7 — Play on Enter

**Goal:** Pressing `Enter` plays the selected station. `Space` stops. Status bar reflects state.

**Tasks:**
1. On `Enter` key event: call `player.play(&stations[selected_idx].stream_url)` and set `app.status_msg = format!("▶ Playing: {}", station.name)`.
2. On `Space` key event: call `player.stop()` and set `app.status_msg = "⏹ Stopped"`.
3. If no station is selected, show `app.status_msg = "Pan map to select a station"`.

**Done when:** You can pan the map, land near BBC Radio 1, press Enter, and hear audio. Press Space to stop.

---

### 🔲 PHASE 8 — Expand Station List

**Goal:** Replace the 5-station `stations.json` with a real global list (~500 stations).

**Tasks:**
1. Fetch stations from the Radio Browser API. Run this one-time script to download:
   ```bash
   curl -s "https://de1.api.radio-browser.info/json/stations/search?limit=500&has_geo_info=true&order=clickcount&reverse=true" \
     | python3 -c "
   import json,sys
   data=json.load(sys.stdin)
   out=[{'id':s['stationuuid'],'name':s['name'],'lat':float(s['geo_lat']),'lon':float(s['geo_long']),'stream_url':s['url_resolved']} for s in data if s.get('geo_lat') and s.get('geo_long') and s.get('url_resolved')]
   print(json.dumps(out,indent=2))
   " > stations.json
   ```
2. Verify `stations.json` has ~400–500 entries with valid lat/lon.
3. Run `cargo run`. Map should display all stations. Verify performance is smooth.

**Done when:** ~500 stations render on the map without lag.

---

### 🔲 PHASE 9 — Polish & Ship

**Goal:** The app is pleasant to use and handles edge cases.

**Tasks (in order of importance):**
1. **Terminal resize:** On resize event, call `map.set_size(new_w, new_h)` and re-render.
2. **Graceful mpv missing:** If `mpv` binary is not found, show error in status bar: `mpv not found — install mpv to play audio`.
3. **Clean exit on panic:** Wrap main with a catch to always restore terminal mode even on crash (use `std::panic::set_hook`).
4. **README update:** Write a short `README.md` with install steps: `cargo install --path .`, then just run `radioterm`.

**Done when:** App handles resize, shows a friendly error if mpv missing, and README is updated.

---

## Rescue Rules (Read when STUCK)

> **Rule 1 — One error at a time.**
> Fix the first compiler error only. Run `cargo check 2>&1 | head -30`. Fix it. Repeat. Never try to fix multiple errors simultaneously.

> **Rule 2 — Terminal stuck/broken after crash?**
> Run `reset` in the shell. That restores terminal state.

> **Rule 3 — mpv not playing audio?**
> Test in isolation: `mpv --no-video https://kexp-mp3-128.streamguys1.com/kexp128.mp3`. If this fails, the stream URL is dead — swap it with a working one.

> **Rule 4 — Map not rendering / blank screen?**
> Check `map.set_size(w, h)` is called with actual terminal dimensions before `map.render()`. Use `crossterm::terminal::size()` to get dimensions.

> **Rule 5 — Stuck on a compile error for more than 2 attempts?**
> Simplify: delete the failing code block, write the simplest possible version that compiles, then add features back one at a time.

> **Rule 6 — Infinite loop / high CPU?**
> Ensure `event::poll(Duration::from_millis(50))` is used — never spin without a sleep/poll. The event loop must always have a timeout.

> **Rule 7 — Don't refactor while implementing.**
> Get each phase working first. Only refactor at the end of a phase when all tests pass.

---

## Quick Reference

| Key | Action |
|---|---|
| Arrows / WASD | Pan map |
| `+` / `-` | Zoom in / out |
| `Enter` | Play selected station |
| `Space` | Stop playback |
| `q` | Quit |

| API | URL |
|---|---|
| Radio Browser | `https://de1.api.radio-browser.info/json/stations/search?limit=500&has_geo_info=true&order=clickcount&reverse=true` |
| TerminalMap docs | `/tmp/TerminalMap/README.md` |
| TerminalMap marker API | `/tmp/TerminalMap/src/marker.rs` |
| TerminalMap widget API | `/tmp/TerminalMap/src/widget.rs` |
