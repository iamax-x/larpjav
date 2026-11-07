//! Download utilities

use crate::error::{DownloadError, LarpResult};
use futures::StreamExt;
use reqwest::Client;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download file with progress and resume support
pub async fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    progress: Option<&mut dyn FnMut(u64, u64)>,
    resume: bool,
) -> LarpResult<u64> {
    // Get file size
    let head = client.head(url).send().await?;
    let total_size = head.content_length().unwrap_or(0);

    // Check existing file for resume
    let mut start_pos = 0u64;
    let mut file = if resume && dest.exists() {
        let metadata = tokio::fs::metadata(dest).await?;
        start_pos = metadata.len();
        if start_pos >= total_size && total_size > 0 {
            return Ok(total_size); // Already complete
        }
        File::options().append(true).open(dest).await?
    } else {
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        File::create(dest).await?
    };

    // Build request with Range header for resume
    let mut request = client.get(url);
    if start_pos > 0 {
        request = request.header("Range", format!("bytes={}-", start_pos));
    }

    let response = request.send().await?;
    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(DownloadError::HttpStatus {
            status: response.status().as_u16(),
            message: response.text().await.unwrap_or_default(),
        }.into());
    }

    if start_pos > 0 && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        // Server doesn't support resume, restart
        file = File::create(dest).await?;
        start_pos = 0;
    }

    let mut stream = response.bytes_stream();
    let mut downloaded = start_pos;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        if let Some(p) = progress {
            p(downloaded, total_size);
        }
    }
    file.flush().await?;
    Ok(downloaded)
}

/// Download to memory
pub async fn download_bytes(client: &Client, url: &str) -> LarpResult<Vec<u8>> {
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(DownloadError::HttpStatus {
            status: response.status().as_u16(),
            message: response.text().await.unwrap_or_default(),
        }.into());
    }
    Ok(response.bytes().await?.to_vec())
}

/// Download with timeout
pub async fn download_with_timeout(
    client: &Client,
    url: &str,
    dest: &Path,
    timeout: Duration,
) -> LarpResult<u64> {
    let mut progress_fn = None;
    tokio::select! {
        result = download_file(client, url, dest, progress_fn.as_mut(), false) => result,
        _ = tokio::time::sleep(timeout) => Err(DownloadError::Timeout { seconds: timeout.as_secs() }.into()),
    }
}

/// Parallel download multiple files
pub async fn download_parallel(
    client: &Client,
    urls: &[(&str, &Path)],
    max_concurrent: usize,
) -> LarpResult<Vec<u64>> {
    use futures::stream::{self, StreamExt};
    stream::iter(urls.iter().map(|(url, dest)| {
        let client = client.clone();
        let url = *url;
        let dest = dest.to_path_buf();
        async move { download_file(&client, url, &dest, None, false).await }
    }))
    .buffer_unordered(max_concurrent)
    .collect()
    .await
}

/// Create HTTP client with sensible defaults
pub fn create_client() -> LarpResult<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .gzip(true)
        .brotli(true)
        .build()?)
}

/// Create HTTP client with custom user agent
pub fn create_client_with_ua(user_agent: &str) -> LarpResult<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .gzip(true)
        .brotli(true)
        .user_agent(user_agent)
        .build()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_download_file() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/test.txt")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("hello world")
            .create_async()
            .await;

        let client = create_client().unwrap();
        let tmp = tempfile::TempDir::new().unwrap();
        let dest = tmp.path().join("test.txt");
        let size = download_file(&client, &format!("{}/test.txt", server.url()), &dest, None, false).await.unwrap();
        assert_eq!(size, 11);
        assert_eq!(tokio::fs::read_to_string(&dest).await.unwrap(), "hello world");
        mock.assert_async().await;
    }
}
