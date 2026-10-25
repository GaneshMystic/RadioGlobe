use std::process::{Child, Command, Stdio};

pub struct Player {
    child: Option<Child>,
}

impl Player {
    pub fn new() -> Self {
        Self { child: None }
    }

    pub fn play(&mut self, url: &str) -> bool {
        self.stop();
        match Command::new("mpv")
            .args(["--no-video", "--really-quiet", url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => {
                self.child = Some(child);
                true
            }
            Err(_) => false,
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.child.is_some()
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}
