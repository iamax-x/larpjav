//! Android App State & Lifecycle

use android_activity::AndroidApp;
use std::sync::Arc;
use larp_jv_core::{
    AuthManager, Downloader, InstanceManager, ModManager, JreManager, MoJsonManager
};
use crate::platform::callback::JniBridge;
use crate::render::surface::AndroidSurface;
use crate::input::raw::InputHandler;
use crate::platform::AndroidPlatform;
use crate::platform::AndroidAssets;
use log::{info, error};

/// App state holding all services and platform integrations
pub struct AndroidAppState {
    app: AndroidApp,
    auth_manager: Arc<AuthManager>,
    downloader: Arc<Downloader>,
    instance_manager: Arc<InstanceManager>,
    mod_manager: Arc<ModManager>,
    jre_manager: Arc<JreManager>,
    mojson_manager: Arc<MoJsonManager>,
    platform: AndroidPlatform,
    assets: AndroidAssets,
    input_handler: InputHandler,
    surface: Option<AndroidSurface>,
    running: bool,
}

impl AndroidAppState {
    pub fn new(
        app: AndroidApp,
        auth_manager: Arc<AuthManager>,
        downloader: Arc<Downloader>,
        instance_manager: Arc<InstanceManager>,
        mod_manager: Arc<ModManager>,
        jre_manager: Arc<JreManager>,
        mojson_manager: Arc<MoJsonManager>,
        platform: AndroidPlatform,
        assets: AndroidAssets,
        input_handler: InputHandler,
    ) -> Self {
        Self {
            app,
            auth_manager,
            downloader,
            instance_manager,
            mod_manager,
            jre_manager,
            mojson_manager,
            platform,
            assets,
            input_handler,
            surface: None,
            running: true,
        }
    }

    pub fn run(&mut self) -> larp_jv_core::LarpResult<()> {
        info!("Starting main event loop");
        
        // Initialize wgpu surface when we have a window
        // This will be called when the Android window is created
        
        // Main loop - poll events
        while self.running {
            self.app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
                self.handle_event(event);
            });
        }
        
        Ok(())
    }

    fn handle_event(&mut self, event: android_activity::Event) {
        use android_activity::Event;
        
        match event {
            Event::Start => {
                info!("App started");
            }
            Event::Resume => {
                info!("App resumed");
            }
            Event::Pause => {
                info!("App paused");
            }
            Event::Stop => {
                info!("App stopped");
            }
            Event::Destroy => {
                info!("App destroyed");
                self.running = false;
            }
            Event::WindowCreated => {
                info!("Window created - initializing wgpu");
                // TODO: Initialize wgpu surface here
            }
            Event::WindowDestroyed => {
                info!("Window destroyed");
                self.surface = None;
            }
            Event::Input(input_event) => {
                self.input_handler.handle_input(input_event);
            }
            Event::ConfigurationChanged => {
                info!("Configuration changed");
            }
            Event::LowMemory => {
                info!("Low memory warning");
            }
            Event::TrimMemory(level) => {
                info!("Trim memory: {:?}", level);
            }
            _ => {}
        }
    }
}

impl Drop for AndroidAppState {
    fn drop(&mut self) {
        info!("AndroidAppState dropped");
    }
}