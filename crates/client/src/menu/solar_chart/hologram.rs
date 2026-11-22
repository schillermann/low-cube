use std::f64::consts::TAU;
use glam::{DVec3, Vec3};
use world::{
    AsteroidBelt, BodyId, BodySnapshot, CelestialAtlas, MeshTarget, OrbitElements, OrbitalLocus,
    Quad, SolarCatalog,
};
use crate::drive::{ARC_NODES, CourseMode, NavigationCourse, OrrerySnapshot, TransferArc};
use crate::menu::glyph::PixelStamp;
use crate::menu::widget::ScreenBounds;
use super::viewport::{ChartSelection, ChartViewport};

/// Holographic 3D Solar System Map view rendering cubic Sun, planets, moons, belt, orbits, and curved arcs.
#[derive(Clone, Copy, Debug)]
pub(super) struct ChartHologram {
    snapshot: OrrerySnapshot,
    viewport: ChartViewport,
    selection: ChartSelection,
    mode_and_aspect: (CourseMode, f32),
}

impl ChartHologram {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        snapshot: OrrerySnapshot,
        viewport: ChartViewport,
        selection: ChartSelection,
        mode_and_aspect: (CourseMode, f32),
    ) -> Self {
        Self { snapshot, viewport, selection, mode_and_aspect }
    }

    fn clipped(x: f32, y: f32) -> bool {
        !(-0.92..=0.92).contains(&x) || !(-0.76..=0.74).contains(&y)
    }

    fn emit_line_2d(a: (f32, f32), b: (f32, f32), half_w: f32, col: Vec3, glow: f32, target: &mut dyn MeshTarget) {
        if Self::clipped(a.0, a.1) && Self::clipped(b.0, b.1) {
            return;
        }
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = dx.hypot(dy);
        if len < 1e-5 {
            return;
        }
        let (nx, ny) = (-dy / len * half_w, dx / len * half_w);
        let p0 = Vec3::new(a.0 - nx, a.1 - ny, 0.0);
        let p1 = Vec3::new(b.0 - nx, b.1 - ny, 0.0);
        let p2 = Vec3::new(b.0 + nx, b.1 + ny, 0.0);
        let p3 = Vec3::new(a.0 + nx, a.1 + ny, 0.0);
        target.push_quad(Quad::glowing(p0, p1, p2, p3, Vec3::Z, col, glow));
    }

    fn emit_iso_cube(cx: f32, cy: f32, r: f32, ux: f32, col: Vec3, ringed: bool, target: &mut dyn MeshTarget) {
        if Self::clipped(cx, cy) {
            return;
        }
        let rx = r * ux;
        let off = (0.38 * rx, 0.34 * r);
        let front = [
            (cx - rx, cy - r),
            (cx + rx, cy - r),
            (cx + rx, cy + r),
            (cx - rx, cy + r),
        ];
        let back = front.map(|(x, y)| (x + off.0, y + off.1));
        ScreenBounds::new(cx - rx * 0.85, cy - r * 0.85, cx + rx * 0.85, cy + r * 0.85)
            .emit_fill(col * 0.42, target);
        for i in 0..4 {
            let j = (i + 1) % 4;
            Self::emit_line_2d(front[i], front[j], 0.0022, col, 0.95, target);
            Self::emit_line_2d(back[i], back[j], 0.0018, col * 0.82, 0.88, target);
            Self::emit_line_2d(front[i], back[i], 0.0018, col * 0.82, 0.88, target);
        }
        if ringed {
            let (rw, rh) = (rx * 2.15, r * 0.72);
            let ring = [(cx - rw, cy), (cx, cy + rh), (cx + rw, cy), (cx, cy - rh)];
            for i in 0..4 {
                Self::emit_line_2d(ring[i], ring[(i + 1) % 4], 0.0020, Vec3::new(0.92, 0.84, 0.62), 0.90, target);
            }
        }
    }

    fn emit_orbit_ellipse(
        &self,
        orbit: &dyn OrbitalLocus,
        focus: DVec3,
        semi_major: f64,
        ux: f32,
        col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        if semi_major <= 1.0 || semi_major < self.viewport.half_span() * 0.025 {
            return;
        }
        orbit.trace_ellipse(focus, semi_major, 48, &mut |pt_a, pt_b| {
            let a2 = self.viewport.project(pt_a, ux);
            let b2 = self.viewport.project(pt_b, ux);
            Self::emit_line_2d(a2, b2, 0.0014, col, -0.35, target);
        });
    }

    fn emit_belt_torus(&self, sun: DVec3, au: f64, ux: f32, target: &mut dyn MeshTarget) {
        let (r_in, r_out) = AsteroidBelt::inner_outer_au();
        if r_out * au < self.viewport.half_span() * 0.04 {
            return;
        }
        let col = Vec3::new(0.52, 0.48, 0.44);
        for idx in 0..72usize {
            let h = (idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let frac = (((h >> 12) & 0xFF) as f64) / 255.0;
            let r = (r_in + (r_out - r_in) * frac) * au;
            let ang = (idx as f64) * TAU / 72.0 + frac * 0.08;
            let (sx, sy) = self.viewport.project(sun + DVec3::new(r * ang.cos(), 0.0, r * ang.sin()), ux);
            if !Self::clipped(sx, sy) {
                ScreenBounds::new(sx - 0.003 * ux, sy - 0.003, sx + 0.003 * ux, sy + 0.003).emit_fill(col, target);
            }
        }
    }

    fn chart_cube_radius(&self, s: &BodySnapshot, earth_half: f32) -> f32 {
        let metric_r = ((s.half_size() / self.viewport.half_span().max(1.0)) as f32) * 0.72;
        let ref_earth = f64::from(earth_half.max(4.0));
        let schematic = if s.id() == BodyId::SOL {
            let sun_rel = (s.half_size() / (ref_earth * 109.2)).max(0.01) as f32;
            (0.024 * sun_rel.sqrt()).clamp(0.006, 0.060)
        } else {
            let body_rel = (s.half_size() / ref_earth).max(0.01) as f32;
            (0.011 * body_rel.powf(0.32)).clamp(0.004, 0.048)
        };
        metric_r.max(schematic).min(0.45)
    }

    fn emit_heliosphere_bodies(
        &self,
        snaps: &[BodySnapshot],
        props: world::SolarProportions,
        ux: f32,
        target: &mut dyn MeshTarget,
    ) {
        let au = props.astronomical_unit();
        let sun_c = snaps[0].center();
        self.emit_belt_torus(sun_c, au, ux, target);
        let entries = SolarCatalog::new().entries();
        for (idx, s) in snaps.iter().enumerate().skip(1) {
            let parent_c = snaps.iter().find(|p| p.id() == s.parent_id()).map_or(sun_c, |p| p.center());
            let ring_col = if s.parent_id() == BodyId::SOL {
                Vec3::new(0.18, 0.44, 0.62)
            } else {
                Vec3::new(0.24, 0.36, 0.52)
            };
            let orbit = entries[idx].orbit();
            self.emit_orbit_ellipse(&orbit, parent_c, s.orbit_radius(), ux, ring_col, target);
        }

        for s in snaps {
            if s.parent_id() != BodyId::SOL && s.id() != BodyId::SOL && s.orbit_radius() < self.viewport.half_span() * 0.035 {
                continue;
            }
            let (bx, by) = self.viewport.project(s.center(), ux);
            if Self::clipped(bx, by) {
                continue;
            }
            let (col, _) = s.palette();
            let rad = self.chart_cube_radius(s, props.earth_half());
            Self::emit_iso_cube(bx, by, rad, ux, col, s.nature().ringed_giant(), target);
            if self.selection == ChartSelection::Locked(s.id()) {
                let b = rad * 1.65;
                ScreenBounds::new(bx - b * ux, by - b, bx + b * ux, by + b)
                    .emit_outline(Vec3::new(0.42, 1.00, 0.88), 0.0032 * ux, 0.0032, target);
            }
            if s.parent_id() == BodyId::SOL || self.viewport.half_span() < au * 0.35 {
                PixelStamp::new(0.0040 * ux, 0.0058, col * 0.85 + Vec3::splat(0.25))
                    .emit_centered(s.name(), bx, by - rad - 0.020, target);
            }
        }
    }

    fn emit_trajectories(&self, atlas: CelestialAtlas, ship_pos: Vec3, vel: Vec3, course: NavigationCourse, ux: f32, target: &mut dyn MeshTarget) {
        let horizon = (self.viewport.half_span() / f64::from(vel.length().max(28.0))).clamp(8.0, 3_600.0);
        let coast = TransferArc::coasting_preview(ship_pos, vel, horizon, atlas);
        for i in 0..(ARC_NODES - 1) {
            let a = self.viewport.project(coast[i].as_dvec3(), ux);
            let b = self.viewport.project(coast[i + 1].as_dvec3(), ux);
            Self::emit_line_2d(a, b, 0.0020, Vec3::new(0.32, 0.92, 0.56), 0.90, target);
        }
        if let NavigationCourse::Charted(ref arc) = course {
            arc.feed_nodes(|nodes, _| {
                for i in 0..(ARC_NODES - 1) {
                    let a = self.viewport.project(nodes[i].as_dvec3(), ux);
                    let b = self.viewport.project(nodes[i + 1].as_dvec3(), ux);
                    Self::emit_line_2d(a, b, 0.0028, Vec3::new(0.30, 0.94, 1.00), 0.98, target);
                }
                let (tx, ty) = self.viewport.project(nodes[ARC_NODES - 1].as_dvec3(), ux);
                if !Self::clipped(tx, ty) {
                    ScreenBounds::new(tx - 0.018 * ux, ty - 0.018, tx + 0.018 * ux, ty + 0.018)
                        .emit_outline(Vec3::new(0.45, 1.00, 0.86), 0.003 * ux, 0.003, target);
                }
            });
            if let Some(flip_pt) = arc.flip_marker() {
                let (fx, fy) = self.viewport.project(flip_pt.as_dvec3(), ux);
                if !Self::clipped(fx, fy) {
                    let d = 0.020;
                    let dia = [(fx, fy + d), (fx + d * ux, fy), (fx, fy - d), (fx - d * ux, fy)];
                    for i in 0..4 {
                        Self::emit_line_2d(dia[i], dia[(i + 1) % 4], 0.0026, Vec3::new(1.00, 0.84, 0.28), 0.98, target);
                    }
                    PixelStamp::new(0.0040 * ux, 0.0058, Vec3::new(1.00, 0.88, 0.36))
                        .emit_centered("50% FLIP", fx, fy + 0.032, target);
                }
            }
        }
        let (sx, sy) = self.viewport.project(ship_pos.as_dvec3(), ux);
        if !Self::clipped(sx, sy) {
            ScreenBounds::new(sx - 0.010 * ux, sy - 0.010, sx + 0.010 * ux, sy + 0.010)
                .emit_fill(Vec3::new(1.00, 0.92, 0.36), target);
            PixelStamp::new(0.0042 * ux, 0.0060, Vec3::new(1.00, 0.96, 0.56))
                .emit_centered("SHIP", sx, sy + 0.024, target);
        }
    }

    pub(super) fn emit(self, target: &mut dyn MeshTarget) {
        let (mode, ux) = self.mode_and_aspect;
        let (bg, rim, btn, gold) = (
            Vec3::new(0.03, 0.06, 0.11),
            Vec3::new(0.26, 0.72, 0.92),
            Vec3::new(0.11, 0.18, 0.28),
            Vec3::new(0.98, 0.84, 0.32),
        );
        ScreenBounds::new(-0.95, -0.94, 0.95, 0.94).emit_framed(bg, rim, 0.006, target);
        ScreenBounds::new(-0.92, -0.76, 0.92, 0.74).emit_framed(Vec3::new(0.015, 0.03, 0.06), Vec3::new(0.16, 0.36, 0.52), 0.003, target);
        PixelStamp::title(gold).emit_centered("SOLAR SYSTEM HOLOGRAM CHART", 0.0, 0.85, target);

        self.snapshot.transfer(|atlas, ship_pos, (_, vel), course| {
            if let Some(eph) = atlas.ephemeris() {
                let snaps = eph.snapshot_all(0.0);
                self.emit_heliosphere_bodies(&snaps, eph.proportions(), ux, target);
            } else {
                atlas.transfer(|home_h, moon_c, moon_h| {
                    let span = self.viewport.half_span().max(1.0) as f32;
                    let er = ((home_h / span) * 0.72).clamp(0.012, 0.35);
                    let (ex, ey) = self.viewport.project(DVec3::ZERO, ux);
                    Self::emit_iso_cube(ex, ey, er, ux, Vec3::new(0.22, 0.82, 0.64), false, target);
                    PixelStamp::new(0.0042 * ux, 0.0060, Vec3::new(0.62, 0.96, 0.84)).emit_centered("EARTH", ex, ey - er - 0.018, target);
                    if moon_h > 0.0 {
                        let mr = ((moon_h / span) * 0.72).clamp(0.007, 0.25);
                        let pocket_orbit = OrbitElements::keplerian(BodyId::EARTH, 1.0, 0.0, (0.0, 0.0, 0.0));
                        self.emit_orbit_ellipse(&pocket_orbit, DVec3::ZERO, moon_c.as_dvec3().length(), ux, Vec3::new(0.24, 0.44, 0.64), target);
                        let (mx, my) = self.viewport.project(moon_c.as_dvec3(), ux);
                        Self::emit_iso_cube(mx, my, mr, ux, Vec3::new(0.80, 0.86, 0.96), false, target);
                        PixelStamp::new(0.0042 * ux, 0.0060, Vec3::new(0.84, 0.90, 0.98)).emit_centered("LUNA", mx, my - mr - 0.016, target);
                    }
                });
            }
            self.emit_trajectories(atlas, ship_pos, vel, course, ux, target);
        });

        let hint = "CLICK PLANET/MOON: PLOT INTERCEPT  |  SCROLL: ZOOM  |  DRAG: PAN";
        PixelStamp::new(0.0042 * ux, 0.0060, Vec3::new(0.64, 0.82, 0.94)).emit_centered(hint, 0.0, 0.78, target);
        let pt = self.viewport.cursor();
        ChartViewport::mode_button().emit_button(mode.label(), pt, btn, mode == CourseMode::Brachistochrone, target);
        ChartViewport::zoom_in_button().emit_button("ZOOM +", pt, btn, false, target);
        ChartViewport::zoom_out_button().emit_button("ZOOM -", pt, btn, false, target);
        ChartViewport::ship_button().emit_button("CENTER SHIP", pt, btn, false, target);
        ChartViewport::close_button().emit_button("CLOSE [M]", pt, btn, false, target);
    }
}
