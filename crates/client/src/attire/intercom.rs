use glam::{Mat4, Quat, Vec3};
use world::MeshTarget;
use super::crown::HeadSculpture;
use super::limb::{FacetedSolid, TaperedPrism};
use super::pose::BodyPose;
use super::rhythm::StrideRhythm;
use super::EvaAttire;

/// `HeadSculpture` decorator mounting a hexagonal flight-comms earcup and boom microphone onto any head.
#[derive(Clone, Copy, Debug)]
pub struct HeadsetHead<S> {
    origin: S,
    accent: Vec3,
    chassis: Vec3,
}

impl<S> HeadsetHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, accent: Vec3, chassis: Vec3) -> Self {
        Self { origin, accent, chassis }
    }

    pub const fn aviation(origin: S) -> Self {
        Self::new(origin, Vec3::new(0.26, 0.88, 1.00), Vec3::new(0.16, 0.22, 0.30))
    }
}

impl<S: HeadSculpture> HeadSculpture for HeadsetHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let crown = pose.crown_frame();
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);

        TaperedPrism::hexagonal(
            Vec3::new(hw(0.092), hs(0.094), hw(-0.002)),
            Vec3::new(hw(0.114), hs(0.094), hw(-0.002)),
            hw(0.028),
            hw(0.022),
            self.accent,
        )
        .emit(&crown, true, target);
        TaperedPrism::new(
            4,
            Vec3::new(hw(0.106), hs(0.080), hw(0.006)),
            Vec3::new(hw(0.044), hs(0.028), hw(0.098)),
            hw(0.006),
            hw(0.006),
            hw(0.005),
            hw(0.005),
            self.chassis,
        )
        .emit(&crown, true, target);
        TaperedPrism::hexagonal(
            Vec3::new(hw(0.046), hs(0.028), hw(0.096)),
            Vec3::new(hw(0.032), hs(0.028), hw(0.102)),
            hw(0.010),
            hw(0.009),
            self.accent,
        )
        .emit(&crown, true, target);
    }
}

/// 3D Intercom flight-comms bust composing `EvaAttire` upper-body pressure shell and `HeadsetHead` decorator.
#[derive(Clone, Copy, Debug)]
pub struct IntercomBust {
    attire: EvaAttire,
    head_angles: (f32, f32),
}

impl IntercomBust {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(attire: EvaAttire, head_angles: (f32, f32)) -> Self {
        Self { attire, head_angles }
    }

    pub fn emit(self, aspect: f32, target: &mut dyn MeshTarget) -> Mat4 {
        let (head_yaw, head_pitch) = self.head_angles;
        let view_rot = Quat::from_rotation_y(head_yaw.clamp(-0.52, 0.52))
            * Quat::from_rotation_x(head_pitch.clamp(-0.36, 0.36));
        let raw_pose = BodyPose::new(
            Vec3::new(0.0, 0.98, 0.0),
            Quat::IDENTITY,
            view_rot,
            0.80,
            StrideRhythm::idle(),
            super::CrewPhysique::OrbitalCadet,
        )
        .with_seated(1.0);
        let scaled = self.attire.emit_bust_with_headset(&raw_pose, target);

        let hs = |y: f32| scaled.crown_size(y);
        let hw = |r: f32| scaled.crown_span(r);
        let neck = scaled.neck_frame();
        let focus = neck.point(0.0, hs(0.072), 0.022);
        let eye = neck.point(hw(-0.135), hs(0.104), 0.545);
        let up = (neck.point(0.0, 1.0, 0.0) - neck.point(0.0, 0.0, 0.0)).normalize_or_zero();
        Mat4::perspective_rh(31.5_f32.to_radians(), aspect.max(0.5), 0.08, 4.0)
            * Mat4::look_at_rh(eye, focus, up)
    }
}
