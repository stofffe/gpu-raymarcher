use crate::Context;
use wgpu::PresentMode;
use winit::{
    dpi::PhysicalSize,
    window::{CursorGrabMode, Fullscreen},
};

/// Enables/Disables vsync
pub fn set_vsync(ctx: &mut Context, vsync: bool) {
    let present_mode = if vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    ctx.render.reconfigure_present_mode(present_mode);
}

/// Enables/Disables borderless windowed mode
pub fn set_fullscreen(ctx: &mut Context, fullscreen: bool) {
    let fullscreen_mode = if fullscreen {
        Some(Fullscreen::Borderless(None))
    } else {
        None
    };
    ctx.render.window.set_fullscreen(fullscreen_mode);
}

/// Enables/Disables window resizing
pub fn set_resizeable(ctx: &mut Context, resizable: bool) {
    ctx.render.window.set_resizable(resizable);
}

/// Sets the inner size of the window
pub fn set_size(ctx: &mut Context, size: (u32, u32)) {
    ctx.render
        .window
        .set_inner_size(PhysicalSize::new(size.0, size.1));
}

/// Enables/Disables the cursor
/// If disabled: Turns off cursor and locks cursor to middle of window
pub fn set_cursor_enabled(ctx: &mut Context, enabled: bool) {
    ctx.render.window.set_cursor_visible(enabled);
    let grab_mode = if enabled {
        CursorGrabMode::None
    } else {
        CursorGrabMode::Locked
    };
    // TODO handle error
    ctx.render.window.set_cursor_grab(grab_mode).unwrap();
}
