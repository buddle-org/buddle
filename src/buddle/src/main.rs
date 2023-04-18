use std::error::Error;
use std::io;

use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use buddle_math::{Mat4, UVec2, Vec3, Vec4};
use buddle_nif::Nif;
use buddle_render::Context;
use buddle_render::Model;
use buddle_render::{Camera, FlatMaterial, Material, Mesh};
use buddle_wad::{Archive, Interner};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("buddle")
        .build(&event_loop)
        .unwrap();

    let physical_size = window.inner_size();
    let mut ctx = Context::new(
        &window,
        UVec2::new(physical_size.width, physical_size.height),
    );

    let camera = Camera::perspective(
        Vec3::new(0.0, 75.0, 100.0),
        Vec3::new(0.0, 50.0, -1.0),
        72.0,
    );

    let rast = camera.rasterize(&ctx);

    let root = Archive::heap("Root.wad", false).unwrap();
    let mut intern = Interner::new(&root);

    let handle = intern.intern("Character/Owl/Owl_Gamma.nif").unwrap();
    let data = intern.fetch_mut(handle).unwrap();
    let mut cursor = io::Cursor::new(data);
    let owl_gamma = Nif::parse(&mut cursor).unwrap();

    let model = Model::from_nif(&ctx, owl_gamma).unwrap();

    let texture = ctx.create_render_texture(UVec2::new(1024, 1024));

    let mut rb = rast.new_frame(&ctx);
    rb.set_clear_color(Vec4::new(1.0, 1.0, 1.0, 1.0));

    model.render_to(&mut rb, Mat4::from_rotation_y(180.0f32.to_radians()));

    rb.render_to_texture(&ctx, &texture);

    let material: Box<dyn Material> = Box::new(FlatMaterial::new(&ctx, &texture.texture));
    let mesh = Mesh::make_plane(&ctx);
    let plane = Model::from_mesh(mesh, material);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::Resized(physical_size) => {
                    ctx.resize(UVec2::new(physical_size.width, physical_size.height));
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    ctx.resize(UVec2::new(new_inner_size.width, new_inner_size.height));
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let mut rb = rast.new_frame(&ctx);

                model.render_to(&mut rb, Mat4::from_rotation_y(180.0f32.to_radians()));
                plane.render_to(&mut rb, Mat4::from_scale(Vec3::splat(50.0)));

                rb.submit(&ctx).unwrap();
            }
            _ => {}
        }
    });
}
