//! Complete metadata task

use crate::downloader::DownloadTask;
use crate::error::LarpResult;
use std::path::Path;

/// Complete metadata after download
pub async fn complete_metadata(task: &mut DownloadTask) -> LarpResult<()> {
    // Update file metadata
    let metadata = tokio::fs::metadata(&task.destination).await?;
    task.total_size = metadata.len();
    task.completed_at = Some(std::time::SystemTime::now());

    // Set file times if needed
    // ...

    Ok(())
}

/// Extract archive if needed
pub async fn extract_if_archive(task: &DownloadTask, dest_dir: &Path) -> LarpResult<Vec<std::path::PathBuf>> {
    use crate::utils::zip::extract_zip;

    let path = &task.destination;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "zip" => {
            extract_zip(path, dest_dir)?;
            let entries = crate::utils::zip::list_zip_entries(path)?;
            Ok(entries.into_iter().map(|e| dest_dir.join(e)).collect())
        }
        "tar" | "gz" | "tgz" => {
            // Use tar crate
            let file = tokio::fs::File::open(path).await?;
            let mut archive = tar::Archive::new(file);
            archive.unpack(dest_dir)?;
            Ok(vec![])
        }
        _ => Ok(vec![]),
    }
}

/// Post-download processing
pub struct PostDownloadProcessor;

impl PostDownloadProcessor {
    pub async fn process(task: &mut DownloadTask, options: ProcessOptions) -> LarpResult<ProcessResult> {
        let mut result = ProcessResult::default();

        if options.verify {
            let verified = crate::downloader::verify::verify_file(
                &task.destination,
                task.metadata.expected_hash.as_deref(),
                task.metadata.hash_algorithm,
                None,
            ).await?;
            result.verified = verified;
        }

        if options.extract {
            let extracted = extract_if_archive(task, &options.extract_dir).await?;
            result.extracted_files = extracted;
        }

        if options.complete_metadata {
            complete_metadata(task).await?;
        }

        Ok(result)
    }
}

/// Process options
#[derive(Debug, Clone)]
pub struct ProcessOptions {
    pub verify: bool,
    pub extract: bool,
    pub complete_metadata: bool,
    pub extract_dir: std::path::PathBuf,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            verify: true,
            extract: false,
            complete_metadata: true,
            extract_dir: std::path::PathBuf::from("."),
        }
    }
}

/// Process result
#[derive(Debug, Clone, Default)]
pub struct ProcessResult {
    pub verified: bool,
    pub extracted_files: Vec<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_options() {
        let opts = ProcessOptions::default();
        assert!(opts.verify);
        assert!(!opts.extract);
    }
}