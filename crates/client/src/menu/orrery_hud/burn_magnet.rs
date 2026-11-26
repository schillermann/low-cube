use glam::{Vec2, Vec3};
use world::MeshTarget;
use crate::drive::{FlightKinematics, TransferArc};
use super::overlays::OpticOverlay;
use super::sight_stage::{COMPASS_RADIUS, DirectionSight, HudSightStage};

const GUIDE_AMBER: Vec3 = Vec3::new(0.96, 0.74, 0.28);

/// `OpticOverlay` decorator projecting the Newtonian `BURN` diamond magnet and lead tether (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct BurnMagnetOverlay<O> {
    origin: O,
    stage: HudSightStage,
    kinematics: FlightKinematics,
    arc: TransferArc,
}

impl<O> BurnMagnetOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: O,
        stage: HudSightStage,
        kinematics: FlightKinematics,
        arc: TransferArc,
    ) -> Self {
        Self { origin, stage, kinematics, arc }
    }

    fn emit_diamond_magnet(&self, center: Vec2, color: Vec3, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        for (r, width, glow) in [(0.026, 0.0024, 0.98), (0.013, 0.0017, 0.94)] {
            let n = pen.step_ndc(center, Vec2::new(0.0, r));
            let e = pen.step_ndc(center, Vec2::new(r, 0.0));
            let s = pen.step_ndc(center, Vec2::new(0.0, -r));
            let w = pen.step_ndc(center, Vec2::new(-r, 0.0));
            for (a, b) in [(n, e), (e, s), (s, w), (w, n)] {
                pen.emit_beam(a, b, width, color, glow, target);
            }
        }
    }
}

impl<O: OpticOverlay> OpticOverlay for BurnMagnetOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        let pen = self.stage.pen();
        let vector = self.kinematics.burn_guidance(&self.arc);
        self.kinematics.transfer(|_, nose_axis, _| {
            vector.present(nose_axis, |burn_unit, base_col, cue| {
                let sight = self.stage.project_direction(burn_unit, COMPASS_RADIUS - 0.018);
                let pt = match sight {
                    DirectionSight::InField { ndc, .. } => ndc,
                    DirectionSight::RimClamped { rim_point, outward, .. } => {
                        pen.emit_rim_chevron(rim_point, outward, 0.024, base_col, target);
                        rim_point
                    }
                };
                self.emit_diamond_magnet(pt, base_col, target);
                pen.emit_anchor_tether(pt, (0.026, 0.028), 6, 0.0017, GUIDE_AMBER, target);
                let cue_pt = pen.step_ndc(pt, Vec2::new(0.0, 0.038));
                pen.emit_caption(cue, cue_pt, (0.0040, 0.0058), base_col, target);
            });
        });
    }
}
