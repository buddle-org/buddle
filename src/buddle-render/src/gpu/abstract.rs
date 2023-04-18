//! Type abstractions

use buddle_math::UVec2;

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
    pub(crate) size: UVec2,
}

pub struct Mesh {
    pub num_triangles: u32,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) model_buffer: wgpu::Buffer,
    pub(crate) model_bind_group: wgpu::BindGroup,
}
