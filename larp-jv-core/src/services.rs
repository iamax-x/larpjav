//! Services — Port of GameService, ProgressService, DownloadService

use crate::error::LarpResult;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Game Service — Port of GameService.java
/// Manages the game lifecycle, rendering, and input
pub struct GameService {
    renderer: Option<Arc<Mutex<crate::game::renderer::GameRenderer>>>,
    input_processor: Option<Arc<Mutex<crate::game::input::InGameEventProcessor>>>,
    running: bool,
}

impl GameService {
    pub fn new() -> Self {
        Self {
            renderer: None,
            input_processor: None,
            running: false,
        }
    }

    pub fn set_renderer(&mut self, renderer: Arc<Mutex<crate::game::renderer::GameRenderer>>) {
        self.renderer = Some(renderer);
    }

    pub fn set_input_processor(&mut self, processor: Arc<Mutex<crate::game::input::InGameEventProcessor>>) {
        self.input_processor = Some(processor);
    }

    pub async fn start(&mut self) -> LarpResult<()> {
        self.running = true;
        log::info!("GameService started");
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.running = false;
        if let Some(renderer) = &self.renderer {
            renderer.lock().await.stop();
        }
        log::info!("GameService stopped");
    }

    pub async fn render_frame(&mut self) -> LarpResult<()> {
        if let Some(renderer) = &self.renderer {
            renderer.lock().await.render_frame()?;
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub async fn handle_input(&mut self) -> LarpResult<()> {
        // Process input events
        if let Some(processor) = &self.input_processor {
            processor.lock().await.clear_deltas();
        }
        Ok(())
    }
}

impl Default for GameService {
    fn default() -> Self {
        Self::new()
    }
}

/// Download Service — Port of DownloadService.java (Android Service)
/// Handles background downloads with notifications
pub struct DownloadService {
    downloader: Arc<crate::downloader::engine::DownloadEngine>,
    active_downloads: Arc<Mutex<std::collections::HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl DownloadService {
    pub fn new(downloader: Arc<crate::downloader::engine::DownloadEngine>) -> Self {
        Self {
            downloader,
            active_downloads: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn start_download(&self, task_id: String, url: String, dest: std::path::PathBuf) -> LarpResult<()> {
        let downloader = self.downloader.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = downloader.download_file(&url, &dest, None).await {
                log::error!("Download failed for {}: {}", task_id, e);
            }
        });
        
        self.active_downloads.lock().await.insert(task_id, handle);
        Ok(())
    }

    pub async fn cancel_download(&self, task_id: &str) -> bool {
        let mut downloads = self.active_downloads.lock().await;
        if let Some(handle) = downloads.remove(task_id) {
            handle.abort();
            true
        } else {
            false
        }
    }

    pub async fn cancel_all(&self) {
        let mut downloads = self.active_downloads.lock().await;
        for (_, handle) in downloads.drain() {
            handle.abort();
        }
    }

    pub async fn get_active_count(&self) -> usize {
        self.active_downloads.lock().await.len()
    }
}

/// Auth Service — Manages authentication state
pub struct AuthService {
    account_manager: Arc<crate::auth::manager::AccountManager>,
    current_account: Arc<Mutex<Option<crate::auth::account::Account>>>,
}

impl AuthService {
    pub fn new(account_manager: Arc<crate::auth::manager::AccountManager>) -> Self {
        Self {
            account_manager,
            current_account: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn login_microsoft(&self) -> LarpResult<crate::auth::account::Account> {
        let account = self.account_manager.login_microsoft().await?;
        *self.current_account.lock().await = Some(account.clone());
        Ok(account)
    }

    pub async fn login_elyby(&self, username: &str, password: &str) -> LarpResult<crate::auth::account::Account> {
        let account = self.account_manager.login_elyby(username, password).await?;
        *self.current_account.lock().await = Some(account.clone());
        Ok(account)
    }

    pub async fn login_offline(&self, username: &str) -> LarpResult<crate::auth::account::Account> {
        let account = self.account_manager.login_offline(username).await?;
        *self.current_account.lock().await = Some(account.clone());
        Ok(account)
    }

    pub async fn logout(&self) {
        *self.current_account.lock().await = None;
    }

    pub async fn get_current_account(&self) -> Option<crate::auth::account::Account> {
        self.current_account.lock().await.clone()
    }

    pub async fn refresh_skin(&self) -> LarpResult<()> {
        if let Some(account) = self.current_account.lock().await.as_ref() {
            self.account_manager.refresh_skin(account).await?;
        }
        Ok(())
    }
}

/// Instance Service — Manages instance operations
pub struct InstanceService {
    instance_manager: Arc<crate::instances::manager::InstanceManager>,
    instance_installer: Arc<crate::instances::installer::InstanceInstaller>,
}

impl InstanceService {
    pub fn new(
        instance_manager: Arc<crate::instances::manager::InstanceManager>,
        instance_installer: Arc<crate::instances::installer::InstanceInstaller>,
    ) -> Self {
        Self {
            instance_manager,
            instance_installer,
        }
    }

    pub async fn create_instance(&self, name: &str, version: &str) -> LarpResult<crate::value::Instance> {
        self.instance_manager.create_instance(name, version).await
    }

    pub async fn delete_instance(&self, name: &str) -> LarpResult<()> {
        self.instance_manager.delete_instance(name).await
    }

    pub async fn clone_instance(&self, name: &str, new_name: &str) -> LarpResult<crate::value::Instance> {
        self.instance_manager.clone_instance(name, new_name).await
    }

    pub async fn launch_instance(&self, name: &str) -> LarpResult<()> {
        let mut instance = self.instance_manager.get_instance(name).await?
            .ok_or_else(|| crate::error::LarpError::Other("Instance not found".to_string()))?;
        
        // Ensure version is installed
        if let Some(version) = &instance.version {
            self.instance_installer.install_version(&mut instance, version).await?;
        }
        
        // Launch game (would start GameService)
        log::info!("Launching instance: {}", name);
        Ok(())
    }

    pub async fn install_mod_loader(&self, name: &str, loader: &str, loader_version: &str) -> LarpResult<()> {
        let mut instance = self.instance_manager.get_instance(name).await?
            .ok_or_else(|| crate::error::LarpError::Other("Instance not found".to_string()))?;
        
        if let Some(mc_version) = &instance.version {
            self.instance_installer.install_mod_loader(&mut instance, loader, loader_version, mc_version).await?;
        }
        
        Ok(())
    }
}

/// Settings Service — Manages global and per-instance settings
pub struct SettingsService {
    global_settings: Arc<Mutex<crate::value::GlobalSettings>>,
    instance_settings: Arc<Mutex<std::collections::HashMap<String, crate::value::InstanceSettings>>>,
}

impl SettingsService {
    pub fn new() -> Self {
        Self {
            global_settings: Arc::new(Mutex::new(crate::value::GlobalSettings::default())),
            instance_settings: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get_global(&self) -> crate::value::GlobalSettings {
        self.global_settings.lock().await.clone()
    }

    pub async fn set_global(&self, settings: crate::value::GlobalSettings) {
        *self.global_settings.lock().await = settings;
    }

    pub async fn get_instance(&self, instance_name: &str) -> crate::value::InstanceSettings {
        self.instance_settings.lock().await
            .get(instance_name)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn set_instance(&self, instance_name: &str, settings: crate::value::InstanceSettings) {
        self.instance_settings.lock().await.insert(instance_name.to_string(), settings);
    }

    pub async fn save(&self) -> LarpResult<()> {
        // Save to disk
        Ok(())
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new()
    }
}