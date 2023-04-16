use std::error::Error;

use cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector2};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use buddle_render::Camera;
use buddle_render::Context;
use buddle_render::Vertex;

const SHADER: &str = "struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct CameraMatrices {
    view_matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>
};

struct ModelMatrices {
    mvp: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> camera: CameraMatrices;

@group(1) @binding(0)
var<uniform> model: ModelMatrices;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = model.mvp * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}";

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let physical_size = window.inner_size();
    let mut ctx = Context::new(
        &window,
        Vector2::new(physical_size.width, physical_size.height),
    );

    let triangle = ctx.create_mesh(
        &[
            Vertex {
                position: [0f32, 0.4f32, 1f32],
                color: [1f32, 0f32, 0f32],
                tex_coords: [0f32, 0f32],
            },
            Vertex {
                position: [0.5f32, -0.4f32, 1f32],
                color: [0f32, 1f32, 0f32],
                tex_coords: [0f32, 0f32],
            },
            Vertex {
                position: [-0.5f32, -0.4f32, 1f32],
                color: [0f32, 0f32, 1f32],
                tex_coords: [0f32, 0f32],
            },
        ],
        &[0, 1, 2],
    );

    let shader = ctx.create_shader(
        SHADER,
        vec![
            &ctx.create_bind_group_layout(1),
            &ctx.create_bind_group_layout(1),
        ],
    );

    let camera = Camera::perspective(Point3::origin(), Point3::new(0.0, 0.0, 1.0), 72.0);
    let rast = camera.rasterize(&ctx);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::Resized(physical_size) => {
                    ctx.resize(Vector2::new(physical_size.width, physical_size.height));
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    ctx.resize(Vector2::new(new_inner_size.width, new_inner_size.height));
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let mut rb = rast.new_frame(&ctx);

                rb.add_draw_call(&triangle, &shader, Matrix4::identity());

                rb.submit(&ctx).unwrap();
            }
            _ => {}
        }
    });
}
