use glam::Vec3;
use crate::relief::vehicle_assembly::{CockpitAxes, PilotOrder};
use super::super::pose::CraftPose;
use super::CraftBlueprint;

pub const MAX_WIRE_CELLS: usize = 24;
pub const MAX_EXHAUST_JETS: usize = 16;

/// Structural or propulsion wireframe role of a voxel block inside `HullWireframe`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellRole {
    HullCube,
    NoseApex,
    MainBell,
    FusionBell,
    AttitudePod,
    ActivePod,
}

/// Chromatic exhaust family distinguishing RCS vernier gas, chemical bells, and fusion plasma.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JetTone {
    VernierCyan,
    BellAmber,
    FusionBlue,
}

/// Quantized 3D ship-local voxel block position and wireframe archetype.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireframeCell {
    locus: [i8; 3],
    role: CellRole,
}

impl WireframeCell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(locus: [i8; 3], role: CellRole) -> Self {
        Self { locus, role }
    }

    pub const fn vacant() -> Self {
        Self::new([0; 3], CellRole::HullCube)
    }

    pub fn from_local(local: Vec3, scale: f32, role: CellRole) -> Self {
        let q = |v: f32| ((v * scale).clamp(-1.0, 1.0) * 100.0).round() as i8;
        Self::new([q(local.x), q(local.y), q(local.z)], role)
    }

    fn unpack(self) -> (CellRole, Vec3) {
        (
            self.role,
            Vec3::new(
                f32::from(self.locus[0]) * 0.01,
                f32::from(self.locus[1]) * 0.01,
                f32::from(self.locus[2]) * 0.01,
            ),
        )
    }
}

/// Quantized 3D nozzle mouth, ship-local exhaust unit vector, and firing throttle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExhaustJet {
    mouth: [i8; 3],
    dir: [i8; 3],
    intensity: (JetTone, u8),
}

impl ExhaustJet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mouth: [i8; 3], dir: [i8; 3], intensity: (JetTone, u8)) -> Self {
        Self { mouth, dir, intensity }
    }

    pub const fn vacant() -> Self {
        Self::new([0; 3], [0, -100, 0], (JetTone::VernierCyan, 0))
    }

    pub fn active(local_mouth: Vec3, scale: f32, local_dir: Vec3, tone: JetTone, throttle: f32) -> Self {
        let qm = |v: f32| ((v * scale).clamp(-1.2, 1.2) * 100.0).round().clamp(-125.0, 125.0) as i8;
        let u = local_dir.normalize_or_zero();
        let qd = |v: f32| (v.clamp(-1.0, 1.0) * 100.0).round() as i8;
        let thr = (throttle.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self::new(
            [qm(local_mouth.x), qm(local_mouth.y), qm(local_mouth.z)],
            [qd(u.x), qd(u.y), qd(u.z)],
            (tone, thr),
        )
    }

    fn unpack(self) -> (JetTone, Vec3, Vec3, f32) {
        let (tone, thr) = self.intensity;
        let m = Vec3::new(
            f32::from(self.mouth[0]) * 0.01,
            f32::from(self.mouth[1]) * 0.01,
            f32::from(self.mouth[2]) * 0.01,
        );
        let d = Vec3::new(
            f32::from(self.dir[0]) * 0.01,
            f32::from(self.dir[1]) * 0.01,
            f32::from(self.dir[2]) * 0.01,
        )
        .normalize_or_zero();
        (tone, m, d, f32::from(thr) / 255.0)
    }
}

/// Encapsulated ship basis, net 3D steering effect vector, and populated slice counts.
#[derive(Clone, Copy, Debug)]
pub struct SteerCue {
    axes: CockpitAxes,
    vector: Vec3,
    counts: (u8, u8),
}

impl SteerCue {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(axes: CockpitAxes, vector: Vec3, counts: (u8, u8)) -> Self {
        Self { axes, vector, counts }
    }

    pub const fn silent() -> Self {
        Self::new(CockpitAxes::new(Vec3::X, Vec3::Y, Vec3::Z), Vec3::ZERO, (0, 0))
    }
}

/// Zero-allocation 3D wireframe hull, live exhaust jets, and net 3D steering vector (`Ursache -> Wirkung`).
#[derive(Clone, Copy, Debug)]
pub struct HullWireframe {
    cells: [WireframeCell; MAX_WIRE_CELLS],
    jets: [ExhaustJet; MAX_EXHAUST_JETS],
    half_extent: f32,
    cue: SteerCue,
}

impl HullWireframe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        cells: [WireframeCell; MAX_WIRE_CELLS],
        jets: [ExhaustJet; MAX_EXHAUST_JETS],
        half_extent: f32,
        cue: SteerCue,
    ) -> Self {
        Self { cells, jets, half_extent, cue }
    }

    pub const fn silent() -> Self {
        Self::new(
            [WireframeCell::vacant(); MAX_WIRE_CELLS],
            [ExhaustJet::vacant(); MAX_EXHAUST_JETS],
            0.22,
            SteerCue::silent(),
        )
    }

    pub(super) fn sample(blueprint: &CraftBlueprint, order: &PilotOrder, pose: CraftPose) -> Self {
        if !blueprint.spacefaring() || blueprint.parts().is_empty() {
            return Self::silent();
        }
        let car_frame = pose.surface_frame();
        let lift = blueprint.wheel_ground_lift();
        let axes = CockpitAxes::from_pose(pose);
        let com = blueprint.mass_center(lift, &car_frame);
        let demands = blueprint.rcs_demands(order, pose, &car_frame);

        let (min_l, max_l) = blueprint.parts().iter().fold(
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
            |(lo, hi), part| {
                let p = axes.to_local(part.world_mass_center(lift, &car_frame));
                (lo.min(p - Vec3::splat(0.25)), hi.max(p + Vec3::splat(0.25)))
            },
        );
        let hull_mid = (min_l + max_l) * 0.5;
        let max_span = ((max_l - min_l).max_element() * 0.5).max(0.55);
        let inv_span = 1.0 / max_span;
        let cube_half = (0.24 * inv_span).clamp(0.09, 0.28);

        let mut cell_buf = ([WireframeCell::vacant(); MAX_WIRE_CELLS], 0usize);
        let mut jet_buf = ([ExhaustJet::vacant(); MAX_EXHAUST_JETS], 0usize);

        for part in blueprint.parts() {
            part.collect_wireframe(
                lift,
                &car_frame,
                axes,
                (com, hull_mid, inv_span),
                (order, demands.0, demands.1),
                (&mut cell_buf, &mut jet_buf),
            );
        }

        let imp = blueprint.sample_impulse(order, pose);
        let lin_dv = if blueprint.fusion_equipped() {
            imp.fusion_velocity_delta(1.0)
        } else {
            imp.velocity_delta(1.0)
        };
        let (omega_step, _) = imp.integrate_attitude(Vec3::ZERO, order.guidance(), 1.0);
        let up_w = axes.to_world(Vec3::Y);
        let fwd_w = axes.to_world(Vec3::Z);
        let swing_w = omega_step.cross(up_w) + omega_step.cross(fwd_w) * 0.85;
        let raw_steer = axes.to_local(lin_dv / 14.0 + swing_w * 0.72);
        let steer_mag = raw_steer.length();
        let steer_local = if steer_mag > 1.0 {
            raw_steer / steer_mag
        } else if steer_mag < 0.03 {
            Vec3::ZERO
        } else {
            raw_steer
        };

        let cue = SteerCue::new(axes, steer_local, (cell_buf.1 as u8, jet_buf.1 as u8));
        Self::new(cell_buf.0, jet_buf.0, cube_half, cue)
    }

    pub fn visit_cells(&self, mut visitor: impl FnMut(CellRole, Vec3, f32)) {
        let len = (self.cue.counts.0 as usize).min(MAX_WIRE_CELLS);
        for cell in &self.cells[..len] {
            let (role, pos) = cell.unpack();
            visitor(role, pos, self.half_extent);
        }
    }

    pub fn visit_jets(&self, mut visitor: impl FnMut(JetTone, Vec3, Vec3, f32)) {
        let len = (self.cue.counts.1 as usize).min(MAX_EXHAUST_JETS);
        for jet in &self.jets[..len] {
            let (tone, mouth, dir, thr) = jet.unpack();
            visitor(tone, mouth, dir, thr);
        }
    }

    /// Transfers `(ship_axes, local_steer_vector, exhaust_active)` to the HUD without getters.
    pub fn transfer_steer<R>(&self, f: impl FnOnce(CockpitAxes, Vec3, bool) -> R) -> R {
        f(self.cue.axes, self.cue.vector, self.cue.counts.1 > 0)
    }
}
