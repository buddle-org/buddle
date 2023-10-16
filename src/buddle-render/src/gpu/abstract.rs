//! Type abstractions

use buddle_math::{Vec2, Vec3};

use crate::{Context, Texture, Vertex};

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

pub struct RenderTexture {
    pub texture: Texture,
    pub depth: Texture,
}

pub struct Mesh {
    pub num_triangles: u32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) model_buffer: wgpu::Buffer,
    pub(crate) model_bind_group: wgpu::BindGroup,
}

impl Mesh {
    pub fn make_plane(ctx: &Context) -> Self {
        let vertices = vec![
            Vertex::new(
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(1.0, 0.5, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, 0.0, 1.0),
                Vec3::new(0.0, 0.5, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, 0.0, -1.0),
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vec3::new(1.0, 0.0, -1.0),
                Vec3::new(1.0, 0.5, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec2::new(1.0, 0.0),
            ),
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];

        ctx.create_mesh(vertices, indices)
    }

    pub fn make_screen_plane(ctx: &Context) -> Self {
        let vertices = vec![
            Vertex::new(
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(1.0, 0.5, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, 1.0, 0.0),
                Vec3::new(0.0, 0.5, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(0.0, 1.0),
            ),
            Vertex::new(
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(1.0, 0.5, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
        ];
        let indices = vec![2, 0, 3, 0, 2, 1];

        ctx.create_mesh(vertices, indices)
    }

    pub fn make_box(ctx: &Context) -> Self {
        let vertices = vec![
            Vertex::new(
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(1.0, 0.5, 1.0),
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, 1.0, 1.0),
                Vec3::new(0.0, 0.5, 1.0),
                Vec3::new(-1.0, 1.0, 1.0),
                Vec2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, 1.0, -1.0),
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::new(-1.0, 1.0, -1.0),
                Vec2::new(0.0, 1.0),
            ),
            Vertex::new(
                Vec3::new(1.0, 1.0, -1.0),
                Vec3::new(1.0, 0.5, 0.0),
                Vec3::new(1.0, 1.0, -1.0),
                Vec2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vec3::new(1.0, -1.0, 1.0),
                Vec3::new(1.0, 0.5, 1.0),
                Vec3::new(1.0, -1.0, 1.0),
                Vec2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, -1.0, 1.0),
                Vec3::new(0.0, 0.5, 1.0),
                Vec3::new(-1.0, -1.0, 1.0),
                Vec2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::new(-1.0, -1.0, -1.0),
                Vec2::new(0.0, 1.0),
            ),
            Vertex::new(
                Vec3::new(1.0, -1.0, -1.0),
                Vec3::new(1.0, 0.5, 0.0),
                Vec3::new(1.0, -1.0, -1.0),
                Vec2::new(1.0, 1.0),
            ),
        ];

        let indices = vec![
            // +Y
            0, 1, 2, 0, 2, 3, // +Z
            0, 4, 5, 0, 5, 1, // +X
            0, 3, 7, 0, 7, 4, // -X
            1, 5, 6, 1, 6, 2, // -Z
            2, 6, 7, 2, 7, 3, // -Y
            4, 6, 5, 4, 7, 6,
        ];

        ctx.create_mesh(vertices, indices)
    }
}
