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
            ((d_lat * d_lat + d_lon * d_lon) * 1_000_000.0) as i64
        })
        .map(|(i, _)| i)
}

// ── Marker sync ───────────────────────────────────────────────────────────────

fn sync_markers(map: &mut MapState, stations: &[Station], selected_idx: Option<usize>) {
    map.clear_markers();
    for (i, s) in stations.iter().enumerate() {
        let is_selected = selected_idx == Some(i);
        let color = if is_selected { 226 } else { 82 };
        map.add_marker(MapMarker {
            lat: s.lat,
            lon: s.lon,
            label: None,
            color,
            shape: MarkerShape::Dot,
            animation: MarkerAnimation::None,
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

fn pan(map: &mut MapState, app: &mut App, stations: &[Station], dlat: f64, dlon: f64) {
    map.center_lat = (map.center_lat + dlat).clamp(-85.0, 85.0);
    map.center_lon += dlon;
    if map.center_lon > 180.0 { map.center_lon -= 360.0; }
    if map.center_lon < -180.0 { map.center_lon += 360.0; }

    app.selected = nearest(stations, map.center_lat, map.center_lon);
    if !app.is_playing && !app.is_buffering {
        update_status_idle(app, stations);
    }
    sync_markers(map, stations, app.selected);
}

// ── Jump-to-station helper ────────────────────────────────────────────────────

fn jump_to(map: &mut MapState, app: &mut App, stations: &[Station], idx: usize, player: &mut Player) {
    let s = &stations[idx];
    map.center_lat = s.lat;
    map.center_lon = s.lon;
    app.selected = Some(idx);
    
    if app.is_playing || app.is_buffering {
        app.update_status_buffering(&s.name, &s.city);
        if player.play(&s.stream_url) {
            app.update_status_playing(&s.name, &s.city);
        } else {
            app.status = "Error: mpv not installed or stream failed".into();
        }
    } else {
        update_status_idle(app, stations);
    }
    sync_markers(map, stations, app.selected);
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    let stations = stations::load()?;
    let mut app = App::new();
    let mut player = Player::new();

    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let (mut cols, mut rows) = terminal::size()?;

    let mut map = MapState::new(MapConfig::default()).await?;
    map.set_size_from_terminal(cols, rows);
    map.fit_world();

    app.selected = nearest(&stations, map.center_lat, map.center_lon);
    update_status_idle(&mut app, &stations);
    sync_markers(&mut map, &stations, app.selected);

    let mut needs_redraw = true;

    loop {
        // Only poll events with timeout; if no event, we don't redraw unless stream reaps or needs redraw
        let event_occurred = event::poll(Duration::from_millis(50))?;
        
        if event_occurred {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    needs_redraw = true;
                    match k.code {
                        KeyCode::Char('q') => break,

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

                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            map.zoom_by(0.5);
                        }
                        KeyCode::Char('-') => {
                            map.zoom_by(-0.5);
                        }

                        KeyCode::Char('n') => {
                            if let Some(idx) = app.selected {
                                let next = (idx + 1) % stations.len();
                                jump_to(&mut map, &mut app, &stations, next, &mut player);
                            }
                        }
                        KeyCode::Char('p') => {
                            if let Some(idx) = app.selected {
                                let prev = if idx == 0 { stations.len() - 1 } else { idx - 1 };
                                jump_to(&mut map, &mut app, &stations, prev, &mut player);
                            }
                        }

                        KeyCode::Enter => {
                            if let Some(i) = app.selected {
                                let s = &stations[i];
                                app.update_status_buffering(&s.name, &s.city);
                                
                                // Render immediately to show buffering state
                                let frame = map.render().await?;
                                print!("\x1B[H{}", frame);
                                let map_rows = rows.saturating_sub(3);
                                execute!(stdout, MoveTo(cols / 2, map_rows / 2))?;
                                print!("\x1B[1;33m⌖\x1B[0m");
                                render_bottom_ui(&app.status, cols, rows);
                                stdout.flush()?;

                                if player.play(&s.stream_url) {
                                    app.update_status_playing(&s.name, &s.city);
                                } else {
                                    app.status = "Error: mpv not installed. Run: sudo apt install mpv".into();
                                }
                            }
                        }

                        KeyCode::Char(' ') => {
                            player.stop();
                            app.update_status_stopped();
                        }

                        _ => {
                            needs_redraw = false;
                        }
                    }
                }

                Event::Resize(w, h) => {
                    cols = w;
                    rows = h;
                    map.set_size_from_terminal(cols, rows);
                    needs_redraw = true;
                }

                _ => {}
            }
        }

        // Check if mpv process has exited on its own
        if player.is_playing() && player.try_reap() {
            app.update_status_stopped();
            app.status = format!("⚠ Stream ended  {}", app.status);
            needs_redraw = true;
        }

        if needs_redraw {
            let frame = map.render().await?;
            print!("\x1B[H{}", frame);

            let map_rows = rows.saturating_sub(3);
            let center_x = cols / 2;
            let center_y = map_rows / 2;
            if map_rows > 0 {
                execute!(stdout, MoveTo(center_x, center_y))?;
                print!("\x1B[1;33m⌖\x1B[0m");
            }

            render_bottom_ui(&app.status, cols, rows);
            stdout.flush()?;
            needs_redraw = false;
        }
    }

    player.stop();
    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    Ok(())
}

fn pan_step(zoom: f64) -> f64 {
    let step = 10.0 / 2.0_f64.powf(zoom);
    step.clamp(0.02, 15.0)
}

fn render_bottom_ui(status: &str, cols: u16, rows: u16) {
    if rows < 2 {
        return;
    }
    let sep_row = rows.saturating_sub(2);
    let status_row = rows.saturating_sub(1);

    // 1. Render separator line (subtle dim gray), strictly clamped to `cols` width
    let _ = execute!(std::io::stdout(), MoveTo(0, sep_row));
    let sep_len = (cols as usize).min(800);
    print!("\x1B[2K\x1B[90m{}\x1B[0m", "─".repeat(sep_len));

    // 2. Render status bar with safe truncation/padding to prevent wrapping/warping
    let _ = execute!(std::io::stdout(), MoveTo(0, status_row));
    print!("\x1B[2K");

    let max_len = cols as usize;
    let truncated_status = if status.chars().count() > max_len {
        let truncated: String = status.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        status.to_string()
    };

    if let Some(split_pos) = truncated_status.find("  |  ") {
        let info = &truncated_status[..split_pos];
        let hints = &truncated_status[split_pos..];
        print!("\x1B[1;36m{}\x1B[0m", info);
        print!("\x1B[2;37m{}\x1B[0m", hints);
    } else {
        print!("\x1B[1;36m{}\x1B[0m", truncated_status);
    }
}
