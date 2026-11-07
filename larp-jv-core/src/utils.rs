//! Utility functions for larp.jv

pub mod hash;
pub mod zip;
pub mod json;
pub mod file;
pub mod download;
pub mod math;
pub mod gpu;
pub mod memory;
pub mod keycode;
pub mod options;
pub mod notification;
pub mod signature;
pub mod locale;
pub mod collections;
pub mod maven;
pub mod lifecycle;
pub mod scoped;

// Re-export commonly used types
pub use hash::{sha1_file, sha256_file, md5_file, sha1_bytes, sha256_bytes, md5_bytes, verify_hash, Sha1Hasher, Sha256Hasher, Md5Hasher, HashAlgorithm};
pub use zip::{extract_zip, extract_zip_entry, list_zip_entries, create_zip, read_zip_entry, zip_contains};
pub use json::{parse_json, to_json_pretty, to_json_compact, merge_json, get_by_pointer, set_by_pointer, deep_clone, is_empty};
pub use file::{read_to_string, read_to_bytes, write_string, write_bytes, append_string, copy_with_progress, copy_dir, remove_dir_all, ensure_dir, file_size, exists, is_dir, is_file, canonicalize, list_dir, list_dir_recursive, glob_files, atomic_write, available_space};
pub use math::{clamp, lerp, smoothstep, smootherstep, map_range, deg2rad, rad2deg, distance2d, distance3d, round_to, approx_eq, next_pow2, prev_pow2, signum_i32, signum_f32};
pub use gpu::{GpuVendor, GpuInfo, GpuWorkaround, is_integrated_gpu, recommended_backend};
pub use memory::{MemoryRegion, parse_self_maps, find_memory_holes, process_memory_usage, ProcessMemory, align_up, align_down, is_aligned, page_size};
pub use keycode::{android, glfw, minecraft, android_to_glfw, glfw_to_minecraft, android_to_minecraft};
pub use options::{parse_options, write_options, get_option, get_option_bool, get_option_int, get_option_float, set_option, keys};
pub use notification::{Notification, NotificationPriority, NotificationCategory, NotificationAction, NotificationActionType, NotificationManager, DesktopNotificationManager, AndroidNotificationManager, NoopNotificationManager, info, download_progress};
pub use signature::{verify_jar_signature, get_signature_files, read_manifest, parse_manifest};
pub use locale::{Locale, system_locale, format_datetime, format_relative, format_number};
pub use collections::{merge_maps, get_or_insert, filter_map, map_values, group_by, partition, unique, chunks, flatten};
pub use maven::{MavenCoordinate, resolve_library, download_to_cache};
pub use lifecycle::{LifecycleState, LifecycleManager, TaskHandle, with_timeout, with_retry, TaskGroup};
pub use scoped::{set_thread_local, get_thread_local, remove_thread_local, clear_thread_local, ScopedStorage, with_scoped, ContextVar};

use crate::error::LarpResult;

/// Compute SHA-1 hash of bytes
pub fn sha1(data: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute SHA-256 hash of bytes
pub fn sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute MD5 hash of bytes
pub fn md5(data: &[u8]) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Verify file hash
pub fn verify_file_hash(path: &std::path::Path, expected_hash: &str, algorithm: HashAlgorithm) -> LarpResult<bool> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let actual = match algorithm {
        HashAlgorithm::Sha1 => sha1(&buffer),
        HashAlgorithm::Sha256 => sha256(&buffer),
        HashAlgorithm::Md5 => md5(&buffer),
    };
    Ok(actual.eq_ignore_ascii_case(expected_hash))
}

/// Hash algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Md5,
}

impl HashAlgorithm {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sha1" | "sha-1" => Some(Self::Sha1),
            "sha256" | "sha-256" => Some(Self::Sha256),
            "md5" => Some(Self::Md5),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_functions() {
        let data = b"test data";
        assert_eq!(sha1(data), "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
        assert_eq!(sha256(data), "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08");
        assert_eq!(md5(data), "098f6bcd4621d373cade4e832627b4f6");
    }
}
