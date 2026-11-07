//! GLES Render Spec — port of GLESRenderSpec.java

use wgpu;

pub struct GlesRenderSpec;

impl GlesRenderSpec {
    pub fn new() -> Self {
        Self
    }

    pub fn get_limits(&self) -> wgpu::Limits {
        wgpu::Limits {
            max_texture_dimension_2d: 8192,
            max_texture_dimension_3d: 2048,
            max_buffer_size: 256 * 1024 * 1024,
            max_bind_groups: 4,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 4,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 4,
            max_storage_textures_per_shader_stage: 4,
            max_uniform_buffers_per_shader_stage: 12,
            max_uniform_buffer_binding_size: 64 * 1024,
            max_storage_buffer_binding_size: 128 * 1024 * 1024,
            max_vertex_buffers: 8,
            max_vertex_attributes: 16,
            max_vertex_attribute_offset: 2047,
            max_inter_stage_shader_components: 60,
            max_compute_workgroup_storage_size: 16384,
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_workgroups_per_dimension: 65535,
            ..Default::default()
        }
    }

    pub fn get_features(&self) -> wgpu::Features {
        wgpu::Features::empty()
    }
}

impl Default for GlesRenderSpec {
    fn default() -> Self {
        Self::new()
    }
}