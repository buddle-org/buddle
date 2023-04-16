use std::error::Error;

use cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector2};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use buddle_render::Vertex;
use buddle_render::{BindGroupLayoutEntry, Camera};
use buddle_render::{Context, TextureDimensions};

const FLAT_TEXTURE: &str = include_str!("shaders/flat_texture.wgsl");

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("buddle").build(&event_loop).unwrap();

    let physical_size = window.inner_size();
    let mut ctx = Context::new(
        &window,
        Vector2::new(physical_size.width, physical_size.height),
    );

    let triangle = ctx.create_mesh(
        &[
            Vertex {
                position: [0f32, 0.4f32, -1f32], // Center top
                color: [1f32, 0f32, 0f32],
                tex_coords: [0.5f32, 0f32],
            },
            Vertex {
                position: [-0.5f32, -0.4f32, -1f32], // Left down
                color: [0f32, 1f32, 0f32],
                tex_coords: [0f32, 1f32],
            },
            Vertex {
                position: [0.5f32, -0.4f32, -1f32], // Right down
                color: [0f32, 0f32, 1f32],
                tex_coords: [1f32, 1f32],
            },
        ],
        &[0, 1, 2],
    );

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

    #[rustfmt::skip]
    let texture = ctx.create_texture(&[
        215, 38, 0, 255,
        9, 86, 191, 255,
        236, 212, 7, 255,
        55, 151, 17, 255], Vector2::new(2, 2));

    let material = ctx.create_material(shader, texture);

    let camera = Camera::perspective(Point3::origin(), Point3::new(0.0, 0.0, -1.0), 72.0);
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

                rb.add_draw_call(&triangle, &material, Matrix4::identity());

                rb.submit(&ctx).unwrap();
            }
            _ => {}
        }
    });
}
