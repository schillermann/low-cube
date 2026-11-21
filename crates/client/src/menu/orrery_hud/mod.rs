mod canvas;

use glam::{Mat4, Vec3};
use world::{CelestialAtlas, MeshTarget};
use crate::drive::{ARC_NODES, NavigationCourse, OrrerySnapshot, TransferArc};
use super::glyph::PixelStamp;
use super::widget::ScreenBounds;
use self::canvas::OrreryCanvas;

struct OrreryFrame {
    bases: ([Vec3; 3], [Vec3; 3]),
    earth_holo: Vec3,
    moon_world: Vec3,
    radii: (f32, f32, f32, f32),
}

impl OrreryFrame {
    fn from_atlas(atlas: CelestialAtlas) -> Self {
        atlas.transfer(|home_half, moon_center, moon_half| {
            let dist = moon_center.length().max(48.0);
            let world_u = if moon_center.length_squared() > 1.0 { moon_center / dist } else { Vec3::Y };
            let ref_axis = if world_u.x.abs() < 0.85 { Vec3::X } else { Vec3::Z };
            let world_w = world_u.cross(ref_axis).normalize_or_zero();
            let world_v = world_w.cross(world_u).normalize_or_zero();

            let earth_holo = Vec3::new(-0.36, -0.08, 0.14);
            let moon_holo = Vec3::new(0.52, 0.22, -0.18);
            let span = (moon_holo - earth_holo).length();
            let holo_u = (moon_holo - earth_holo) / span;
            let holo_w = (Vec3::Y - holo_u * holo_u.y).normalize_or_zero();
            let holo_v = holo_u.cross(holo_w).normalize_or_zero();

            let scale = span / dist;
            let vis_earth = (home_half * scale).clamp(0.18, 0.24);
            let vis_moon = if moon_half > 0.0 { (moon_half * scale).clamp(0.10, 0.14) } else { 0.0 };
            Self {
                bases: ([world_u, world_v, world_w], [holo_u, holo_v, holo_w]),
                earth_holo,
                moon_world: moon_center,
                radii: (scale, home_half.max(1.0), vis_earth, vis_moon),
            }
        })
    }

    fn rotate_dir(&self, d: Vec3) -> Vec3 {
        let ([wu, wv, ww], [hu, hv, hw]) = self.bases;
        hu * d.dot(wu) + hv * d.dot(wv) + hw * d.dot(ww)
    }

    fn map_point(&self, world_pt: Vec3) -> Vec3 {
        let (scale, home_half, vis_earth, vis_moon) = self.radii;
        let base = self.earth_holo + self.rotate_dir(world_pt) * scale;
        let home_cheb = world_pt.abs().max_element();
        let earth_lift = (vis_earth - home_half * scale).max(0.0);
        let after_earth = if earth_lift > 1e-4 && home_cheb < home_half * 2.2 {
            let w = (1.0 - ((home_cheb - home_half) / (home_half * 1.2)).clamp(0.0, 1.0)).max(0.0);
            let dir = self.rotate_dir(world_pt).normalize_or_zero();
            base + dir * (earth_lift * w)
        } else {
            base
        };
        if vis_moon <= 0.0 {
            return after_earth;
        }
        let moon_half = 16.0_f32;
        let rel_moon = world_pt - self.moon_world;
        let moon_cheb = rel_moon.abs().max_element();
        let moon_lift = (vis_moon - moon_half * scale).max(0.0);
        if moon_lift > 1e-4 && moon_cheb < moon_half * 2.6 {
            let w = (1.0 - ((moon_cheb - moon_half) / (moon_half * 1.6)).clamp(0.0, 1.0)).max(0.0);
            let dir = self.rotate_dir(rel_moon).normalize_or_zero();
            after_earth + dir * (moon_lift * w)
        } else {
            after_earth
        }
    }
}

/// Nose-Camera Variante-1 HUD projecting `[ + ]` lunar target brackets and the bottom-right 3D Steuerkonsolen-Orrery.
#[derive(Clone, Copy, Debug)]
pub struct NoseOrreryHud {
    snapshot: OrrerySnapshot,
    view_proj: Mat4,
    inv_aspect: f32,
}

impl NoseOrreryHud {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_parts(snapshot: OrrerySnapshot, view_proj: Mat4, inv_aspect: f32) -> Self {
        Self { snapshot, view_proj, inv_aspect }
    }

    pub fn new(snapshot: OrrerySnapshot, view_proj: Mat4, aspect: f32) -> Self {
        Self::from_parts(snapshot, view_proj, (1.0 / aspect.max(0.5)).clamp(0.25, 1.5))
    }

    fn emit_target_brackets(&self, target_world: Vec3, target: &mut dyn MeshTarget) {
        let clip = self.view_proj * target_world.extend(1.0);
        if clip.w <= 0.15 {
            return;
        }
        let (nx, ny) = (clip.x / clip.w, clip.y / clip.w);
        if nx.abs() > 1.06 || ny.abs() > 1.06 {
            return;
        }
        let ux = self.inv_aspect;
        let (rx, ry) = (0.056 * ux, 0.056);
        let (ax, ay) = (0.021 * ux, 0.021);
        let (tx, ty) = (0.0034 * ux, 0.0034);
        let bracket_col = Vec3::new(0.28, 0.96, 0.78);
        let cross_col = Vec3::new(0.46, 0.96, 1.00);
        for sx in [-1.0_f32, 1.0_f32] {
            for sy in [-1.0_f32, 1.0_f32] {
                let (cx, cy) = (nx + sx * rx, ny + sy * ry);
                let (hx0, hx1) = if sx < 0.0 { (cx, cx + ax) } else { (cx - ax, cx) };
                let (vy0, vy1) = if sy < 0.0 { (cy, cy + ay) } else { (cy - ay, cy) };
                ScreenBounds::new(hx0, cy - ty, hx1, cy + ty).emit_fill(bracket_col, target);
                ScreenBounds::new(cx - tx, vy0, cx + tx, vy1).emit_fill(bracket_col, target);
            }
        }
        ScreenBounds::new(nx - 0.013 * ux, ny - 0.0022, nx + 0.013 * ux, ny + 0.0022).emit_fill(cross_col, target);
        ScreenBounds::new(nx - 0.0022 * ux, ny - 0.013, nx + 0.0022 * ux, ny + 0.013).emit_fill(cross_col, target);
    }

    fn emit_charted_route(
        &self,
        arc: &TransferArc,
        frame: &OrreryFrame,
        canvas: OrreryCanvas,
        (ship_pos, flight_vec): (Vec3, Vec3),
        (cx, cy): (f32, f32),
        target: &mut dyn MeshTarget,
    ) {
        let mut target_world = Vec3::ZERO;
        arc.feed_nodes(|nodes, target_norm| {
            target_world = nodes[ARC_NODES - 1];
            let cyan = Vec3::new(0.32, 0.94, 1.00);
            let glow_cyan = Vec3::new(0.18, 0.78, 0.96);
            for i in 0..(ARC_NODES - 1) {
                let a = frame.map_point(nodes[i]);
                let b = frame.map_point(nodes[i + 1]);
                canvas.emit_strut_3d(a, b, 0.0046, glow_cyan, -0.26, target);
                canvas.emit_strut_3d(a, b, 0.0021, cyan, 0.98, target);
            }
            let holo_tgt = frame.map_point(target_world);
            let n_holo = frame.rotate_dir(target_norm).normalize_or_zero();
            let ref_up = if n_holo.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
            let t1 = n_holo.cross(ref_up).normalize_or_zero() * 0.038;
            let t2 = n_holo.cross(t1).normalize_or_zero() * 0.038;
            let mark_col = Vec3::new(0.42, 1.00, 0.86);
            canvas.emit_strut_3d(holo_tgt - t1, holo_tgt + t1, 0.0019, mark_col, 0.98, target);
            canvas.emit_strut_3d(holo_tgt - t2, holo_tgt + t2, 0.0019, mark_col, 0.98, target);
        });
        self.emit_target_brackets(target_world, target);

        let telemetry = arc.sample_telemetry(ship_pos, flight_vec);
        telemetry.transfer(|anchor, _cross_m, angle_deg, aligned| {
            let ux = self.inv_aspect;
            if !aligned {
                let amber = Vec3::new(1.00, 0.68, 0.22);
                let a_holo = frame.map_point(anchor);
                let s_holo = frame.map_point(ship_pos);
                canvas.emit_strut_3d(s_holo, a_holo, 0.0018, amber, 0.96, target);
            }
            let (col, label) = if aligned {
                (Vec3::new(0.58, 1.00, 0.82), format!("ON COURSE {:.1}", angle_deg))
            } else {
                (Vec3::new(1.00, 0.72, 0.28), format!("OFF COURSE {:.1}", angle_deg))
            };
            PixelStamp::new(0.0046 * ux, 0.0068, col).emit_centered(&label, cx + 0.02 * ux, cy + 0.245, target);
        });
    }

    pub fn emit(self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect;
        let (cx, cy) = (1.0 - 0.36 * ux, -0.57);
        let canvas = OrreryCanvas::new((cx, cy), 0.255, ux);
        self.snapshot.transfer(|atlas, ship_pos, (nose_axis, flight_vec), course| {
            let frame = OrreryFrame::from_atlas(atlas);
            let (_, _, vis_earth, vis_moon) = frame.radii;
            let earth_holo = frame.earth_holo;
            let moon_holo = frame.map_point(frame.moon_world);

            canvas.emit_projector_base(target);
            canvas.emit_gravity_shell(earth_holo, vis_earth * 1.68, 14, Vec3::new(0.22, 0.88, 0.72), target);
            canvas.emit_wire_cube(earth_holo, vis_earth, Vec3::new(0.20, 0.96, 0.72), 0.14, target);

            if vis_moon > 0.0 {
                canvas.emit_gravity_shell(moon_holo, vis_moon * 1.85, 10, Vec3::new(0.68, 0.82, 0.98), target);
                canvas.emit_wire_cube(moon_holo, vis_moon, Vec3::new(0.84, 0.92, 1.00), 0.15, target);
            }

            match course {
                NavigationCourse::Uncharted => {
                    if vis_moon > 0.0 {
                        let base_col = Vec3::new(0.20, 0.62, 0.80);
                        canvas.emit_strut_3d(earth_holo, moon_holo, 0.0014, base_col, -0.34, target);
                    }
                }
                NavigationCourse::Charted(ref arc) => {
                    self.emit_charted_route(arc, &frame, canvas, (ship_pos, flight_vec), (cx, cy), target);
                }
            }

            let ship_holo = frame.map_point(ship_pos);
            let nose_holo = frame.rotate_dir(nose_axis);
            let flight_holo = frame.rotate_dir(flight_vec);
            canvas.emit_rocket_symbol(ship_holo, nose_holo, flight_holo, target);
        });
    }
}
