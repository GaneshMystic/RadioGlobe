use crate::app::App;
use crate::stations::Station;
use crossterm::{cursor::MoveTo, execute};

pub fn render_bottom_ui(
    app: &App,
    stations: &[Station],
    _zoom: f64,
    cols: u16,
    rows: u16,
) -> anyhow::Result<()> {
    if rows < 4 {
        return Ok(());
    }

    let max_len = cols as usize;
    let mut stdout = std::io::stdout();

    let get_color = |ch: char, default_color: &'static str| -> &'static str {
        if let Some((c, _)) = app.last_key {
            if c == ch || (ch == '+' && (c == '+' || c == '=' || c == 'i')) || (ch == 'h' && (c == 'h' || c == 'a' || c == 'j' || c == 'k' || c == 'l')) {
                return "\x1B[1;33m";
            }
        }
        default_color
    };

    let play_col = get_color(' ', "\x1B[1;36m");
    let fav_col = get_color('f', "\x1B[1;36m");
    let zoom_col = get_color('i', "\x1B[1;36m");
    let pan_col = get_color('h', "\x1B[1;36m");
    let quit_col = get_color('q', "\x1B[1;36m");
    let r = "\x1B[0m";

    let selected_station = app.selected.and_then(|i| stations.get(i));
    let playing_station = app.playing_idx.and_then(|i| stations.get(i));

    let is_hovering_other = match (app.selected, app.playing_idx) {
        (Some(sel), Some(play)) => sel != play,
        (Some(_), None) => true, // hovering/selecting when not playing
        _ => false,
    };

    let (info_str, info_color) = if app.is_buffering {
        let name = selected_station.map(|s| s.name.as_str()).unwrap_or("Station");
        (format!("󰑮 Connecting to {}...", name), "\x1B[1;36m")
    } else if app.is_playing && !is_hovering_other {
        // State A: Active Playing
        let s = playing_station.or(selected_station);
        if let Some(s) = s {
            let fav_icon = if app.favorites.contains(&s.id) { "♥" } else { "♡" };
            let loc_str = if s.city.is_empty() {
                "".to_string()
            } else {
                format!(" — {}", s.city)
            };
            let text = format!("󰐊 LIVE: {} {}{}", fav_icon, s.name, loc_str);
            (text, "\x1B[1;36m")
        } else {
            ("󰐊 LIVE".to_string(), "\x1B[1;36m")
        }
    } else {
        // State B: Hover Preview
        let s = selected_station;
        if let Some(s) = s {
            let loc_str = if s.city.is_empty() {
                "".to_string()
            } else {
                format!(" — {}", s.city)
            };
            if app.is_playing {
                (format!("󰐎 PREVIEW: {}{}", s.name, loc_str), "\x1B[90m")
            } else {
                let fav_icon = if app.favorites.contains(&s.id) { "♥" } else { "♡" };
                (format!("󰐎 PREVIEW: {} {}{}", fav_icon, s.name, loc_str), "\x1B[90m")
            }
        } else {
            ("󰐎 PREVIEW".to_string(), "\x1B[90m")
        }
    };

    let show_full_shortcuts = app.show_shortcuts && rows >= 12;

    if show_full_shortcuts {
        let sep1_row = rows.saturating_sub(4);
        let info_row = rows.saturating_sub(3);
        let sep2_row = rows.saturating_sub(2);
        let ctrl_row = rows.saturating_sub(1);

        // 1. Separator above info
        execute!(stdout, MoveTo(0, sep1_row))?;
        print!("\x1B[2K\x1B[90m{}\x1B[0m", "─".repeat(max_len));

        // 2. Station Info (Top)
        execute!(stdout, MoveTo(0, info_row))?;
        print!("\x1B[2K");
        let trunc_info = if info_str.chars().count() > max_len {
            let t: String = info_str.chars().take(max_len.saturating_sub(1)).collect();
            format!("{}…", t)
        } else {
            info_str
        };
        print!("{}{}\x1B[0m", info_color, trunc_info);

        // 3. Separator above shortcuts
        execute!(stdout, MoveTo(0, sep2_row))?;
        print!("\x1B[2K\x1B[90m{}\x1B[0m", "─".repeat(max_len));

        // 4. Control Bar based on width W
        execute!(stdout, MoveTo(0, ctrl_row))?;
        print!("\x1B[2K");

        let ctrl_str = if max_len >= 80 {
            format!(
                " {play_col}󰐊{r} [Space] Play │ {fav_col}󰋑{r} [f] Fav │ {zoom_col}󰍉{r} [+/-] Zoom │ {pan_col}{r} [hjkl] Pan │ {quit_col}󰈆{r} [q] Quit  [Tab]"
            )
        } else if max_len >= 60 {
            format!(
                " {play_col}󰐊{r} [Spc] │ {fav_col}󰋑{r} [f] │ {zoom_col}󰍉{r} [+/-] │ {pan_col}{r} [hjkl] │ {quit_col}󰈆{r} [q] [Tab]"
            )
        } else if max_len >= 40 {
            format!(
                " {fav_col}󰋑{r} │ {play_col}󰒮  󰐊  󰒭{r} │ +  90%  - │ {pan_col}   {r}"
            )
        } else if max_len >= 28 {
            format!(
                " {fav_col}󰋑{r} │ {play_col}󰒮  󰐊  󰒭{r} │ + 90% -"
            )
        } else {
            format!(
                " {play_col}󰒮     󰐊     󰒭{r}"
            )
        };

        let trunc_ctrl = if ctrl_str.chars().count() > max_len {
            let t: String = ctrl_str.chars().take(max_len.saturating_sub(1)).collect();
            format!("{}…", t)
        } else {
            ctrl_str
        };
        print!("{}", trunc_ctrl);

    } else {
        // Compact 2-line mode
        let sep_row = rows.saturating_sub(2);
        let info_row = rows.saturating_sub(1);

        execute!(stdout, MoveTo(0, sep_row))?;
        print!("\x1B[2K\x1B[90m{}\x1B[0m", "─".repeat(max_len));

        execute!(stdout, MoveTo(0, info_row))?;
        print!("\x1B[2K");

        let trunc_info = if info_str.chars().count() > max_len {
            let t: String = info_str.chars().take(max_len.saturating_sub(1)).collect();
            format!("{}…", t)
        } else {
            info_str
        };
        print!("{}{}\x1B[0m", info_color, trunc_info);
    }

    Ok(())
}
