use std::collections::HashSet;
use std::time::Instant;

pub struct App {
    pub selected: Option<usize>,
    pub playing_idx: Option<usize>,
    pub station_info: String,
    pub is_playing: bool,
    pub is_buffering: bool,
    pub favorites: HashSet<String>,
    pub show_shortcuts: bool,
    pub last_key: Option<(char, Instant)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected: None,
            playing_idx: None,
            station_info: String::new(),
            is_playing: false,
            is_buffering: false,
            favorites: HashSet::new(),
            show_shortcuts: true,
            last_key: None,
        }
    }

    pub fn flash_key(&mut self, ch: char) {
        self.last_key = Some((ch, Instant::now()));
    }

    pub fn toggle_shortcuts(&mut self) {
        self.show_shortcuts = !self.show_shortcuts;
    }

    pub fn toggle_favorite(&mut self, station_id: &str, station_name: &str, city: &str) -> bool {
        let is_fav = if self.favorites.contains(station_id) {
            self.favorites.remove(station_id);
            false
        } else {
            self.favorites.insert(station_id.to_string());
            true
        };
        self.update_status_for_current(station_id, station_name, city);
        is_fav
    }

    pub fn update_status_for_current(&mut self, station_id: &str, station_name: &str, city: &str) {
        let fav_icon = if self.favorites.contains(station_id) { "♥" } else { "♡" };

        if self.is_buffering {
            self.station_info = format!("󰑮 Connecting to {}...", station_name);
        } else if self.is_playing {
            self.station_info = if city.is_empty() {
                format!("󰐊 LIVE: {} {}", fav_icon, station_name)
            } else {
                format!("󰐊 LIVE: {} {}  —  {}", fav_icon, station_name, city)
            };
        } else {
            self.station_info = if city.is_empty() {
                format!("󰓃 {} {}", fav_icon, station_name)
            } else {
                format!("󰓃 {} {}  —  {}", fav_icon, station_name, city)
            };
        }
    }

    pub fn update_status_idle(&mut self, station_id: &str, station_name: &str, city: &str) {
        let fav_icon = if self.favorites.contains(station_id) { "♥" } else { "♡" };
        self.station_info = if city.is_empty() {
            format!("󰓃 {} {}", fav_icon, station_name)
        } else {
            format!("󰓃 {} {}  —  {}", fav_icon, station_name, city)
        };
    }

    pub fn update_status_buffering(&mut self, station_name: &str, _city: &str) {
        self.is_buffering = true;
        self.is_playing = false;
        self.station_info = format!("󰑮 Connecting to {}...", station_name);
    }

    pub fn update_status_playing(&mut self, station_id: &str, station_name: &str, city: &str) {
        self.is_playing = true;
        self.is_buffering = false;
        let fav_icon = if self.favorites.contains(station_id) { "♥" } else { "♡" };
        self.station_info = if city.is_empty() {
            format!("󰐊 LIVE: {} {}", fav_icon, station_name)
        } else {
            format!("󰐊 LIVE: {} {}  —  {}", fav_icon, station_name, city)
        };
    }

    pub fn update_status_stopped(&mut self, station_id: &str, station_name: &str, city: &str) {
        self.is_playing = false;
        self.is_buffering = false;
        let fav_icon = if self.favorites.contains(station_id) { "♥" } else { "♡" };
        self.station_info = if city.is_empty() {
            format!("󰓛 Stopped: {} {}", fav_icon, station_name)
        } else {
            format!("󰓛 Stopped: {} {}  —  {}", fav_icon, station_name, city)
        };
    }
}
