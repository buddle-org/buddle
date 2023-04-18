//! Batches and dispatches draw calls to the GPU

use wgpu::TextureView;

use buddle_math::{Mat4, Vec4};

use crate::camera::ModelMatrices;
use crate::gpu::{context::Context, Mesh};
use crate::{Material, RenderTexture};

pub(crate) struct DrawCall<'a> {
    mesh: &'a Mesh,
    material: &'a Box<dyn Material>,
    model_matrix: Mat4,
}

pub struct RenderBuffer<'a, 'b> {
    pub(crate) draw_calls: Vec<DrawCall<'a>>,
    pub(crate) clear_color: Vec4,
    camera_bind_group: &'b wgpu::BindGroup,
    view_mat: Mat4,
    proj_mat: Mat4,
}

impl<'a, 'b> RenderBuffer<'a, 'b> {
    pub fn new(camera_bind_group: &'b wgpu::BindGroup, view_mat: Mat4, proj_mat: Mat4) -> Self {
        RenderBuffer {
            draw_calls: Vec::new(),
            clear_color: Vec4::ZERO,
            camera_bind_group,
            view_mat,
            proj_mat,
        }
    }

    pub fn set_clear_color(&mut self, color: Vec4) {
        self.clear_color = color;
    }

    // The mutable Mesh borrow is important, because Buffers can only be updated at
    // the beginning of a submit call, meaning that we can only draw each mesh
    // once (having all the transform matrices)
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
        self.render_to_texture_view(ctx, &texture.texture.view, &texture.depth.view)
    }

    fn render_to_texture_view(&self, ctx: &Context, view: &TextureView, depth: &TextureView) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color.x as f64,
                            g: self.clear_color.y as f64,
                            b: self.clear_color.z as f64,
                            a: self.clear_color.w as f64,
                        }),
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

            for draw_call in &self.draw_calls {
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

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn submit(self, ctx: &Context) -> Result<(), ()> {
        let output_res = ctx.surface.surface.get_current_texture();

        if output_res.is_err() {
            match output_res {
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => ctx.reconfigure(),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => return Err(()),
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
                _ => unreachable!(),
            }
            return self.submit(ctx);
        }

        let output = output_res.unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_to_texture_view(ctx, &view, &ctx.depth_buffer.view);

        output.present();

        Ok(())
    }
}
