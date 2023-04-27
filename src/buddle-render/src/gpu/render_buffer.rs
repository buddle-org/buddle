//! Batches and dispatches draw calls to the GPU

use wgpu::{BlendComponent, BlendFactor, BlendOperation};
use buddle_math::{Mat4};

use crate::camera::ModelMatrices;
use crate::gpu::{context::Context, Mesh, OIT_COMPOSITE, SCREEN};
use crate::{
    BindGroupLayoutEntry, Material, RenderTexture, SimplifiedPipelineConfig, Texture,
    TextureDimensions, MSAA,
};

pub(crate) struct DrawCall<'a> {
    mesh: &'a Mesh,
    material: &'a Box<dyn Material>,
    model_matrix: Mat4,
}

pub struct RenderBuffer<'a, 'b> {
    pub(crate) draw_calls: Vec<DrawCall<'a>>,
    camera_bind_group: &'b wgpu::BindGroup,
    view_mat: Mat4,
    proj_mat: Mat4,
}

impl<'a, 'b> RenderBuffer<'a, 'b> {
    pub fn new(camera_bind_group: &'b wgpu::BindGroup, view_mat: Mat4, proj_mat: Mat4) -> Self {
        RenderBuffer {
            draw_calls: Vec::new(),
            camera_bind_group,
            view_mat,
            proj_mat,
        }
    }

    pub fn add_draw_call(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Box<dyn Material>,
        model_matrix: Mat4,
    ) {
        self.draw_calls.push(DrawCall {
            mesh,
            material,
            model_matrix,
        });
    }

    pub fn render_to_texture(&self, ctx: &Context, texture: &RenderTexture) {
        self.render_to_view(ctx, &texture.texture.view, &texture.depth.view)
    }

    fn draw_to_pass<'c>(&self, ctx: &Context, mut render_pass: wgpu::RenderPass<'c>)
    where
        'b: 'c,
        'a: 'c,
    {
        for draw_call in &self.draw_calls {
            if !draw_call.material.has_opaque_pixels() {
                continue;
            }

            ctx.update_buffer(
                &draw_call.mesh.model_buffer,
                &[ModelMatrices::new(
                    self.view_mat,
                    self.proj_mat,
                    draw_call.model_matrix,
                )],
            );

            render_pass.set_pipeline(&draw_call.material.get_shader().pipeline);

            render_pass.set_bind_group(0, self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &draw_call.mesh.model_bind_group, &[]);
            render_pass.set_bind_group(2, &draw_call.material.get_bind_group(), &[]);

            render_pass.set_vertex_buffer(0, draw_call.mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                draw_call.mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..draw_call.mesh.num_triangles, 0, 0..1);
        }
    }

    fn draw_to_pass_oit<'c>(&self, ctx: &Context, mut render_pass: wgpu::RenderPass<'c>)
    where
        'b: 'c,
        'a: 'c,
    {
        for draw_call in &self.draw_calls {
            if !draw_call.material.has_transparent_pixels() {
                continue;
            }

            ctx.update_buffer(
                &draw_call.mesh.model_buffer,
                &[ModelMatrices::new(
                    self.view_mat,
                    self.proj_mat,
                    draw_call.model_matrix,
                )],
            );

            render_pass.set_pipeline(&draw_call.material.get_transparent_shader().pipeline);

            render_pass.set_bind_group(0, self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &draw_call.mesh.model_bind_group, &[]);
            render_pass.set_bind_group(2, &draw_call.material.get_bind_group(), &[]);

            render_pass.set_vertex_buffer(0, draw_call.mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                draw_call.mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..draw_call.mesh.num_triangles, 0, 0..1);
        }
    }

    fn render_to_view(&self, ctx: &Context, view: &wgpu::TextureView, depth: &wgpu::TextureView) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.draw_to_pass(ctx, render_pass);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    fn render_to_view_oit(
        &self,
        ctx: &Context,
        accum: &wgpu::TextureView,
        reveal: &wgpu::TextureView,
        depth: &wgpu::TextureView,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: accum,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: true,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: reveal,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: None,
                    stencil_ops: None,
                }),
            });

            self.draw_to_pass_oit(ctx, render_pass);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    fn render_oit_composite(
        &self,
        ctx: &Context,
        target: &wgpu::TextureView,
        opaque: &Texture,
        accum: &Texture,
        reveal: &Texture,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        let plane = Mesh::make_screen_plane(ctx);
        let bgl = ctx.create_bind_group_layout(vec![
            BindGroupLayoutEntry::Texture{dim: TextureDimensions::D2, filtering: true},
            BindGroupLayoutEntry::Sampler{filtering: true},
        ]);

        let composite_shader = ctx.create_shader(
            OIT_COMPOSITE,
            vec![&bgl, &bgl],
            SimplifiedPipelineConfig {
                wireframe: false,
                msaa: MSAA::Off,
                targets: vec![wgpu::ColorTargetState {
                    format: ctx.surface.config.format,
                    blend: Some(wgpu::BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        }
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
                depth_settings: None,
            },
        );

        let screen_shader = ctx.create_shader(
            SCREEN,
            vec![&bgl],
            SimplifiedPipelineConfig {
                wireframe: false,
                msaa: MSAA::Off,
                targets: vec![wgpu::ColorTargetState {
                    format: ctx.surface.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
                depth_settings: None,
            },
        );

        let opaque_bg = ctx.create_bind_group(
            &bgl,
            vec![
                wgpu::BindingResource::TextureView(&opaque.view),
                wgpu::BindingResource::Sampler(&opaque.sampler),
            ],
        );

        let accum_bg = ctx.create_bind_group(
            &bgl,
            vec![
                wgpu::BindingResource::TextureView(&accum.view),
                wgpu::BindingResource::Sampler(&accum.sampler),
            ],
        );

        let reveal_bg = ctx.create_bind_group(
            &bgl,
            vec![
                wgpu::BindingResource::TextureView(&reveal.view),
                wgpu::BindingResource::Sampler(&reveal.sampler),
            ],
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // Copy opaque to screen

            render_pass.set_pipeline(&screen_shader.pipeline);
            render_pass.set_bind_group(0, &opaque_bg, &[]);
            render_pass.set_vertex_buffer(0, plane.vertex_buffer.slice(..));
            render_pass.set_index_buffer(plane.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..plane.num_triangles, 0, 0..1);

            // Composite transparent to screen

            render_pass.set_pipeline(&composite_shader.pipeline);
            render_pass.set_bind_group(0, &accum_bg, &[]);
            render_pass.set_bind_group(1, &reveal_bg, &[]);
            render_pass.set_vertex_buffer(0, plane.vertex_buffer.slice(..));
            render_pass.set_index_buffer(plane.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..plane.num_triangles, 0, 0..1);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    fn get_output(&self, ctx: &Context) -> Result<wgpu::SurfaceTexture, ()> {
        let output_res = ctx.surface.surface.get_current_texture();

        if let Ok(tex) = output_res {
            Ok(tex)
        } else {
            match output_res {
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => ctx.reconfigure(),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => return Err(()),
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
                _ => unreachable!(),
            }
            self.get_output(ctx)
        }
    }

    pub fn submit(self, ctx: &Context) -> Result<(), ()> {
        let output = self.get_output(ctx)?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_to_view(ctx, &ctx.oit_opaque.view, &ctx.depth_buffer.view);
        self.render_to_view_oit(
            ctx,
            &ctx.oit_accum.view,
            &ctx.oit_reveal.view,
            &ctx.depth_buffer.view,
        );

        self.render_oit_composite(ctx, &view, &ctx.oit_opaque, &ctx.oit_accum, &ctx.oit_reveal);

        output.present();

        Ok(())
    }
}
