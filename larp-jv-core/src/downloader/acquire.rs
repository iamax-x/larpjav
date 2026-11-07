//! Acquirable task metadata for streaming

use crate::downloader::{DownloadTask, AcquirableTaskMetadata};
use crate::error::LarpResult;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Streaming download with acquire semantics
pub struct StreamingDownload {
    task: DownloadTask,
    stream: Pin<Box<dyn Stream<Item = LarpResult<bytes::Bytes>> + Send>>,
    buffer: bytes::BytesMut,
    chunk_size: usize,
}

impl StreamingDownload {
    pub fn new(
        task: DownloadTask,
        stream: Pin<Box<dyn Stream<Item = LarpResult<bytes::Bytes>> + Send>>,
        chunk_size: usize,
    ) -> Self {
        Self {
            task,
            stream,
            buffer: bytes::BytesMut::new(),
            chunk_size,
        }
    }

    pub fn into_inner(self) -> (DownloadTask, Pin<Box<dyn Stream<Item = LarpResult<bytes::Bytes>> + Send>>) {
        (self.task, self.stream)
    }
}

impl Stream for StreamingDownload {
    type Item = LarpResult<bytes::Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

/// Acquirable task for progressive downloads
pub struct AcquirableTask {
    metadata: AcquirableTaskMetadata,
}

impl AcquirableTask {
    pub fn new(task: DownloadTask) -> Self {
        Self {
            metadata: AcquirableTaskMetadata::new(task),
        }
    }

    pub fn metadata(&self) -> &AcquirableTaskMetadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut AcquirableTaskMetadata {
        &mut self.metadata
    }

    pub fn is_available(&self) -> bool {
        self.metadata.available
    }

    pub fn set_available(&mut self, available: bool) {
        self.metadata.available = available;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquirable_task() {
        let task = crate::downloader::DownloadTask::new("https://example.com/file", std::path::Path::new("/tmp/file"), None);
        let acquirable = AcquirableTask::new(task);
        assert!(!acquirable.is_available());
    }
}