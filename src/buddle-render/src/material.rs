use crate::{BindGroupLayoutEntry, Context, Shader, Texture};

pub trait Material {
    fn get_shader(&self) -> &Shader;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

pub struct FlatMaterial {
    pub(crate) shader: Shader,
    pub(crate) diffuse: Texture,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl FlatMaterial {
    pub fn new(ctx: &Context, shader: Shader, diffuse: Texture) -> Self {
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

        FlatMaterial {
            shader,
            diffuse,
            bind_group,
        }
    }
}

impl Material for FlatMaterial {
    fn get_shader(&self) -> &Shader {
        &self.shader
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
