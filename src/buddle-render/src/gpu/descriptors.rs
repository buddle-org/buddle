//! Describing what we want and have to the GPU

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MSAA {
    Off,
    On(u32),
}

#[derive(Copy, Clone, PartialEq)]
pub struct SimplifiedPipelineConfig {
    pub wireframe: bool,
    pub msaa: MSAA,
}

/// See docs for [`wgpu::TextureViewDimension`]
#[derive(Copy, Clone)]
pub enum TextureDimensions {
    D1,
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

impl Into<wgpu::TextureViewDimension> for &TextureDimensions {
    fn into(self) -> wgpu::TextureViewDimension {
        match self {
            TextureDimensions::D1 => wgpu::TextureViewDimension::D1,
            TextureDimensions::D2 => wgpu::TextureViewDimension::D2,
            TextureDimensions::D2Array => wgpu::TextureViewDimension::D2Array,
            TextureDimensions::Cube => wgpu::TextureViewDimension::Cube,
            TextureDimensions::CubeArray => wgpu::TextureViewDimension::CubeArray,
            TextureDimensions::D3 => wgpu::TextureViewDimension::D3,
        }
    }
}

pub enum BindGroupLayoutEntry {
    Buffer,
    Sampler,
    Texture(TextureDimensions),
}
