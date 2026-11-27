use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use crate::relief::{CraftIdentity, CraftPose, RigArm, SanctuaryPlinth, VehicleAxes};

/// Rigid 6-DOF kinematics of a `GridBody` (`4` strictly private fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyMotion {
    pose: CraftPose,
    linear_vel: Vec3,
    angular_vel: Vec3,
    proper_accel: Vec3,
}

impl BodyMotion {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        pose: CraftPose,
        linear_vel: Vec3,
        angular_vel: Vec3,
        proper_accel: Vec3,
    ) -> Self {
        Self { pose, linear_vel, angular_vel, proper_accel }
    }

    pub const fn stationary(pose: CraftPose) -> Self {
        Self::new(pose, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO)
    }

    pub const fn planetary_identity() -> Self {
        let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
        Self::stationary(CraftPose::parked(Vec3::ZERO, axes, Vec3::Z))
    }

    pub const fn pose(self) -> CraftPose {
        self.pose
    }

    pub const fn linear_vel(self) -> Vec3 {
        self.linear_vel
    }

    pub const fn angular_vel(self) -> Vec3 {
        self.angular_vel
    }

    pub const fn proper_accel(self) -> Vec3 {
        self.proper_accel
    }

    pub const fn with_pose(self, pose: CraftPose) -> Self {
        Self::new(pose, self.linear_vel, self.angular_vel, self.proper_accel)
    }

    pub const fn with_velocity(self, linear_vel: Vec3, angular_vel: Vec3) -> Self {
        Self::new(self.pose, linear_vel, angular_vel, self.proper_accel)
    }

    pub fn with_proper_accel(self, proper_accel: Vec3) -> Self {
        let burn_ratio = (proper_accel.dot(self.pose.normal()) / 9.81).clamp(0.0, 2.5);
        let next_pose = if burn_ratio > 0.02 {
            self.pose.with_motion(burn_ratio, self.pose.steer_turn())
        } else {
            self.pose
        };
        Self::new(next_pose, self.linear_vel, self.angular_vel, proper_accel)
    }

    pub fn shifted_by(self, pos_shift: Vec3, next_vel: Vec3) -> Self {
        Self::new(
            self.pose.with_origin(self.pose.origin() + pos_shift),
            next_vel,
            self.angular_vel,
            self.proper_accel,
        )
    }

    /// Integrates 6-DOF translation and rotation over `dt` under `gravitational_dv` (`g * dt`).
    pub fn step_six_dof(self, dt: f32, gravitational_dv: Vec3) -> Self {
        if dt <= 1e-7 {
            return self;
        }
        let next_pose_rot = if self.angular_vel.length_squared() > 1e-10 {
            self.pose.rotated(Quat::from_scaled_axis(self.angular_vel * dt))
        } else {
            self.pose
        };
        let burn_world = if self.proper_accel.length_squared() > 1e-8 {
            let rot_delta = next_pose_rot.hull_rotation() * self.pose.hull_rotation().inverse();
            rot_delta * self.proper_accel
        } else {
            Vec3::ZERO
        };
        let dv = burn_world * dt + gravitational_dv;
        let next_vel = self.linear_vel + dv;
        let next_origin = self.pose.origin() + (self.linear_vel + next_vel) * (0.5 * dt);
        Self::new(next_pose_rot.with_origin(next_origin), next_vel, self.angular_vel, burn_world)
    }
}

/// Rigid parent-relative attachment coupling a docked `GridBody` to a host `GridBody` (`3` private fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockedLink {
    host: CraftIdentity,
    local_offset: Vec3,
    local_rot: Quat,
}

impl DockedLink {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(host: CraftIdentity, local_offset: Vec3, local_rot: Quat) -> Self {
        Self { host, local_offset, local_rot }
    }

    pub const fn host(self) -> CraftIdentity {
        self.host
    }

    pub fn synced_motion(self, host_motion: BodyMotion, current_pose: CraftPose) -> BodyMotion {
        let host_pose = host_motion.pose();
        let world_origin = host_pose.world_point_at(self.local_offset);
        let target_rot = (host_pose.hull_rotation() * self.local_rot).normalize();
        let delta_rot = target_rot * current_pose.hull_rotation().inverse();
        let synced_pose = current_pose.rotated(delta_rot).with_origin(world_origin);
        let r = world_origin - host_pose.origin();
        let rim_vel = host_motion.angular_vel().cross(r);
        BodyMotion::new(
            synced_pose,
            host_motion.linear_vel() + rim_vel,
            host_motion.angular_vel(),
            host_motion.proper_accel(),
        )
    }
}

/// Polymorphic physical anchoring state of a `GridBody` (`0` `Option` fields).
#[derive(Clone, Copy, Debug)]
pub enum BodyMooring {
    CrustAnchored(SanctuaryPlinth),
    MagneticFlange(RigArm),
    SurfaceSettled(SurfaceFrame),
    FreeFlight,
    DockedPort(DockedLink),
}

impl BodyMooring {
    pub const fn crust_anchored(plinth: SanctuaryPlinth) -> Self {
        Self::CrustAnchored(plinth)
    }

    pub const fn free_flight() -> Self {
        Self::FreeFlight
    }

    pub const fn free_moving(self) -> bool {
        matches!(self, Self::FreeFlight)
    }

    pub const fn docked_link(self) -> Option<DockedLink> {
        match self {
            Self::DockedPort(link) => Some(link),
            _ => None,
        }
    }
}
