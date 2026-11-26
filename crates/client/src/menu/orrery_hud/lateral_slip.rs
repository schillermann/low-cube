use glam::{Vec2, Vec3};
use world::MeshTarget;
use crate::drive::TouchdownTrace;
use super::overlays::OpticOverlay;
use super::sight_stage::{BELLY_GUARD_RADIUS, DirectionSight, HudSightStage};

const SLIP_CYAN: Vec3 = Vec3::new(0.34, 0.94, 1.00);
const SLIP_WARN: Vec3 = Vec3::new(1.00, 0.68, 0.24);

/// `OpticOverlay` decorator projecting horizontal retrograde/slip guidance in Belly-Camera (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct LateralSlipOverlay<O> {
    origin: O,
    stage: HudSightStage,
    trace: TouchdownTrace,
}

impl<O> LateralSlipOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: O, stage: HudSightStage, trace: TouchdownTrace) -> Self {
        Self { origin, stage, trace }
    }

    fn emit_slip_reticle(&self, center: Vec2, color: Vec3, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        let r = 0.018;
        pen.emit_ring(center, r, 14, 0.0020, color, 0.98, target);
        let d = r * 0.68;
        for (a, b) in [
            (Vec2::new(-d, -d), Vec2::new(d, d)),
            (Vec2::new(-d, d), Vec2::new(d, -d)),
        ] {
            pen.emit_beam(pen.step_ndc(center, a), pen.step_ndc(center, b), 0.0019, color, 0.98, target);
        }
    }
}

impl<O: OpticOverlay> OpticOverlay for LateralSlipOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        let pen = self.stage.pen();
        self.trace.visit_predicted(|fp| {
            fp.visit_slip(0.35, |slip_dir, slip_speed| {
                let color = if slip_speed > 3.5 { SLIP_WARN } else { SLIP_CYAN };
                let sight = self.stage.project_direction(slip_dir, BELLY_GUARD_RADIUS - 0.030);
                let pt = match sight {
                    DirectionSight::InField { ndc, .. } => ndc,
                    DirectionSight::RimClamped { rim_point, outward, .. } => {
                        pen.emit_rim_chevron(rim_point, outward, 0.018, color, target);
                        rim_point
                    }
                };
                self.emit_slip_reticle(pt, color, target);
                pen.emit_anchor_tether(pt, (0.024, 0.022), 4, 0.0014, color, target);
                let caption = format!("SLIP {:.1}", slip_speed);
                let caption_pt = pen.step_ndc(pt, Vec2::new(0.0, 0.032));
                pen.emit_caption(&caption, caption_pt, (0.0036, 0.0054), color, target);
            });
        });
    }
}
