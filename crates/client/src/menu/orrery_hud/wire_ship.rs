use glam::{Mat4, Vec2, Vec3};
use world::{CellRole, HullWireframe, JetTone, MeshTarget, Quad, MAX_EXHAUST_JETS};
use crate::menu::glyph::PixelStamp;
use crate::menu::widget::ScreenBounds;
use super::overlays::OpticOverlay;
use super::wire_stage::{
    AMBER_WIRE, CARD_GLASS, CARD_RIM, CYAN_WIRE, GHOST_HULL, IDLE_POD, STEER_HALO, STEER_PEAK,
    CameraStage,
};

/// Bottom-left camera-synchronized 3D ghost-hull & thruster HUD contrasting `EXHAUST` vs `STEER >`.
#[derive(Clone, Copy, Debug)]
pub struct WireframeShipCard {
    wireframe: HullWireframe,
    view_proj: Mat4,
    inv_aspect: f32,
}

impl WireframeShipCard {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(wireframe: HullWireframe, view_proj: Mat4, inv_aspect: f32) -> Self {
        Self { wireframe, view_proj, inv_aspect }
    }

    pub fn emit(self, target: &mut dyn MeshTarget) {
        let ux = self.inv_aspect.max(0.25);
        let (x0, y0, y1) = (-0.945, -0.920, -0.300);
        let x1 = (x0 + 1.02 * ux).min(-0.345);
        let bounds = (x0, y0, x1, y1);

        target.push_quad(Quad::glowing(
            Vec3::new(x0, y0, 0.0),
            Vec3::new(x1, y0, 0.0),
            Vec3::new(x1, y1, 0.0),
            Vec3::new(x0, y1, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            CARD_GLASS,
            -0.78,
        ));
        ScreenBounds::new(x0, y0, x1, y1).emit_outline(CARD_RIM, 0.0028 * ux, 0.0028, target);

        let center = Vec2::new(x0 + (x1 - x0) * 0.48, y0 + (y1 - y0) * 0.45);
        self.wireframe.transfer_steer(|ship_axes, steer_local, exhaust_lit| {
            let stage = CameraStage::from_view_proj(ship_axes, self.view_proj, center, 0.19, ux);

            let mut dedicated_nose = false;
            let mut fuselage_crown: Option<(Vec3, Vec3)> = None;
            let (mut rcs_count, mut bell_count) = (0u8, 0u8);

            // Pass 1: Faint ghosted outer fuselage envelope + idle thrusters (zero interior block grid)
            self.wireframe.visit_cells(|role, pos, half| match role {
                CellRole::Fuselage => {
                    stage.emit_box(pos, half, (GHOST_HULL, -0.24), 0.0013, target);
                    fuselage_crown = Some((pos, half));
                }
                CellRole::NoseApex => {
                    dedicated_nose = true;
                    stage.emit_nose_pyramid(pos, half, target);
                }
                CellRole::MainBell => {
                    bell_count = bell_count.saturating_add(1);
                    stage.emit_bell_skirt(pos, half, (IDLE_POD, -0.55), target);
                }
                CellRole::FusionBell => {
                    bell_count = bell_count.saturating_add(1);
                    stage.emit_bell_skirt(pos, half, (Vec3::new(0.32, 0.78, 0.96), -0.58), target);
                }
                CellRole::AttitudePod => {
                    rcs_count = rcs_count.saturating_add(1);
                    stage.emit_pod_rosette(pos, half.x * 0.95, false, target);
                }
                CellRole::ActiveMainBell | CellRole::ActiveFusionBell => {
                    bell_count = bell_count.saturating_add(1);
                }
                CellRole::ActivePod => {
                    rcs_count = rcs_count.saturating_add(1);
                }
            });

            if !dedicated_nose {
                if let Some((c_pos, c_half)) = fuselage_crown {
                    let cap_h = Vec3::new(c_half.x * 0.65, c_half.y.min(0.18), c_half.z * 0.65);
                    let cap_pos = Vec3::new(c_pos.x, c_pos.y + c_half.y + cap_h.y, c_pos.z);
                    stage.emit_nose_pyramid(cap_pos, cap_h, target);
                }
            }

            // Pass 2: Active firing thruster housings in high-contrast glowing amber/sapphire
            self.wireframe.visit_cells(|role, pos, half| match role {
                CellRole::ActiveMainBell => {
                    stage.emit_bell_skirt(pos, half, (AMBER_WIRE, 0.98), target);
                }
                CellRole::ActiveFusionBell => {
                    stage.emit_bell_skirt(pos, half, (Vec3::new(0.46, 0.92, 1.00), 0.98), target);
                }
                CellRole::ActivePod => {
                    stage.emit_pod_rosette(pos, half.x * 1.08, true, target);
                }
                _ => {}
            });

            let tally = format!("RCS:{}  BELL:{}", rcs_count, bell_count);
            PixelStamp::new(0.0040 * ux, 0.0060, IDLE_POD)
                .emit_centered(&tally, x0 + (x1 - x0) * 0.50, y1 - 0.032, target);

            // Pass 3: Depth-sorted 3D exhaust jets with muzzle ignition sparks
            let mut sorted = [(0.0_f32, JetTone::VernierCyan, Vec3::ZERO, Vec3::NEG_Y, 0.0_f32); MAX_EXHAUST_JETS];
            let mut jlen = 0usize;
            self.wireframe.visit_jets(|tone, mouth, dir, thr| {
                if jlen < MAX_EXHAUST_JETS {
                    sorted[jlen] = (stage.cam_depth(mouth), tone, mouth, dir, thr);
                    jlen += 1;
                }
            });
            sorted[..jlen].sort_by(|a, b| a.0.total_cmp(&b.0));
            for &(_, tone, mouth, dir, thr) in &sorted[..jlen] {
                stage.emit_jet(tone, mouth, dir, thr, bounds, target);
            }

            if exhaust_lit {
                PixelStamp::new(0.0046 * ux, 0.0070, CYAN_WIRE)
                    .emit_centered("EXHAUST", x0 + 0.16 * ux, y0 + 0.048, target);
            }

            let anchor = fuselage_crown.map_or(Vec3::ZERO, |(p, h)| Vec3::new(p.x, p.y + h.y * 0.55, p.z));
            Self::emit_steer_arrow(stage, anchor, steer_local, bounds, ux, target);
        });
    }

    fn emit_steer_arrow(
        stage: CameraStage,
        anchor_local: Vec3,
        steer_local: Vec3,
        bounds: (f32, f32, f32, f32),
        ux: f32,
        target: &mut dyn MeshTarget,
    ) {
        let mag = steer_local.length();
        if mag <= 0.03 {
            return;
        }
        let reach = 0.62 + 0.68 * mag.clamp(0.0, 1.0);
        let a = stage.project(anchor_local);
        let raw_b = stage.project(anchor_local + (steer_local / mag) * reach);
        let b = Vec2::new(
            raw_b.x.clamp(bounds.0 + 0.06 * ux, bounds.2 - 0.06 * ux),
            raw_b.y.clamp(bounds.1 + 0.06, bounds.3 - 0.06),
        );
        let m = Vec2::new((b.x - a.x) / ux, b.y - a.y);
        let len = m.length();
        if len <= 0.012 {
            return;
        }
        let dir = m / len;
        let s = Vec2::new(-dir.y * ux, dir.x);
        let f = Vec2::new(dir.x * ux, dir.y);
        stage.emit_beam(a, b, 0.0082, STEER_HALO, -0.32, target);
        stage.emit_beam(a, b, 0.0034, STEER_PEAK, 0.98, target);
        stage.emit_beam(b - f * 0.026 + s * 0.022, b, 0.0032, STEER_PEAK, 0.98, target);
        stage.emit_beam(b - f * 0.026 - s * 0.022, b, 0.0032, STEER_PEAK, 0.98, target);

        let caption = if dir.x < -0.25 { "< STEER" } else { "STEER >" };
        let lx = (b.x + dir.x * 0.04 * ux).clamp(bounds.0 + 0.14 * ux, bounds.2 - 0.14 * ux);
        let ly = (b.y + 0.045).clamp(bounds.1 + 0.045, bounds.3 - 0.042);
        PixelStamp::new(0.0048 * ux, 0.0072, STEER_PEAK).emit_centered(caption, lx, ly, target);
    }
}

/// `OpticOverlay` decorator layering the camera-synchronized 3D wireframe ship HUD onto rocket cameras.
#[derive(Clone, Copy, Debug)]
pub struct WireframeShipOverlay<O> {
    origin: O,
    wireframe: HullWireframe,
    view_proj: Mat4,
    inv_aspect: f32,
}

impl<O> WireframeShipOverlay<O> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: O, wireframe: HullWireframe, view_proj: Mat4, inv_aspect: f32) -> Self {
        Self { origin, wireframe, view_proj, inv_aspect }
    }
}

impl<O: OpticOverlay> OpticOverlay for WireframeShipOverlay<O> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.origin.emit(target);
        WireframeShipCard::new(self.wireframe, self.view_proj, self.inv_aspect).emit(target);
    }
}
