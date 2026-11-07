//! ZIP archive utilities

use crate::error::LarpResult;
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;
use zip::write::FileOptions;

/// Extract ZIP archive to directory
pub fn extract_zip(archive_path: &Path, dest_dir: &Path) -> LarpResult<()> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.name());
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}

/// Extract specific entry from ZIP
pub fn extract_zip_entry(archive_path: &Path, entry_name: &str, dest_path: &Path) -> LarpResult<()> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut entry = archive.by_name(entry_name)?;
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut outfile = std::fs::File::create(dest_path)?;
    std::io::copy(&mut entry, &mut outfile)?;
    Ok(())
}

/// List entries in ZIP archive
pub fn list_zip_entries(archive_path: &Path) -> LarpResult<Vec<String>> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        entries.push(entry.name().to_string());
    }
    Ok(entries)
}

/// Create ZIP archive from directory
pub fn create_zip(source_dir: &Path, archive_path: &Path) -> LarpResult<()> {
    let file = std::fs::File::create(archive_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    add_dir_to_zip(&mut zip, source_dir, source_dir, options)?;
    zip.finish()?;
    Ok(())
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    root: &Path,
    dir: &Path,
    options: FileOptions,
) -> LarpResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(root).unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            add_dir_to_zip(zip, root, &path, options)?;
        } else {
            zip.start_file(name, options)?;
            let mut file = std::fs::File::open(&path)?;
            std::io::copy(&mut file, zip)?;
        }
    }
    Ok(())
}

/// Read a file from ZIP into memory
pub fn read_zip_entry(archive_path: &Path, entry_name: &str) -> LarpResult<Vec<u8>> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut entry = archive.by_name(entry_name)?;
    let mut buffer = Vec::new();
    entry.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Check if ZIP contains entry
pub fn zip_contains(archive_path: &Path, entry_name: &str) -> LarpResult<bool> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    Ok(archive.by_name(entry_name).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_create_and_extract_zip() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source");
        let archive = tmp.path().join("test.zip");
        let dest = tmp.path().join("dest");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("file1.txt"), "hello").unwrap();
        std::fs::create_dir_all(source.join("subdir")).unwrap();
        std::fs::write(source.join("subdir/file2.txt"), "world").unwrap();

        create_zip(&source, &archive).unwrap();
        extract_zip(&archive, &dest).unwrap();

        assert_eq!(std::fs::read_to_string(dest.join("file1.txt")).unwrap(), "hello");
        assert_eq!(std::fs::read_to_string(dest.join("subdir/file2.txt")).unwrap(), "world");
    }
}
