//! Progress listener

use crate::error::DownloadError;

/// Progress listener trait
pub trait ProgressListener: Send + Sync {
    fn on_start(&self, total: u64);
    fn on_progress(&self, downloaded: u64, total: u64);
    fn on_complete(&self);
    fn on_error(&self, error: &DownloadError);
}

/// No-op progress listener
pub struct NoopProgressListener;

impl ProgressListener for NoopProgressListener {
    fn on_start(&self, _total: u64) {}
    fn on_progress(&self, _downloaded: u64, _total: u64) {}
    fn on_complete(&self) {}
    fn on_error(&self, _error: &DownloadError) {}
}

/// Callback-based progress listener
pub struct CallbackProgressListener {
    on_start: Option<Box<dyn Fn(u64) + Send + Sync>>,
    on_progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    on_complete: Option<Box<dyn Fn() + Send + Sync>>,
    on_error: Option<Box<dyn Fn(&DownloadError) + Send + Sync>>,
}

impl CallbackProgressListener {
    pub fn new() -> Self {
        Self {
            on_start: None,
            on_progress: None,
            on_complete: None,
            on_error: None,
        }
    }

    pub fn on_start<F: Fn(u64) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_start = Some(Box::new(f));
        self
    }

    pub fn on_progress<F: Fn(u64, u64) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_progress = Some(Box::new(f));
        self
    }

    pub fn on_complete<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_complete = Some(Box::new(f));
        self
    }

    pub fn on_error<F: Fn(&DownloadError) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_error = Some(Box::new(f));
        self
    }
}

impl Default for CallbackProgressListener {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressListener for CallbackProgressListener {
    fn on_start(&self, total: u64) {
        if let Some(f) = &self.on_start {
            f(total);
        }
    }

    fn on_progress(&self, downloaded: u64, total: u64) {
        if let Some(f) = &self.on_progress {
            f(downloaded, total);
        }
    }

    fn on_complete(&self) {
        if let Some(f) = &self.on_complete {
            f();
        }
    }

    fn on_error(&self, error: &DownloadError) {
        if let Some(f) = &self.on_error {
            f(error);
        }
    }
}

/// Multi-progress listener
pub struct MultiProgressListener {
    listeners: Vec<Box<dyn ProgressListener>>,
}

impl MultiProgressListener {
    pub fn new() -> Self {
        Self { listeners: Vec::new() }
    }

    pub fn add<P: ProgressListener + 'static>(&mut self, listener: P) {
        self.listeners.push(Box::new(listener));
    }
}

impl Default for MultiProgressListener {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressListener for MultiProgressListener {
    fn on_start(&self, total: u64) {
        for listener in &self.listeners {
            listener.on_start(total);
        }
    }

    fn on_progress(&self, downloaded: u64, total: u64) {
        for listener in &self.listeners {
            listener.on_progress(downloaded, total);
        }
    }

    fn on_complete(&self) {
        for listener in &self.listeners {
            listener.on_complete();
        }
    }

    fn on_error(&self, error: &DownloadError) {
        for listener in &self.listeners {
            listener.on_error(error);
        }
    }
}

/// Progress state for UI
#[derive(Debug, Clone)]
pub struct ProgressState {
    pub total: u64,
    pub downloaded: u64,
    pub speed: u64, // bytes/sec
    pub eta: Option<std::time::Duration>,
}

impl ProgressState {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            downloaded: 0,
            speed: 0,
            eta: None,
        }
    }

    pub fn update(&mut self, downloaded: u64, elapsed: std::time::Duration) {
        self.downloaded = downloaded;
        if elapsed.as_secs() > 0 {
            self.speed = downloaded / elapsed.as_secs();
        }
        if self.speed > 0 && self.total > downloaded {
            self.eta = Some(std::time::Duration::from_secs((self.total - downloaded) / self.speed));
        }
    }

    pub fn progress(&self) -> f32 {
        if self.total == 0 { 0.0 } else { self.downloaded as f32 / self.total as f32 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_progress_listener() {
        let mut called = false;
        let listener = CallbackProgressListener::new()
            .on_complete(move || called = true);
        listener.on_complete();
        assert!(called);
    }

    #[test]
    fn test_progress_state() {
        let mut state = ProgressState::new(1000);
        state.update(500, std::time::Duration::from_secs(10));
        assert_eq!(state.progress(), 0.5);
        assert_eq!(state.speed, 50);
    }
}