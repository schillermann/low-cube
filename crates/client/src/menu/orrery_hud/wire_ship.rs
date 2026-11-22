use glam::{Mat4, Vec2, Vec3};
use world::{CellRole, HullWireframe, MeshTarget, Quad};
use crate::menu::glyph::PixelStamp;
use crate::menu::widget::ScreenBounds;
use super::overlays::OpticOverlay;
use super::wire_stage::{AMBER_WIRE, CARD_GLASS, CARD_RIM, CYAN_WIRE, CameraStage};

/// Bottom-left camera-synchronized 3D wireframe rocket card contrasting `EXHAUST` against `STEER >`.
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
            let mut crown_cell: Option<(Vec3, f32)> = None;

            self.wireframe.visit_cells(|role, pos, half| {
                match role {
                    CellRole::HullCube => {
                        stage.emit_box(pos, Vec3::splat(half), CYAN_WIRE, 0.0022, target);
                        if crown_cell.is_none_or(|(prev, _)| pos.y > prev.y) {
                            crown_cell = Some((pos, half));
                        }
                    }
                    CellRole::NoseApex => {
                        dedicated_nose = true;
                        stage.emit_nose_pyramid(pos, half, target);
                    }
                    CellRole::MainBell => {
                        let col = if exhaust_lit { AMBER_WIRE } else { CYAN_WIRE };
                        stage.emit_bell_skirt(pos, half, col, target);
                    }
                    CellRole::FusionBell => {
                        stage.emit_bell_skirt(pos, half, Vec3::new(0.42, 0.88, 1.00), target);
                    }
                    CellRole::AttitudePod => {
                        stage.emit_box(pos, Vec3::splat(half * 0.78), CYAN_WIRE, 0.0019, target);
                    }
                    CellRole::ActivePod => {
                        stage.emit_box(pos, Vec3::splat(half * 0.82), AMBER_WIRE, 0.0025, target);
                    }
                }
            });

            if !dedicated_nose {
                if let Some((top_pos, half)) = crown_cell {
                    let cap_center = Vec3::new(top_pos.x, top_pos.y + half * 1.65, top_pos.z);
                    stage.emit_nose_pyramid(cap_center, half * 0.82, target);
                }
            }

            self.wireframe.visit_jets(|tone, mouth, dir, thr| {
                stage.emit_jet(tone, mouth, dir, thr, bounds, target);
            });

            if exhaust_lit {
                PixelStamp::new(0.0046 * ux, 0.0070, CYAN_WIRE)
                    .emit_centered("EXHAUST", x0 + 0.16 * ux, y0 + 0.048, target);
            }

            let anchor = crown_cell.map_or(Vec3::ZERO, |(p, h)| Vec3::new(p.x * 0.4, p.y + h * 0.4, p.z * 0.4));
            stage.emit_steer_vector(anchor, steer_local, bounds, target);
        });
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
