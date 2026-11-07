//! larp-jv-core — Pure Rust core library for larp.jv Minecraft launcher
//!
//! This crate contains all platform-independent logic:
//! - Authentication (Microsoft, Ely.by, Offline)
//! - Download engine with verification
//! - Instance and version management
//! - Mod loader support (Forge, Fabric, Quilt, NeoForge, OptiFine, LegacyFabric)
//! - JRE management and Java runner
//! - Mojang JSON parsing (version manifests, rules, libraries)
//! - Utilities (hash, zip, json, file, math, GPU, memory)

#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]
#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod auth;
pub mod downloader;
pub mod error;
pub mod instances;
pub mod jre;
pub mod mojson;
pub mod modding;
pub mod platform;
pub mod utils;
pub mod value;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::auth::{Account, AccountManager, AuthType, BackgroundLogin, LoginListener};
    pub use crate::downloader::{Downloader, DownloaderBuilder, DownloadTask, ProgressListener};
    pub use crate::error::{LarpError, LarpResult};
    pub use crate::instances::{Instance, InstanceManager, VersionManifest};
    pub use crate::jre::{JavaRunner, JreManager, JreVersion};
    pub use crate::mojson::{Library, LibraryArtifact, Rule, VersionManifest as MoJsonManifest};
    pub use crate::modding::{ModLoader, ModInstaller, ModManager};
    pub use crate::utils::{HashUtils, ZipUtils, FileUtils, JsonUtils, DownloadUtils};
    pub use crate::value::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_imports() {
        // Verify all prelude items are accessible
        let _ = LarpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    }
}