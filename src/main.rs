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

fn nearest(stations: &[Station], lat: f64, lon: f64) -> Option<usize> {
    stations
        .iter()
        .enumerate()
        .min_by_key(|(_, s)| {
            let d_lat = s.lat - lat;
            let d_lon = s.lon - lon;
            ((d_lat * d_lat + d_lon * d_lon) * 1e6) as i64
        })
        .map(|(i, _)| i)
}

fn sync_markers(map: &mut MapState, stations: &[Station], selected_idx: Option<usize>) {
    map.clear_markers();
    for (i, s) in stations.iter().enumerate() {
        let is_selected = selected_idx == Some(i);
        let color = if is_selected { 226 } else { 82 }; // yellow vs green
        map.add_marker(MapMarker {
            lat: s.lat,
            lon: s.lon,
            label: Some(s.name.clone()),
            color,
            shape: MarkerShape::Dot,
            animation: MarkerAnimation::Pulse,
            id: s.id.clone(),
        });
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Panic hook to clean up terminal on unexpected exit
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    let stations = stations::load()?;
    let mut app = App::new();
    let mut player = Player::new();

    // Terminal initialization
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let (mut cols, mut rows) = terminal::size()?;
    let mut map = MapState::new(MapConfig::default()).await?;
    let map_rows = rows.saturating_sub(1);
    map.set_size((cols as usize) * 2, (map_rows as usize) * 4);

    // Initial station selection and marker setup
    app.selected = nearest(&stations, map.center_lat, map.center_lon);
    if let Some(idx) = app.selected {
        app.status = format!(
            "Nearest: {} | Enter: play | Space: stop | q: quit",
            stations[idx].name
        );
    }
    sync_markers(&mut map, &stations, app.selected);

    loop {
        // Poll for events (50ms timeout)
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Char('w') => {
                        map.center_lat += 1.0;
                        app.selected = nearest(&stations, map.center_lat, map.center_lon);
                        if let Some(idx) = app.selected {
                            if !player.is_playing() {
                                app.status = format!(
                                    "Nearest: {} | Enter: play | Space: stop | q: quit",
                                    stations[idx].name
                                );
                            }
                        }
                        sync_markers(&mut map, &stations, app.selected);
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        map.center_lat -= 1.0;
                        app.selected = nearest(&stations, map.center_lat, map.center_lon);
                        if let Some(idx) = app.selected {
                            if !player.is_playing() {
                                app.status = format!(
                                    "Nearest: {} | Enter: play | Space: stop | q: quit",
                                    stations[idx].name
                                );
                            }
                        }
                        sync_markers(&mut map, &stations, app.selected);
                    }
                    KeyCode::Left | KeyCode::Char('a') => {
                        map.center_lon -= 2.0;
                        app.selected = nearest(&stations, map.center_lat, map.center_lon);
                        if let Some(idx) = app.selected {
                            if !player.is_playing() {
                                app.status = format!(
                                    "Nearest: {} | Enter: play | Space: stop | q: quit",
                                    stations[idx].name
                                );
                            }
                        }
                        sync_markers(&mut map, &stations, app.selected);
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        map.center_lon += 2.0;
                        app.selected = nearest(&stations, map.center_lat, map.center_lon);
                        if let Some(idx) = app.selected {
                            if !player.is_playing() {
                                app.status = format!(
                                    "Nearest: {} | Enter: play | Space: stop | q: quit",
                                    stations[idx].name
                                );
                            }
                        }
                        sync_markers(&mut map, &stations, app.selected);
                    }
                    KeyCode::Char('+') => {
                        map.zoom = (map.zoom + 0.5).min(18.0);
                    }
                    KeyCode::Char('-') => {
                        map.zoom = (map.zoom - 0.5).max(0.0);
                    }
                    KeyCode::Enter => {
                        if let Some(i) = app.selected {
                            if player.play(&stations[i].stream_url) {
                                app.status = format!(
                                    "▶  {} | Space: stop | q: quit",
                                    stations[i].name
                                );
                            } else {
                                app.status =
                                    "Error: mpv not installed or failed to spawn. Run: sudo apt install mpv"
                                        .into();
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        player.stop();
                        if let Some(i) = app.selected {
                            app.status = format!(
                                "⏹  Stopped ({}) | Enter: play | q: quit",
                                stations[i].name
                            );
                        } else {
                            app.status = "⏹  Stopped | Enter: play | q: quit".into();
                        }
                    }
                    _ => {}
                },
                Event::Resize(w, h) => {
                    cols = w;
                    rows = h;
                    let map_rows = rows.saturating_sub(1);
                    map.set_size((cols as usize) * 2, (map_rows as usize) * 4);
                }
                _ => {}
            }
        }

        // Advance animation tick
        map.advance_tick();

        // Render map
        let frame = map.render().await?;
        print!("\x1B[H{}", frame);

        // Center crosshair
        let center_x = cols / 2;
        let center_y = rows.saturating_sub(1) / 2;
        execute!(stdout, MoveTo(center_x, center_y))?;
        print!("⌖");

        // Status bar on bottom row
        execute!(stdout, MoveTo(0, rows.saturating_sub(1)))?;
        print!("\x1B[2K{:<width$}", app.status, width = cols as usize);

        stdout.flush()?;
    }

    // Teardown
    player.stop();
    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    Ok(())
}
