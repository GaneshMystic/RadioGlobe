mod app;
mod player;
mod stations;
mod ui;

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
    widget::MapState,
};

// ── Nearest-station helper ────────────────────────────────────────────────    

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

fn sync_markers(map: &mut MapState, _stations: &[Station], _selected_idx: Option<usize>, _app: &App) {
    map.clear_markers();
}

// ── Status-bar update helper ────────────────────────────────────────────────  

fn update_status_idle(app: &mut App, stations: &[Station]) {
    if let Some(idx) = app.selected {
        let s = &stations[idx];
        app.update_status_idle(&s.id, &s.name, &s.city);
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
    sync_markers(map, stations, app.selected, app);
}

// ── Jump-to-station helper ────────────────────────────────────────────────    

fn jump_to(map: &mut MapState, app: &mut App, stations: &[Station], idx: usize, player: &mut Player) {
    let s = &stations[idx];
    map.center_lat = s.lat;
    map.center_lon = s.lon;
    app.selected = Some(idx);
    
    if app.is_playing || app.is_buffering {
        app.update_status_buffering(&s.name, &s.city);
        if player.play(&s.stream_url) {
            app.playing_idx = Some(idx);
            app.update_status_playing(&s.id, &s.name, &s.city);
        } else {
            app.station_info = "Error: mpv not installed or stream failed".into();
        }
    } else {
        update_status_idle(app, stations);
    }
    sync_markers(map, stations, app.selected, app);
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
    sync_markers(&mut map, &stations, app.selected, &app);

    let mut needs_redraw = true;
    let mut running = true;

    while running {
        // Drain all pending events in the queue to batch key repeats and prevent stuttering
        if event::poll(Duration::from_millis(16))? {
            while event::poll(Duration::from_millis(0))? {
                match event::read()? {
                    Event::Key(k) if k.kind == KeyEventKind::Press => {
                        needs_redraw = true;
                        match k.code {
                            KeyCode::Char('q') => {
                                running = false;
                                break;
                            }

                            // Toggle shortcuts bar with Tab
                            KeyCode::Tab => {
                                app.toggle_shortcuts();
                            }

                            // Pan: h (Left)
                            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
                                app.flash_key('h');
                                let step = pan_step(map.zoom);
                                pan(&mut map, &mut app, &stations, 0.0, -step * 2.0);
                            }
                            // Pan: j (Down)
                            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                                app.flash_key('j');
                                let step = pan_step(map.zoom);
                                pan(&mut map, &mut app, &stations, -step, 0.0);
                            }
                            // Pan: k (Up)
                            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                                app.flash_key('k');
                                let step = pan_step(map.zoom);
                                pan(&mut map, &mut app, &stations, step, 0.0);
                            }
                            // Pan: l (Right)
                            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                                app.flash_key('l');
                                let step = pan_step(map.zoom);
                                pan(&mut map, &mut app, &stations, 0.0, step * 2.0);
                            }

                            // Zoom In: i
                            KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('i') => {
                                app.flash_key('i');
                                map.zoom_by(0.5);
                            }
                            // Zoom Out: o
                            KeyCode::Char('-') | KeyCode::Char('o') => {
                                app.flash_key('o');
                                map.zoom_by(-0.5);
                            }

                            // Station navigation (n / p)
                            KeyCode::Char('n') => {
                                app.flash_key('n');
                                if let Some(idx) = app.selected {
                                    let next = (idx + 1) % stations.len();
                                    jump_to(&mut map, &mut app, &stations, next, &mut player);
                                }
                            }
                            KeyCode::Char('p') => {
                                app.flash_key('p');
                                if let Some(idx) = app.selected {
                                    let prev = if idx == 0 { stations.len() - 1 } else { idx - 1 };
                                    jump_to(&mut map, &mut app, &stations, prev, &mut player);
                                }
                            }

                            // Toggle Favorite (f)
                            KeyCode::Char('f') => {
                                app.flash_key('f');
                                if let Some(idx) = app.selected {
                                    let s = &stations[idx];
                                    app.toggle_favorite(&s.id, &s.name, &s.city);
                                    sync_markers(&mut map, &stations, app.selected, &app);
                                }
                            }

                            // Enter or Space: Play / Pause
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                let is_space = k.code == KeyCode::Char(' ');
                                if is_space {
                                    app.flash_key(' ');
                                    if player.is_playing() {
                                        player.stop();
                                        app.playing_idx = None;
                                        if let Some(i) = app.selected {
                                            let s = &stations[i];
                                            app.update_status_stopped(&s.id, &s.name, &s.city);
                                        }
                                        continue;
                                    }
                                } else {
                                    app.flash_key('E');
                                }

                                if let Some(i) = app.selected {
                                    let s = &stations[i];
                                    app.update_status_buffering(&s.name, &s.city);
                                    
                                    let frame = map.render().await?;
                                    print!("\x1B[H{}", frame);
                                    let reserved = if app.show_shortcuts && rows >= 4 && cols >= 60 { 4 } else if app.show_shortcuts && rows >= 3 { 3 } else { 2 };
                                    let map_rows = rows.saturating_sub(reserved);
                                    if map_rows > 0 {
                                        execute!(stdout, MoveTo(cols / 2, map_rows / 2))?;
                                        print!("\x1B[1;33m⌖\x1B[0m");
                                    }
                                    ui::render_bottom_ui(&app, &stations, map.zoom, cols, rows)?;
                                    stdout.flush()?;

                                    if player.play(&s.stream_url) {
                                        app.playing_idx = Some(i);
                                        app.update_status_playing(&s.id, &s.name, &s.city);
                                    } else {
                                        app.station_info = "Error: mpv not installed. Run: sudo apt install mpv".into();
                                    }
                                }
                            }

                            _ => {}
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
        }

        // Clear key flash after 150ms
        if let Some((_, time)) = app.last_key {
            if time.elapsed() > Duration::from_millis(150) {
                app.last_key = None;
                needs_redraw = true;
            }
        }

        if player.is_playing() && player.try_reap() {
            app.playing_idx = None;
            if let Some(i) = app.selected {
                let s = &stations[i];
                app.update_status_stopped(&s.id, &s.name, &s.city);
            }
            needs_redraw = true;
        }

        if needs_redraw {
            let frame = map.render().await?;
            print!("\x1B[H{}", frame);

            let reserved = if app.show_shortcuts && rows >= 4 && cols >= 60 { 4 } else if app.show_shortcuts && rows >= 3 { 3 } else { 2 };
            let map_rows = rows.saturating_sub(reserved);
            let center_x = cols / 2;
            let center_y = map_rows / 2;
            if map_rows > 0 {
                execute!(stdout, MoveTo(center_x, center_y))?;
                print!("\x1B[1;33m⌖\x1B[0m");
            }

            ui::render_bottom_ui(&app, &stations, map.zoom, cols, rows)?;
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
