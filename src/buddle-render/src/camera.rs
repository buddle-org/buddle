//! Representation of a camera in 3D-space

use cgmath::{Matrix, Matrix4, Point3, SquareMatrix, Transform, Vector3};
use wgpu::{BindGroup, Buffer, BufferUsages};

use crate::gpu::{Context, RenderBuffer};

#[derive(PartialEq, Copy, Clone)]
pub enum CameraType {
    Perspective,
    Orthographic,
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub cull_near: f32,
    pub cull_far: f32,
    pub cam_type: CameraType,
}

pub struct Rasterizer {
    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl Camera {
    pub fn perspective(position: Point3<f32>, target: Point3<f32>, fov: f32) -> Self {
        Camera {
            position,
            target,
            up: Vector3::unit_y(),
            fov,
            cull_near: 0.01,
            cull_far: 1000.0,
            cam_type: CameraType::Perspective,
        }
    }

    pub fn orthographic(position: Point3<f32>, target: Point3<f32>, near_plane_width: f32) -> Self {
        Camera {
            position,
            target,
            up: Vector3::unit_y(),
            fov: near_plane_width,
            cull_near: 0.01,
            cull_far: 1000.0,
            cam_type: CameraType::Orthographic,
        }
    }

    pub fn rasterize(self, ctx: &Context) -> Rasterizer {
        let camera_matrices = CameraMatrices::new(Matrix4::identity(), Matrix4::identity());
        let camera_buffer = ctx.create_buffer(
            &[camera_matrices],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let camera_bind_group = ctx.create_bind_group(vec![&camera_buffer]);
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
            Matrix4::look_at_rh(self.camera.position, self.camera.target, self.camera.up);
        let proj_matrix;
        if self.camera.cam_type == CameraType::Perspective {
            let top = self.camera.cull_near * (self.camera.fov * 0.5).to_radians().tan();
            let right = top * aspect;

            proj_matrix = cgmath::frustum(
                -right,
                right,
                -top,
                top,
                self.camera.cull_near,
                self.camera.cull_far,
            );
        } else {
            let top = self.camera.fov / 2.0;
            let right = top * aspect;

            proj_matrix = cgmath::ortho(
                -right,
                right,
                -top,
                top,
                self.camera.cull_near,
                self.camera.cull_far,
            );
        }

        #[rustfmt::skip]
        pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

        let final_proj_mat = OPENGL_TO_WGPU_MATRIX * proj_matrix;

        ctx.update_buffer(
            &self.camera_buffer,
            &[CameraMatrices::new(view_matrix, proj_matrix)],
        );

        RenderBuffer::new(&self.camera_bind_group, view_matrix, final_proj_mat)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraMatrices {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
}

impl CameraMatrices {
    pub fn new(view_matrix: Matrix4<f32>, proj_matrix: Matrix4<f32>) -> Self {
        Self {
            view_matrix: view_matrix.into(),
            proj_matrix: proj_matrix.into(),
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
    pub fn new(
        view_matrix: Matrix4<f32>,
        proj_matrix: Matrix4<f32>,
        model_matrix: Matrix4<f32>,
    ) -> Self {
        let model_view_matrix = view_matrix * model_matrix;

        Self {
            mvp: ((proj_matrix * view_matrix) * model_matrix).into(),
            model_matrix: model_matrix.into(),
            normal_matrix: model_view_matrix
                .inverse_transform()
                .unwrap()
                .transpose()
                .into(),
        }
    }
}
