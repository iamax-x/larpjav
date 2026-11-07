//! Download verification

use crate::error::{DownloadError, LarpError, LarpResult};
use crate::utils::hash::{verify_hash, HashAlgorithm};
use std::path::Path;

/// Verify file hash
pub async fn verify_file_hash(
    path: &Path,
    expected_hash: &str,
    algorithm: HashAlgorithm,
) -> LarpResult<bool> {
    verify_hash(path, expected_hash, algorithm)
}

/// Verify file size
pub async fn verify_file_size(path: &Path, expected_size: u64) -> LarpResult<bool> {
    let metadata = tokio::fs::metadata(path).await?;
    Ok(metadata.len() == expected_size)
}

/// Verify file (hash and/or size)
pub async fn verify_file(
    path: &Path,
    expected_hash: Option<&str>,
    algorithm: Option<HashAlgorithm>,
    expected_size: Option<u64>,
) -> LarpResult<()> {
    if let (Some(hash), Some(algo)) = (expected_hash, algorithm) {
        let verified = verify_hash(path, hash, algo).await?;
        if !verified {
            return Err(DownloadError::HashMismatch {
                expected: hash.to_string(),
                actual: "mismatch".to_string(),
            }.into());
        }
    }

    if let Some(size) = expected_size {
        let verified = verify_file_size(path, size).await?;
        if !verified {
            let actual = tokio::fs::metadata(path).await?.len();
            return Err(DownloadError::SizeMismatch { expected: size, actual }.into());
        }
    }

    Ok(())
}

/// Verification exception
#[derive(Debug, thiserror::Error)]
pub enum VerificationException {
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: u64, actual: u64 },

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<VerificationException> for DownloadError {
    fn from(e: VerificationException) -> Self {
        match e {
            VerificationException::HashMismatch { expected, actual } => DownloadError::HashMismatch { expected, actual },
            VerificationException::SizeMismatch { expected, actual } => DownloadError::SizeMismatch { expected, actual },
            VerificationException::FileNotFound(path) => DownloadError::Other(format!("File not found: {}", path)),
            VerificationException::Io(e) => DownloadError::Io(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_verify_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        let path = file.path();

        // Test hash verification
        let result = verify_file_hash(path, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed", HashAlgorithm::Sha1).await.unwrap();
        assert!(result);

        // Test size verification
        let result = verify_file_size(path, 11).await.unwrap();
        assert!(result);

        // Test combined verification
        verify_file(path, Some("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"), Some(HashAlgorithm::Sha1), Some(11)).await.unwrap();
    }
}