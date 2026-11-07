//! Tasks — Port of AsyncAssetManager, AsyncVersionList, DataMigrator, NativesExtractor, SpeedCalculator

use crate::error::LarpResult;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// Async Asset Manager — Port of AsyncAssetManager.java
/// Manages loading of assets (textures, fonts, etc.) in background
pub struct AsyncAssetManager {
    load_tasks: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<LarpResult<Vec<u8>>>>>>,
    cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl AsyncAssetManager {
    pub fn new() -> Self {
        Self {
            load_tasks: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn load_asset(&self, key: &str, loader: impl FnOnce() -> LarpResult<Vec<u8>> + Send + 'static) -> LarpResult<Vec<u8>> {
        // Check cache first
        if let Some(data) = self.cache.lock().await.get(key) {
            return Ok(data.clone());
        }
        
        // Check if already loading
        let mut tasks = self.load_tasks.lock().await;
        if let Some(handle) = tasks.get(key) {
            return handle.await??;
        }
        
        // Start new load task
        let key_owned = key.to_string();
        let cache = self.cache.clone();
        let handle = tokio::spawn(async move {
            let data = loader()?;
            cache.lock().await.insert(key_owned.clone(), data.clone());
            Ok(data)
        });
        
        tasks.insert(key.to_string(), handle);
        
        // Wait for completion
        let handle = tasks.remove(key).unwrap();
        handle.await?
    }

    pub async fn preload(&self, key: &str, loader: impl FnOnce() -> LarpResult<Vec<u8>> + Send + 'static) {
        let key_owned = key.to_string();
        let cache = self.cache.clone();
        let handle = tokio::spawn(async move {
            if let Ok(data) = loader() {
                cache.lock().await.insert(key_owned, data);
            }
        });
        self.load_tasks.lock().await.insert(key.to_string(), handle);
    }

    pub async fn clear_cache(&self) {
        self.cache.lock().await.clear();
    }

    pub async fn remove(&self, key: &str) {
        self.cache.lock().await.remove(key);
    }
}

impl Default for AsyncAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Async Version List — Port of AsyncVersionList.java
/// Loads version manifest asynchronously
pub struct AsyncVersionList {
    manifest_url: String,
    cache: Arc<Mutex<Option<crate::mojson::client::VersionManifest>>>,
    loading: Arc<Mutex<bool>>,
}

impl AsyncVersionList {
    pub fn new(manifest_url: String) -> Self {
        Self {
            manifest_url,
            cache: Arc::new(Mutex::new(None)),
            loading: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn get_versions(&self, mojson: &crate::mojson::client::MoJsonClient) -> LarpResult<crate::mojson::client::VersionManifest> {
        // Check cache
        if let Some(cached) = self.cache.lock().await.as_ref() {
            return Ok(cached.clone());
        }
        
        // Check if already loading
        let mut loading = self.loading.lock().await;
        if *loading {
            // Wait for loading to complete (simplified - in real impl use notify)
            drop(loading);
            while self.cache.lock().await.is_none() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            return Ok(self.cache.lock().await.as_ref().unwrap().clone());
        }
        
        *loading = true;
        drop(loading);
        
        // Load manifest
        let manifest = mojson.fetch_version_manifest().await?;
        *self.cache.lock().await = Some(manifest.clone());
        *self.loading.lock().await = false;
        
        Ok(manifest)
    }

    pub async fn invalidate_cache(&self) {
        *self.cache.lock().await = None;
    }
}

/// Data Migrator — Port of DataMigrator.java
/// Handles migration of data between versions
pub struct DataMigrator {
    migrations: Vec<Box<dyn Migration>>,
}

pub trait Migration: Send + Sync {
    fn version(&self) -> u32;
    fn migrate(&self, data_dir: &Path) -> LarpResult<()>;
}

impl DataMigrator {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    pub fn add_migration(&mut self, migration: Box<dyn Migration>) {
        self.migrations.push(migration);
    }

    pub async fn migrate(&self, data_dir: &Path, from_version: u32) -> LarpResult<()> {
        for migration in &self.migrations {
            if migration.version() > from_version {
                log::info!("Running migration v{}", migration.version());
                migration.migrate(data_dir)?;
            }
        }
        Ok(())
    }
}

impl Default for DataMigrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Natives Extractor — Port of NativesExtractor.java
/// Extracts native libraries from JARs
pub struct NativesExtractor;

impl NativesExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_natives(&self, jar_path: &Path, dest_dir: &Path, classifier: &str) -> LarpResult<()> {
        use crate::utils::zip::extract_jar;
        
        // Extract the natives JAR
        extract_jar(jar_path, dest_dir)?;
        
        // Organize by platform
        let platform_dir = dest_dir.join(classifier);
        if platform_dir.exists() {
            for entry in std::fs::read_dir(&platform_dir)? {
                let entry = entry?;
                let dest = dest_dir.join(entry.file_name());
                std::fs::rename(entry.path(), dest)?;
            }
            std::fs::remove_dir(platform_dir)?;
        }
        
        Ok(())
    }

    pub fn extract_all_natives(&self, jars: &[(std::path::PathBuf, String)], dest_dir: &Path) -> LarpResult<()> {
        for (jar, classifier) in jars {
            self.extract_natives(jar, dest_dir, classifier)?;
        }
        Ok(())
    }
}

impl Default for NativesExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Speed Calculator — Port of SpeedCalculator.java
/// Calculates download/install speeds
pub struct SpeedCalculator {
    samples: Vec<(Instant, u64)>, // (timestamp, bytes)
    window_duration: std::time::Duration,
}

impl SpeedCalculator {
    pub fn new(window_seconds: u64) -> Self {
        Self {
            samples: Vec::new(),
            window_duration: std::time::Duration::from_secs(window_seconds),
        }
    }

    pub fn add_sample(&mut self, bytes: u64) {
        let now = Instant::now();
        self.samples.push((now, bytes));
        self.cleanup_old_samples(now);
    }

    pub fn get_speed_bps(&self) -> f64 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        
        let first = self.samples.first().unwrap();
        let last = self.samples.last().unwrap();
        
        let elapsed = last.0.duration_since(first.0).as_secs_f64();
        if elapsed <= 0.0 {
            return 0.0;
        }
        
        let bytes_diff = last.1.saturating_sub(first.1) as f64;
        bytes_diff / elapsed
    }

    pub fn get_speed_mbps(&self) -> f64 {
        self.get_speed_bps() / (1024.0 * 1024.0)
    }

    fn cleanup_old_samples(&mut self, now: Instant) {
        let cutoff = now - self.window_duration;
        self.samples.retain(|(t, _)| *t > cutoff);
    }

    pub fn reset(&mut self) {
        self.samples.clear();
    }
}

impl Default for SpeedCalculator {
    fn default() -> Self {
        Self::new(5) // 5 second window
    }
}