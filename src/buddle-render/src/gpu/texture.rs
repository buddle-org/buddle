use crate::{Context, TextureDimensions};
use buddle_math::UVec2;

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) dimensions: TextureDimensions,
    pub(crate) size: UVec2,
}

impl Texture {
    pub fn missing(ctx: &Context) -> Self {
        ctx.create_texture(
            &[
                0, 0, 0, 255, 255, 0, 220, 255, 255, 0, 220, 255, 0, 0, 0, 255,
            ],
            UVec2::new(2, 2),
        )
    }
}
