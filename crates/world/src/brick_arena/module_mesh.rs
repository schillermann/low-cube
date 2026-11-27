use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use crate::relief::MountDomain;
use super::coord::CellCoord;
use super::module::{MachineKind, ScalableKind, SpaceModule};
use super::module_spec::ModuleSpec;

const HOLOGRAM_GLOW: f32 = 1.9;

/// Sculpted low-poly Space-Age 3D representation of a `ModuleSpec` anchored at `anchor` (`3` fields).
#[derive(Clone, Copy, Debug)]
pub struct ModuleSculpture {
    anchor: CellCoord,
    spec: ModuleSpec,
    datum: f32,
}

impl ModuleSculpture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(anchor: CellCoord, spec: ModuleSpec, datum: f32) -> Self {
        Self { anchor, spec, datum }
    }

    fn bounds(self) -> (f32, f32, f32, f32, f32, f32) {
        let (u, v, _) = self.anchor.components();
        let (du, dv, dw) = self.spec.span().dimensions();
        let step = GridCell::SPAN;
        let u0 = (u as f32) * step;
        let v0 = (v as f32) * step;
        let h0 = self.anchor.floor_elev(self.datum);
        (u0, u0 + (du as f32) * step, v0, v0 + (dv as f32) * step, h0, h0 + (dw as f32) * step)
    }

    pub fn emit(self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (u0, u1, v0, v1, h0, h1) = self.bounds();
        let (b_span, t_span, h_span) = (u1 - u0, v1 - v0, h1 - h0);
        let titanium = Vec3::new(0.66, 0.72, 0.80);
        let carbon = Vec3::new(0.20, 0.24, 0.30);
        let copper = Vec3::new(0.84, 0.48, 0.22);
        let phosphor = Vec3::new(0.18, 0.94, 0.68);
        let arc_orange = Vec3::new(0.98, 0.56, 0.16);
        let cyan = Vec3::new(0.24, 0.88, 0.98);

        match self.spec.module() {
            SpaceModule::Machine(MachineKind::ArcFurnace) => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.04, v1 - 0.04, h0, h0 + h_span * 0.72, frame, carbon, titanium, true, target);
                BlockMedium::emit_box(u0 + 0.16, u1 - 0.16, v0 + 0.16, v1 - 0.16, h0 + h_span * 0.72, h1, frame, titanium, copper, false, target);
                Self::emit_glow_panel(u0 + 0.20, u1 - 0.20, v0 + 0.02, h0 + 0.28, h0 + 0.68, frame, arc_orange, 2.2, target);
            }
            SpaceModule::Machine(MachineKind::AssemblyBench) => {
                BlockMedium::emit_box(u0 + 0.03, u1 - 0.03, v0 + 0.03, v1 - 0.03, h0, h0 + h_span * 0.52, frame, carbon, titanium, true, target);
                BlockMedium::emit_box(u0 + 0.08, u1 - 0.08, v0 + 0.10, v1 - 0.10, h0 + h_span * 0.52, h1, frame, titanium, cyan, false, target);
            }
            SpaceModule::Machine(MachineKind::ElectrolysisPlant) => {
                let mid_u = (u0 + u1) * 0.5;
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.04, v1 - 0.04, h0, h0 + 0.24, frame, carbon, titanium, true, target);
                BlockMedium::emit_box(u0 + 0.08, mid_u - 0.04, v0 + 0.10, v1 - 0.10, h0 + 0.24, h1 - 0.06, frame, cyan, titanium, false, target);
                BlockMedium::emit_box(mid_u + 0.04, u1 - 0.08, v0 + 0.10, v1 - 0.10, h0 + 0.24, h1 - 0.06, frame, copper, titanium, false, target);
            }
            SpaceModule::Machine(MachineKind::AirlockChamber) => {
                BlockMedium::emit_box(u0, u1, v0, v1, h0, h0 + 0.12, frame, carbon, titanium, true, target);
                BlockMedium::emit_box(u0, u1, v0, v1, h1 - 0.14, h1, frame, carbon, titanium, true, target);
                BlockMedium::emit_box(u0, u0 + 0.12, v0, v1, h0 + 0.12, h1 - 0.14, frame, titanium, titanium, false, target);
                BlockMedium::emit_box(u1 - 0.12, u1, v0, v1, h0 + 0.12, h1 - 0.14, frame, titanium, titanium, false, target);
                Self::emit_glow_panel(u0 + 0.28, u1 - 0.28, v0 + 0.01, h1 - 0.24, h1 - 0.16, frame, phosphor, 1.8, target);
            }
            SpaceModule::Machine(MachineKind::PilotSeat) => {
                BlockMedium::emit_box(u0 + 0.14, u1 - 0.14, v0 + 0.14, v1 - 0.14, h0, h0 + 0.45, frame, carbon, titanium, true, target);
                BlockMedium::emit_box(u0 + 0.16, u1 - 0.16, v1 - 0.28, v1 - 0.08, h0 + 0.45, h1 - 0.08, frame, carbon, copper, false, target);
            }
            SpaceModule::Machine(MachineKind::PhosphorConsole) => {
                BlockMedium::emit_box(u0 + 0.05, u1 - 0.05, v0 + 0.05, v1 - 0.05, h0, h0 + h_span * 0.62, frame, carbon, titanium, true, target);
                BlockMedium::emit_box(u0 + 0.10, u1 - 0.10, v0 + 0.12, v1 - 0.12, h0 + h_span * 0.62, h1, frame, carbon, phosphor, false, target);
                Self::emit_glow_panel(u0 + 0.16, u1 - 0.16, v0 + 0.10, h0 + h_span * 0.66, h1 - 0.06, frame, phosphor, 1.9, target);
            }
            SpaceModule::Machine(MachineKind::DockingRing) => {
                BlockMedium::emit_box(u0 + 0.02, u1 - 0.02, v0 + 0.02, v0 + 0.18, h0, h1, frame, titanium, copper, true, target);
                BlockMedium::emit_box(u0 + 0.02, u1 - 0.02, v1 - 0.18, v1 - 0.02, h0, h1, frame, titanium, copper, true, target);
                BlockMedium::emit_box(u0 + 0.02, u0 + 0.18, v0 + 0.18, v1 - 0.18, h0, h1, frame, titanium, copper, true, target);
                BlockMedium::emit_box(u1 - 0.18, u1 - 0.02, v0 + 0.18, v1 - 0.18, h0, h1, frame, titanium, copper, true, target);
            }
            SpaceModule::Machine(MachineKind::CrewBerth) => {
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v0 + 0.04, v1 - 0.04, h0, h0 + 0.34, frame, titanium, Vec3::new(0.84, 0.88, 0.94), true, target);
                BlockMedium::emit_box(u0 + 0.04, u1 - 0.04, v1 - 0.22, v1 - 0.04, h0 + 0.34, h1 - 0.12, frame, carbon, cyan, false, target);
            }
            SpaceModule::Scalable(ScalableKind::PropulsionBell) => {
                let inset_u = b_span * 0.22;
                let inset_v = t_span * 0.22;
                BlockMedium::emit_box(u0 + inset_u, u1 - inset_u, v0 + inset_v, v1 - inset_v, h0 + h_span * 0.55, h1, frame, titanium, copper, true, target);
                BlockMedium::emit_box(u0 + b_span * 0.06, u1 - b_span * 0.06, v0 + t_span * 0.06, v1 - t_span * 0.06, h0, h0 + h_span * 0.55, frame, carbon, titanium, true, target);
            }
            SpaceModule::Scalable(ScalableKind::PropellantTank) => {
                BlockMedium::emit_box(u0 + b_span * 0.06, u1 - b_span * 0.06, v0 + t_span * 0.06, v1 - t_span * 0.06, h0 + 0.06, h1 - 0.06, frame, Vec3::new(0.86, 0.89, 0.94), copper, true, target);
                BlockMedium::emit_box(u0 + b_span * 0.04, u1 - b_span * 0.04, v0 + t_span * 0.04, v1 - t_span * 0.04, h0 + h_span * 0.44, h0 + h_span * 0.56, frame, copper, copper, false, target);
            }
            SpaceModule::Scalable(ScalableKind::ThermalFin) => {
                BlockMedium::emit_box(u0 + 0.03, u1 - 0.03, v0 + t_span * 0.36, v1 - t_span * 0.36, h0, h1, frame, carbon, arc_orange, true, target);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_glow_panel(u0: f32, u1: f32, v: f32, h0: f32, h1: f32, frame: &SurfaceFrame, color: Vec3, glow: f32, target: &mut dyn MeshTarget) {
        let [p0, p1, p2, p3] = [
            frame.point_at(u0, v, h0),
            frame.point_at(u1, v, h0),
            frame.point_at(u1, v, h1),
            frame.point_at(u0, v, h1),
        ];
        target.push_quad(Quad::glowing(p0, p1, p2, p3, -frame.bitangent(), color, glow));
    }
}

/// Luminous `B × T × H` 3D drag-hologram preview for multi-block & scalable Space-Age modules (`3` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpanHologram {
    anchor: CellCoord,
    spec: ModuleSpec,
    domain: MountDomain,
}

impl SpanHologram {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(anchor: CellCoord, spec: ModuleSpec, domain: MountDomain) -> Self {
        Self { anchor, spec, domain }
    }

    pub const fn spec(self) -> ModuleSpec { self.spec }
    pub const fn anchor(self) -> CellCoord { self.anchor }

    pub fn emit(self, datum: f32, frame: &SurfaceFrame, obstructed: bool, target: &mut dyn MeshTarget) {
        let (u, v, _) = self.anchor.components();
        let (du, dv, dw) = self.spec.span().dimensions();
        let step = GridCell::SPAN;
        let (u0, u1) = ((u as f32) * step, ((u + i32::from(du)) as f32) * step);
        let (v0, v1) = ((v as f32) * step, ((v + i32::from(dv)) as f32) * step);
        let (h0, h1) = (self.anchor.floor_elev(datum), self.anchor.floor_elev(datum) + (dw as f32) * step);
        let tone = if obstructed {
            Vec3::new(0.98, 0.28, 0.24)
        } else if self.domain == MountDomain::CraftRig {
            Vec3::new(0.20, 0.90, 1.00)
        } else {
            Vec3::new(0.24, 0.96, 0.62)
        };
        let arm = 0.14_f32;
        let bar = 0.022_f32;
        for &(cu, su) in &[(u0, 1.0_f32), (u1, -1.0_f32)] {
            for &(cv, sv) in &[(v0, 1.0_f32), (v1, -1.0_f32)] {
                for &(ch, sh) in &[(h0, 1.0_f32), (h1, -1.0_f32)] {
                    let (ua, ub) = if su > 0.0 { (cu, cu + arm) } else { (cu - arm, cu) };
                    let (va, vb) = if sv > 0.0 { (cv, cv + arm) } else { (cv - arm, cv) };
                    let (ha, hb) = if sh > 0.0 { (ch, ch + arm) } else { (ch - arm, ch) };
                    let (bu0, bu1) = if su > 0.0 { (cu, cu + bar) } else { (cu - bar, cu) };
                    let (bv0, bv1) = if sv > 0.0 { (cv, cv + bar) } else { (cv - bar, cv) };
                    let (bh0, bh1) = if sh > 0.0 { (ch, ch + bar) } else { (ch - bar, ch) };
                    Self::emit_glow_beam(ua, ub, bv0, bv1, bh0, bh1, frame, tone, target);
                    Self::emit_glow_beam(bu0, bu1, va, vb, bh0, bh1, frame, tone, target);
                    Self::emit_glow_beam(bu0, bu1, bv0, bv1, ha, hb, frame, tone, target);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_glow_beam(u0: f32, u1: f32, v0: f32, v1: f32, h0: f32, h1: f32, frame: &SurfaceFrame, color: Vec3, target: &mut dyn MeshTarget) {
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        let [b0, b1, b2, b3] = [frame.point_at(u0, v0, h0), frame.point_at(u1, v0, h0), frame.point_at(u1, v1, h0), frame.point_at(u0, v1, h0)];
        let [p0, p1, p2, p3] = [frame.point_at(u0, v0, h1), frame.point_at(u1, v0, h1), frame.point_at(u1, v1, h1), frame.point_at(u0, v1, h1)];
        target.push_quad(Quad::glowing(p0, p1, p2, p3, n, color, HOLOGRAM_GLOW));
        target.push_quad(Quad::glowing(b0, b3, b2, b1, -n, color, HOLOGRAM_GLOW));
        target.push_quad(Quad::glowing(b0, b1, p1, p0, -b, color, HOLOGRAM_GLOW));
        target.push_quad(Quad::glowing(p3, p2, b2, b3, b, color, HOLOGRAM_GLOW));
        target.push_quad(Quad::glowing(b3, b0, p0, p3, -t, color, HOLOGRAM_GLOW));
        target.push_quad(Quad::glowing(b1, b2, p2, p1, t, color, HOLOGRAM_GLOW));
    }
}
