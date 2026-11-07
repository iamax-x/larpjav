//! Game Rendering Pipeline — Port of GameRenderer, RenderSpec, SurfaceProvider

use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat, PresentMode, CompositeAlphaMode};
use std::sync::Arc;
use crate::value::RenderBackendType;

/// Render backend specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    GLES,
    Mesa,
    Vulkan,
    AWT,
    Dummy,
}

/// Render specification trait
pub trait RenderSpec: Send + Sync {
    fn backend_type(&self) -> RenderBackend;
    fn initialize(&mut self, device: &Device, queue: &Queue) -> crate::error::LarpResult<()>;
    fn render(&mut self, surface: &Surface, config: &SurfaceConfiguration) -> crate::error::LarpResult<()>;
    fn resize(&mut self, width: u32, height: u32);
    fn get_supported_formats(&self) -> Vec<TextureFormat>;
}

/// GLES Render Spec — Port of GLESRenderSpec.java
pub struct GLESRenderSpec {
    initialized: bool,
    shader_program: Option<u32>, // Would be wgpu::RenderPipeline in real impl
}

impl GLESRenderSpec {
    pub fn new() -> Self {
        Self {
            initialized: false,
            shader_program: None,
        }
    }
}

impl Default for GLESRenderSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderSpec for GLESRenderSpec {
    fn backend_type(&self) -> RenderBackend {
        RenderBackend::GLES
    }

    fn initialize(&mut self, device: &Device, queue: &Queue) -> crate::error::LarpResult<()> {
        // Create render pipeline for GLES
        // Port from GLESRenderSpec.java initialize()
        self.initialized = true;
        log::info!("GLESRenderSpec initialized");
        Ok(())
    }

    fn render(&mut self, surface: &Surface, config: &SurfaceConfiguration) -> crate::error::LarpResult<()> {
        if !self.initialized {
            return Err(crate::error::LarpError::Other("Not initialized".to_string()));
        }
        // Render frame - port from GLESRenderSpec.java render()
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        // Handle resize
    }

    fn get_supported_formats(&self) -> Vec<TextureFormat> {
        vec![
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgba8UnormSrgb,
        ]
    }
}

/// Mesa Render Spec — Port of MesaRenderSpec.java
pub struct MesaRenderSpec {
    initialized: bool,
}

impl MesaRenderSpec {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Default for MesaRenderSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderSpec for MesaRenderSpec {
    fn backend_type(&self) -> RenderBackend {
        RenderBackend::Mesa
    }

    fn initialize(&mut self, device: &Device, queue: &Queue) -> crate::error::LarpResult<()> {
        self.initialized = true;
        log::info!("MesaRenderSpec initialized");
        Ok(())
    }

    fn render(&mut self, surface: &Surface, config: &SurfaceConfiguration) -> crate::error::LarpResult<()> {
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {}

    fn get_supported_formats(&self) -> Vec<TextureFormat> {
        vec![TextureFormat::Bgra8UnormSrgb]
    }
}

/// Renderer Cache — Port of RendererCache.java
pub struct RendererCache {
    specs: Vec<Box<dyn RenderSpec>>,
    active_spec: Option<usize>,
}

impl RendererCache {
    pub fn new() -> Self {
        Self {
            specs: Vec::new(),
            active_spec: None,
        }
    }

    pub fn register_spec(&mut self, spec: Box<dyn RenderSpec>) {
        self.specs.push(spec);
    }

    pub fn set_active(&mut self, index: usize) -> crate::error::LarpResult<()> {
        if index < self.specs.len() {
            self.active_spec = Some(index);
            Ok(())
        } else {
            Err(crate::error::LarpError::Other("Invalid spec index".to_string()))
        }
    }

    pub fn get_active(&mut self) -> Option<&mut dyn RenderSpec> {
        self.active_spec.and_then(|i| self.specs.get_mut(i)).map(|b| b.as_mut())
    }

    pub fn list_specs(&self) -> Vec<RenderBackend> {
        self.specs.iter().map(|s| s.backend_type()).collect()
    }
}

impl Default for RendererCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Surface Provider Trait — Port of SurfaceProvider.java
pub trait SurfaceProvider: Send + Sync {
    fn create_surface(&self, instance: &wgpu::Instance, width: u32, height: u32) -> crate::error::LarpResult<Surface<'static>>;
    fn get_current_texture(&self) -> crate::error::LarpResult<wgpu::SurfaceTexture>;
    fn present(&self) -> crate::error::LarpResult<()>;
    fn resize(&mut self, width: u32, height: u32);
}

/// SurfaceView Surface Provider — Port of SurfaceViewSurfaceProvider.java
pub struct SurfaceViewSurfaceProvider {
    surface: Option<Surface<'static>>,
    config: Option<SurfaceConfiguration>,
    width: u32,
    height: u32,
}

impl SurfaceViewSurfaceProvider {
    pub fn new() -> Self {
        Self {
            surface: None,
            config: None,
            width: 0,
            height: 0,
        }
    }
}

impl Default for SurfaceViewSurfaceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceProvider for SurfaceViewSurfaceProvider {
    fn create_surface(&self, _instance: &wgpu::Instance, _width: u32, _height: u32) -> crate::error::LarpResult<Surface<'static>> {
        // Implemented in Android-specific code via ANativeWindow
        Err(crate::error::LarpError::Other("Use AndroidSurface".to_string()))
    }

    fn get_current_texture(&self) -> crate::error::LarpResult<wgpu::SurfaceTexture> {
        self.surface.as_ref()
            .ok_or_else(|| crate::error::LarpError::Other("No surface".to_string()))?
            .get_current_texture()
            .map_err(|e| crate::error::LarpError::Other(format!("Failed to get texture: {:?}", e)))
    }

    fn present(&self) -> crate::error::LarpResult<()> {
        if let Some(surface) = &self.surface {
            let frame = surface.get_current_texture()
                .map_err(|e| crate::error::LarpError::Other(format!("Failed to get frame: {:?}", e)))?;
            frame.present();
            Ok(())
        } else {
            Err(crate::error::LarpError::Other("No surface".to_string()))
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.config) {
            config.width = width;
            config.height = height;
            // Reconfigure would need device
        }
    }
}

/// TextureView Surface Provider — Port of TextureViewSurfaceProvider.java
pub struct TextureViewSurfaceProvider {
    surface: Option<Surface<'static>>,
    config: Option<SurfaceConfiguration>,
    width: u32,
    height: u32,
}

impl TextureViewSurfaceProvider {
    pub fn new() -> Self {
        Self {
            surface: None,
            config: None,
            width: 0,
            height: 0,
        }
    }
}

impl Default for TextureViewSurfaceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceProvider for TextureViewSurfaceProvider {
    fn create_surface(&self, _instance: &wgpu::Instance, _width: u32, _height: u32) -> crate::error::LarpResult<Surface<'static>> {
        Err(crate::error::LarpError::Other("Use AndroidSurface".to_string()))
    }

    fn get_current_texture(&self) -> crate::error::LarpResult<wgpu::SurfaceTexture> {
        self.surface.as_ref()
            .ok_or_else(|| crate::error::LarpError::Other("No surface".to_string()))?
            .get_current_texture()
            .map_err(|e| crate::error::LarpError::Other(format!("Failed to get texture: {:?}", e)))
    }

    fn present(&self) -> crate::error::LarpResult<()> {
        if let Some(surface) = &self.surface {
            let frame = surface.get_current_texture()
                .map_err(|e| crate::error::LarpError::Other(format!("Failed to get frame: {:?}", e)))?;
            frame.present();
            Ok(())
        } else {
            Err(crate::error::LarpError::Other("No surface".to_string()))
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.config) {
            config.width = width;
            config.height = height;
        }
    }
}

/// Game Renderer — Port of GameRenderer.java
pub struct GameRenderer {
    render_spec: Option<Box<dyn RenderSpec>>,
    surface_provider: Option<Box<dyn SurfaceProvider>>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    width: u32,
    height: u32,
    running: bool,
}

impl GameRenderer {
    pub fn new() -> Self {
        Self {
            render_spec: None,
            surface_provider: None,
            device: None,
            queue: None,
            width: 0,
            height: 0,
            running: false,
        }
    }

    pub fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        render_spec: Box<dyn RenderSpec>,
        surface_provider: Box<dyn SurfaceProvider>,
        width: u32,
        height: u32,
    ) -> crate::error::LarpResult<()> {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());
        self.width = width;
        self.height = height;

        render_spec.initialize(&device, &queue)?;
        self.render_spec = Some(render_spec);
        self.surface_provider = Some(surface_provider);
        self.running = true;

        log::info!("GameRenderer initialized: {}x{}", width, height);
        Ok(())
    }

    pub fn render_frame(&mut self) -> crate::error::LarpResult<()> {
        if let (Some(spec), Some(provider)) = (&mut self.render_spec, &mut self.surface_provider) {
            spec.render(
                provider.get_current_texture()?.surface(),
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: TextureFormat::Bgra8UnormSrgb,
                    width: self.width,
                    height: self.height,
                    present_mode: PresentMode::Fifo,
                    alpha_mode: CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                }
            )?;
            provider.present()?;
        }
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let Some(spec) = &mut self.render_spec {
            spec.resize(width, height);
        }
        if let Some(provider) = &mut self.surface_provider {
            provider.resize(width, height);
        }
    }

    pub fn set_render_spec(&mut self, spec: Box<dyn RenderSpec>) -> crate::error::LarpResult<()> {
        if let Some(device) = &self.device {
            if let Some(queue) = &self.queue {
                spec.initialize(device, queue)?;
                self.render_spec = Some(spec);
            }
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

impl Default for GameRenderer {
    fn default() -> Self {
        Self::new()
    }
}