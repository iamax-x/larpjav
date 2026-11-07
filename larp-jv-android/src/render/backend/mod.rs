//! Render Platform Abstraction — port of Platform.java + backends

use wgpu;
use std::sync::Arc;

pub trait RenderBackend: Send + Sync {
    fn initialize(&mut self) -> crate::error::LarpResult<()>;
    fn get_surface(&self) -> Option<&wgpu::Surface>;
    fn get_adapter(&self) -> Option<&wgpu::Adapter>;
    fn get_device(&self) -> Option<&wgpu::Device>;
    fn get_queue(&self) -> Option<&wgpu::Queue>;
}

/// AWT Backend (headless/software)
pub struct AwtBackend {
    surface: Option<wgpu::Surface<'static>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

impl AwtBackend {
    pub fn new() -> Self {
        Self {
            surface: None,
            adapter: None,
            device: None,
            queue: None,
        }
    }
}

impl Default for AwtBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for AwtBackend {
    fn initialize(&mut self) -> crate::error::LarpResult<()> {
        Ok(())
    }
    fn get_surface(&self) -> Option<&wgpu::Surface> {
        self.surface.as_ref()
    }
    fn get_adapter(&self) -> Option<&wgpu::Adapter> {
        self.adapter.as_ref()
    }
    fn get_device(&self) -> Option<&wgpu::Device> {
        self.device.as_ref()
    }
    fn get_queue(&self) -> Option<&wgpu::Queue> {
        self.queue.as_ref()
    }
}

/// GLFW Backend (desktop)
pub struct GlfwBackend {
    surface: Option<wgpu::Surface<'static>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

impl GlfwBackend {
    pub fn new() -> Self {
        Self {
            surface: None,
            adapter: None,
            device: None,
            queue: None,
        }
    }
}

impl Default for GlfwBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for GlfwBackend {
    fn initialize(&mut self) -> crate::error::LarpResult<()> {
        Ok(())
    }
    fn get_surface(&self) -> Option<&wgpu::Surface> {
        self.surface.as_ref()
    }
    fn get_adapter(&self) -> Option<&wgpu::Adapter> {
        self.adapter.as_ref()
    }
    fn get_device(&self) -> Option<&wgpu::Device> {
        self.device.as_ref()
    }
    fn get_queue(&self) -> Option<&wgpu::Queue> {
        self.queue.as_ref()
    }
}

/// SDL Backend (Android/desktop)
pub struct SdlBackend {
    surface: Option<wgpu::Surface<'static>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

impl SdlBackend {
    pub fn new() -> Self {
        Self {
            surface: None,
            adapter: None,
            device: None,
            queue: None,
        }
    }
}

impl Default for SdlBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for SdlBackend {
    fn initialize(&mut self) -> crate::error::LarpResult<()> {
        Ok(())
    }
    fn get_surface(&self) -> Option<&wgpu::Surface> {
        self.surface.as_ref()
    }
    fn get_adapter(&self) -> Option<&wgpu::Adapter> {
        self.adapter.as_ref()
    }
    fn get_device(&self) -> Option<&wgpu::Device> {
        self.device.as_ref()
    }
    fn get_queue(&self) -> Option<&wgpu::Queue> {
        self.queue.as_ref()
    }
}

/// Dummy Backend (headless/testing)
pub struct DummyBackend;

impl DummyBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DummyBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for DummyBackend {
    fn initialize(&mut self) -> crate::error::LarpResult<()> {
        Ok(())
    }
    fn get_surface(&self) -> Option<&wgpu::Surface> { None }
    fn get_adapter(&self) -> Option<&wgpu::Adapter> { None }
    fn get_device(&self) -> Option<&wgpu::Device> { None }
    fn get_queue(&self) -> Option<&wgpu::Queue> { None }
}

/// Platform abstraction
pub struct Platform {
    pub backend: Box<dyn RenderBackend>,
}

impl Platform {
    pub fn new(backend: Box<dyn RenderBackend>) -> Self {
        Self { backend }
    }
}