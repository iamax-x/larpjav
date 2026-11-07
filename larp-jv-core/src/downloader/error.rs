//! Download error types

use crate::error::LarpError;
use thiserror::Error;

/// Download-specific errors
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTTP {status}: {message}")]
    HttpStatus { status: u16, message: String },

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("File size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: u64, actual: u64 },

    #[error("Resume not supported by server")]
    ResumeNotSupported,

    #[error("Partial download cannot be resumed: {0}")]
    CannotResume(String),

    #[error("Destination exists and overwrite is false")]
    DestinationExists,

    #[error("Insufficient disk space: need {needed}, have {available}")]
    InsufficientSpace { needed: u64, available: u64 },

    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Too many redirects")]
    TooManyRedirects,

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task cancelled")]
    Cancelled,

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Other: {0}")]
    Other(String),
}

impl From<DownloadError> for LarpError {
    fn from(e: DownloadError) -> Self {
        LarpError::Download(e)
    }
}

impl DownloadError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            DownloadError::Timeout { .. }
                | DownloadError::Network(_)
                | DownloadError::HttpStatus { status: 500..=599, .. }
                | DownloadError::Io(_)
        )
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            DownloadError::Network(_) => "NETWORK_ERROR",
            DownloadError::HttpStatus { .. } => "HTTP_ERROR",
            DownloadError::HashMismatch { .. } => "HASH_MISMATCH",
            DownloadError::SizeMismatch { .. } => "SIZE_MISMATCH",
            DownloadError::ResumeNotSupported => "RESUME_NOT_SUPPORTED",
            DownloadError::CannotResume(_) => "CANNOT_RESUME",
            DownloadError::DestinationExists => "DESTINATION_EXISTS",
            DownloadError::InsufficientSpace { .. } => "INSUFFICIENT_SPACE",
            DownloadError::Timeout { .. } => "TIMEOUT",
            DownloadError::TooManyRedirects => "TOO_MANY_REDIRECTS",
            DownloadError::InvalidUrl(_) => "INVALID_URL",
            DownloadError::Io(_) => "IO_ERROR",
            DownloadError::Cancelled => "CANCELLED",
            DownloadError::VerificationFailed(_) => "VERIFICATION_FAILED",
            DownloadError::Other(_) => "UNKNOWN_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_error_codes() {
        let err = DownloadError::HashMismatch { expected: "abc".to_string(), actual: "def".to_string() };
        assert_eq!(err.error_code(), "HASH_MISMATCH");
        assert!(!err.is_recoverable());

        let err = DownloadError::Timeout { seconds: 30 };
        assert_eq!(err.error_code(), "TIMEOUT");
        assert!(err.is_recoverable());
    }
}