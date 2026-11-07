//! File download implementation

use crate::downloader::DownloadTask;
use crate::error::{DownloadError, LarpError, LarpResult};
use crate::utils::download::download_file;
use reqwest::Client;
use std::path::Path;

/// Download a single file
pub async fn download_single_file(
    client: &Client,
    url: &str,
    dest: &Path,
    progress: Option<&mut dyn FnMut(u64, u64)>,
    resume: bool,
) -> LarpResult<u64> {
    download_file(client, url, dest, progress, resume).await
}

/// Download file with custom options
pub async fn download_file_with_options(
    client: &Client,
    url: &str,
    dest: &Path,
    options: FileDownloadOptions,
) -> LarpResult<u64> {
    let progress_fn = options.progress.map(|mut p| &mut p as &mut dyn FnMut(u64, u64));
    download_file(client, url, dest, progress_fn, options.resume).await
}

/// File download options
#[derive(Debug, Clone)]
pub struct FileDownloadOptions {
    pub resume: bool,
    pub expected_hash: Option<String>,
    pub hash_algorithm: Option<crate::utils::hash::HashAlgorithm>,
    pub expected_size: Option<u64>,
    pub timeout: Option<std::time::Duration>,
    pub progress: Option<Box<dyn FnMut(u64, u64) + Send>>,
    pub headers: std::collections::HashMap<String, String>,
}

impl FileDownloadOptions {
    pub fn new() -> Self {
        Self {
            resume: false,
            expected_hash: None,
            hash_algorithm: None,
            expected_size: None,
            timeout: None,
            progress: None,
            headers: std::collections::HashMap::new(),
        }
    }

    pub fn with_resume(mut self, resume: bool) -> Self {
        self.resume = resume;
        self
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

    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_progress<F: FnMut(u64, u64) + Send + 'static>(mut self, progress: F) -> Self {
        self.progress = Some(Box::new(progress));
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

impl Default for FileDownloadOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify downloaded file
pub async fn verify_download(
    path: &Path,
    expected_hash: &str,
    algorithm: crate::utils::hash::HashAlgorithm,
) -> LarpResult<bool> {
    crate::utils::hash::verify_hash(path, expected_hash, algorithm)
}

/// Get file info from HEAD request
pub async fn get_file_info(client: &Client, url: &str) -> LarpResult<FileInfo> {
    let resp = client.head(url).send().await?;
    if !resp.status().is_success() {
        return Err(DownloadError::HttpStatus {
            status: resp.status().as_u16(),
            message: resp.text().await.unwrap_or_default(),
        }.into());
    }

    let content_length = resp.content_length();
    let content_type = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).map(String::from);
    let accept_ranges = resp.headers().get("accept-ranges").and_then(|v| v.to_str().ok()) == Some("bytes");
    let etag = resp.headers().get("etag").and_then(|v| v.to_str().ok()).map(String::from);
    let last_modified = resp.headers().get("last-modified").and_then(|v| v.to_str().ok()).map(String::from);

    Ok(FileInfo {
        url: url.to_string(),
        content_length,
        content_type,
        accept_ranges,
        etag,
        last_modified,
    })
}

/// File info from HEAD request
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub url: String,
    pub content_length: Option<u64>,
    pub content_type: Option<String>,
    pub accept_ranges: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl FileInfo {
    pub fn supports_resume(&self) -> bool {
        self.accept_ranges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_download_options() {
        let opts = FileDownloadOptions::new()
            .with_resume(true)
            .with_hash("abc123".to_string(), crate::utils::hash::HashAlgorithm::Sha256)
            .with_size(1024);
        assert!(opts.resume);
        assert_eq!(opts.expected_hash, Some("abc123".to_string()));
    }
}