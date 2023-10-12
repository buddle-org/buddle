//! Representation of a camera in 3D-space

use wgpu::{BindGroup, Buffer, BufferUsages};

use buddle_math::{Mat4, Vec3};

use crate::gpu::{Context, RenderBuffer};
use crate::BindGroupLayoutEntry;

#[derive(PartialEq, Copy, Clone)]
pub enum CameraType {
    Perspective,
    Orthographic,
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub cull_near: f32,
    pub cull_far: f32,
    pub cam_type: CameraType,
}

pub struct Rasterizer {
    pub camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl Camera {
    pub fn perspective(position: Vec3, target: Vec3, fov: f32) -> Self {
        Camera {
            position,
            target,
            up: Vec3::Y,
            fov,
            cull_near: 0.01,
            cull_far: 1000.0,
            cam_type: CameraType::Perspective,
        }
    }

    pub fn orthographic(position: Vec3, target: Vec3, near_plane_width: f32) -> Self {
        Camera {
            position,
            target,
            up: Vec3::Y,
            fov: near_plane_width,
            cull_near: 0.01,
            cull_far: 1000.0,
            cam_type: CameraType::Orthographic,
        }
    }

    pub fn rasterize(self, ctx: &Context) -> Rasterizer {
        let camera_matrices = CameraData::new(Mat4::IDENTITY, Mat4::IDENTITY, Vec3::ZERO);
        let camera_buffer = ctx.create_buffer(
            &[camera_matrices],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let camera_bind_group = ctx.create_bind_group(
            &ctx.create_bind_group_layout(vec![BindGroupLayoutEntry::Buffer]),
            vec![camera_buffer.as_entire_binding()],
        );
        Rasterizer::new(self, camera_buffer, camera_bind_group)
    }
}

impl Rasterizer {
    pub(crate) fn new(camera: Camera, camera_buffer: Buffer, camera_bind_group: BindGroup) -> Self {
        Rasterizer {
            camera,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn new_frame(&self, ctx: &Context) -> RenderBuffer {
        let aspect = ctx.surface.config.width as f32 / ctx.surface.config.height as f32;

        let view_matrix =
            Mat4::look_at_rh(self.camera.position, self.camera.target, self.camera.up);
        let proj_matrix;
        if self.camera.cam_type == CameraType::Perspective {
            proj_matrix = Mat4::perspective_rh_gl(
                self.camera.fov.to_radians(),
                aspect,
                self.camera.cull_near,
                self.camera.cull_far,
            );
        } else {
            let top = self.camera.fov / 2.0;
            let right = top * aspect;

            proj_matrix = Mat4::orthographic_rh_gl(
                -right,
                right,
                -top,
                top,
                self.camera.cull_near,
                self.camera.cull_far,
            );
        }

        #[rustfmt::skip]
        pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_slice(&[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        ]);

        let final_proj_mat = OPENGL_TO_WGPU_MATRIX * proj_matrix;

        ctx.update_buffer(
            &self.camera_buffer,
            &[CameraData::new(
                view_matrix,
                proj_matrix,
                self.camera.position,
            )],
        );

        RenderBuffer::new(&self.camera_bind_group, view_matrix, final_proj_mat)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraData {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    position: [f32; 3],
}

impl CameraData {
    pub fn new(view_matrix: Mat4, proj_matrix: Mat4, position: Vec3) -> Self {
        Self {
            view_matrix: view_matrix.to_cols_array_2d(),
            proj_matrix: proj_matrix.to_cols_array_2d(),
            position: position.to_array(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ModelMatrices {
    mvp: [[f32; 4]; 4],
    model_matrix: [[f32; 4]; 4],
    normal_matrix: [[f32; 4]; 4],
}

impl ModelMatrices {
    pub fn new(view_matrix: Mat4, proj_matrix: Mat4, model_matrix: Mat4) -> Self {
        let model_view_matrix = view_matrix * model_matrix;

        Self {
            mvp: ((proj_matrix * view_matrix) * model_matrix).to_cols_array_2d(),
            model_matrix: model_matrix.to_cols_array_2d(),
            normal_matrix: model_view_matrix.inverse().transpose().to_cols_array_2d(),
        }
    }
}
