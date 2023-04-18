use std::rc::Rc;

use crate::gpu::FLAT_TEXTURE;
use crate::{BindGroupLayoutEntry, Context, Shader, Texture, TextureDimensions};

pub trait Material {
    fn get_shader(&self) -> &Rc<Shader>;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

pub struct FlatMaterial {
    pub(crate) shader: Rc<Shader>,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl FlatMaterial {
    pub fn new(ctx: &Context, diffuse: &Texture) -> Self {
        let shader = ctx.create_shader(
            FLAT_TEXTURE,
            vec![
                &ctx.create_bind_group_layout(vec![BindGroupLayoutEntry::Buffer]),
                &ctx.create_bind_group_layout(vec![BindGroupLayoutEntry::Buffer]),
                &ctx.create_bind_group_layout(vec![
                    BindGroupLayoutEntry::Texture(TextureDimensions::D2),
                    BindGroupLayoutEntry::Sampler,
                ]),
            ],
        );

        let bind_group = ctx.create_bind_group(
            ctx.create_bind_group_layout(vec![
                BindGroupLayoutEntry::Texture(diffuse.dimensions),
                BindGroupLayoutEntry::Sampler,
            ]),
            vec![
                wgpu::BindingResource::TextureView(&diffuse.view),
                wgpu::BindingResource::Sampler(&diffuse.sampler),
            ],
        );

        FlatMaterial { shader, bind_group }
    }
}

impl Material for FlatMaterial {
    fn get_shader(&self) -> &Rc<Shader> {
        &self.shader
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
