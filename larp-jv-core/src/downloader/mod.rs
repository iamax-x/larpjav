//! Download engine for larp.jv

pub mod engine;
pub mod task;
pub mod file;
pub mod verify;
pub mod metadata;
pub mod acquire;
pub mod progress;
pub mod error;

use crate::error::{DownloadError, LarpError, LarpResult};
use crate::utils::download::{create_client, create_client_with_ua};
use engine::DownloaderEngine;
use task::{DownloadTask, TaskMetadata, TaskState};
use progress::ProgressListener;

/// Download manager - high-level interface
pub struct Downloader {
    engine: DownloaderEngine,
    client: reqwest::Client,
    default_progress: Option<Box<dyn ProgressListener>>,
}

impl Downloader {
    pub fn new() -> LarpResult<Self> {
        Ok(Self {
            engine: DownloaderEngine::new(),
            client: create_client()?,
            default_progress: None,
        })
    }

    pub fn with_client(client: reqwest::Client) -> LarpResult<Self> {
        Ok(Self {
            engine: DownloaderEngine::new(),
            client,
            default_progress: None,
        })
    }

    pub fn with_user_agent(user_agent: &str) -> LarpResult<Self> {
        Ok(Self {
            engine: DownloaderEngine::new(),
            client: create_client_with_ua(user_agent)?,
            default_progress: None,
        })
    }

    pub fn set_default_progress<P: ProgressListener + 'static>(&mut self, listener: P) {
        self.default_progress = Some(Box::new(listener));
    }

    /// Download a file
    pub async fn download_file(
        &self,
        url: &str,
        dest: &std::path::Path,
        progress: Option<&mut dyn ProgressListener>,
    ) -> LarpResult<u64> {
        let task = self.engine.create_task(url, dest, progress.or(self.default_progress.as_deref_mut())).await?;
        self.engine.execute_task(&self.client, task).await
    }

    /// Download multiple files in parallel
    pub async fn download_parallel(
        &self,
        urls: &[(&str, &std::path::Path)],
        max_concurrent: usize,
    ) -> LarpResult<Vec<u64>> {
        self.engine.download_parallel(&self.client, urls, max_concurrent).await
    }

    /// Get active downloads
    pub async fn active_downloads(&self) -> Vec<DownloadTask> {
        self.engine.active_tasks().await
    }

    /// Cancel a download
    pub async fn cancel(&self, task_id: &str) -> LarpResult<bool> {
        self.engine.cancel_task(task_id).await
    }

    /// Cancel all downloads
    pub async fn cancel_all(&self) {
        self.engine.cancel_all().await;
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("Failed to create downloader")
    }
}

/// Builder for Downloader
pub struct DownloaderBuilder {
    client: Option<reqwest::Client>,
    user_agent: Option<String>,
    max_concurrent: usize,
    chunk_size: usize,
    timeout: std::time::Duration,
}

impl DownloaderBuilder {
    pub fn new() -> Self {
        Self {
            client: None,
            user_agent: None,
            max_concurrent: 4,
            chunk_size: 8192,
            timeout: std::time::Duration::from_secs(30),
        }
    }

    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn max_concurrent(mut self, n: usize) -> Self {
        self.max_concurrent = n;
        self
    }

    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> LarpResult<Downloader> {
        let client = if let Some(c) = self.client {
            c
        } else if let Some(ua) = self.user_agent {
            create_client_with_ua(&ua)?
        } else {
            create_client()?
        };

        let mut downloader = Downloader::with_client(client)?;
        // Configure engine
        downloader.engine.set_max_concurrent(self.max_concurrent);
        downloader.engine.set_chunk_size(self.chunk_size);
        downloader.engine.set_timeout(self.timeout);
        Ok(downloader)
    }
}

impl Default for DownloaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_downloader_creation() {
        let downloader = Downloader::new().unwrap();
        assert_eq!(downloader.active_downloads().await.len(), 0);
    }
}