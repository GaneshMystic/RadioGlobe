use std::process::{Child, Command, Stdio};

pub struct Player {
    child: Option<Child>,
}

impl Player {
    pub fn new() -> Self {
        Self { child: None }
    }

    /// Spawn mpv for the given URL. Returns `true` on success, `false` if mpv
    /// is not installed or failed to start.
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

    /// Kill the current stream if one is running.
    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            // Reap the child so it doesn't become a zombie
            let _ = c.wait();
        }
    }

    /// Returns `true` if a stream is currently running.
    pub fn is_playing(&self) -> bool {
        self.child.is_some()
    }

    /// Non-blocking check: if the child has already exited, clean it up and
    /// return `true`. Otherwise returns `false`.
    pub fn try_reap(&mut self) -> bool {
        if let Some(ref mut c) = self.child {
            match c.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited — take and drop it
                    self.child.take();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}
