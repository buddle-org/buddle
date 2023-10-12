mod controller;

use std::error::Error;
use std::io;
use winit::dpi::PhysicalPosition;

use winit::event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, WindowBuilder};

use crate::controller::CameraController;
use buddle_math::{Mat4, UVec2, Vec2, Vec3};
use buddle_nif::Nif;
use buddle_render::Context;
use buddle_render::Model;
use buddle_render::{Camera, Material};
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
        Vec3::new(-100.0, 75.0, 0.0),
        Vec3::new(0.0, 50.0, -1.0),
        72.0,
    );

    let mut controller = CameraController::new();

    let mut rast = camera.rasterize(&ctx);

    let root = Archive::heap("Root.wad", false).unwrap();
    let mut intern = Interner::new(&root);

    let handle = intern.intern("WC_Z01_Golem_Court.nif").unwrap();
    let data = intern.fetch_mut(handle).unwrap();
    let mut cursor = io::Cursor::new(data);
    let owl_gamma = Nif::parse(&mut cursor).unwrap();

    let gamma = Model::from_nif(&ctx, owl_gamma).unwrap();

    let mut last_mouse_pos = Vec2::ZERO;
    let mut capture_mouse = true;

    window.set_cursor_visible(false);
    let _ = window.set_cursor_grab(CursorGrabMode::Confined);

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
                WindowEvent::CursorMoved { position, .. } if capture_mouse => {
                    let pos = Vec2::new(position.x as f32, position.y as f32);
                    controller.add_mouse_delta(pos - last_mouse_pos);

                    let window_size = window.inner_size();
                    let center =
                        PhysicalPosition::new(window_size.width / 2, window_size.height / 2);
                    window.set_cursor_position(center).unwrap();

                    last_mouse_pos = Vec2::new(center.x as f32, center.y as f32);
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => {
                match event {
                    //DeviceEvent::MouseMotion { delta} => {
                    //    controller.add_mouse_delta(Vec2::new(delta.0 as f32, delta.1 as f32));
                    //}
                    DeviceEvent::Key(input) => {
                        let pressed = input.state == ElementState::Pressed;
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::W) => controller.set_key_state(0, pressed),
                            Some(VirtualKeyCode::S) => controller.set_key_state(1, pressed),
                            Some(VirtualKeyCode::A) => controller.set_key_state(2, pressed),
                            Some(VirtualKeyCode::D) => controller.set_key_state(3, pressed),
                            Some(VirtualKeyCode::Space) => controller.set_key_state(4, pressed),
                            Some(VirtualKeyCode::LShift) => controller.set_key_state(5, pressed),
                            Some(VirtualKeyCode::Escape) if pressed => {
                                window.set_cursor_visible(capture_mouse);
                                capture_mouse = !capture_mouse;

                                if capture_mouse {
                                    let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                                } else {
                                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                                }

                                let window_size = window.inner_size();
                                let center = PhysicalPosition::new(
                                    window_size.width / 2,
                                    window_size.height / 2,
                                );
                                window.set_cursor_position(center).unwrap();

                                last_mouse_pos = Vec2::new(center.x as f32, center.y as f32);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                controller.update_free(&mut rast.camera);

                let mut rb = rast.new_frame(&ctx);

                gamma.render_to(&mut rb, Mat4::IDENTITY);

                rb.submit(&ctx).unwrap();
            }
            _ => {}
        }
    });
}
