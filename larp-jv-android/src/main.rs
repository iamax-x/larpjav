//! larp-jv-android — Android frontend para larp.jv
//! Entry point via ndk-glue

use android_activity::AndroidApp;
use android_logger::Config;
use log::{LevelFilter, info, error};
use std::sync::Arc;
use larp_jv_core::{
    AuthManager, Downloader, InstanceManager, ModManager, JreManager, MoJsonManager
};
use crate::platform::{callback::JniBridge, app::AndroidAppState};
use crate::render::surface::AndroidSurface;
use crate::input::raw::InputHandler;

mod platform;
mod render;
mod input;
mod native;
mod ui;

#[no_mangle]
fn android_main(app: AndroidApp) {
    // 1. Logger Android
    android_logger::init_once(
        Config::default()
            .with_tag("larp.jv")
            .with_max_level(LevelFilter::Debug),
    );

    info!("🚀 larp.jv starting on Android");

    // 2. Inicializa core services
    let auth_manager = Arc::new(AuthManager::new());
    let downloader = Arc::new(Downloader::new().expect("Downloader init failed"));
    let instance_manager = Arc::new(
        InstanceManager::new("/data/data/com.larp.jv/files/instances")
            .expect("InstanceManager init failed")
    );
    let mod_manager = Arc::new(ModManager::new().expect("ModManager init failed"));
    let jre_manager = Arc::new(
        JreManager::new("/data/data/com.larp.jv/files/jres")
            .expect("JreManager init failed")
    );
    let mojson_manager = Arc::new(
        MoJsonManager::new("/data/data/com.larp.jv/files/mojson")
            .expect("MoJsonManager init failed")
    );

    // 3. JNI Bridge
    let vm = Arc::new(unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut _) }.expect("Invalid JavaVM"));
    let activity = unsafe { jni::objects::JObject::from_raw(app.activity_as_ptr() as *mut _) };
    let jni_bridge = Arc::new(JniBridge::new(vm.clone(), activity));

    // 4. Platform traits implementation
    let platform = AndroidPlatform::new(jni_bridge.clone());

    // 5. Asset loading
    let assets = AndroidAssets::new(app.activity());

    // 6. Input handler
    let mut input_handler = InputHandler::new(jni_bridge.clone());

    // 7. App state
    let mut app_state = AndroidAppState::new(
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
    );

    // 8. Main event loop
    if let Err(e) = app_state.run() {
        error!("App error: {}", e);
    }
}

struct AndroidPlatform {
    jni_bridge: Arc<JniBridge>,
}

impl AndroidPlatform {
    fn new(jni_bridge: Arc<JniBridge>) -> Self {
        Self { jni_bridge }
    }
}

struct AndroidAssets {
    activity: jni::objects::JObject<'static>,
}

impl AndroidAssets {
    fn new(activity: jni::objects::JObject) -> Self {
        Self { activity }
    }
}