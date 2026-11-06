use glam::{Mat3, Quat, Vec3};
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::controls::{KeyboardControls, LocomotionTarget};

const MAX_DRIVE_SPEED: f32 = 14.0;
const MAX_REVERSE_SPEED: f32 = 8.0;
const THROTTLE_ACCEL: f32 = 16.0;
const REVERSE_ACCEL: f32 = 11.0;
const BRAKE_DECEL: f32 = 32.0;
const COAST_DECEL: f32 = 3.5;
const STEER_RATE: f32 = 2.2;
const SEAT_EYE_SUPPORT: f32 = 1.0;
const REVERSE_ENGAGE_DWELL: f32 = 0.22;

struct PedalInput {
    throttle: f32,
    steer: f32,
    handbrake: bool,
}

impl PedalInput {
    const fn idle() -> Self {
        Self { throttle: 0.0, steer: 0.0, handbrake: false }
    }
}

impl LocomotionTarget for PedalInput {
    fn travel(&mut self, direction: Vec3) {
        self.throttle = direction.y;
        self.steer = direction.x;
    }

    fn jump(&mut self) {
        self.handbrake = true;
    }

    fn sneak(&mut self) {
        self.handbrake = true;
    }

    fn sprint(&mut self) {}
}

/// Longitudinal speed, reverse-shift dwell, wheel spin angle, and steering wheel turn state.
#[derive(Clone, Copy, Debug)]
pub struct DriveKinematics {
    speed: f32,
    reverse_dwell: f32,
    wheel_spin: f32,
    steer_turn: f32,
}

impl DriveKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(speed: f32, reverse_dwell: f32, wheel_spin: f32, steer_turn: f32) -> Self {
        Self { speed, reverse_dwell, wheel_spin, steer_turn }
    }

    pub const fn parked() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    fn advance(&mut self, pedals: &PedalInput, wheel_radius: f32, dt: f32) -> f32 {
        if pedals.handbrake {
            self.reverse_dwell = 0.0;
            if self.speed > 0.0 {
                self.speed = (self.speed - BRAKE_DECEL * dt).max(0.0);
            } else {
                self.speed = (self.speed + BRAKE_DECEL * dt).min(0.0);
            }
        } else if pedals.throttle > 0.1 {
            self.reverse_dwell = 0.0;
            if self.speed < 0.0 {
                self.speed = (self.speed + BRAKE_DECEL * dt).min(0.0);
            } else {
                self.speed = (self.speed + THROTTLE_ACCEL * dt).min(MAX_DRIVE_SPEED);
            }
        } else if pedals.throttle < -0.1 {
            if self.speed > 0.0 {
                self.speed = (self.speed - BRAKE_DECEL * dt).max(0.0);
                self.reverse_dwell = 0.0;
            } else if self.speed == 0.0 {
                self.reverse_dwell += dt;
                if self.reverse_dwell >= REVERSE_ENGAGE_DWELL {
                    self.speed = (self.speed - REVERSE_ACCEL * dt).max(-MAX_REVERSE_SPEED);
                }
            } else {
                self.speed = (self.speed - REVERSE_ACCEL * dt).max(-MAX_REVERSE_SPEED);
            }
        } else {
            self.reverse_dwell = 0.0;
            if self.speed > 0.0 {
                self.speed = (self.speed - COAST_DECEL * dt).max(0.0);
            } else {
                self.speed = (self.speed + COAST_DECEL * dt).min(0.0);
            }
        }
        let target_steer = if pedals.steer.abs() > 0.1 { pedals.steer.signum() } else { 0.0 };
        self.steer_turn += (target_steer - self.steer_turn) * (14.0 * dt).clamp(0.0, 1.0);
        self.wheel_spin += (self.speed * dt) / wheel_radius.max(0.12);
        target_steer
    }
}

/// Active driven vehicle state encapsulating the multi-block car assembly and kinematics.
pub struct DrivenCar {
    assembly: VehicleAssembly,
    kinematics: DriveKinematics,
}

impl DrivenCar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(assembly: VehicleAssembly, kinematics: DriveKinematics) -> Self {
        Self { assembly, kinematics }
    }

    pub const fn boarded(assembly: VehicleAssembly) -> Self {
        Self::new(assembly, DriveKinematics::parked())
    }

    pub fn body_rotation(&self) -> Quat {
        let up = self.assembly.normal().normalize_or_zero();
        let raw_fwd = self.assembly.heading();
        let fwd = (raw_fwd - up * raw_fwd.dot(up)).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        Quat::from_mat3(&Mat3::from_cols(right, up, -fwd)).normalize()
    }

    pub const fn steer_turn(&self) -> f32 {
        self.kinematics.steer_turn
    }

    pub fn seat_position(&self, world: &dyn PlanetaryWorld) -> Vec3 {
        let origin = self.assembly.origin();
        let up = world.surface_up_at(origin);
        let ground = world.constrain_to_surface(origin + up * 4.0, SEAT_EYE_SUPPORT);
        ground + up * (0.12 + self.assembly.seat_floor_offset())
    }

    pub fn step(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> (Vec3, f32) {
        let mut pedals = PedalInput::idle();
        controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
        let wheel_r = self.assembly.wheel_radius();
        let sign = self.kinematics.advance(&pedals, wheel_r, dt);
        let mut steer_px = 0.0;
        if sign.abs() > 0.1 {
            let rev_flip = if self.kinematics.speed < -0.05 { -1.0 } else { 1.0 };
            let yaw_delta = -sign * rev_flip * STEER_RATE * dt;
            self.assembly.steer(yaw_delta);
            steer_px = (sign * rev_flip * STEER_RATE * dt) / 0.003;
        }
        let cur_origin = self.assembly.origin();
        let next_raw = cur_origin + self.assembly.heading() * (self.kinematics.speed * dt);
        let up = world.surface_up_at(next_raw);
        let next_ground = world.constrain_to_surface(next_raw + up * 4.0, 0.0);
        self.assembly.relocate(next_ground, up);
        (next_ground + up * (SEAT_EYE_SUPPORT + 0.12 + self.assembly.seat_floor_offset()), steer_px)
    }

    pub fn into_assembly(self) -> VehicleAssembly {
        self.assembly
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.assembly.emit_driven(self.kinematics.wheel_spin, self.kinematics.steer_turn, target);
    }
}

/// Polymorphic avatar driving state avoiding `Option<DrivenCar>`.
pub enum VehiclePilot {
    Pedestrian,
    Seated(DrivenCar),
}

impl VehiclePilot {
    pub const fn driving(&self) -> bool { matches!(self, Self::Seated(_)) }

    pub fn pose_state(&self) -> Option<(Quat, f32)> {
        match self {
            Self::Pedestrian => None,
            Self::Seated(car) => Some((car.body_rotation(), car.steer_turn())),
        }
    }

    pub fn dismount(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if let Self::Seated(car) = std::mem::replace(self, Self::Pedestrian) {
            let _ = world.park_vehicle(car.into_assembly());
            true
        } else {
            false
        }
    }

    pub fn step(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> Option<(Vec3, f32)> {
        match self {
            Self::Pedestrian => None,
            Self::Seated(car) => Some(car.step(dt, controls, world)),
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if let Self::Seated(car) = self {
            car.emit(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::{Quat, Vec3};
    use winit::keyboard::KeyCode;
    use world::{BiomedCubeWorld, BlockMedium, BlockMold, MeshTarget, PlanetaryWorld, Quad, SurfaceFrame};
    use crate::attire::{MartialStyle, StrideRhythm};
    use crate::avatar::{Avatar, PlanetaryAvatar};
    use crate::controls::{KeyMotion, KeyboardControls};
    use crate::posture::AvatarPosture;

    struct VertCollector(Vec<Vec3>);
    impl MeshTarget for VertCollector {
        fn push_quad(&mut self, quad: Quad) {
            quad.transfer(|v0, v1, v2, v3, _, _| self.0.extend_from_slice(&[v0, v1, v2, v3]));
        }
    }

    #[test]
    fn test_wheel_types_spin_reverse_after_brake_and_seated_head_and_hands() {
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let m0 = BlockMold::cube().in_medium(BlockMedium::CarWheel);
        let radii = [
            m0.shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
            m0.next_form().shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
            m0.next_form().next_form().shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
            m0.next_form().next_form().next_form().shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
        ];
        assert!(radii[0] != radii[1] && radii[1] != radii[2] && radii[2] != radii[3] && radii[0] != radii[2]);

        let mut world = BiomedCubeWorld::beveled(32, 1.0);
        let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
        let rig_mold = BlockMold::cube().in_medium(BlockMedium::CantileverRig);
        let eng_mold = BlockMold::cube().turn_clockwise().in_medium(BlockMedium::CarEngine);
        let seat_mold = BlockMold::cube().in_medium(BlockMedium::CabinSeat);
        let wheel_mold = m0.next_form().next_form();
        let av_pos = Vec3::new(0.25, g0, 1.75);
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), Vec3::new(0.0, -0.8, -0.6).normalize(), av_pos, rig_mold));
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), Vec3::new(0.0, -0.8, -0.6).normalize(), av_pos, eng_mold));
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, av_pos, seat_mold));
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av_pos, m0.next_form()));
        assert_eq!(world.harvest_surface(Vec3::new(0.75, g0 + 1.80, 1.25), Vec3::NEG_Z), Some(BlockMedium::CarWheel));
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av_pos, wheel_mold));
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 2.35, 0.75), Vec3::NEG_Y, av_pos, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)));
        assert!((world.constrain_to_surface(Vec3::new(1.25, 22.0, 1.25), 0.0).y - (g0 + 2.0)).abs() < 1e-4);
        assert!((world.constrain_to_surface(Vec3::new(1.25, 22.0, 0.75), 0.0).y - (g0 + 2.0)).abs() < 1e-4);

        let mut crane_op = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 22.0, 1.25), 1.0));
        crane_op.rotate_degrees(0.0, 35.0);
        assert!(crane_op.use_right_hand(&mut world));

        let mut driver = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.25, 22.0, 1.75), 1.0));
        driver.rotate_degrees(0.0, 58.0);
        assert!(driver.use_right_hand(&mut world));

        let view_fwd = |a: &PlanetaryAvatar| {
            let inv = a.view_projection_matrix(1.0).inverse();
            (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - inv.project_point3(Vec3::ZERO)).normalize()
        };
        let initial_dir = view_fwd(&driver);
        driver.rotate_degrees(240.0, 0.0);
        assert!(view_fwd(&driver).dot(initial_dir) > 0.10);
        let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        let start_x = cam_pos(&driver).x;
        let (mut verts_t0, mut verts_t1) = (VertCollector(Vec::new()), VertCollector(Vec::new()));
        driver.emit_appearance(&mut verts_t0);
        let mut gas = KeyMotion::idle();
        gas.register_key(KeyCode::KeyW, true);
        for _ in 0..20 { driver.step_motion(1.0 / 60.0, &gas, &world); }
        assert!(cam_pos(&driver).x > start_x + 0.5);
        driver.emit_appearance(&mut verts_t1);
        assert_ne!(verts_t0.0, verts_t1.0);
        let mut hold_s = KeyMotion::idle();
        hold_s.register_key(KeyCode::KeyS, true);
        for _ in 0..18 { driver.step_motion(1.0 / 60.0, &hold_s, &world); }
        let braked_x = cam_pos(&driver).x;
        for _ in 0..35 { driver.step_motion(1.0 / 60.0, &hold_s, &world); }
        assert!(cam_pos(&driver).x < braked_x - 0.35);
        let posture = AvatarPosture::crouching();
        let pose_c = posture.body_pose(Vec3::ZERO, Quat::IDENTITY, Quat::IDENTITY, StrideRhythm::idle()).with_surface_rotation(Quat::IDENTITY).with_seated_steer(0.0);
        let pose_l = posture.body_pose(Vec3::ZERO, Quat::IDENTITY, Quat::from_rotation_y(0.9), StrideRhythm::idle()).with_surface_rotation(Quat::IDENTITY).with_seated_steer(0.0);
        assert_eq!(pose_c.torso_frame().point(0.2, 0.5, 0.3), pose_l.torso_frame().point(0.2, 0.5, 0.3));
        assert_eq!(pose_c.leg_joints(1.0), pose_l.leg_joints(1.0));
        assert_ne!(pose_c.crown_frame().point(0.0, 0.1, 0.1), pose_l.crown_frame().point(0.0, 0.1, 0.1));

        let (pose_left, pose_right) = (pose_c.with_seated_steer(-1.0), pose_c.with_seated_steer(1.0));
        let r_left = pose_left.arm(1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0);
        let r_right = pose_right.arm(1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0);
        let l_left = pose_left.arm(-1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0);
        let l_right = pose_right.arm(-1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0);
        assert!(r_left.y > r_right.y + 0.08 && l_right.y > l_left.y + 0.08);
    }
}
