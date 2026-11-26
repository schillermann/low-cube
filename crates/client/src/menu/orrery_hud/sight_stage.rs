use glam::{Mat4, Vec2, Vec3};
use crate::menu::widget::ScreenPoint;
use super::hud_pen::HudPen;

pub const COMPASS_RADIUS: f32 = 0.43;
pub const BELLY_GUARD_RADIUS: f32 = 0.43;

/// Polymorphic 360-degree camera-space ray projection without `Option`.
#[derive(Clone, Copy, Debug)]
pub enum DirectionSight {
    InField {
        ndc: Vec2,
        angle_deg: f32,
    },
    RimClamped {
        rim_point: Vec2,
        outward: Vec2,
        angle_deg: f32,
    },
}

/// Polymorphic 3D world-point projection with continuous off-screen guard clamping.
#[derive(Clone, Copy, Debug)]
pub enum WorldSight {
    OnScreen(Vec2),
    OffScreen {
        rim_point: Vec2,
        outward: Vec2,
    },
}

/// Camera-synchronized Tell-Don't-Ask NDC projection stage for `OpticOverlay` decorators (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct HudSightStage {
    view_proj: Mat4,
    reticle: ScreenPoint,
    inv_aspect: f32,
}

impl HudSightStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(view_proj: Mat4, reticle: ScreenPoint, inv_aspect: f32) -> Self {
        Self { view_proj, reticle, inv_aspect }
    }

    /// Creates the aspect-calibrated 2D `HudPen` anchored at the boresight reticle.
    pub fn pen(self) -> HudPen {
        HudPen::new(self.reticle.transfer(Vec2::new), self.inv_aspect)
    }

    /// Continuously projects any 3D vector across all 360 degrees onto the FOV or compass rim.
    pub fn project_direction(self, world_dir: Vec3, ring_radius: f32) -> DirectionSight {
        let unit = world_dir.normalize_or_zero();
        let anchor = self.reticle.transfer(Vec2::new);
        let pen = self.pen();
        if unit.length_squared() <= 0.25 {
            return DirectionSight::InField { ndc: anchor, angle_deg: 0.0 };
        }
        let clip = self.view_proj * unit.extend(0.0);
        let angle_deg = clip.w.clamp(-1.0, 1.0).acos().to_degrees();
        let safe_w = clip.w.max(0.12);
        let virt_ndc = Vec2::new(clip.x / safe_w, clip.y / safe_w);
        let delta = pen.metric_delta(anchor, virt_ndc);
        let dist = delta.length();
        if clip.w > 0.12 && dist <= ring_radius {
            return DirectionSight::InField { ndc: virt_ndc, angle_deg };
        }
        let outward = if dist > 1e-4 { delta / dist } else { Vec2::X };
        DirectionSight::RimClamped {
            rim_point: pen.step_ndc(anchor, outward * ring_radius),
            outward,
            angle_deg,
        }
    }

    /// Projects a finite 3D world point onto the screen or smoothly clamps it onto `guard_radius`.
    pub fn project_world(self, world_pt: Vec3, guard_radius: f32) -> WorldSight {
        let anchor = self.reticle.transfer(Vec2::new);
        let pen = self.pen();
        let clip = self.view_proj * world_pt.extend(1.0);
        let safe_w = clip.w.max(0.12);
        let virt_ndc = Vec2::new(clip.x / safe_w, clip.y / safe_w);
        if clip.w > 0.12 && virt_ndc.x.abs() <= 0.90 && virt_ndc.y.abs() <= 0.84 {
            return WorldSight::OnScreen(virt_ndc);
        }
        let delta = pen.metric_delta(anchor, virt_ndc);
        let dist = delta.length();
        let outward = if dist > 1e-4 { delta / dist } else { Vec2::NEG_Y };
        WorldSight::OffScreen {
            rim_point: pen.step_ndc(anchor, outward * guard_radius),
            outward,
        }
    }

    /// Invokes `f(ndc)` strictly when `world_pt` projects in front of the camera (`w > 0.12`).
    pub fn visit_front_ndc(self, world_pt: Vec3, f: impl FnOnce(Vec2)) {
        let clip = self.view_proj * world_pt.extend(1.0);
        if clip.w > 0.12 {
            let ndc = Vec2::new(clip.x / clip.w, clip.y / clip.w);
            if ndc.x.abs() <= 1.25 && ndc.y.abs() <= 1.25 {
                f(ndc);
            }
        }
    }
}
