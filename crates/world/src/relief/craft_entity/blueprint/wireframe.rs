use glam::Vec3;
use crate::relief::vehicle_assembly::{CockpitAxes, PilotOrder};
use super::super::pose::CraftPose;
use super::contour::HullContour;
use super::wire_primitives::{
    CellRole, ExhaustJet, JetTone, WireframeCell, WireframeEdge, MAX_EXHAUST_JETS,
    MAX_WIRE_CELLS, MAX_WIRE_EDGES,
};
use super::CraftBlueprint;

/// Encapsulated ship basis, net 3D steering effect vector, and populated `(edge, cell, jet)` counts.
#[derive(Clone, Copy, Debug)]
pub struct SteerCue {
    axes: CockpitAxes,
    vector: Vec3,
    counts: (u8, u8, u8),
}

impl SteerCue {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(axes: CockpitAxes, vector: Vec3, counts: (u8, u8, u8)) -> Self {
        Self { axes, vector, counts }
    }

    pub const fn silent() -> Self {
        Self::new(CockpitAxes::new(Vec3::X, Vec3::Y, Vec3::Z), Vec3::ZERO, (0, 0, 0))
    }
}

/// Zero-allocation 3D merged outer-hull crease wireframe, discrete thruster stations, and `STEER` vector.
#[derive(Clone, Copy, Debug)]
pub struct HullWireframe {
    edges: [WireframeEdge; MAX_WIRE_EDGES],
    cells: [WireframeCell; MAX_WIRE_CELLS],
    jets: [ExhaustJet; MAX_EXHAUST_JETS],
    cue: SteerCue,
}

impl HullWireframe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        edges: [WireframeEdge; MAX_WIRE_EDGES],
        cells: [WireframeCell; MAX_WIRE_CELLS],
        jets: [ExhaustJet; MAX_EXHAUST_JETS],
        cue: SteerCue,
    ) -> Self {
        Self { edges, cells, jets, cue }
    }

    pub const fn silent() -> Self {
        Self::new(
            [WireframeEdge::vacant(); MAX_WIRE_EDGES],
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

        let mut solids = Vec::with_capacity(blueprint.parts().len());
        let mut all_min = Vec3::splat(f32::INFINITY);
        let mut all_max = Vec3::splat(f32::NEG_INFINITY);

        for part in blueprint.parts() {
            part.push_hull_solid(lift, &mut solids);
            let p = axes.to_local(part.world_mass_center(lift, &car_frame));
            all_min = all_min.min(p - Vec3::splat(0.25));
            all_max = all_max.max(p + Vec3::splat(0.25));
        }

        let contour = HullContour::from_solids(&solids);
        contour.visit_segments(|a, b| {
            for q in [a, b] {
                let p = axes.to_local(car_frame.point_at(q.x, q.z, q.y));
                all_min = all_min.min(p);
                all_max = all_max.max(p);
            }
        });

        let hull_mid = (all_min + all_max) * 0.5;
        let max_span = ((all_max - all_min).max_element() * 0.5).max(0.60);
        let inv_span = 1.0 / max_span;

        let mut edge_buf = ([WireframeEdge::vacant(); MAX_WIRE_EDGES], 0usize);
        contour.visit_segments(|a, b| {
            if edge_buf.1 < MAX_WIRE_EDGES {
                let al = axes.to_local(car_frame.point_at(a.x, a.z, a.y)) - hull_mid;
                let bl = axes.to_local(car_frame.point_at(b.x, b.z, b.y)) - hull_mid;
                edge_buf.0[edge_buf.1] = WireframeEdge::segment(al, bl, inv_span);
                edge_buf.1 += 1;
            }
        });

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

        let cue = SteerCue::new(
            axes,
            steer_local,
            (edge_buf.1 as u8, cell_buf.1 as u8, jet_buf.1 as u8),
        );
        Self::new(edge_buf.0, cell_buf.0, jet_buf.0, cue)
    }

    pub fn visit_edges(&self, mut visitor: impl FnMut(Vec3, Vec3)) {
        let len = (self.cue.counts.0 as usize).min(MAX_WIRE_EDGES);
        for edge in &self.edges[..len] {
            let (a, b) = edge.unpack();
            visitor(a, b);
        }
    }

    pub fn visit_cells(&self, mut visitor: impl FnMut(CellRole, Vec3, Vec3)) {
        let len = (self.cue.counts.1 as usize).min(MAX_WIRE_CELLS);
        for cell in &self.cells[..len] {
            let (role, pos, half) = cell.unpack();
            visitor(role, pos, half);
        }
    }

    pub fn visit_jets(&self, mut visitor: impl FnMut(JetTone, Vec3, Vec3, f32)) {
        let len = (self.cue.counts.2 as usize).min(MAX_EXHAUST_JETS);
        for jet in &self.jets[..len] {
            let (tone, mouth, dir, thr) = jet.unpack();
            visitor(tone, mouth, dir, thr);
        }
    }

    /// Transfers `(ship_axes, local_steer_vector, exhaust_active)` to the HUD without getters.
    pub fn transfer_steer<R>(&self, f: impl FnOnce(CockpitAxes, Vec3, bool) -> R) -> R {
        f(self.cue.axes, self.cue.vector, self.cue.counts.2 > 0)
    }
}
