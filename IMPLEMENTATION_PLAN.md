# RadioTerm — Implementation Plan

> **For the AI:** Read only the current phase. Complete it, verify it works, then move to the next.
> Never skip phases. When stuck, go directly to **Rescue Rules**.

---

## Project Layout

```
radio/                          ← project root (you are here)
├── Cargo.toml                  ← workspace root
├── stations.json               ← bundled station data
├── IMPLEMENTATION_PLAN.md      ← this file
├── TerminalMap/                ← local map library (do NOT edit)
│   ├── Cargo.toml
│   └── src/
│       ├── widget.rs           ← MapState API  (read this for map usage)
│       ├── marker.rs           ← MapMarker API (read this for dots/pins)
│       └── ...
└── src/
    ├── main.rs                 ← event loop, terminal setup/teardown
    ├── app.rs                  ← App state, selection logic
    ├── map_view.rs             ← MapState wrapper + marker sync
    ├── player.rs               ← mpv child process controller
    └── stations.rs             ← Station struct, load stations.json
```

---

## Key APIs (memorise these, reference TerminalMap/src/ for details)

```rust
// --- Map setup ---
use terminalmap::{widget::MapState, config::MapConfig};
let mut map = MapState::new(MapConfig::default()).await?;
map.set_size(width_chars, height_chars);      // terminal size in chars
let frame: String = map.render().await?;      // returns ANSI string, print it

// --- Pan / Zoom ---
map.center_lat += delta;
map.center_lon += delta;
map.zoom += 1.0;  // zoom range: 0.0 (world) to 18.0 (street)

// --- Markers ---
use terminalmap::marker::{MapMarker, MarkerShape, MarkerAnimation};
map.add_marker(MapMarker {
    lat, lon,
    label: Some("Station Name".into()),
    color: 82,                          // xterm-256 color (82 = green, 226 = yellow)
    shape: MarkerShape::Dot,
    animation: MarkerAnimation::Pulse,
    id: "unique_id".into(),
});
map.clear_markers();                    // call before re-adding to update colors
map.advance_tick();                     // call once per frame for animations

// --- Nearest station (selection logic) ---
fn nearest(stations: &[Station], lat: f64, lon: f64) -> Option<usize> {
    stations.iter().enumerate()
        .min_by_key(|(_, s)| ((s.lat-lat).powi(2) + (s.lon-lon).powi(2) * 1e6) as i64)
        .map(|(i, _)| i)
}
```

---

## Cargo.toml (copy exactly)

```toml
[package]
name = "radioterm"
version = "0.1.0"
edition = "2021"

[dependencies]
terminalmap = { path = "TerminalMap" }
ratatui     = "0.29"
crossterm   = "0.28"
tokio       = { version = "1", features = ["full"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
anyhow      = "1"
```

> **Note:** `crossterm` and `ratatui` versions must match what TerminalMap uses.
> Check `TerminalMap/Cargo.toml` if you get version conflict errors.

---

## Phases

---

### ✅ PHASE 0 — Repo
**Status: DONE** — git repo exists at project root.

---

### ✅ PHASE 1 — Cargo Setup

**Goal:** `cargo build` succeeds with a minimal main.

1. Create `Cargo.toml` using the template above.
2. Create `src/main.rs`:
   ```rust
   #[tokio::main]
   async fn main() {
       println!("RadioTerm starting...");
   }
   ```
3. Run `cargo build`. Fix errors only — no features yet.

**Done when:** `cargo build` exits with code 0, no errors.

---

### ✅ PHASE 2 — Stations

**Goal:** Load and parse station data from a bundled JSON file.

1. Create `stations.json` in project root:
   ```json
   [
     {"id":"bbc1","name":"BBC Radio 1","lat":51.5,"lon":-0.1,"stream_url":"http://stream.live.vc.bbcmedia.co.uk/bbc_radio_one"},
     {"id":"kexp","name":"KEXP 90.3","lat":47.6,"lon":-122.3,"stream_url":"https://kexp-mp3-128.streamguys1.com/kexp128.mp3"},
     {"id":"fip","name":"FIP Radio","lat":48.8,"lon":2.3,"stream_url":"https://icecast.radiofrance.fr/fip-midfi.mp3"},
     {"id":"kera","name":"NPR KERA","lat":32.7,"lon":-96.8,"stream_url":"https://nprdfw.streamguys1.com/nprdfw.mp3"},
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
       let raw = include_str!("../stations.json");
       Ok(serde_json::from_str(raw)?)
   }
   ```
3. In `main.rs`: declare `mod stations;`, call `stations::load()`, print count.

**Done when:** `cargo run` prints `5 stations loaded`.

---

### 🔲 PHASE 3 — Player

**Goal:** Play and stop an audio stream using `mpv` as a child process.

1. Create `src/player.rs`:
   ```rust
   use std::process::{Child, Command, Stdio};

   pub struct Player { child: Option<Child> }

   impl Player {
       pub fn new() -> Self { Self { child: None } }

       pub fn play(&mut self, url: &str) {
           self.stop();
           self.child = Command::new("mpv")
               .args(["--no-video", "--really-quiet", url])
               .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
               .spawn().ok();
       }

       pub fn stop(&mut self) {
           if let Some(mut c) = self.child.take() { let _ = c.kill(); }
       }

       pub fn is_playing(&self) -> bool { self.child.is_some() }
   }

   impl Drop for Player { fn drop(&mut self) { self.stop(); } }
   ```
2. In `main.rs`, test: create Player, call `play(kexp_url)`, `std::thread::sleep(5s)`, `stop()`.

**Done when:** `cargo run` plays 5 seconds of audio, exits cleanly.

---

### 🔲 PHASE 4 — TUI Shell + Map

**Goal:** Full-screen terminal with a scrollable, zoomable map. `q` quits cleanly.

1. In `main.rs`, set up raw mode + alternate screen:
   ```rust
   use crossterm::{terminal, execute};
   terminal::enable_raw_mode()?;
   execute!(std::io::stdout(), terminal::EnterAlternateScreen)?;
   // On exit (before returning from main):
   terminal::disable_raw_mode()?;
   execute!(std::io::stdout(), terminal::LeaveAlternateScreen)?;
   ```
2. Get terminal size:
   ```rust
   let (w, h) = terminal::size()?;  // returns (cols, rows) as u16
   ```
3. Create map, set size, enter event loop:
   ```rust
   let mut map = MapState::new(MapConfig::default()).await?;
   map.set_size(w as usize, h as usize - 1); // -1 for status bar row
   ```
4. Event loop (use `event::poll` — never spin without timeout):
   ```rust
   use crossterm::event::{self, Event, KeyCode};
   loop {
       if event::poll(std::time::Duration::from_millis(50))? {
           if let Event::Key(k) = event::read()? {
               match k.code {
                   KeyCode::Char('q') => break,
                   KeyCode::Up    | KeyCode::Char('w') => map.center_lat += 1.0,
                   KeyCode::Down  | KeyCode::Char('s') => map.center_lat -= 1.0,
                   KeyCode::Left  | KeyCode::Char('a') => map.center_lon -= 2.0,
                   KeyCode::Right | KeyCode::Char('d') => map.center_lon += 2.0,
                   KeyCode::Char('+') => map.zoom = (map.zoom + 0.5).min(18.0),
                   KeyCode::Char('-') => map.zoom = (map.zoom - 0.5).max(0.0),
                   _ => {}
               }
           }
       }
       // Render
       let frame = map.render().await?;
       print!("\x1B[H{}", frame);  // move cursor to top-left, then print
       use std::io::Write;
       std::io::stdout().flush()?;
   }
   ```

**Do NOT add stations or player yet.**

**Done when:** Map fills terminal, pans and zooms with keys, `q` exits without corrupting the terminal.

---

### 🔲 PHASE 5 — Markers

**Goal:** Each station renders as a pulsing green dot on the correct geographic position.

1. After creating the map and loading stations, add markers:
   ```rust
   for s in &stations {
       map.add_marker(MapMarker {
           lat: s.lat, lon: s.lon,
           label: Some(s.name.clone()),
           color: 82,
           shape: MarkerShape::Dot,
           animation: MarkerAnimation::Pulse,
           id: s.id.clone(),
       });
   }
   ```
2. Inside the render loop, call `map.advance_tick()` before `map.render()`.

**Done when:** All 5 station dots appear pulsing on the map at the right positions.

---

### 🔲 PHASE 6 — Crosshair + Selection

**Goal:** A `⌖` crosshair drawn at screen center. The nearest station is highlighted yellow. Status bar shows station name.

1. Create `src/app.rs`:
   ```rust
   pub struct App {
       pub selected: Option<usize>,
       pub status: String,
   }
   impl App {
       pub fn new() -> Self { Self { selected: None, status: "Pan to select a station".into() } }
   }
   ```
2. After each pan/key event, recompute nearest station using the `nearest()` function from Key APIs above. Store index in `app.selected`.
3. Re-sync markers each frame:
   ```rust
   map.clear_markers();
   for (i, s) in stations.iter().enumerate() {
       let color = if app.selected == Some(i) { 226 } else { 82 }; // yellow vs green
       map.add_marker(MapMarker { lat: s.lat, lon: s.lon, color,
           label: Some(s.name.clone()), shape: MarkerShape::Dot,
           animation: MarkerAnimation::Pulse, id: s.id.clone() });
   }
   ```
4. Draw crosshair at screen center after rendering map:
   ```rust
   use crossterm::cursor::MoveTo;
   execute!(stdout, MoveTo(w/2, h/2 - 1))?;
   print!("⌖");
   ```
5. Draw status bar on last row:
   ```rust
   execute!(stdout, MoveTo(0, h - 1))?;
   print!("{:<width$}", app.status, width = w as usize);
   ```
6. Update `app.status` with selected station name after each selection change.

**Done when:** Panning updates the yellow highlight and status bar correctly.

---

### 🔲 PHASE 7 — Playback

**Goal:** `Enter` plays selected station. `Space` stops. Status bar reflects state.

1. In event loop, add key handlers:
   ```rust
   KeyCode::Enter => {
       if let Some(i) = app.selected {
           player.play(&stations[i].stream_url);
           app.status = format!("▶  {} | Space: stop | q: quit", stations[i].name);
       }
   }
   KeyCode::Char(' ') => {
       player.stop();
       app.status = "⏹  Stopped | Enter: play | q: quit".into();
   }
   ```
2. Wire up `player` (created in Phase 3) into the main loop.

**Done when:** Pan to BBC Radio 1 → Enter → hear audio. Space → silence.

---

### 🔲 PHASE 8 — Expand Stations

**Goal:** Replace 5 test stations with ~500 real global stations from Radio Browser API.

1. Run this one-time script from the project root:
   ```bash
   curl -s "https://de1.api.radio-browser.info/json/stations/search?limit=500&has_geo_info=true&order=clickcount&reverse=true" \
   | python3 -c "
   import json,sys
   data=json.load(sys.stdin)
   out=[{'id':s['stationuuid'],'name':s['name'],
         'lat':float(s['geo_lat']),'lon':float(s['geo_long']),
         'stream_url':s['url_resolved']}
        for s in data if s.get('geo_lat') and s.get('geo_long') and s.get('url_resolved')]
   print(json.dumps(out,indent=2))
   " > stations.json
   ```
2. Run `cargo run`. Verify ~400–500 dots on the map. Verify smooth performance.

**Done when:** ~500 stations render without visible lag.

---

### 🔲 PHASE 9 — Polish

**Tasks (in priority order):**
1. **Resize:** On `Event::Resize(w, h)`, call `map.set_size(w as usize, h as usize - 1)`.
2. **mpv missing:** Wrap `player.play()` — if `Command::new("mpv").spawn()` fails, set `app.status = "Error: mpv not installed. Run: sudo apt install mpv"`.
3. **Panic cleanup:** Add at top of main:
   ```rust
   let default_hook = std::panic::take_hook();
   std::panic::set_hook(Box::new(move |info| {
       let _ = crossterm::terminal::disable_raw_mode();
       let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
       default_hook(info);
   }));
   ```
4. **Update README.md** with: what the app does, `cargo run` to launch, key controls table.
5. **git commit** all changes with message `feat: complete radioterm`.

**Done when:** All 4 tasks above are complete and `cargo run` launches cleanly.

---

## Rescue Rules

> **Stuck on compile error?**
> Run `cargo check 2>&1 | head -20`. Fix the FIRST error only. Repeat. Never fix multiple errors at once.

> **Terminal is broken/garbled after a crash?**
> Run `reset` in shell. This restores terminal state.

> **mpv not playing?**
> Test directly: `mpv --no-video https://kexp-mp3-128.streamguys1.com/kexp128.mp3`
> If that fails, stream URL is dead — pick a different one from `stations.json`.

> **Map is blank / not rendering?**
> Ensure `map.set_size(w, h)` is called with real terminal dimensions before `map.render()`.
> Check that `\x1B[H` (cursor home) is printed before the frame string.

> **CPU spiking / app freezing?**
> The event loop MUST use `event::poll(Duration::from_millis(50))`. Never use a bare loop without a timeout.

> **Dependency version conflict?**
> Open `TerminalMap/Cargo.toml`, check exact versions of `crossterm` and `ratatui`, mirror them in your `Cargo.toml`.

> **Stuck for more than 2 attempts on the same error?**
> Delete the failing block. Write the simplest thing that compiles. Add features back one line at a time.

---

## Key Controls Reference

| Key | Action |
|---|---|
| `W` `A` `S` `D` / Arrows | Pan map |
| `+` / `-` | Zoom in / out |
| `Enter` | Play selected station |
| `Space` | Stop playback |
| `q` | Quit |
