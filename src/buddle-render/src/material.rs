use std::rc::Rc;

use crate::gpu::{FLAT_TEXTURE, OIT_FLAT_TEXTURE};
use crate::{
    BindGroupLayoutEntry, Context, DepthSettings, Shader, SimplifiedPipelineConfig, Texture,
    TextureDimensions, MSAA,
};

pub trait Material {
    fn get_shader(&self) -> &Rc<Shader>;
    fn get_transparent_shader(&self) -> &Rc<Shader>;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
    fn has_transparent_pixels(&self) -> bool;
    fn has_opaque_pixels(&self) -> bool;
}

pub struct FlatMaterial {
    shader: Rc<Shader>,
    transparent_shader: Rc<Shader>,
    transparent: bool,
    opaque: bool,
    bind_group: wgpu::BindGroup,
}

impl FlatMaterial {
    pub fn new(ctx: &Context, diffuse: &Texture, transparent: bool, opaque: bool) -> Self {
        let config = SimplifiedPipelineConfig {
            wireframe: false,
            msaa: MSAA::Off,
            targets: vec![wgpu::ColorTargetState {
                format: ctx.surface.config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            }],
            depth_settings: Some(DepthSettings{compare: wgpu::CompareFunction::Less, write: true}),
        };

        let transparent_config = SimplifiedPipelineConfig {
            wireframe: false,
            msaa: MSAA::Off,
            targets: vec![
                wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                },
                wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::Zero,
                            dst_factor: wgpu::BlendFactor::OneMinusSrc,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                },
            ],
            depth_settings: Some(DepthSettings {
                compare: wgpu::CompareFunction::Less,
                write: false,
            }),
        };

        let buffer_bind_gl = ctx.create_bind_group_layout(vec![BindGroupLayoutEntry::Buffer]);
        let texture_gl = ctx.create_bind_group_layout(vec![
            BindGroupLayoutEntry::Texture {
                dim: TextureDimensions::D2,
                filtering: true,
            },
            BindGroupLayoutEntry::Sampler { filtering: true },
        ]);

        let shader = ctx.create_shader(
            FLAT_TEXTURE,
            vec![&buffer_bind_gl, &buffer_bind_gl, &texture_gl],
            config,
        );

        let transparent_shader = ctx.create_shader(
            OIT_FLAT_TEXTURE,
            vec![&buffer_bind_gl, &buffer_bind_gl, &texture_gl],
            transparent_config,
        );

        let bind_group = ctx.create_bind_group(
            &texture_gl,
            vec![
                wgpu::BindingResource::TextureView(&diffuse.view),
                wgpu::BindingResource::Sampler(&diffuse.sampler),
            ],
        );

        FlatMaterial {
            shader,
            transparent_shader,
            bind_group,
            transparent,
            opaque,
        }
    }
}

impl Material for FlatMaterial {
    fn get_shader(&self) -> &Rc<Shader> {
        &self.shader
    }

    fn get_transparent_shader(&self) -> &Rc<Shader> {
        &self.transparent_shader
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn has_transparent_pixels(&self) -> bool {
        self.transparent
    }

    fn has_opaque_pixels(&self) -> bool {
        self.opaque
    }
}
