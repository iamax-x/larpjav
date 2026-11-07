//! JNI Bridge: CallbackBridge.kt (Kotlin) ↔ Rust
//! O MojoLauncher já tem CallbackBridge.java — portamos para Kotlin + Rust

use jni::{JavaVM, objects::{JObject, JString}, sys::{jlong, jboolean, jint}};
use std::sync::Arc;
use larp_jv_core::LarpResult;

/// JNI Bridge: CallbackBridge.kt (Kotlin) ↔ Rust
pub struct JniBridge {
    vm: Arc<jni::JavaVM>,
    activity: jni::objects::JObject<'static>,
}

impl JniBridge {
    pub fn new(vm: Arc<jni::JavaVM>, activity: jni::objects::JObject) -> Self {
        let env = vm.get_env().unwrap();
        let global_activity = env.new_global_ref(activity).unwrap();
        Self { 
            vm, 
            activity: global_activity.into() 
        }
    }

    pub fn vibrate(&self, duration_ms: u64) {
        if let Ok(env) = self.vm.get_env() {
            let vibrator = env.get_system_service(self.activity.as_obj(), "vibrator").ok();
            if let Some(vibrator) = vibrator {
                let _ = env.call_method(
                    vibrator, 
                    "vibrate", 
                    "(J)V", 
                    &[jni::objects::JValue::Long(duration_ms as i64)]
                );
            }
        }
    }

    pub fn request_permission(&self, permission: &str) -> bool {
        if let Ok(env) = self.vm.get_env() {
            let jperm = env.new_string(permission).unwrap();
            let result = env.call_method(
                self.activity.as_obj(),
                "requestPermission",
                "(Ljava/lang/String;)Z",
                &[jni::objects::JValue::Object(&jperm)]
            ).ok();
            result.and_then(|v| v.z()).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn check_permission(&self, permission: &str) -> bool {
        if let Ok(env) = self.vm.get_env() {
            let jperm = env.new_string(permission).unwrap();
            let result = env.call_method(
                self.activity.as_obj(),
                "checkPermission",
                "(Ljava/lang/String;)Z",
                &[jni::objects::JValue::Object(&jperm)]
            ).ok();
            result.and_then(|v| v.z()).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn get_clipboard_text(&self) -> String {
        if let Ok(env) = self.vm.get_env() {
            let clipboard = env.get_system_service(self.activity.as_obj(), "clipboard").ok();
            if let Some(clipboard) = clipboard {
                if let Ok(text) = env.call_method(clipboard, "getText", "()Ljava/lang/CharSequence;", &[]) {
                    if let Ok(obj) = text.l() {
                        return env.get_string(&JString::from(obj)).unwrap().to_string();
                    }
                }
            }
        }
        String::new()
    }

    pub fn set_clipboard_text(&self, text: &str) {
        if let Ok(env) = self.vm.get_env() {
            let clipboard = env.get_system_service(self.activity.as_obj(), "clipboard").ok();
            if let Some(clipboard) = clipboard {
                let jtext = env.new_string(text).unwrap();
                let _ = env.call_method(
                    clipboard,
                    "setText",
                    "(Ljava/lang/CharSequence;)V",
                    &[jni::objects::JValue::Object(&jtext)]
                );
            }
        }
    }
}

// JNI exports (chamados do Kotlin)
#[no_mangle]
pub extern "system" fn Java_com_larp_jv_CallbackBridge_nativeInit(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    _bridge_ptr: jlong,
) {
    // Initialize any Rust globals needed by JNI
    log::debug!("JNI Bridge initialized");
}

#[no_mangle]
pub extern "system" fn Java_com_larp_jv_CallbackBridge_nativeGetClipboardText(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
    bridge_ptr: jlong,
) -> jni::objects::JString {
    let bridge = unsafe { &*(bridge_ptr as *const crate::platform::callback::JniBridge) };
    let text = bridge.get_clipboard_text();
    env.new_string(text).unwrap().into()
}

#[no_mangle]
pub extern "system" fn Java_com_larp_jv_CallbackBridge_nativeSetClipboardText(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
    bridge_ptr: jlong,
    text: jni::objects::JString,
) {
    let bridge = unsafe { &*(bridge_ptr as *const crate::platform::callback::JniBridge) };
    let text: String = env.get_string(&text).unwrap().into();
    bridge.set_clipboard_text(&text);
}

#[no_mangle]
pub extern "system" fn Java_com_larp_jv_CallbackBridge_nativeVibrate(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    bridge_ptr: jlong,
    duration: jlong,
) {
    let bridge = unsafe { &*(bridge_ptr as *const crate::platform::callback::JniBridge) };
    bridge.vibrate(duration as u64);
}

#[no_mangle]
pub extern "system" fn Java_com_larp_jv_CallbackBridge_nativeRequestPermission(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
    bridge_ptr: jlong,
    permission: jni::objects::JString,
) -> jboolean {
    let bridge = unsafe { &*(bridge_ptr as *const crate::platform::callback::JniBridge) };
    let perm: String = env.get_string(&permission).unwrap().into();
    bridge.request_permission(&perm) as jboolean
}

#[no_mangle]
pub extern "system" fn Java_com_larp_jv_CallbackBridge_nativeCheckPermission(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
    bridge_ptr: jlong,
    permission: jni::objects::JString,
) -> jboolean {
    let bridge = unsafe { &*(bridge_ptr as *const crate::platform::callback::JniBridge) };
    let perm: String = env.get_string(&permission).unwrap().into();
    bridge.check_permission(&perm) as jboolean
}