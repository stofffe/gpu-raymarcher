use glam::{Mat3, Mat4, Vec3};

use crate::Context;

pub fn set_camera_pos(ctx: &mut Context, pos: Vec3) {
    ctx.render.globals.camera_pos = pos;
}

pub fn get_camera_pos(ctx: &mut Context) -> Vec3 {
    ctx.render.globals.camera_pos
}

pub fn set_camera_rot(ctx: &mut Context, rot: Mat3) {
    ctx.render.globals.camera_rot = rot;
}

pub fn get_camera_rot(ctx: &mut Context) -> Mat3 {
    ctx.render.globals.camera_rot
}

pub fn set_focal_length(ctx: &mut Context, focal_length: f32) {
    ctx.render.globals.focal_length = focal_length;
}

pub fn get_focal_length(ctx: &mut Context, focal_length: f32) -> f32 {
    ctx.render.globals.focal_length
}

// pub fn add_camera_pos(ctx: &mut Context, pos: Vec3) {
//     ctx.render.globals.camera_pos += pos;
// }

// pub fn add_camera_rot(ctx: &mut Context, rot: Mat4) {
//     ctx.render.globals.camera_rot *= rot;
// }
