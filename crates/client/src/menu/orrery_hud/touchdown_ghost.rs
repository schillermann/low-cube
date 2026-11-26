use std::f32::consts::TAU;
use glam::{Vec2, Vec3};
use world::MeshTarget;
use crate::drive::{TOUCHDOWN_NODES, TouchdownFootprint, TouchdownTrace};
use super::overlays::OpticOverlay;
use super::sight_stage::{BELLY_GUARD_RADIUS, HudSightStage, WorldSight};

/// `OpticOverlay` decorator projecting the 3D ballistic descent trail and terrain touchdown ring (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct TouchdownGhostOverlay<O> {
    origin: O,
    stage: HudSightStage,
    trace: TouchdownTrace,
}

impl<O> TouchdownGhostOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: O, stage: HudSightStage, trace: TouchdownTrace) -> Self {
        Self { origin, stage, trace }
    }

    fn emit_ballistic_trail(&self, trail: &[Vec3; TOUCHDOWN_NODES], color: Vec3, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        for idx in 1..(TOUCHDOWN_NODES - 1) {
            let a_world = trail[idx];
            let b_world = trail[idx + 1];
            if (b_world - a_world).length_squared() <= 1e-4 {
                continue;
            }
            self.stage.visit_front_ndc(a_world, |a_ndc| {
                self.stage.visit_front_ndc(b_world, |b_ndc| {
                    let mid = a_ndc.lerp(b_ndc, 0.62);
                    pen.emit_beam(a_ndc, mid, 0.0020, color, 0.96, target);
                });
            });
        }
    }

    fn emit_surface_footprint(&self, fp: &TouchdownFootprint, color: Vec3, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        fp.transfer(|trail, surf_up, _, radius| {
            let impact = trail[TOUCHDOWN_NODES - 1];
            let up = surf_up.normalize_or_zero();
            let ref_axis = if up.y.abs() < 0.90 { Vec3::Y } else { Vec3::X };
            let t1 = up.cross(ref_axis).normalize_or_zero();
            let t2 = up.cross(t1).normalize_or_zero();
            let segs = 16usize;
            for i in 0..segs {
                let a0 = (i as f32) * TAU / (segs as f32);
                let a1 = ((i + 1) as f32) * TAU / (segs as f32);
                let w0 = impact + (t1 * a0.cos() + t2 * a0.sin()) * radius + up * 0.04;
                let w1 = impact + (t1 * a1.cos() + t2 * a1.sin()) * radius + up * 0.04;
                self.stage.visit_front_ndc(w0, |p0| {
                    self.stage.visit_front_ndc(w1, |p1| {
                        pen.emit_beam(p0, p1, 0.0024, color, 0.98, target);
                    });
                });
            }
            let inner = radius * 0.42;
            for axis in [t1, t2] {
                let w0 = impact - axis * inner + up * 0.04;
                let w1 = impact + axis * inner + up * 0.04;
                self.stage.visit_front_ndc(w0, |p0| {
                    self.stage.visit_front_ndc(w1, |p1| {
                        pen.emit_beam(p0, p1, 0.0018, color, 0.96, target);
                    });
                });
            }
            let spire_top = impact + up * (radius * 0.95);
            self.stage.visit_front_ndc(impact + up * 0.04, |base_ndc| {
                self.stage.visit_front_ndc(spire_top, |top_ndc| {
                    pen.emit_beam(base_ndc, top_ndc, 0.0019, color, 0.94, target);
                });
            });
        });
    }
}

impl<O: OpticOverlay> OpticOverlay for TouchdownGhostOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        let pen = self.stage.pen();
        self.trace.visit_predicted(|fp| {
            let tone = fp.classify_tone();
            let color = tone.color();
            fp.transfer(|trail, _, _, _| {
                self.emit_ballistic_trail(trail, color, target);
                let impact = trail[TOUCHDOWN_NODES - 1];
                match self.stage.project_world(impact, BELLY_GUARD_RADIUS) {
                    WorldSight::OnScreen(ndc) => {
                        self.emit_surface_footprint(fp, color, target);
                        pen.emit_ring(ndc, 0.016, 12, 0.0018, color, 0.96, target);
                        let label_pt = pen.step_ndc(ndc, Vec2::new(0.0, -0.036));
                        pen.emit_caption(tone.badge_label(), label_pt, (0.0038, 0.0056), color, target);
                    }
                    WorldSight::OffScreen { rim_point, outward } => {
                        pen.emit_rim_chevron(rim_point, outward, 0.026, color, target);
                        let badge = pen.step_ndc(rim_point, -outward * 0.024);
                        pen.emit_ring(badge, 0.013, 10, 0.0019, color, 0.96, target);
                        let label_pt = pen.step_ndc(rim_point, Vec2::new(0.0, -0.036));
                        pen.emit_caption(tone.badge_label(), label_pt, (0.0038, 0.0056), color, target);
                    }
                }
            });
        });
    }
}
