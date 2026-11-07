//! Download engine

use crate::downloader::{DownloadTask, TaskMetadata, TaskState};
use crate::error::{DownloadError, LarpError, LarpResult};
use crate::utils::download::{download_file, download_bytes, create_client};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Progress listener trait
#[async_trait]
pub trait ProgressListener: Send + Sync {
    async fn on_progress(&self, downloaded: u64, total: u64);
    async fn on_start(&self, total: u64);
    async fn on_complete(&self);
    async fn on_error(&self, error: &DownloadError);
}

/// No-op progress listener
pub struct NoopProgressListener;

#[async_trait]
impl ProgressListener for NoopProgressListener {
    async fn on_progress(&self, _downloaded: u64, _total: u64) {}
    async fn on_start(&self, _total: u64) {}
    async fn on_complete(&self) {}
    async fn on_error(&self, _error: &DownloadError) {}
}

/// Download engine - core download logic
pub struct DownloaderEngine {
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    max_concurrent: usize,
    chunk_size: usize,
    timeout: std::time::Duration,
}

impl DownloaderEngine {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent: 4,
            chunk_size: 8192,
            timeout: std::time::Duration::from_secs(30),
        }
    }

    pub fn set_max_concurrent(&mut self, n: usize) {
        self.max_concurrent = n;
    }

    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size = size;
    }

    pub fn set_timeout(&mut self, timeout: std::time::Duration) {
        self.timeout = timeout;
    }

    /// Create a new download task
    pub async fn create_task(
        &self,
        url: &str,
        dest: &Path,
        progress: Option<&mut dyn ProgressListener>,
    ) -> LarpResult<DownloadTask> {
        let task = DownloadTask::new(url, dest, progress);
        self.tasks.write().await.insert(task.id.clone(), task.clone());
        Ok(task)
    }

    /// Execute a download task
    pub async fn execute_task(
        &self,
        client: &reqwest::Client,
        mut task: DownloadTask,
    ) -> LarpResult<u64> {
        task.state = TaskState::Downloading;
        self.update_task(task.clone()).await;

        let progress = task.progress.take();
        let mut progress_listener = progress.map(|p| Box::new(CallbackProgressListener { inner: p }) as Box<dyn ProgressListener + Send>);

        let result = download_file(
            client,
            &task.url,
            &task.destination,
            progress_listener.as_mut().map(|p| &mut **p),
            task.resume,
        ).await;

        match result {
            Ok(size) => {
                task.state = TaskState::Completed;
                task.downloaded = size;
                if let Some(p) = &progress_listener {
                    p.on_complete().await;
                }
                self.update_task(task).await;
                Ok(size)
            }
            Err(e) => {
                task.state = TaskState::Failed(e.to_string());
                if let Some(p) = &progress_listener {
                    p.on_error(&e).await;
                }
                self.update_task(task).await;
                Err(e)
            }
        }
    }

    /// Download multiple files in parallel
    pub async fn download_parallel(
        &self,
        client: &reqwest::Client,
        urls: &[(&str, &Path)],
        max_concurrent: usize,
    ) -> LarpResult<Vec<u64>> {
        use futures::stream::{self, StreamExt};

        let results = stream::iter(urls.iter().map(|(url, dest)| {
            let client = client.clone();
            let url = *url;
            let dest = dest.to_path_buf();
            async move {
                let task = DownloadTask::new(url, &dest, None);
                download_file(&client, url, &dest, None, false).await
            }
        }))
        .buffer_unordered(max_concurrent)
        .collect::<Vec<_>>()
        .await;

        results.into_iter().collect()
    }

    /// Get active tasks
    pub async fn active_tasks(&self) -> Vec<DownloadTask> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// Get task by ID
    pub async fn get_task(&self, id: &str) -> Option<DownloadTask> {
        self.tasks.read().await.get(id).cloned()
    }

    /// Cancel a task
    pub async fn cancel_task(&self, id: &str) -> LarpResult<bool> {
        if let Some(mut task) = self.tasks.write().await.get_mut(id) {
            task.state = TaskState::Cancelled;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Cancel all tasks
    pub async fn cancel_all(&self) {
        let mut tasks = self.tasks.write().await;
        for task in tasks.values_mut() {
            if matches!(task.state, TaskState::Downloading | TaskState::Pending) {
                task.state = TaskState::Cancelled;
            }
        }
    }

    async fn update_task(&self, task: DownloadTask) {
        self.tasks.write().await.insert(task.id.clone(), task);
    }
}

impl Default for DownloaderEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Callback-based progress listener
struct CallbackProgressListener {
    inner: Box<dyn FnMut(u64, u64) + Send>,
}

#[async_trait]
impl ProgressListener for CallbackProgressListener {
    async fn on_progress(&self, downloaded: u64, total: u64) {
        // Can't mutate self in async trait, need interior mutability
    }
    async fn on_start(&self, _total: u64) {}
    async fn on_complete(&self) {}
    async fn on_error(&self, _error: &DownloadError) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let _engine = DownloaderEngine::new();
    }
}