//! GPU utilities

use crate::error::LarpResult;
use std::collections::HashMap;

/// GPU vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    Arm,
    Broadcom,
    Imagination,
    Vivante,
    VeriSilicon,
    Unknown,
}

/// GPU info
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub renderer: String,
    pub version: String,
    pub device_name: Option<String>,
    pub driver_version: Option<String>,
    pub api_version: Option<String>,
    pub max_texture_size: Option<u32>,
    pub max_vertex_attributes: Option<u32>,
    pub extensions: Vec<String>,
    pub limits: HashMap<String, u64>,
}

impl GpuInfo {
    /// Detect GPU info from wgpu
    #[cfg(feature = "wgpu")]
    pub fn from_wgpu_adapter(adapter: &wgpu::Adapter) -> Self {
        let info = adapter.get_info();
        let features = adapter.features();
        let limits = adapter.limits();

        let vendor = match info.vendor {
            0x10DE => GpuVendor::Nvidia,
            0x1002 => GpuVendor::Amd,
            0x8086 => GpuVendor::Intel,
            0x106B => GpuVendor::Apple,
            0x5143 => GpuVendor::Qualcomm,
            0x13B5 => GpuVendor::Arm,
            0x14E4 => GpuVendor::Broadcom,
            0x1010 => GpuVendor::Imagination,
            0x1000 => GpuVendor::Vivante,
            _ => GpuVendor::Unknown,
        };

        let mut ext = Vec::new();
        if features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) { ext.push("texture_compression_bc".into()); }
        if features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) { ext.push("texture_compression_etc2".into()); }
        if features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC) { ext.push("texture_compression_astc".into()); }
        if features.contains(wgpu::Features::ANISOTROPIC_FILTERING) { ext.push("anisotropic_filtering".into()); }
        if features.contains(wgpu::Features::MULTI_DRAW_INDIRECT) { ext.push("multi_draw_indirect".into()); }
        if features.contains(wgpu::Features::TEXTURE_BINDING_ARRAY) { ext.push("texture_binding_array".into()); }
        if features.contains(wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY) { ext.push("storage_resource_binding_array".into()); }
        if features.contains(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY) { ext.push("sampled_texture_and_storage_buffer_array".into()); }

        let mut lim = HashMap::new();
        lim.insert("max_texture_size_2d".into(), limits.max_texture_dimension_2d as u64);
        lim.insert("max_texture_size_3d".into(), limits.max_texture_dimension_3d as u64);
        lim.insert("max_buffer_size".into(), limits.max_buffer_size as u64);
        lim.insert("max_bind_groups".into(), limits.max_bind_groups as u64);
        lim.insert("max_uniform_buffer_size".into(), limits.max_uniform_buffer_binding_size as u64);
        lim.insert("max_storage_buffer_size".into(), limits.max_storage_buffer_binding_size as u64);

        Self {
            vendor,
            renderer: info.name,
            version: info.driver,
            device_name: Some(info.name.clone()),
            driver_version: Some(info.driver.clone()),
            api_version: Some(info.backend.to_string()),
            max_texture_size: Some(limits.max_texture_dimension_2d),
            max_vertex_attributes: Some(limits.max_vertex_attributes),
            extensions: ext,
            limits: lim,
        }
    }

    /// Check if extension is supported
    pub fn has_extension(&self, ext: &str) -> bool {
        self.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
    }

    /// Get vendor-specific workarounds
    pub fn workarounds(&self) -> Vec<GpuWorkaround> {
        let mut workarounds = Vec::new();
        match self.vendor {
            GpuVendor::Nvidia => {
                workarounds.push(GpuWorkaround::PreferVulkan);
                workarounds.push(GpuWorkaround::AvoidPushConstants);
            }
            GpuVendor::Amd => {
                workarounds.push(GpuWorkaround::PreferVulkan);
            }
            GpuVendor::Intel => {
                workarounds.push(GpuWorkaround::AvoidLargeUniformBuffers);
            }
            GpuVendor::Apple => {
                workarounds.push(GpuWorkaround::PreferMetal);
                workarounds.push(GpuWorkaround::AvoidComputeShaders);
            }
            GpuVendor::Qualcomm | GpuVendor::Arm => {
                workarounds.push(GpuWorkaround::PreferGles);
                workarounds.push(GpuWorkaround::AvoidLargeTextures);
            }
            _ => {}
        }
        workarounds
    }
}

/// GPU workarounds for specific vendors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuWorkaround {
    PreferVulkan,
    PreferMetal,
    PreferGles,
    AvoidPushConstants,
    AvoidLargeUniformBuffers,
    AvoidLargeTextures,
    AvoidComputeShaders,
    ForceSingleThreaded,
}

/// Detect if running on integrated graphics
pub fn is_integrated_gpu(info: &GpuInfo) -> bool {
    matches!(info.vendor, GpuVendor::Intel | GpuVendor::Apple | GpuVendor::Qualcomm | GpuVendor::Arm)
}

/// Get recommended backend for platform
pub fn recommended_backend() -> wgpu::Backends {
    #[cfg(target_os = "windows")]
    return wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
    #[cfg(target_os = "linux")]
    return wgpu::Backends::VULKAN;
    #[cfg(target_os = "android")]
    return wgpu::Backends::VULKAN | wgpu::Backends::GLES;
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux", target_os = "android")))]
    return wgpu::Backends::all();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_vendor_detection() {
        // Just test enum values
        assert_eq!(GpuVendor::Nvidia as u32, 0);
    }
}
