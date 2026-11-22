use glam::Vec3;
use crate::relief::vehicle_assembly::{CockpitAxes, PilotOrder};
use super::super::pose::CraftPose;
use super::CraftBlueprint;

pub const MAX_WIRE_CELLS: usize = 24;
pub const MAX_EXHAUST_JETS: usize = 16;

/// Propulsion or structural silhouette role inside `HullWireframe`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellRole {
    Fuselage,
    NoseApex,
    MainBell,
    ActiveMainBell,
    FusionBell,
    ActiveFusionBell,
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

/// Quantized 3D ship-local station center, half-extents, and `CellRole`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireframeCell {
    locus: [i8; 3],
    span: [u8; 3],
    role: CellRole,
}

impl WireframeCell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(locus: [i8; 3], span: [u8; 3], role: CellRole) -> Self {
        Self { locus, span, role }
    }

    pub const fn vacant() -> Self {
        Self::new([0; 3], [16; 3], CellRole::Fuselage)
    }

    pub fn prism(local_center: Vec3, local_half: Vec3, scale: f32, role: CellRole) -> Self {
        let ql = |v: f32| ((v * scale).clamp(-1.15, 1.15) * 100.0).round().clamp(-120.0, 120.0) as i8;
        let qs = |v: f32| ((v * scale).clamp(0.06, 1.00) * 100.0).round().clamp(6.0, 100.0) as u8;
        Self::new(
            [ql(local_center.x), ql(local_center.y), ql(local_center.z)],
            [qs(local_half.x), qs(local_half.y), qs(local_half.z)],
            role,
        )
    }

    fn unpack(self) -> (CellRole, Vec3, Vec3) {
        let pos = Vec3::new(
            f32::from(self.locus[0]) * 0.01,
            f32::from(self.locus[1]) * 0.01,
            f32::from(self.locus[2]) * 0.01,
        );
        let half = Vec3::new(
            f32::from(self.span[0]) * 0.01,
            f32::from(self.span[1]) * 0.01,
            f32::from(self.span[2]) * 0.01,
        );
        (self.role, pos, half)
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
        let qm = |v: f32| ((v * scale).clamp(-1.24, 1.24) * 100.0).round().clamp(-125.0, 125.0) as i8;
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

/// Zero-allocation 3D wireframe hull envelope, discrete thruster stations, and `STEER` vector.
#[derive(Clone, Copy, Debug)]
pub struct HullWireframe {
    cells: [WireframeCell; MAX_WIRE_CELLS],
    jets: [ExhaustJet; MAX_EXHAUST_JETS],
    cue: SteerCue,
}

impl HullWireframe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        cells: [WireframeCell; MAX_WIRE_CELLS],
        jets: [ExhaustJet; MAX_EXHAUST_JETS],
        cue: SteerCue,
    ) -> Self {
        Self { cells, jets, cue }
    }

    pub const fn silent() -> Self {
        Self::new(
            [WireframeCell::vacant(); MAX_WIRE_CELLS],
            [ExhaustJet::vacant(); MAX_EXHAUST_JETS],
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

        let mut all_min = Vec3::splat(f32::INFINITY);
        let mut all_max = Vec3::splat(f32::NEG_INFINITY);
        let mut core_min = Vec3::splat(f32::INFINITY);
        let mut core_max = Vec3::splat(f32::NEG_INFINITY);

        for part in blueprint.parts() {
            let p = axes.to_local(part.world_mass_center(lift, &car_frame));
            all_min = all_min.min(p - Vec3::splat(0.25));
            all_max = all_max.max(p + Vec3::splat(0.25));
            if !part.attitude_pod() && !part.main_bell() {
                core_min = core_min.min(p - Vec3::splat(0.24));
                core_max = core_max.max(p + Vec3::splat(0.24));
            }
        }
        let hull_mid = (all_min + all_max) * 0.5;
        let max_span = ((all_max - all_min).max_element() * 0.5).max(0.60);
        let inv_span = 1.0 / max_span;

        let mut cell_buf = ([WireframeCell::vacant(); MAX_WIRE_CELLS], 0usize);
        let mut jet_buf = ([ExhaustJet::vacant(); MAX_EXHAUST_JETS], 0usize);

        if core_min.x <= core_max.x {
            let c_mid = (core_min + core_max) * 0.5 - hull_mid;
            let c_half = ((core_max - core_min) * 0.5).max(Vec3::splat(0.20));
            cell_buf.0[0] = WireframeCell::prism(c_mid, c_half, inv_span, CellRole::Fuselage);
            cell_buf.1 = 1;
        }

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
        let (raw_omega, _) = imp.integrate_attitude(Vec3::ZERO, order.guidance(), 1.0);
        let omega_step = if order.guidance_engaged() {
            let cmd_dir = demands.1.normalize_or_zero();
            if cmd_dir.length_squared() > 0.5 {
                cmd_dir * raw_omega.dot(cmd_dir).max(0.0)
            } else {
                Vec3::ZERO
            }
        } else {
            raw_omega
        };
        let up_w = axes.to_world(Vec3::Y);
        let fwd_w = axes.to_world(Vec3::Z);
        let swing_w = omega_step.cross(up_w) + omega_step.cross(fwd_w) * 0.85;
        let lin_mag = lin_dv.length();
        let lin_cue = if lin_mag > 1e-4 {
            (lin_dv / lin_mag) * (lin_mag / 3.5).clamp(0.0, 1.0)
        } else {
            Vec3::ZERO
        };
        let raw_steer = axes.to_local(lin_cue + swing_w * 0.55);
        let steer_mag = raw_steer.length();
        let steer_local = if steer_mag > 1.0 {
            raw_steer / steer_mag
        } else if steer_mag < 0.03 {
            Vec3::ZERO
        } else {
            raw_steer
        };

        let cue = SteerCue::new(axes, steer_local, (cell_buf.1 as u8, jet_buf.1 as u8));
        Self::new(cell_buf.0, jet_buf.0, cue)
    }

    pub fn visit_cells(&self, mut visitor: impl FnMut(CellRole, Vec3, Vec3)) {
        let len = (self.cue.counts.0 as usize).min(MAX_WIRE_CELLS);
        for cell in &self.cells[..len] {
            let (role, pos, half) = cell.unpack();
            visitor(role, pos, half);
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
