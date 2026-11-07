//! Progress Keeper — Port of ProgressKeeper, ProgressListener, ProgressState, TaskCountListener, DownloaderProgressWrapper

use crate::error::LarpResult;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;

/// Progress State — Port of ProgressState.java
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressState {
    Waiting,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Progress Listener Trait — Port of ProgressListener.java
pub trait ProgressListener: Send + Sync {
    fn on_progress(&self, task_id: &str, progress: f32, message: &str);
    fn on_state_change(&self, task_id: &str, state: ProgressState);
    fn on_error(&self, task_id: &str, error: &str);
}

/// Task Count Listener — Port of TaskCountListener.java
pub trait TaskCountListener: Send + Sync {
    fn on_task_count_changed(&self, total: usize, completed: usize, failed: usize);
}

/// Progress Info
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub task_id: String,
    pub name: String,
    pub state: ProgressState,
    pub progress: f32, // 0.0 - 1.0
    pub message: String,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub speed_bps: f64,
}

/// Downloader Progress Wrapper — Port of DownloaderProgressWrapper.java
pub struct DownloaderProgressWrapper {
    listeners: Vec<Arc<dyn ProgressListener>>,
    task_count_listeners: Vec<Arc<dyn TaskCountListener>>,
    tasks: Arc<RwLock<HashMap<String, ProgressInfo>>>,
    total_tasks: usize,
    completed_tasks: usize,
    failed_tasks: usize,
}

impl DownloaderProgressWrapper {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
            task_count_listeners: Vec::new(),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
        }
    }

    pub fn add_listener(&mut self, listener: Arc<dyn ProgressListener>) {
        self.listeners.push(listener);
    }

    pub fn add_task_count_listener(&mut self, listener: Arc<dyn TaskCountListener>) {
        self.task_count_listeners.push(listener);
    }

    pub fn remove_listener(&mut self, listener: &Arc<dyn ProgressListener>) {
        self.listeners.retain(|l| !Arc::ptr_eq(l, listener));
    }

    pub async fn start_task(&self, task_id: &str, name: &str, total_bytes: u64) {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.to_string(), ProgressInfo {
            task_id: task_id.to_string(),
            name: name.to_string(),
            state: ProgressState::Running,
            progress: 0.0,
            message: "Starting...".to_string(),
            started_at: Some(Instant::now()),
            completed_at: None,
            bytes_downloaded: 0,
            bytes_total: total_bytes,
            speed_bps: 0.0,
        });
        
        for listener in &self.listeners {
            listener.on_state_change(task_id, ProgressState::Running);
            listener.on_progress(task_id, 0.0, "Starting...");
        }
    }

    pub async fn update_progress(&self, task_id: &str, bytes_downloaded: u64, message: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            let now = Instant::now();
            if let Some(started) = task.started_at {
                let elapsed = now.duration_since(started).as_secs_f64();
                if elapsed > 0.0 {
                    task.speed_bps = bytes_downloaded as f64 / elapsed;
                }
            }
            task.bytes_downloaded = bytes_downloaded;
            task.progress = if task.bytes_total > 0 {
                bytes_downloaded as f32 / task.bytes_total as f32
            } else {
                0.0
            };
            task.message = message.to_string();
            
            // Notify listeners
            for listener in &self.listeners {
                listener.on_progress(task_id, task.progress, message);
            }
        }
    }

    pub async fn complete_task(&self, task_id: &str, message: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.state = ProgressState::Completed;
            task.progress = 1.0;
            task.message = message.to_string();
            task.completed_at = Some(Instant::now());
            
            self.completed_tasks += 1;
            self.notify_task_count();
            
            for listener in &self.listeners {
                listener.on_state_change(task_id, ProgressState::Completed);
                listener.on_progress(task_id, 1.0, message);
            }
        }
    }

    pub async fn fail_task(&self, task_id: &str, error: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.state = ProgressState::Failed;
            task.message = error.to_string();
            task.completed_at = Some(Instant::now());
            
            self.failed_tasks += 1;
            self.notify_task_count();
            
            for listener in &self.listeners {
                listener.on_state_change(task_id, ProgressState::Failed);
                listener.on_error(task_id, error);
            }
        }
    }

    pub async fn pause_task(&self, task_id: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.state = ProgressState::Paused;
            
            for listener in &self.listeners {
                listener.on_state_change(task_id, ProgressState::Paused);
            }
        }
    }

    pub async fn resume_task(&self, task_id: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.state = ProgressState::Running;
            
            for listener in &self.listeners {
                listener.on_state_change(task_id, ProgressState::Running);
            }
        }
    }

    pub async fn get_progress(&self, task_id: &str) -> Option<ProgressInfo> {
        self.tasks.read().await.get(task_id).cloned()
    }

    pub async fn get_all_progress(&self) -> Vec<ProgressInfo> {
        self.tasks.read().await.values().cloned().collect()
    }

    pub async fn get_overall_progress(&self) -> f32 {
        let tasks = self.tasks.read().await;
        if tasks.is_empty() {
            return 1.0;
        }
        let total: f32 = tasks.values().map(|t| t.progress).sum();
        total / tasks.len() as f32
    }

    fn notify_task_count(&self) {
        for listener in &self.task_count_listeners {
            listener.on_task_count_changed(self.total_tasks, self.completed_tasks, self.failed_tasks);
        }
    }

    pub fn set_total_tasks(&mut self, total: usize) {
        self.total_tasks = total;
        self.completed_tasks = 0;
        self.failed_tasks = 0;
        self.notify_task_count();
    }

    pub async fn clear_completed(&self) {
        let mut tasks = self.tasks.write().await;
        tasks.retain(|_, v| v.state != ProgressState::Completed);
    }
}

impl Default for DownloaderProgressWrapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress Service — Port of ProgressService.java
pub struct ProgressService {
    wrapper: DownloaderProgressWrapper,
}

impl ProgressService {
    pub fn new() -> Self {
        Self {
            wrapper: DownloaderProgressWrapper::new(),
        }
    }

    pub fn wrapper(&self) -> &DownloaderProgressWrapper {
        &self.wrapper
    }

    pub fn wrapper_mut(&mut self) -> &mut DownloaderProgressWrapper {
        &mut self.wrapper
    }
}

impl Default for ProgressService {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress Service Keeper — Port of ProgressServiceKeeper.java
pub struct ProgressServiceKeeper {
    services: HashMap<String, Arc<ProgressService>>,
}

impl ProgressServiceKeeper {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, key: &str) -> Arc<ProgressService> {
        self.services.entry(key.to_string())
            .or_insert_with(|| Arc::new(ProgressService::new()))
            .clone()
    }

    pub fn remove(&mut self, key: &str) -> Option<Arc<ProgressService>> {
        self.services.remove(key)
    }

    pub fn get(&self, key: &str) -> Option<Arc<ProgressService>> {
        self.services.get(key).cloned()
    }
}

impl Default for ProgressServiceKeeper {
    fn default() -> Self {
        Self::new()
    }
}