use glam::{uvec2, Mat3, Vec3};

use crate::Context;

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
    ctx.render.globals.focal_length = focal_length;
}

/// Resizes the render texture
pub fn resize(ctx: &mut Context, width: u32, height: u32) {
    ctx.render.globals.screen_dim = uvec2(width, height);
}
