use glam::{Vec2, Vec3};
use world::MeshTarget;
use crate::drive::FlightKinematics;
use super::overlays::OpticOverlay;
use super::sight_stage::{COMPASS_RADIUS, DirectionSight, HudSightStage};

const PROGRADE_CYAN: Vec3 = Vec3::new(0.32, 0.96, 1.00);
const RETRO_AMBER: Vec3 = Vec3::new(1.00, 0.68, 0.24);
const TETHER_CYAN: Vec3 = Vec3::new(0.24, 0.80, 0.94);

/// `OpticOverlay` decorator projecting the Newtonian `(o)` Prograde / `(x)` Retrograde flight-path vector (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct ProgradeDriftOverlay<O> {
    origin: O,
    stage: HudSightStage,
    kinematics: FlightKinematics,
}

impl<O> ProgradeDriftOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: O, stage: HudSightStage, kinematics: FlightKinematics) -> Self {
        Self { origin, stage, kinematics }
    }

    fn emit_prograde_glyph(&self, center: Vec2, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        let r = 0.021;
        pen.emit_ring(center, r, 16, 0.0022, PROGRADE_CYAN, 0.98, target);
        pen.emit_ring(center, 0.0036, 8, 0.0020, PROGRADE_CYAN, 0.98, target);
        for (a, b) in [
            (Vec2::new(-r - 0.015, 0.0), Vec2::new(-r, 0.0)),
            (Vec2::new(r, 0.0), Vec2::new(r + 0.015, 0.0)),
            (Vec2::new(0.0, r), Vec2::new(0.0, r + 0.015)),
        ] {
            pen.emit_beam(pen.step_ndc(center, a), pen.step_ndc(center, b), 0.0021, PROGRADE_CYAN, 0.98, target);
        }
    }

    fn emit_retrograde_glyph(&self, center: Vec2, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        let r = 0.020;
        pen.emit_ring(center, r, 16, 0.0021, RETRO_AMBER, 0.98, target);
        let d = r * 0.68;
        for (a, b) in [
            (Vec2::new(-d, -d), Vec2::new(d, d)),
            (Vec2::new(-d, d), Vec2::new(d, -d)),
        ] {
            pen.emit_beam(pen.step_ndc(center, a), pen.step_ndc(center, b), 0.0020, RETRO_AMBER, 0.98, target);
        }
    }
}

impl<O: OpticOverlay> OpticOverlay for ProgradeDriftOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        let pen = self.stage.pen();
        self.kinematics.transfer(|_, _, velocity| {
            let speed = velocity.length();
            if speed <= 0.5 {
                return;
            }
            let sight = self.stage.project_direction(velocity / speed, COMPASS_RADIUS - 0.032);
            let (pt, angle_deg, clamped) = match sight {
                DirectionSight::InField { ndc, angle_deg } => (ndc, angle_deg, false),
                DirectionSight::RimClamped { rim_point, outward, angle_deg } => {
                    let col = if angle_deg > 95.0 { RETRO_AMBER } else { PROGRADE_CYAN };
                    pen.emit_rim_chevron(rim_point, outward, 0.018, col, target);
                    (rim_point, angle_deg, true)
                }
            };
            let retro = angle_deg > 95.0;
            if retro {
                self.emit_retrograde_glyph(pt, target);
            } else {
                self.emit_prograde_glyph(pt, target);
            }
            let tether_col = if retro { RETRO_AMBER } else { TETHER_CYAN };
            pen.emit_anchor_tether(pt, (0.026, 0.025), 5, 0.0014, tether_col, target);
            if !clamped || speed >= 2.0 {
                let col = if retro { RETRO_AMBER } else { PROGRADE_CYAN };
                let label = format!("{:.0} M/S", speed);
                let label_pt = pen.step_ndc(pt, Vec2::new(0.0, -0.036));
                pen.emit_caption(&label, label_pt, (0.0036, 0.0054), col, target);
            }
        });
    }
}
