use glam::{Mat4, Quat, Vec3};
use world::MeshTarget;
use super::limb::{FacetedSolid, TaperedPrism};
use super::pose::BodyPose;
use super::rhythm::StrideRhythm;
use super::YouthAttire;

/// 3D Intercom flight-comms bust composing `YouthAttire` upper-body garments, crown, and cyan headset.
#[derive(Clone, Copy, Debug)]
pub struct IntercomBust {
    attire: YouthAttire,
    head_angles: (f32, f32),
}

impl IntercomBust {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(attire: YouthAttire, head_angles: (f32, f32)) -> Self {
        Self { attire, head_angles }
    }

    pub fn emit(self, aspect: f32, target: &mut dyn MeshTarget) -> Mat4 {
        let (head_yaw, head_pitch) = self.head_angles;
        let view_rot = Quat::from_rotation_y(head_yaw.clamp(-0.52, 0.52))
            * Quat::from_rotation_x(head_pitch.clamp(-0.36, 0.36));
        let base_pose = BodyPose::new(
            Vec3::new(0.0, 0.98, 0.0),
            Quat::IDENTITY,
            view_rot,
            0.80,
            StrideRhythm::idle(),
            super::YouthPhysique::SlenderSquire,
        )
        .with_seated(1.0);
        let scaled = self.attire.emit_upper_bust(&base_pose, target);

        let crown = scaled.crown_frame();
        let hs = |y: f32| scaled.crown_size(y);
        let hw = |r: f32| scaled.crown_span(r);
        let cyan = Vec3::new(0.26, 0.88, 1.00);
        let slate = Vec3::new(0.16, 0.22, 0.30);

        TaperedPrism::hexagonal(
            Vec3::new(hw(0.092), hs(0.094), hw(-0.002)),
            Vec3::new(hw(0.114), hs(0.094), hw(-0.002)),
            hw(0.028),
            hw(0.022),
            cyan,
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
            slate,
        )
        .emit(&crown, true, target);
        TaperedPrism::hexagonal(
            Vec3::new(hw(0.046), hs(0.028), hw(0.096)),
            Vec3::new(hw(0.032), hs(0.028), hw(0.102)),
            hw(0.010),
            hw(0.009),
            cyan,
        )
        .emit(&crown, true, target);

        let neck = scaled.neck_frame();
        let focus = neck.point(0.0, hs(0.075), 0.02);
        let eye = neck.point(0.0, hs(0.092), 0.58);
        let up = (neck.point(0.0, 1.0, 0.0) - neck.point(0.0, 0.0, 0.0)).normalize_or_zero();
        Mat4::perspective_rh(34.0_f32.to_radians(), aspect.max(0.5), 0.08, 4.0)
            * Mat4::look_at_rh(eye, focus, up)
    }
}
