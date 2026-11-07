//! File system utilities

use crate::error::LarpResult;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Read entire file to string
pub fn read_to_string(path: &Path) -> LarpResult<String> {
    Ok(fs::read_to_string(path)?)
}

/// Read entire file to bytes
pub fn read_to_bytes(path: &Path) -> LarpResult<Vec<u8>> {
    Ok(fs::read(path)?)
}

/// Write string to file (create parent dirs)
pub fn write_string(path: &Path, content: &str) -> LarpResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Write bytes to file (create parent dirs)
pub fn write_bytes(path: &Path, content: &[u8]) -> LarpResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Append string to file
pub fn append_string(path: &Path, content: &str) -> LarpResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Copy file with progress callback
pub fn copy_with_progress<F>(
    src: &Path,
    dst: &Path,
    mut progress: F,
) -> LarpResult<u64>
where
    F: FnMut(u64, u64),
{
    let mut src_file = fs::File::open(src)?;
    let total = src_file.metadata()?.len();
    let mut dst_file = fs::File::create(dst)?;
    let mut buffer = [0u8; 8192];
    let mut copied = 0u64;
    loop {
        let n = src_file.read(&mut buffer)?;
        if n == 0 { break; }
        dst_file.write_all(&buffer[..n])?;
        copied += n as u64;
        progress(copied, total);
    }
    Ok(copied)
}

/// Recursively copy directory
pub fn copy_dir(src: &Path, dst: &Path) -> LarpResult<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Remove directory recursively
pub fn remove_dir_all(path: &Path) -> LarpResult<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Ensure directory exists
pub fn ensure_dir(path: &Path) -> LarpResult<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Get file size
pub fn file_size(path: &Path) -> LarpResult<u64> {
    Ok(fs::metadata(path)?.len())
}

/// Check if path exists
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Check if path is directory
pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}

/// Check if path is file
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

/// Get canonical path (resolve symlinks)
pub fn canonicalize(path: &Path) -> LarpResult<PathBuf> {
    Ok(fs::canonicalize(path)?)
}

/// List files in directory (non-recursive)
pub fn list_dir(path: &Path) -> LarpResult<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        entries.push(entry?.path());
    }
    Ok(entries)
}

/// List files recursively
pub fn list_dir_recursive(path: &Path) -> LarpResult<Vec<PathBuf>> {
    let mut entries = Vec::new();
    fn walk(dir: &Path, entries: &mut Vec<PathBuf>) -> LarpResult<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            entries.push(path.clone());
            if path.is_dir() {
                walk(&path, entries)?;
            }
        }
        Ok(())
    }
    walk(path, &mut entries)?;
    Ok(entries)
}

/// Find files matching glob pattern
pub fn glob_files(pattern: &str) -> LarpResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    for entry in glob::glob(pattern)? {
        results.push(entry?);
    }
    Ok(results)
}

/// Atomic write: write to temp file then rename
pub fn atomic_write(path: &Path, content: &[u8]) -> LarpResult<()> {
    let tmp_path = path.with_extension("tmp");
    write_bytes(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Get available disk space
pub fn available_space(path: &Path) -> LarpResult<u64> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut free_bytes = 0u64;
        unsafe {
            GetDiskFreeSpaceExW(
                windows::core::PCWSTR(path_wide.as_ptr()),
                Some(&mut free_bytes),
                None,
                None,
            )?;
        }
        Ok(free_bytes)
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::MetadataExt;
        let stat = fs::metadata(path)?;
        Ok(stat.blocks() * 512) // Approximate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_ops() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.txt");
        write_string(&file, "hello").unwrap();
        assert_eq!(read_to_string(&file).unwrap(), "hello");
        append_string(&file, " world").unwrap();
        assert_eq!(read_to_string(&file).unwrap(), "hello world");
    }

    #[test]
    fn test_copy_dir() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("file.txt"), "content").unwrap();
        fs::write(src.join("sub/file2.txt"), "content2").unwrap();

        copy_dir(&src, &dst).unwrap();
        assert!(dst.join("file.txt").exists());
        assert!(dst.join("sub/file2.txt").exists());
    }
}
