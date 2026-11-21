use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::super::cell::GridCell;
use super::super::wedge::WedgeHeading;

/// Single thruster horn station on a 5-way `RcsManifold` pod.
#[derive(Clone, Copy, Debug)]
pub struct RcsHorn {
    origin: Vec3,
    exhaust: Vec3,
    ref_up: Vec3,
    envelope: (f32, f32),
}

impl RcsHorn {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: Vec3, exhaust: Vec3, ref_up: Vec3, envelope: (f32, f32)) -> Self {
        Self { origin, exhaust, ref_up, envelope }
    }

    pub const fn origin(self) -> Vec3 {
        self.origin
    }

    pub const fn exhaust_dir(self) -> Vec3 {
        self.exhaust
    }

    pub fn thrust_dir(self) -> Vec3 {
        -self.exhaust
    }

    pub const fn ref_up(self) -> Vec3 {
        self.ref_up
    }

    pub const fn envelope(self) -> (f32, f32) {
        self.envelope
    }
}

/// 5-nozzle spacecraft RCS thruster manifold shared 1:1 between 3D mesh emission and rigid-body physics.
#[derive(Clone, Copy, Debug)]
pub struct RcsManifold {
    heading: WedgeHeading,
}

impl RcsManifold {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading) -> Self {
        Self { heading }
    }

    /// Resolves the 5 physical horn stations (`[Outward, Dorsal, Ventral, Port, Starboard]`) in `frame`.
    pub fn horns(self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame) -> [RcsHorn; 5] {
        let (uc, vc) = cell.center();
        let center = frame.point_at(uc, vc, floor_elev + 0.25);
        let up = frame.normal().normalize_or_zero();
        let out_dir = (-self.heading.forward_vector(frame)).normalize_or_zero();
        let side_dir = out_dir.cross(up).normalize_or_zero();

        let dorsal_ex = (up * 0.96 + out_dir * 0.28).normalize_or_zero();
        let ventral_ex = (-up * 0.96 + out_dir * 0.28).normalize_or_zero();
        let port_ex = (-side_dir * 0.96 + out_dir * 0.28).normalize_or_zero();
        let starboard_ex = (side_dir * 0.96 + out_dir * 0.28).normalize_or_zero();

        [
            RcsHorn::new(center + out_dir * 0.015, out_dir, up, (0.145, 0.064)),
            RcsHorn::new(center + up * 0.086 - out_dir * 0.042, dorsal_ex, side_dir, (0.132, 0.052)),
            RcsHorn::new(center - up * 0.086 - out_dir * 0.042, ventral_ex, side_dir, (0.132, 0.052)),
            RcsHorn::new(center - side_dir * 0.086 - out_dir * 0.042, port_ex, up, (0.132, 0.052)),
            RcsHorn::new(center + side_dir * 0.086 - out_dir * 0.042, starboard_ex, up, (0.132, 0.052)),
        ]
    }

    /// Evaluates `[0.0..=1.0]` valve throttles across all 5 horns from linear and `r × F` torque demand.
    pub fn resolve_throttles(
        self,
        cell: GridCell,
        floor_elev: f32,
        frame: &SurfaceFrame,
        center_of_mass: Vec3,
        linear_demand: Vec3,
        torque_demand: Vec3,
    ) -> [f32; 5] {
        let horns = self.horns(cell, floor_elev, frame);
        let up = frame.normal().normalize_or_zero();
        let out_dir = (-self.heading.forward_vector(frame)).normalize_or_zero();
        let side_dir = out_dir.cross(up).normalize_or_zero();

        let vert_lin = linear_demand.dot(up);
        let planar_lin = linear_demand - up * vert_lin;

        let mut raw = [
            (-out_dir).dot(planar_lin).clamp(0.0, 1.0),
            (-vert_lin).clamp(0.0, 1.0),
            vert_lin.clamp(0.0, 1.0),
            side_dir.dot(planar_lin).clamp(0.0, 1.0),
            (-side_dir).dot(planar_lin).clamp(0.0, 1.0),
        ];

        let rot_mag = torque_demand.length();
        if rot_mag > 1e-4 {
            let rot_dir = torque_demand / rot_mag;
            for (idx, horn) in horns.iter().enumerate() {
                let lever = horn.origin() - center_of_mass;
                let align = lever.cross(horn.thrust_dir()).dot(rot_dir);
                if align > 0.032 {
                    let rot_share = ((align / 0.30) * rot_mag).clamp(0.0, 1.0);
                    raw[idx] = (raw[idx] + rot_share).clamp(0.0, 1.0);
                }
            }
        }

        [
            raw[0],
            (raw[1] - raw[2]).max(0.0),
            (raw[2] - raw[1]).max(0.0),
            (raw[3] - raw[4]).max(0.0),
            (raw[4] - raw[3]).max(0.0),
        ]
    }
}
