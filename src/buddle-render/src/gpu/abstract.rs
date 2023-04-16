//! Type abstractions

use cgmath::Vector2;

use crate::TextureDimensions;

pub struct Surface {
    pub(crate) surface: wgpu::Surface,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl Surface {
    pub fn configure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.config);
    }
}

pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) dimensions: TextureDimensions,
    pub(crate) size: Vector2<u32>,
}

pub struct Mesh {
    pub num_triangles: u32,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) model_buffer: wgpu::Buffer,
    pub(crate) model_bind_group: wgpu::BindGroup,
}

pub struct Material {
    pub(crate) shader: Shader,
    pub(crate) diffuse: Texture,
    pub(crate) bind_group: wgpu::BindGroup,
}
