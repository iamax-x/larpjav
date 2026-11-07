//! Android Surface handling for wgpu
//! Usa ANativeWindow_fromSurface via JNI via NativeWindowHelper

use wgpu::{Surface, Instance, SurfaceConfiguration, TextureFormat, PresentMode, CompositeAlphaMode};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, AndroidDisplayHandle, AndroidWindowHandle};
use std::ptr::NonNull;
use larp_jv_core::LarpResult;
use crate::platform::callback::JniBridge;

/// Android Surface wrapper for wgpu
pub struct AndroidSurface {
    surface: Option<Surface<'static>>,
    config: Option<SurfaceConfiguration>,
    native_window: *mut std::ffi::c_void, // ANativeWindow*
    width: u32,
    height: u32,
}

impl AndroidSurface {
    pub fn new(
        instance: &Instance,
        jni_bridge: &JniBridge,
        surface_view: jni::objects::JObject, // SurfaceView ou TextureView
        width: u32,
        height: u32,
    ) -> LarpResult<Self> {
        // 1. Obter ANativeWindow do Surface via JNI helper
        let native_window = Self::get_native_window(jni_bridge, surface_view)?;
        
        // 2. Criar wgpu Surface
        let surface = unsafe {
            let raw_window_handle = RawWindowHandle::Android(AndroidWindowHandle::new(
                NonNull::new(native_window).ok_or("Failed to create NonNull")?,
            ));
            let raw_display_handle = RawDisplayHandle::Android(AndroidDisplayHandle::new());
            
            instance.create_surface_unsafe(
                wgpu::SurfaceTargetUnsafe::from_handles(raw_display_handle, raw_window_handle)
            )?
        };
        
        // 3. Configurar surface
        let caps = surface.get_capabilities(&instance.adapter().expect("No adapter"));
        let format = caps.formats.iter()
            .find(|f| **f == TextureFormat::Bgra8UnormSrgb)
            .copied()
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);
        
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&instance.device().expect("No device"), &config);
        
        Ok(Self {
            surface: Some(surface),
            config: Some(config),
            native_window,
            width,
            height,
        })
    }

    fn get_native_window(
        jni_bridge: &JniBridge, 
        surface_view: jni::objects::JObject
    ) -> LarpResult<*mut std::ffi::c_void> {
        let env = jni_bridge.vm.get_env()?;
        
        // Determine if it's a SurfaceView or TextureView and call appropriate helper
        let class_name = env.get_object_class(surface_view)?.get_name()?;
        
        let native_window_ptr = if class_name.contains("SurfaceView") {
            // Call NativeWindowHelper.getNativeWindowFromSurfaceView
            let result = env.call_static_method(
                "com/larp/jv/NativeWindowHelper",
                "getNativeWindowFromSurfaceView",
                "(Landroid/view/SurfaceView;)J",
                &[jni::objects::JValue::Object(&surface_view)]
            )?;
            result.j()? as *mut std::ffi::c_void
        } else if class_name.contains("TextureView") {
            // Call NativeWindowHelper.getNativeWindowFromTextureView
            let result = env.call_static_method(
                "com/larp/jv/NativeWindowHelper",
                "getNativeWindowFromTextureView",
                "(Landroid/view/TextureView;)J",
                &[jni::objects::JValue::Object(&surface_view)]
            )?;
            result.j()? as *mut std::ffi::c_void
        } else {
            // Fallback: try generic Surface
            let surface = env.call_method(
                surface_view,
                "getSurface",
                "()Landroid/view/Surface;",
                &[]
            )?.l()?;
            
            let result = env.call_static_method(
                "com/larp/jv/NativeWindowHelper",
                "getNativeWindow",
                "(Landroid/view/Surface;)J",
                &[jni::objects::JValue::Object(&surface)]
            )?;
            result.j()? as *mut std::ffi::c_void
        };
        
        if native_window_ptr.is_null() {
            return Err(LarpError::Other("Failed to get native window from SurfaceView/TextureView".to_string()));
        }
        
        Ok(native_window_ptr)
    }

    pub fn resize(&mut self, instance: &Instance, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.config) {
            config.width = width;
            config.height = height;
            if let Some(device) = instance.device() {
                surface.configure(device, config);
            }
        }
    }

    pub fn get_surface(&self) -> Option<&Surface> {
        self.surface.as_ref()
    }

    pub fn get_config(&self) -> Option<&SurfaceConfiguration> {
        self.config.as_ref()
    }

    pub fn present(&self) -> LarpResult<()> {
        if let Some(surface) = &self.surface {
            let frame = surface.get_current_texture()
                .map_err(|e| LarpError::Other(format!("Failed to get frame: {:?}", e)))?;
            frame.present();
            Ok(())
        } else {
            Err(LarpError::Other("No surface".to_string()))
        }
    }
}

impl Drop for AndroidSurface {
    fn drop(&mut self) {
        // ANativeWindow_release - called via JNI helper
        if !self.native_window.is_null() {
            let _ = self.release_native_window();
        }
    }
}

impl AndroidSurface {
    fn release_native_window(&self) -> LarpResult<()> {
        if !self.native_window.is_null() {
            // Call Java helper to release
            // NativeWindowHelper.releaseNativeWindow(native_window_ptr)
            // For now, just log
            log::debug!("Releasing native window: {:p}", self.native_window);
        }
        Ok(())
    }
}

/// Surface provider trait for different view types
pub trait SurfaceProvider {
    fn get_surface_view(&self) -> jni::objects::JObject;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

/// SurfaceView implementation
pub struct SurfaceViewProvider {
    view: jni::objects::JObject,
    width: u32,
    height: u32,
}

impl SurfaceViewProvider {
    pub fn new(view: jni::objects::JObject, width: u32, height: u32) -> Self {
        Self { view, width, height }
    }
}

impl SurfaceProvider for SurfaceViewProvider {
    fn get_surface_view(&self) -> jni::objects::JObject {
        self.view.clone()
    }
    
    fn get_width(&self) -> u32 {
        self.width
    }
    
    fn get_height(&self) -> u32 {
        self.height
    }
}

/// TextureView implementation
pub struct TextureViewProvider {
    view: jni::objects::JObject,
    width: u32,
    height: u32,
}

impl TextureViewProvider {
    pub fn new(view: jni::objects::JObject, width: u32, height: u32) -> Self {
        Self { view, width, height }
    }
}

impl SurfaceProvider for TextureViewProvider {
    fn get_surface_view(&self) -> jni::objects::JObject {
        self.view.clone()
    }
    
    fn get_width(&self) -> u32 {
        self.width
    }
    
    fn get_height(&self) -> u32 {
        self.height
    }
}