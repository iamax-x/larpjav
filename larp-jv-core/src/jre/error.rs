//! JRE error types

use crate::error::LarpError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JreError {
    #[error("No suitable JRE found for version {version}")]
    NotFound { version: String },

    #[error("JRE download failed: {0}")]
    DownloadFailed(#[from] crate::downloader::DownloadError),

    #[error("JRE extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Invalid JRE structure: {0}")]
    InvalidStructure(String),

    #[error("Java version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("Class file parsing error: {0}")]
    ClassFileParse(String),

    #[error("Class version unsupported: {version}")]
    UnsupportedClassVersion { version: u16 },

    #[error("JVM launch failed: {0}")]
    LaunchFailed(String),

    #[error("JVM process exited with code {code}: {stderr}")]
    ProcessExited { code: i32, stderr: String },

    #[error("Native library not found: {lib}")]
    NativeLibraryMissing { lib: String },

    #[error("Native library load failed: {lib}: {error}")]
    NativeLibraryLoadFailed { lib: String, error: String },

    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),

    #[error("Invalid JVM argument: {0}")]
    InvalidArgument(String),
}

impl From<JreError> for LarpError {
    fn from(e: JreError) -> Self {
        LarpError::Jre(e)
    }
}

impl JreError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            JreError::DownloadFailed(e) if e.is_recoverable()
                | JreError::ExtractionFailed(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jre_error() {
        let err = JreError::NotFound { version: "17".to_string() };
        assert!(!err.is_recoverable());
    }
}