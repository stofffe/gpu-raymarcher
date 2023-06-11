use glam::{uvec2, Mat3, Vec3};

use crate::{render::MAX_SHAPE_AMOUNT, Context, Shape};

/// Sets the internal camera position
pub fn set_camera_pos(ctx: &mut Context, pos: Vec3) {
    ctx.render.globals.camera_pos = pos;
}

/// Sets the internal camera rotation
pub fn set_camera_rot(ctx: &mut Context, rot: Mat3) {
    ctx.render.globals.camera_rot = rot;
}

/// Sets the internal camera focal length
pub fn set_focal_length(ctx: &mut Context, focal_length: f32) {
    debug_assert!(focal_length > 0.0, "focal length must be greater than 0");
    ctx.render.globals.focal_length = focal_length;
}

/// Resizes the render texture
pub fn resize(ctx: &mut Context, width: u32, height: u32) {
    debug_assert!(
        width != 0 || height != 0,
        "screen dimensions can not be zero"
    );
    ctx.render.globals.screen_dim = uvec2(width, height);
    // TODO resize render texture
}

pub fn render_shape(ctx: &mut Context, shape: Shape) {
    debug_assert!(
        ctx.render.shapes.len() < MAX_SHAPE_AMOUNT as usize,
        "can not add more shapes than max: {}",
        MAX_SHAPE_AMOUNT
    );
    ctx.render.shapes.push(shape);
}

pub fn render_shapes(ctx: &mut Context, shapes: Vec<Shape>) {
    for shape in shapes {
        render_shape(ctx, shape);
    }
}
