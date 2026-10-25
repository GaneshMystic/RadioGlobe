pub struct App {
    pub selected: Option<usize>,
    pub status: String,
    pub is_playing: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected: None,
            status: String::new(),
            is_playing: false,
        }
    }

    /// Build status bar text for the idle (browsing) state.
    pub fn update_status_idle(&mut self, station_name: &str, city: &str) {
        self.is_playing = false;
        if city.is_empty() {
            self.status = format!("▸ {}  [Enter] play  [n/p] next/prev  [+/-] zoom  [q] quit", station_name);
        } else {
            self.status = format!("▸ {}  —  {}  [Enter] play  [n/p] next/prev  [+/-] zoom  [q] quit", station_name, city);
        }
    }

    /// Build status bar text for the playing state.
    pub fn update_status_playing(&mut self, station_name: &str, city: &str) {
        self.is_playing = true;
        if city.is_empty() {
            self.status = format!("▶  {}  [Space] stop  [n/p] next/prev  [q] quit", station_name);
        } else {
            self.status = format!("▶  {}  —  {}  [Space] stop  [n/p] next/prev  [q] quit", station_name, city);
        }
    }

    /// Build status bar text for the stopped state.
    pub fn update_status_stopped(&mut self) {
        self.is_playing = false;
        self.status = "⏹  Stopped  [Enter] play  [n/p] next/prev  [q] quit".into();
    }
}
