use glam::Quat;

/// Polymorphic upper-body yaw state restricting full 360-degree turns when shoulders are wedged in a narrow 2x2 shaft.
#[derive(Clone, Copy, Debug)]
pub enum ShoulderArc {
    Unobstructed(f32),
    Wedged(f32, f32),
}

impl ShoulderArc {
    pub fn clamp_yaw(self, yaw: f32) -> f32 {
        match self {
            Self::Unobstructed(_) => yaw,
            Self::Wedged(anchor, play) => yaw.clamp(anchor - play, anchor + play),
        }
    }

    pub fn transition(self, wedged: bool, current_yaw: f32, play: f32) -> Self {
        match (self, wedged) {
            (Self::Unobstructed(free_yaw), true) => Self::Wedged(free_yaw, play),
            (Self::Wedged(anchor, _), true) => Self::Wedged(anchor, play),
            (_, false) => Self::Unobstructed(current_yaw),
        }
    }
}

/// Interface for viewpoint rotation and surface alignment.
pub trait ViewOrientation {
    fn rotate(&mut self, dx: f32, dy: f32);
    fn constrain_shoulders(&mut self, _wedged: bool, _play: f32) {}
    fn combine_with_surface(&self, surface_rot: Quat) -> Quat;
}

/// Encapsulated first-person look orientation without internal flags or hardcoded coefficients.
pub struct MouseLook {
    yaw: f32,
    pitch: f32,
    arc: ShoulderArc,
}

impl MouseLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_arc(yaw: f32, pitch: f32, arc: ShoulderArc) -> Self {
        Self { yaw, pitch, arc }
    }

    pub fn new(yaw: f32, pitch: f32) -> Self {
        Self::with_arc(yaw, pitch, ShoulderArc::Unobstructed(yaw))
    }

    /// Secondary constructor initializing idle zero angles.
    pub fn idle() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl ViewOrientation for MouseLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw = self.arc.clamp_yaw(self.yaw - dx);
        self.pitch = (self.pitch - dy).clamp(-1.4, 1.4);
    }

    fn constrain_shoulders(&mut self, wedged: bool, play: f32) {
        self.arc = self.arc.transition(wedged, self.yaw, play);
        self.yaw = self.arc.clamp_yaw(self.yaw);
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        let local_rot = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
        surface_rot * local_rot
    }
}

/// Decorator scaling mouse rotational delta by an encapsulated sensitivity coefficient.
pub struct SensitivityLook {
    look: Box<dyn ViewOrientation>,
    sensitivity: f32,
}

impl SensitivityLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(look: Box<dyn ViewOrientation>, sensitivity: f32) -> Self {
        Self { look, sensitivity }
    }

    /// Secondary constructor wrapping orientation with calibrated sensitivity (0.003).
    pub fn calibrated(look: Box<dyn ViewOrientation>) -> Self {
        Self::new(look, 0.003)
    }
}

impl ViewOrientation for SensitivityLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        self.look.rotate(dx * self.sensitivity, dy * self.sensitivity);
    }

    fn constrain_shoulders(&mut self, wedged: bool, play: f32) {
        self.look.constrain_shoulders(wedged, play);
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        self.look.combine_with_surface(surface_rot)
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use world::{BiomedCubeWorld, PlanetaryWorld};
    use crate::avatar::{Avatar, PlanetaryAvatar};
    use crate::controls::KeyMotion;
    use super::*;

    #[test]
    fn test_sensitivity_look_rotation() {
        let mut look = SensitivityLook::calibrated(Box::new(MouseLook::idle()));
        look.rotate(10.0, 5.0);
        let q = look.combine_with_surface(Quat::IDENTITY);
        assert_ne!(q, Quat::IDENTITY);
    }

    #[test]
    fn test_avatar_in_two_by_two_shaft_cannot_turn_fully_until_widened() {
        use winit::keyboard::KeyCode;
        use crate::controls::KeyboardControls;
        let mut world = BiomedCubeWorld::beveled(32, 1.0);
        let rim_y = world.constrain_to_surface(Vec3::new(0.25, 20.0, 0.25), 1.0).y;
        let mut walk_in = KeyMotion::idle();
        walk_in.register_key(KeyCode::KeyW, true);
        for _ in 0..3 { let y = world.constrain_to_surface(Vec3::new(0.25, 20.0, 0.25), 0.0).y; assert!(world.excavate_surface(Vec3::new(0.25, y + 1.0, 0.25), Vec3::NEG_Y)); }
        let mut walker_1x1 = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 20.0, 0.65), 1.0));
        for _ in 0..15 { walker_1x1.step_motion(1.0 / 60.0, &walk_in, &world); }
        let hole_1x1_y = walker_1x1.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).y;
        assert!((hole_1x1_y - (rim_y + 0.8)).abs() < 0.15, "Avatar must NOT fall into a 1x1 hole (got {hole_1x1_y} vs rim eye {})", rim_y + 0.8);
        for _ in 0..3 { let y = world.constrain_to_surface(Vec3::new(0.75, 20.0, 0.25), 0.0).y; assert!(world.excavate_surface(Vec3::new(0.75, y + 1.0, 0.25), Vec3::NEG_Y)); }
        let mut walker_2x1 = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.50, 20.0, 0.65), 1.0));
        for _ in 0..25 { walker_2x1.step_motion(1.0 / 60.0, &walk_in, &world); }
        let hole_2x1_y = walker_2x1.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).y;
        assert!(hole_2x1_y < rim_y + 0.8 - 0.8, "Avatar MUST fall into a 2x1 hole (got {hole_2x1_y} vs rim eye {})", rim_y + 0.8);

        let pos = world.constrain_to_surface(Vec3::new(0.50, 20.0, 0.25), 1.0);
        let mut avatar = PlanetaryAvatar::at(pos);
        avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);

        let forward_of = |a: &PlanetaryAvatar| -> Vec3 {
            let inv = a.view_projection_matrix(1.0).inverse();
            (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - inv.project_point3(Vec3::ZERO)).normalize()
        };
        let initial_fwd = forward_of(&avatar);
        avatar.rotate_view(150.0, 0.0);
        let slight_fwd = forward_of(&avatar);
        assert!(slight_fwd.x.abs() > 0.25, "Avatar must be able to turn partially inside a 2x1 shaft, got {slight_fwd:?}");

        avatar.rotate_view(1500.0, 0.0);
        let clamped_right = forward_of(&avatar);
        assert!(clamped_right.dot(initial_fwd) > 0.35, "Avatar in 2x1 shaft must not be able to turn 180/360 degrees because upper body is wider than 1 block, got {clamped_right:?}");

        avatar.rotate_view(-3000.0, 0.0);
        let clamped_left = forward_of(&avatar);
        assert!(clamped_left.dot(initial_fwd) > 0.35, "Avatar in 2x1 shaft must stop against opposite wall when turning the other way, got {clamped_left:?}");

        let broad_attire = crate::attire::YouthAttire::squire()
            .with_adjusted_region(crate::attire::BodyRegion::Torso, 4, 4)
            .with_adjusted_region(crate::attire::BodyRegion::Arms, 4, 4);
        avatar.apply_attire(broad_attire);
        avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
        let broad_clamped = forward_of(&avatar);
        assert!(broad_clamped.dot(initial_fwd) > clamped_left.dot(initial_fwd) + 0.15, "Broader customized upper body (Torso/Arms) must have tighter turning play in a 2x1 shaft ({} vs {})", broad_clamped.dot(initial_fwd), clamped_left.dot(initial_fwd));

        avatar.apply_attire(crate::attire::YouthAttire::squire());
        for u in 0..2 {
            let (x, z) = ((u as f32) * 0.5 + 0.25, 0.75);
            for _ in 0..3 {
                let y = world.constrain_to_surface(Vec3::new(x, 20.0, z), 0.0).y;
                assert!(world.excavate_surface(Vec3::new(x, y + 1.0, z), Vec3::NEG_Y));
            }
        }
        let mut walker_2x2 = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.50, 20.0, 1.15), 1.0));
        for _ in 0..35 { walker_2x2.step_motion(1.0 / 60.0, &walk_in, &world); }
        let hole_2x2_y = walker_2x2.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).y;
        assert!(hole_2x2_y < rim_y + 0.8 - 1.0, "Avatar walking into a 2x2 hole must fall into it instead of gliding across (got {hole_2x2_y} vs rim eye {})", rim_y + 0.8);
        avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
        avatar.rotate_view(1050.0, 0.0);
        let full_turn_fwd = forward_of(&avatar);
        assert!(full_turn_fwd.dot(initial_fwd) < -0.5, "Widening shaft to 2x2 must allow full 180/360-degree rotation when upper body width is smaller than 2 blocks, got {full_turn_fwd:?}");
    }
}
