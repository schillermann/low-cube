use glam::{Mat4, Quat, Vec3};
use world::PlanetaryWorld;
use crate::basis::OrientationBasis;
use crate::controls::KeyboardControls;
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::lens::PerspectiveLens;
use crate::order::LocomotionOrder;
use crate::orientation::{MouseLook, ViewOrientation};

/// Interface for player avatars acting within planetary gravity.
pub trait Avatar {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn toggle_invert_y(&mut self);
    fn toggle_invert_x(&mut self);
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
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

        let mut order = LocomotionOrder::idle();
        let basis = OrientationBasis::aligned(self.surface_up, self.look.as_ref());
        basis.drive_locomotion(controls, &mut order);

        order.dispatch_jump(&mut *self.flight);
        let stride = order.surface_stride(self.surface_up, 6.0 * dt);

        self.position = self.flight.advance(dt, self.position, stride, self.surface_up, world);
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let basis = OrientationBasis::aligned(self.surface_up, self.look.as_ref());
        let proj = self.lens.projection_matrix(aspect);
        let view = basis.view_matrix_at(self.position, 0.8);
        proj * view
    }
}
