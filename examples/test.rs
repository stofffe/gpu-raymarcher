use glam::{quat, vec3, Mat3, Mat4, Quat, Vec3};
use gpu_raymarcher::{
    cmd::{keyboard, mouse, render, time, window},
    Callbacks, Context, KeyCode, KeyModifier,
};

const CAMERA_MOVE_SPEED: f32 = 1.0;
const CAMERA_ROTATE_SPEED: f32 = 1.0;
const CAMERA_ZOOM_SPEED: f32 = 0.01;

struct App {
    camera_pos: Vec3,
    yaw: f32,
    pitch: f32,
    focal_len: f32,
}

impl Callbacks for App {
    fn init(&self, ctx: &mut Context) {
        window::set_cursor_enabled(ctx, false);
    }

    fn update(&mut self, ctx: &mut Context, dt: f32) -> bool {
        self.input(ctx, dt);
        self.draw();

        // println!("{dt}");
        false
    }
}

impl App {
    fn input(&mut self, ctx: &mut Context, dt: f32) {
        let mut move_speed = CAMERA_MOVE_SPEED;
        if keyboard::modifier_pressed(ctx, KeyModifier::Shift) {
            move_speed *= 3.0;
        }

        // Camera rotation
        let (dx, dy) = mouse::mouse_delta(ctx);
        self.yaw += dx * CAMERA_ROTATE_SPEED;
        self.pitch += dy * CAMERA_ROTATE_SPEED;

        if self.pitch > 89.0 {
            self.pitch = 89.0
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0
        }
        let rotation = Mat3::from_rotation_y(self.yaw.to_radians())
            * Mat3::from_rotation_x(self.pitch.to_radians());

        // Camera movement
        let rot_mat = rotation.to_cols_array_2d();
        let right = vec3(rot_mat[0][0], rot_mat[0][1], rot_mat[0][2]).normalize();
        let up = vec3(rot_mat[1][0], rot_mat[1][1], rot_mat[1][2]).normalize();
        let forward = vec3(rot_mat[2][0], rot_mat[2][1], rot_mat[2][2]).normalize();

        let mut movement = Vec3::ZERO;
        if keyboard::key_pressed(ctx, KeyCode::W) {
            movement += forward;
        }
        if keyboard::key_pressed(ctx, KeyCode::S) {
            movement -= forward;
        }
        if keyboard::key_pressed(ctx, KeyCode::D) {
            movement += right;
        }
        if keyboard::key_pressed(ctx, KeyCode::A) {
            movement -= right;
        }
        if movement != Vec3::ZERO {
            movement = movement.normalize() * move_speed * dt;
        }
        self.camera_pos += movement;

        // Zoom
        self.focal_len += mouse::scroll_delta(ctx).1 * CAMERA_ZOOM_SPEED;
        self.focal_len = self.focal_len.max(0.1);

        // Update renderer
        render::set_camera_rot(ctx, rotation);
        render::set_camera_pos(ctx, self.camera_pos);
        render::set_focal_length(ctx, self.focal_len);
    }
    fn draw(&mut self) {}
}

fn main() {
    let app = App {
        camera_pos: vec3(0.0, 0.0, -3.0),
        yaw: 0.0,
        pitch: 0.0,
        focal_len: 1.0,
    };
    gpu_raymarcher::run(app);
}

// Camera rotation
// self.yaw = 90.0;
// println!("{}", self.yaw);
// let dir = vec3(
//     self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
//     self.pitch.to_radians().sin(),
//     self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
// );
// let camera_front = dir.normalize();
// let camera_up = vec3(0.0, 1.0, 0.0);
// let rotation = Mat4::look_to_lh(Vec3::ZERO, camera_front, camera_up);
// render::set_camera_rot(ctx, rotation);
