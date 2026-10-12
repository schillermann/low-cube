use glam::{Mat4, Quat, Vec3};
use world::PlanetaryWorld;
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::orientation::{MouseLook, ViewOrientation};

/// Interface for player avatars acting within planetary gravity.
pub trait Avatar {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn toggle_invert_y(&mut self);
    fn toggle_invert_x(&mut self);
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
}

/// Perspective projection lens encapsulating field of view and clipping planes.
pub struct PerspectiveLens {
    fov_radians: f32,
    near_plane: f32,
    far_plane: f32,
}

impl PerspectiveLens {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(fov_radians: f32, near_plane: f32, far_plane: f32) -> Self {
        Self {
            fov_radians,
            near_plane,
            far_plane,
        }
    }

    /// Secondary constructor with standard first-person field of view.
    pub fn human_eye() -> Self {
        Self::new(65.0f32.to_radians(), 0.1, 200.0)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_radians, aspect, self.near_plane, self.far_plane)
    }
}

/// Local coordinate basis for an avatar on a curved planetary surface.
struct OrientationBasis {
    forward: Vec3,
    right: Vec3,
    up: Vec3,
}

impl OrientationBasis {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(forward: Vec3, right: Vec3, up: Vec3) -> Self {
        Self { forward, right, up }
    }

    /// Secondary constructor aligning basis with surface up vector and view orientation.
    fn aligned(surface_up: Vec3, look: &dyn ViewOrientation) -> Self {
        let up = surface_up.normalize();
        let arbitrary_ref = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::Z };
        let tangent = up.cross(arbitrary_ref).normalize();
        let bitangent = tangent.cross(up).normalize();

        let surface_rot = Quat::from_mat3(&glam::Mat3::from_cols(tangent, up, bitangent));
        let view_rot = look.combine_with_surface(surface_rot);

        let forward = view_rot * Vec3::NEG_Z;
        let right = view_rot * Vec3::X;

        Self::new(forward, right, up)
    }

    fn drive_locomotion(
        &self,
        controls: &dyn KeyboardControls,
        target: &mut dyn LocomotionTarget,
    ) {
        controls.drive_locomotion(self.forward, self.right, target);
    }

    fn view_matrix_at(&self, position: Vec3, eye_height: f32) -> Mat4 {
        let eye = position + self.up * eye_height;
        let target = eye + self.forward;
        Mat4::look_at_rh(eye, target, self.up)
    }
}

/// Fully encapsulated player entity composed of position, gravity alignment, ViewOrientation, PerspectiveLens, and GravitationalFlight.
pub struct PlanetaryAvatar {
    position: Vec3,
    surface_up: Vec3,
    look: Box<dyn ViewOrientation>,
    lens: PerspectiveLens,
    flight: Box<dyn GravitationalFlight>,
}

impl PlanetaryAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        position: Vec3,
        surface_up: Vec3,
        look: Box<dyn ViewOrientation>,
        lens: PerspectiveLens,
        flight: Box<dyn GravitationalFlight>,
    ) -> Self {
        Self {
            position,
            surface_up,
            look,
            lens,
            flight,
        }
    }

    /// Secondary constructor initializing avatar at an initial position.
    pub fn at(initial_position: Vec3) -> Self {
        Self::new(
            initial_position,
            Vec3::Y,
            Box::new(MouseLook::idle()),
            PerspectiveLens::human_eye(),
            Box::new(SurfaceFlight::terrestrial(1.0)),
        )
    }

    fn reorient_surface(&mut self, target_up: Vec3) {
        let rot_to_target = Quat::from_rotation_arc(self.surface_up, target_up);
        self.surface_up = rot_to_target.slerp(Quat::IDENTITY, 0.85) * self.surface_up;
        if self.surface_up.dot(target_up) > 0.999 {
            self.surface_up = target_up;
        }
    }
}

impl LocomotionTarget for PlanetaryAvatar {
    fn travel(&mut self, direction: Vec3) {
        let tangent = direction - self.surface_up * direction.dot(self.surface_up);
        if tangent.length_squared() > 1e-4 {
            self.position += tangent.normalize() * (6.0 * (1.0 / 60.0));
        }
    }

    fn jump(&mut self) {
        self.flight.jump();
    }
}

impl Avatar for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        self.look.rotate(dx, dy);
    }

    fn toggle_invert_y(&mut self) {
        self.look.toggle_invert_y();
    }

    fn toggle_invert_x(&mut self) {
        self.look.toggle_invert_x();
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        self.reorient_surface(world.surface_up_at(self.position));

        let basis = OrientationBasis::aligned(self.surface_up, self.look.as_ref());
        basis.drive_locomotion(controls, self);

        self.position = self.flight.advance(dt, self.position, self.surface_up, world);
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let basis = OrientationBasis::aligned(self.surface_up, self.look.as_ref());
        let proj = self.lens.projection_matrix(aspect);
        let view = basis.view_matrix_at(self.position, 0.8);
        proj * view
    }
}

