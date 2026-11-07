//! Download task types

use crate::error::LarpResult;
use std::path::PathBuf;
use std::time::SystemTime;

/// Download task state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
    Verifying,
}

/// Download task
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub destination: PathBuf,
    pub state: TaskState,
    pub total_size: u64,
    pub downloaded: u64,
    pub resume: bool,
    pub progress: Option<Box<dyn FnMut(u64, u64) + Send>>,
    pub created_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub error: Option<String>,
    pub metadata: TaskMetadata,
}

impl DownloadTask {
    pub fn new(
        url: impl Into<String>,
        destination: &std::path::Path,
        progress: Option<&mut dyn FnMut(u64, u64)>,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            url: url.into(),
            destination: destination.to_path_buf(),
            state: TaskState::Pending,
            total_size: 0,
            downloaded: 0,
            resume: false,
            progress: progress.map(|p| Box::new(p) as Box<dyn FnMut(u64, u64) + Send>),
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            error: None,
            metadata: TaskMetadata::default(),
        }
    }

    pub fn with_resume(mut self, resume: bool) -> Self {
        self.resume = resume;
        self
    }

    pub fn with_metadata(mut self, metadata: TaskMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn progress(&self) -> f32 {
        if self.total_size == 0 {
            0.0
        } else {
            self.downloaded as f32 / self.total_size as f32
        }
    }

    pub fn speed(&self) -> u64 {
        if let Some(started) = self.started_at {
            if let Ok(elapsed) = SystemTime::now().duration_since(started) {
                let secs = elapsed.as_secs().max(1);
                return self.downloaded / secs;
            }
        }
        0
    }

    pub fn eta(&self) -> Option<std::time::Duration> {
        if self.downloaded > 0 && self.total_size > self.downloaded {
            let speed = self.speed();
            if speed > 0 {
                let remaining = self.total_size - self.downloaded;
                return Some(std::time::Duration::from_secs(remaining / speed));
            }
        }
        None
    }
}

/// Task metadata
#[derive(Debug, Clone, Default)]
pub struct TaskMetadata {
    pub expected_hash: Option<String>,
    pub hash_algorithm: Option<crate::utils::hash::HashAlgorithm>,
    pub headers: std::collections::HashMap<String, String>,
    pub priority: i32,
    pub tags: Vec<String>,
}

impl TaskMetadata {
    pub fn with_hash(mut self, hash: String, algorithm: crate::utils::hash::HashAlgorithm) -> Self {
        self.expected_hash = Some(hash);
        self.hash_algorithm = Some(algorithm);
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
}

/// Acquirable task metadata (for streaming)
#[derive(Debug, Clone)]
pub struct AcquirableTaskMetadata {
    pub task: DownloadTask,
    pub available: bool,
    pub stream: Option<Box<dyn futures::Stream<Item = LarpResult<bytes::Bytes>> + Send + Unpin>>,
}

impl AcquirableTaskMetadata {
    pub fn new(task: DownloadTask) -> Self {
        Self {
            task,
            available: false,
            stream: None,
        }
    }
}

/// Check file on disk task
#[derive(Debug, Clone)]
pub struct CheckFileOnDiskTask {
    pub path: PathBuf,
    pub expected_hash: Option<String>,
    pub hash_algorithm: Option<crate::utils::hash::HashAlgorithm>,
    pub expected_size: Option<u64>,
}

impl CheckFileOnDiskTask {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            expected_hash: None,
            hash_algorithm: None,
            expected_size: None,
        }
    }

    pub fn with_hash(mut self, hash: String, algorithm: crate::utils::hash::HashAlgorithm) -> Self {
        self.expected_hash = Some(hash);
        self.hash_algorithm = Some(algorithm);
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.expected_size = Some(size);
        self
    }
}

/// Complete metadata task
#[derive(Debug, Clone)]
pub struct CompleteMetadataTask {
    pub task: DownloadTask,
    pub verified: bool,
    pub extracted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_task() {
        let task = DownloadTask::new("https://example.com/file.zip", std::path::Path::new("/tmp/file.zip"), None);
        assert_eq!(task.state, TaskState::Pending);
        assert_eq!(task.progress(), 0.0);
    }

    #[test]
    fn test_task_metadata() {
        let metadata = TaskMetadata::default()
            .with_hash("abc123".to_string(), crate::utils::hash::HashAlgorithm::Sha256)
            .with_priority(10);
        assert_eq!(metadata.expected_hash, Some("abc123".to_string()));
        assert_eq!(metadata.priority, 10);
    }
}