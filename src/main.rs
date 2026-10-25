mod app;
mod player;
mod stations;

use app::App;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use player::Player;
use stations::Station;
use std::io::Write;
use std::time::Duration;
use terminalmap::{
    config::MapConfig,
    marker::{MapMarker, MarkerAnimation, MarkerShape},
    widget::MapState,
};

// ── Nearest-station helper ────────────────────────────────────────────────────

fn nearest(stations: &[Station], lat: f64, lon: f64) -> Option<usize> {
    stations
        .iter()
        .enumerate()
        .min_by_key(|(_, s)| {
            let d_lat = s.lat - lat;
            let d_lon = s.lon - lon;
            // Simple Euclidean distance² in geo-space (good enough for a UI)
            ((d_lat * d_lat + d_lon * d_lon) * 1_000_000.0) as i64
        })
        .map(|(i, _)| i)
}

// ── Marker sync ───────────────────────────────────────────────────────────────

/// Rebuild all markers: dots only (no label text on the map), no animation.
/// Selected station gets yellow; all others are green.
fn sync_markers(map: &mut MapState, stations: &[Station], selected_idx: Option<usize>) {
    map.clear_markers();
    for (i, s) in stations.iter().enumerate() {
        let is_selected = selected_idx == Some(i);
        let color = if is_selected { 226 } else { 82 }; // yellow vs green
        map.add_marker(MapMarker {
            lat: s.lat,
            lon: s.lon,
            label: None,                     // ← no text on the map
            color,
            shape: MarkerShape::Dot,
            animation: MarkerAnimation::None, // ← no animations
            id: s.id.clone(),
        });
    }
}

// ── Status-bar update helper ──────────────────────────────────────────────────

fn update_status_idle(app: &mut App, stations: &[Station]) {
    if let Some(idx) = app.selected {
        let s = &stations[idx];
        app.update_status_idle(&s.name, &s.city);
    }
}

// ── Pan helper ────────────────────────────────────────────────────────────────

/// Pan the map, re-select nearest station, refresh status (unless playing).
fn pan(map: &mut MapState, app: &mut App, stations: &[Station], dlat: f64, dlon: f64) {
    map.center_lat = (map.center_lat + dlat).clamp(-85.0, 85.0);
    map.center_lon += dlon;
    // Wrap longitude
    if map.center_lon > 180.0 { map.center_lon -= 360.0; }
    if map.center_lon < -180.0 { map.center_lon += 360.0; }

    app.selected = nearest(stations, map.center_lat, map.center_lon);
    if !app.is_playing {
        update_status_idle(app, stations);
    }
    sync_markers(map, stations, app.selected);
}

// ── Jump-to-station helper ────────────────────────────────────────────────────

/// Recenter the map on a station and make it selected.
fn jump_to(map: &mut MapState, app: &mut App, stations: &[Station], idx: usize) {
    let s = &stations[idx];
    map.center_lat = s.lat;
    map.center_lon = s.lon;
    app.selected = Some(idx);
    if !app.is_playing {
        update_status_idle(app, stations);
    }
    sync_markers(map, stations, app.selected);
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Panic hook: always restore terminal state on crash
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    let stations = stations::load()?;
    let mut app = App::new();
    let mut player = Player::new();

    // ── Terminal init ─────────────────────────────────────────────────────────
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let (mut cols, mut rows) = terminal::size()?;

    let mut map = MapState::new(MapConfig::default()).await?;
    // Use the library's helper — it correctly converts cols/rows to braille pixels
    // and reserves rows for the status bar.
    map.set_size_from_terminal(cols, rows);
    map.fit_world(); // start with the whole world visible

    // Initial nearest-station selection
    app.selected = nearest(&stations, map.center_lat, map.center_lon);
    update_status_idle(&mut app, &stations);
    sync_markers(&mut map, &stations, app.selected);

    // ── Event loop ────────────────────────────────────────────────────────────
    loop {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                    // ── Quit ──────────────────────────────────────────────────
                    KeyCode::Char('q') => break,

                    // ── Pan (WASD / arrows) ───────────────────────────────────
                    // Pan step scales with zoom: far out = big jumps, zoomed in = small.
                    KeyCode::Up | KeyCode::Char('w') => {
                        let step = pan_step(map.zoom);
                        pan(&mut map, &mut app, &stations, step, 0.0);
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        let step = pan_step(map.zoom);
                        pan(&mut map, &mut app, &stations, -step, 0.0);
                    }
                    KeyCode::Left | KeyCode::Char('a') => {
                        let step = pan_step(map.zoom);
                        pan(&mut map, &mut app, &stations, 0.0, -step * 2.0);
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        let step = pan_step(map.zoom);
                        pan(&mut map, &mut app, &stations, 0.0, step * 2.0);
                    }

                    // ── Zoom ──────────────────────────────────────────────────
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        map.zoom_by(0.5);
                    }
                    KeyCode::Char('-') => {
                        map.zoom_by(-0.5);
                    }

                    // ── n / p: jump forward / backward through station list ───
                    KeyCode::Char('n') => {
                        if let Some(idx) = app.selected {
                            let next = (idx + 1) % stations.len();
                            jump_to(&mut map, &mut app, &stations, next);
                        }
                    }
                    KeyCode::Char('p') => {
                        if let Some(idx) = app.selected {
                            let prev = if idx == 0 { stations.len() - 1 } else { idx - 1 };
                            jump_to(&mut map, &mut app, &stations, prev);
                        }
                    }

                    // ── Enter: play selected station ──────────────────────────
                    KeyCode::Enter => {
                        if let Some(i) = app.selected {
                            if player.play(&stations[i].stream_url) {
                                app.update_status_playing(&stations[i].name, &stations[i].city);
                            } else {
                                app.status =
                                    "Error: mpv not installed. Run: sudo apt install mpv".into();
                            }
                        }
                    }

                    // ── Space: stop ───────────────────────────────────────────
                    KeyCode::Char(' ') => {
                        player.stop();
                        app.update_status_stopped();
                    }

                    _ => {}
                },

                // ── Terminal resize ───────────────────────────────────────────
                Event::Resize(w, h) => {
                    cols = w;
                    rows = h;
                    map.set_size_from_terminal(cols, rows);
                }

                _ => {}
            }
        }

        // Check if mpv process has exited on its own (e.g. stream died)
        if player.is_playing() && player.try_reap() {
            app.update_status_stopped();
            app.status = format!("⚠  Stream ended  {}", app.status);
        }

        // ── Render ────────────────────────────────────────────────────────────
        let frame = map.render().await?;
        // Move cursor to top-left then print the map frame
        print!("\x1B[H{}", frame);

        // Crosshair at screen centre (map area only, above status bar)
        let map_rows = rows.saturating_sub(1);
        let center_x = cols / 2;
        let center_y = map_rows / 2;
        execute!(stdout, MoveTo(center_x, center_y))?;
        print!("⌖");

        // Status bar: last row, two distinct visual sections
        //   Left: station info (bright)   Right: key hints (dim)
        let status_row = rows.saturating_sub(1);
        execute!(stdout, MoveTo(0, status_row))?;
        // Clear the line first, then write styled status
        print!("\x1B[2K");
        render_status_bar(&app.status, cols as usize);

        stdout.flush()?;
    }

    // ── Teardown ──────────────────────────────────────────────────────────────
    player.stop();
    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    Ok(())
}

// ── Pan step: scales with zoom level ─────────────────────────────────────────

/// Returns a pan step (degrees lat) appropriate for the current zoom.
/// At zoom 0 (world view) → 10°.  At zoom 10 (city level) → ~0.05°.
fn pan_step(zoom: f64) -> f64 {
    // Halve the step for each zoom level: step = 10 / 2^zoom
    let step = 10.0 / 2.0_f64.powf(zoom);
    step.clamp(0.02, 15.0)
}

// ── Status-bar renderer ───────────────────────────────────────────────────────

/// Renders the status bar with station info visually differentiated from hints.
/// Format produced by App::update_status_*:
///   "▸ <name>  —  <city>  [Enter] play  [n/p] …"
///   "▶  <name>  —  <city>  [Space] stop  …"
/// Strategy: bold/bright for the icon+name+city part, dim for the key hints.
fn render_status_bar(status: &str, _cols: usize) {
    // Split at the first "  [" to separate station info from key hints
    if let Some(split_pos) = status.find("  [") {
        let info = &status[..split_pos];
        let hints = &status[split_pos..];
        // Bold + bright white for station info
        print!("\x1B[1;97m{}\x1B[0m", info);
        // Dim for key hints
        print!("\x1B[2m{}\x1B[0m", hints);
    } else {
        print!("{}", status);
    }
}
