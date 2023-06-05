use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    app::{App, Callbacks},
    context::Context,
    render::{HEIGHT, WIDTH},
};

pub(crate) fn new_window() -> (winit::window::Window, winit::event_loop::EventLoop<()>) {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();

    (window, event_loop)
}

pub(crate) async fn run_window<C: Callbacks + 'static>(
    event_loop: EventLoop<()>,
    mut app: App<C>,
    mut ctx: Context,
) {
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            if window_id == ctx.render.window.id() {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        ctx.render.resize_window(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        ctx.render.resize_window(**new_inner_size);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        ctx.input
                            .mouse
                            .set_pos((position.x, position.y), &ctx.render);
                    }
                    WindowEvent::MouseInput { state, button, .. } => match state {
                        ElementState::Pressed => ctx.input.mouse.press_button(*button),
                        ElementState::Released => ctx.input.mouse.release_button(*button),
                    },
                    WindowEvent::CursorLeft { .. } => {
                        ctx.input.mouse.set_on_screen(false);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(keycode) = input.virtual_keycode {
                            match input.state {
                                ElementState::Pressed => ctx.input.keyboard.set_key(keycode),
                                ElementState::Released => ctx.input.keyboard.release_key(keycode),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::DeviceEvent { ref event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => ctx.input.mouse.set_mouse_delta(*delta),
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == ctx.render.window.id() => {
            match ctx.render.render(&ctx.time) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => ctx.render.resize_window(ctx.render.window_size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            if app.update(&mut ctx) {
                *control_flow = ControlFlow::Exit;
            }
            ctx.render.window.request_redraw();
        }
        _ => {}
    });
}
