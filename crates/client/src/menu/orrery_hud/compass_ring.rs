use std::f32::consts::TAU;
use glam::{Vec2, Vec3};
use world::{CelestialAtlas, MeshTarget};
use super::overlays::OpticOverlay;
use super::sight_stage::{COMPASS_RADIUS, DirectionSight, HudSightStage};

const RING_CYAN: Vec3 = Vec3::new(0.22, 0.68, 0.84);
const TICK_CYAN: Vec3 = Vec3::new(0.36, 0.86, 0.98);
const HOME_TEAL: Vec3 = Vec3::new(0.26, 0.96, 0.72);
const LUNA_ICE: Vec3 = Vec3::new(0.76, 0.92, 1.00);

/// `OpticOverlay` decorator projecting the 360-degree *Expanse*-style boresight compass ring and off-axis worlds (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CompassRingOverlay<O> {
    origin: O,
    stage: HudSightStage,
    atlas: CelestialAtlas,
    ship_pos: Vec3,
}

impl<O> CompassRingOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: O,
        stage: HudSightStage,
        atlas: CelestialAtlas,
        ship_pos: Vec3,
    ) -> Self {
        Self { origin, stage, atlas, ship_pos }
    }

    fn emit_compass_dial(&self, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        pen.emit_anchor_ring(COMPASS_RADIUS, 48, 0.0014, RING_CYAN, -0.46, target);
        for step in 0..24 {
            let ang = (step as f32) * TAU / 24.0;
            let dir = Vec2::new(ang.cos(), ang.sin());
            let cardinal = step % 6 == 0;
            let inner = if cardinal { COMPASS_RADIUS - 0.022 } else { COMPASS_RADIUS - 0.011 };
            let width = if cardinal { 0.0022 } else { 0.0013 };
            let col = if cardinal { TICK_CYAN } else { RING_CYAN };
            let glow = if cardinal { 0.92 } else { -0.40 };
            pen.emit_anchor_spoke(dir, (inner, COMPASS_RADIUS), width, col, glow, target);
        }
    }

    fn emit_cube_glyph(&self, center: Vec2, half: f32, color: Vec3, target: &mut dyn MeshTarget) {
        let pen = self.stage.pen();
        let c0 = pen.step_ndc(center, Vec2::new(-half, -half));
        let c1 = pen.step_ndc(center, Vec2::new(half, -half));
        let c2 = pen.step_ndc(center, Vec2::new(half, half));
        let c3 = pen.step_ndc(center, Vec2::new(-half, half));
        for (a, b) in [(c0, c1), (c1, c2), (c2, c3), (c3, c0)] {
            pen.emit_beam(a, b, 0.0018, color, 0.96, target);
        }
    }

    fn emit_world_bearing(
        &self,
        body_center: Vec3,
        body_half: f32,
        name: &str,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let rel = body_center - self.ship_pos;
        let dist = rel.length();
        if body_half <= 0.0 || dist <= body_half * 1.35 {
            return;
        }
        let pen = self.stage.pen();
        match self.stage.project_direction(rel / dist, COMPASS_RADIUS) {
            DirectionSight::InField { ndc, .. } => {
                pen.emit_anchor_notch_toward(
                    ndc,
                    (COMPASS_RADIUS - 0.010, COMPASS_RADIUS + 0.012),
                    0.0022,
                    color,
                    target,
                );
            }
            DirectionSight::RimClamped { rim_point, outward, angle_deg } => {
                pen.emit_rim_chevron(rim_point, outward, 0.022, color, target);
                let badge_pt = pen.step_ndc(rim_point, -outward * 0.024);
                self.emit_cube_glyph(badge_pt, 0.011, color, target);
                let text_pt = pen.step_ndc(rim_point, outward * 0.048);
                let caption = format!("{name} {:.0}", angle_deg);
                pen.emit_caption(&caption, text_pt, (0.0038, 0.0056), color, target);
            }
        }
    }
}

impl<O: OpticOverlay> OpticOverlay for CompassRingOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        self.emit_compass_dial(target);
        self.atlas.transfer(|home_half, moon_center, moon_half| {
            self.emit_world_bearing(Vec3::ZERO, home_half, "HOME", HOME_TEAL, target);
            self.emit_world_bearing(moon_center, moon_half, "LUNA", LUNA_ICE, target);
        });
    }
}
