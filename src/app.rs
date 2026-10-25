pub struct App {
    pub selected: Option<usize>,
    pub status: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected: None,
            status: "Pan to select a station | Enter: play | Space: stop | q: quit".into(),
        }
    }
}
