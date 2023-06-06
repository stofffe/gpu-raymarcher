use winit::event::MouseButton;

use crate::context::Context;

/// Returns the mouse delta for the current frame
pub fn mouse_delta(ctx: &Context) -> (f32, f32) {
    let (dx, dy) = ctx.input.mouse.mouse_delta();
    (dx as f32, dy as f32)
}

/// Returns if mouse is on screen or not
pub fn mouse_on_screen(ctx: &Context) -> bool {
    ctx.input.mouse.on_screen()
}

/// Returns the current physical coordinates for the mouse
pub fn mouse_pos_physical(ctx: &Context) -> (f64, f64) {
    ctx.input.mouse.mouse_pos_physical()
}

/// Returns the current pixel under the mouse
pub fn mouse_pos_pixel(ctx: &Context) -> (u32, u32) {
    ctx.input.mouse.mouse_pos_pixel(&ctx.render)
}

/// Returns true if MouseButton is pressed
/// Accepts repeating
pub fn mouse_button_pressed(ctx: &Context, keycode: MouseButton) -> bool {
    ctx.input.mouse.button_pressed(keycode)
}

/// Returns true if MouseButton was pressed this frame
pub fn mouse_button_just_pressed(ctx: &Context, keycode: MouseButton) -> bool {
    ctx.input.mouse.button_just_pressed(keycode)
}

/// Returns true if MouseButton was released this frame
pub fn mouse_button_released(ctx: &Context, keycode: MouseButton) -> bool {
    ctx.input.mouse.button_released(keycode)
}

/// Returns the scroll delta for the current frame
pub fn scroll_delta(ctx: &Context) -> (f32, f32) {
    let (dx, dy) = ctx.input.mouse.scroll_delta();
    (dx as f32, dy as f32)
}
