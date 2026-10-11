use glam::{Mat4, Quat, Vec3};
use world::PlanetaryWorld;
use crate::controls::KeyboardControls;

/// Interface for player avatars acting within planetary gravity.
pub trait Player {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn toggle_invert_y(&mut self);
    fn toggle_invert_x(&mut self);
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
}

/// Fully encapsulated player entity implementing Player.
pub struct PlayerAvatar {
    position: Vec3,
    surface_up: Vec3,
    yaw: f32,
    pitch: f32,
    invert_y: bool,
    invert_x: bool,
}

impl PlayerAvatar {
    pub fn new(initial_position: Vec3) -> Self {
        Self {
            position: initial_position,
            surface_up: Vec3::Y,
            yaw: 0.0,
            pitch: 0.0,
            invert_y: false,
            invert_x: false,
        }
    }

    fn orientation_basis(&self) -> (Vec3, Vec3, Vec3, Quat) {
        let up = self.surface_up.normalize();
        let arbitrary_ref = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let tangent = up.cross(arbitrary_ref).normalize();
        let bitangent = tangent.cross(up).normalize();

        let surface_rot = Quat::from_mat3(&glam::Mat3::from_cols(tangent, up, bitangent));
        let view_rot = surface_rot * Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);

        let forward = view_rot * Vec3::NEG_Z;
        let right = view_rot * Vec3::X;

        (forward, right, up, view_rot)
    }
}

impl Player for PlayerAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.003;
        let x_sign = if self.invert_x { 1.0 } else { -1.0 };
        let y_sign = if self.invert_y { 1.0 } else { -1.0 };

        self.yaw += dx * sensitivity * x_sign;
        self.pitch = (self.pitch + dy * sensitivity * y_sign).clamp(-1.4, 1.4);
    }

    fn toggle_invert_y(&mut self) {
        self.invert_y = !self.invert_y;
        println!("[Player] Invert Y (Pitch) is now: {}", self.invert_y);
    }

    fn toggle_invert_x(&mut self) {
        self.invert_x = !self.invert_x;
        println!("[Player] Invert X (Yaw) is now: {}", self.invert_x);
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        let target_up = world.surface_up_at(self.position);

        let rot_to_target = Quat::from_rotation_arc(self.surface_up, target_up);
        self.surface_up = rot_to_target.slerp(Quat::IDENTITY, 0.85) * self.surface_up;
        if self.surface_up.dot(target_up) > 0.999 {
            self.surface_up = target_up;
        }

        let (forward, right, up, _) = self.orientation_basis();
        let raw_move = controls.resolve_motion(forward, right);

        let tangent_move = raw_move - up * raw_move.dot(up);
        if tangent_move.length_squared() > 1e-4 {
            self.position += tangent_move.normalize() * (6.0 * dt);
        }

        self.position = world.constrain_to_surface(self.position, 1.0);
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let (forward, _, up, _) = self.orientation_basis();
        let proj = Mat4::perspective_rh(65.0f32.to_radians(), aspect, 0.1, 200.0);
        let eye = self.position + up * 0.8;
        let target = eye + forward;
        let view = Mat4::look_at_rh(eye, target, up);
        proj * view
    }
}
