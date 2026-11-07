//! Hash utilities

use crate::error::LarpResult;
use ring::digest;
use std::io::Read;
use std::path::Path;

/// Compute SHA-1 hash of a file
pub fn sha1_file(path: &Path) -> LarpResult<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize())
}

/// Compute SHA-256 hash of a file
pub fn sha256_file(path: &Path) -> LarpResult<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize())
}

/// Compute MD5 hash of a file
pub fn md5_file(path: &Path) -> LarpResult<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Md5Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize())
}

/// Streaming SHA-1 hasher
pub struct Sha1Hasher {
    ctx: digest::Context,
}

impl Sha1Hasher {
    pub fn new() -> Self {
        Self { ctx: digest::Context::new(&digest::SHA1_FOR_LEGACY_USE_ONLY) }
    }
    pub fn update(&mut self, data: &[u8]) {
        self.ctx.update(data);
    }
    pub fn finalize(self) -> String {
        let digest = self.ctx.finish();
        hex::encode(digest.as_ref())
    }
}

/// Streaming SHA-256 hasher
pub struct Sha256Hasher {
    ctx: digest::Context,
}

impl Sha256Hasher {
    pub fn new() -> Self {
        Self { ctx: digest::Context::new(&digest::SHA256) }
    }
    pub fn update(&mut self, data: &[u8]) {
        self.ctx.update(data);
    }
    pub fn finalize(self) -> String {
        let digest = self.ctx.finish();
        hex::encode(digest.as_ref())
    }
}

/// Streaming MD5 hasher (using ring's MD5 is not available, use md5 crate)
pub struct Md5Hasher {
    ctx: md5::Md5,
}

impl Md5Hasher {
    pub fn new() -> Self {
        Self { ctx: md5::Md5::new() }
    }
    pub fn update(&mut self, data: &[u8]) {
        use md5::Digest;
        self.ctx.update(data);
    }
    pub fn finalize(self) -> String {
        let result = self.ctx.finalize();
        hex::encode(result)
    }
}

/// Compute hash of bytes in memory
pub fn sha1_bytes(data: &[u8]) -> String {
    let digest = digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, data);
    hex::encode(digest.as_ref())
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let digest = digest::digest(&digest::SHA256, data);
    hex::encode(digest.as_ref())
}

pub fn md5_bytes(data: &[u8]) -> String {
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Verify file hash against expected
pub fn verify_hash(path: &Path, expected: &str, algorithm: HashAlgorithm) -> LarpResult<bool> {
    let actual = match algorithm {
        HashAlgorithm::Sha1 => sha1_file(path)?,
        HashAlgorithm::Sha256 => sha256_file(path)?,
        HashAlgorithm::Md5 => md5_file(path)?,
    };
    Ok(actual.eq_ignore_ascii_case(expected))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Md5,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_hash_bytes() {
        assert_eq!(sha1_bytes(b"test"), "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
        assert_eq!(sha256_bytes(b"test"), "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08");
        assert_eq!(md5_bytes(b"test"), "098f6bcd4621d373cade4e832627b4f6");
    }

    #[test]
    fn test_hash_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        let path = file.path();
        assert_eq!(sha1_file(path).unwrap(), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
        assert_eq!(sha256_file(path).unwrap(), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
        assert_eq!(md5_file(path).unwrap(), "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }
}
