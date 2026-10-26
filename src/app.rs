pub struct App {
    pub selected: Option<usize>,
    pub status: String,
    pub is_playing: bool,
    pub is_buffering: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected: None,
            status: String::new(),
            is_playing: false,
            is_buffering: false,
        }
    }

    /// Build status bar text for the idle (browsing) state using Nerd Font icons.
    pub fn update_status_idle(&mut self, station_name: &str, city: &str) {
        self.is_playing = false;
        self.is_buffering = false;
        if city.is_empty() {
            self.status = format!("󰓃 {}  |  [Enter] Play  [n/p] Next/Prev  [WASD] Pan  [+/-] Zoom  [q] Quit", station_name);
        } else {
            self.status = format!("󰓃 {}  —  {}  |  [Enter] Play  [n/p] Next/Prev  [WASD] Pan  [+/-] Zoom  [q] Quit", station_name, city);
        }
    }

    /// Build status bar text for the buffering/connecting state.
    pub fn update_status_buffering(&mut self, station_name: &str, city: &str) {
        self.is_buffering = true;
        self.is_playing = false;
        if city.is_empty() {
            self.status = format!("󰑮 Connecting to {}...  |  [Space] Stop  [q] Quit", station_name);
        } else {
            self.status = format!("󰑮 Connecting to {} — {}...  |  [Space] Stop  [q] Quit", station_name, city);
        }
    }

    /// Build status bar text for the playing state.
    pub fn update_status_playing(&mut self, station_name: &str, city: &str) {
        self.is_playing = true;
        self.is_buffering = false;
        if city.is_empty() {
            self.status = format!("󰐊 LIVE: {}  |  [Space] Stop  [n/p] Next/Prev  [+/-] Zoom  [q] Quit", station_name);
        } else {
            self.status = format!("󰐊 LIVE: {}  —  {}  |  [Space] Stop  [n/p] Next/Prev  [+/-] Zoom  [q] Quit", station_name, city);
        }
    }

    /// Build status bar text for the stopped state.
    pub fn update_status_stopped(&mut self) {
        self.is_playing = false;
        self.is_buffering = false;
        self.status = "󰓛 Stopped  |  [Enter] Play  [n/p] Next/Prev  [WASD] Pan  [+/-] Zoom  [q] Quit".into();
    }
}
