use glam::{DVec2, DVec3};
use world::{BodyId, CelestialAtlas};
use crate::menu::widget::{ScreenBounds, ScreenPoint};

/// Polymorphic selected body state on the Solar System Map avoiding `Option<BodyId>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartSelection {
    Unselected,
    Locked(BodyId),
}

/// Interactive pan, logarithmic zoom, and 3D ecliptic projection of `SolarChartModal` (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct ChartViewport {
    pan: DVec2,
    half_span: f64,
    cursor: ScreenPoint,
    dragging: bool,
}

impl ChartViewport {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pan: DVec2, half_span: f64, cursor: ScreenPoint, dragging: bool) -> Self {
        Self { pan, half_span, cursor, dragging }
    }

    pub fn calibrated(atlas: CelestialAtlas, ship_pos: glam::Vec3, cursor: ScreenPoint) -> Self {
        if let Some(eph) = atlas.ephemeris() {
            let sun = eph.snapshot_all(0.0)[0].center();
            let span = (eph.proportions().astronomical_unit() * 2.15).max(500.0);
            Self::new(DVec2::new(sun.x, sun.z), span, cursor, false)
        } else {
            let sd = ship_pos.as_dvec3();
            Self::new(DVec2::new(sd.x, sd.z), 480.0, cursor, false)
        }
    }

    pub const fn cursor(self) -> ScreenPoint {
        self.cursor
    }

    pub const fn half_span(self) -> f64 {
        self.half_span
    }

    pub fn move_cursor(&mut self, next: ScreenPoint, inv_aspect: f32) {
        if self.dragging {
            let (nx, ny) = next.transfer(|x, y| (x, y));
            let (ox, oy) = self.cursor.transfer(|x, y| (x, y));
            let dx = f64::from(nx - ox) / f64::from(inv_aspect.max(0.25) * 0.78);
            let dy = f64::from(ny - oy) / (0.72 * 0.62);
            self.pan.x -= dx * self.half_span;
            self.pan.y += dy * self.half_span;
        }
        self.cursor = next;
    }

    pub fn begin_drag(&mut self) {
        self.dragging = true;
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    pub fn zoom(&mut self, factor: f64, max_span: f64) {
        self.half_span = (self.half_span * factor).clamp(16.0, max_span.max(1_200.0));
    }

    pub fn recenter_on(&mut self, world_pt: DVec3) {
        self.pan = DVec2::new(world_pt.x, world_pt.z);
    }

    /// Projects an Earth-centered inertial `DVec3` into tilted 2D NDC coordinates inside the chart frame.
    pub fn project(&self, world_pt: DVec3, inv_aspect: f32) -> (f32, f32) {
        let rx = (world_pt.x - self.pan.x) / self.half_span.max(1.0);
        let rz = (world_pt.z - self.pan.y) / self.half_span.max(1.0);
        let ry = world_pt.y / self.half_span.max(1.0);
        let sx = (rx as f32) * 0.78 * inv_aspect;
        let sy = (-rz as f32 * 0.62 + ry as f32 * 0.56) * 0.72 - 0.03;
        (sx, sy)
    }

    pub fn inside_canvas(pt: ScreenPoint) -> bool {
        ScreenBounds::new(-0.92, -0.76, 0.92, 0.74).contains(pt)
    }

    pub const fn mode_button() -> ScreenBounds {
        ScreenBounds::new(-0.90, -0.91, -0.34, -0.80)
    }

    pub const fn zoom_in_button() -> ScreenBounds {
        ScreenBounds::new(-0.30, -0.91, -0.08, -0.80)
    }

    pub const fn zoom_out_button() -> ScreenBounds {
        ScreenBounds::new(-0.05, -0.91, 0.17, -0.80)
    }

    pub const fn ship_button() -> ScreenBounds {
        ScreenBounds::new(0.20, -0.91, 0.52, -0.80)
    }

    pub const fn close_button() -> ScreenBounds {
        ScreenBounds::new(0.55, -0.91, 0.90, -0.80)
    }

    /// Resolves the NDC screen point of a named `BodyId` for clicking or automated UI tests.
    #[cfg(test)]
    pub fn body_screen_point(&self, id: BodyId, atlas: CelestialAtlas, inv_aspect: f32) -> Option<ScreenPoint> {
        if let Some(eph) = atlas.ephemeris() {
            let snap = eph.snapshot_all(0.0).into_iter().find(|s| s.id() == id)?;
            let (sx, sy) = self.project(snap.center(), inv_aspect);
            return Some(ScreenPoint::new(sx, sy));
        }
        atlas.transfer(|_, moon_c, moon_h| match id {
            BodyId::EARTH => {
                let (sx, sy) = self.project(DVec3::ZERO, inv_aspect);
                Some(ScreenPoint::new(sx, sy))
            }
            BodyId::LUNA if moon_h > 0.0 => {
                let (sx, sy) = self.project(moon_c.as_dvec3(), inv_aspect);
                Some(ScreenPoint::new(sx, sy))
            }
            _ => None,
        })
    }

    /// Picks the closest visible celestial cube under `self.cursor` within interactive click tolerance.
    pub fn pick_body(&self, atlas: CelestialAtlas, inv_aspect: f32) -> Option<BodyId> {
        let (cx, cy) = self.cursor.transfer(|x, y| (x, y));
        if let Some(eph) = atlas.ephemeris() {
            let snaps = eph.snapshot_all(0.0);
            let mut best: Option<(f32, BodyId)> = None;
            for s in snaps {
                if s.parent_id() != BodyId::SOL && s.id() != BodyId::SOL && s.orbit_radius() < self.half_span * 0.035 {
                    continue;
                }
                let (bx, by) = self.project(s.center(), inv_aspect);
                if !(-0.93..=0.93).contains(&bx) || !(-0.77..=0.75).contains(&by) {
                    continue;
                }
                let d = ((cx - bx) / inv_aspect.max(0.35)).hypot(cy - by);
                let hit_r = if s.id() == BodyId::SOL { 0.065 } else { 0.052 };
                if d <= hit_r && best.map_or(true, |(b, _)| d < b) {
                    best = Some((d, s.id()));
                }
            }
            return best.map(|(_, id)| id);
        }
        atlas.transfer(|_, moon_c, moon_h| {
            for (id, pt) in [(BodyId::EARTH, DVec3::ZERO), (BodyId::LUNA, moon_c.as_dvec3())] {
                if id == BodyId::LUNA && moon_h <= 0.0 {
                    continue;
                }
                let (bx, by) = self.project(pt, inv_aspect);
                if ((cx - bx) / inv_aspect.max(0.35)).hypot(cy - by) <= 0.06 {
                    return Some(id);
                }
            }
            None
        })
    }
}
