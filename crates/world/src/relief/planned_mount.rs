use glam::Vec3;
use crate::block::{BlockMedium, BlockMold, GridCell};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Construction target domain distinguishing vehicle/crane assembly from static World-Cube earthworks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountDomain {
    WorldCube,
    CraftRig,
}

impl MountDomain {
    pub const fn tint_color(self) -> Vec3 {
        match self {
            Self::CraftRig => Vec3::new(0.16, 0.88, 1.00),
            Self::WorldCube => Vec3::new(0.38, 0.94, 0.56),
        }
    }

    pub const fn bracket_color(self) -> Vec3 {
        match self {
            Self::CraftRig => Vec3::new(0.45, 0.96, 1.00),
            Self::WorldCube => Vec3::new(0.68, 1.00, 0.72),
        }
    }

    pub const fn glow_strength(self) -> f32 {
        match self {
            Self::CraftRig => 0.80,
            Self::WorldCube => 0.72,
        }
    }
}

/// Spatial cell coordinate and elevation datum for a planned block placement.
#[derive(Clone, Copy, Debug)]
pub struct MountSite {
    frame: SurfaceFrame,
    cell: GridCell,
    floor_elev: f32,
    shift: Vec3,
}

impl MountSite {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_shift(frame: SurfaceFrame, cell: GridCell, floor_elev: f32, shift: Vec3) -> Self {
        Self { frame, cell, floor_elev, shift }
    }

    pub const fn new(frame: SurfaceFrame, cell: GridCell, floor_elev: f32) -> Self {
        Self::with_shift(frame, cell, floor_elev, Vec3::ZERO)
    }

    pub fn translated(self, delta: Vec3) -> Self {
        Self::with_shift(self.frame, self.cell, self.floor_elev, self.shift + delta)
    }

    pub fn cell_coordinates(self) -> (i32, i32, f32) {
        self.cell.feed_coordinates(|u, v| (u, v, self.floor_elev))
    }

    fn volume_center(self) -> Vec3 {
        let (uc, vc) = self.cell.center();
        self.frame.point_at(uc, vc, self.floor_elev + BLOCK_STEP * 0.5)
    }
}

struct HologramTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    center_shift: (Vec3, Vec3),
    domain: MountDomain,
}

impl MeshTarget for HologramTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let raw_glow = quad.radiate(|g| g);
        if raw_glow > 1.5 || raw_glow < 0.0 {
            return;
        }
        let (center, shift) = self.center_shift;
        let tint = self.domain.tint_color();
        let glow = self.domain.glow_strength();
        self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, norm, col| {
            let holo_col = col.lerp(tint, 0.70);
            Quad::glowing(
                v0.lerp(center, 0.024) + shift,
                v1.lerp(center, 0.024) + shift,
                v2.lerp(center, 0.024) + shift,
                v3.lerp(center, 0.024) + shift,
                norm,
                holo_col,
                glow,
            )
        }));
    }
}

/// Verified 3D block mount uniting target cell, mold orientation, and construction domain.
#[derive(Clone, Copy, Debug)]
pub struct PlannedMount {
    site: MountSite,
    mold: BlockMold,
    aim_dir: Vec3,
    domain: MountDomain,
}

impl PlannedMount {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(site: MountSite, mold: BlockMold, aim_dir: Vec3, domain: MountDomain) -> Self {
        Self { site, mold, aim_dir, domain }
    }

    pub const fn domain(self) -> MountDomain {
        self.domain
    }

    pub fn cell_coordinates(self) -> (i32, i32, f32) {
        self.site.cell_coordinates()
    }

    pub fn translated(self, delta: Vec3) -> Self {
        Self::new(self.site.translated(delta), self.mold, self.aim_dir, self.domain)
    }

    /// Emits the shape-faithful luminous hologram preview and 3D corner docking brackets.
    pub fn emit_hologram(&self, target: &mut dyn MeshTarget) {
        if let Some(spec) = crate::ModuleSpec::from_mold(self.mold) {
            let (u, v, z) = self.site.cell_coordinates();
            let anchor = crate::CellCoord::from_floor(u, v, z, 0.0);
            let mut shifted = BracketShiftTarget { shift: self.site.shift, col: self.domain.bracket_color(), inner: target };
            crate::SpanHologram::new(anchor, spec, self.domain).emit(0.0, &self.site.frame, false, &mut shifted);
            return;
        }
        let atom = self.mold.atom_for(self.aim_dir, &self.site.frame);
        let center = self.site.volume_center();
        let tint = self.domain.tint_color();
        {
            let mut holo = HologramTarget {
                inner: target,
                center_shift: (center, self.site.shift),
                domain: self.domain,
            };
            atom.at_floor(self.site.floor_elev)
                .with_shape(|s| s.emit_shape(self.site.cell, self.site.floor_elev, &self.site.frame, tint, &mut holo));
        }
        self.emit_docking_brackets(target);
    }

    /// Emits a stretched `B × T × H` drag-hologram preview between `self` and `end_mount`.
    pub fn emit_drag_hologram(&self, end_mount: Self, extra_height_steps: i32, target: &mut dyn MeshTarget) {
        let Some(kind) = crate::SpaceModule::from_medium(self.mold.medium()) else {
            self.emit_hologram(target);
            return;
        };
        let (u0, v0, z0) = self.site.cell_coordinates();
        let (u1, v1, z1) = end_mount.site.cell_coordinates();
        if u0 == u1 && v0 == v1 && extra_height_steps == 0 {
            self.emit_hologram(target);
            return;
        }
        let (anchor, span) = crate::ModuleSpan::dragged_between(
            crate::CellCoord::from_floor(u0, v0, z0, 0.0),
            crate::CellCoord::from_floor(u1, v1, z1, 0.0),
            extra_height_steps,
        );
        let turn = crate::CardinalTurn::from_code(self.mold.chronicle_bytes()[1]);
        let spec = crate::ModuleSpec::new(kind, span, turn);
        let mut shifted = BracketShiftTarget { shift: self.site.shift, col: self.domain.bracket_color(), inner: target };
        crate::SpanHologram::new(anchor, spec, self.domain).emit(0.0, &self.site.frame, false, &mut shifted);
    }

    /// Resolves `(anchor_eye, anchor_dir, ModuleSpec)` for a dragged `B × T × H` scalable module between `self` and `end_mount`.
    pub fn resolve_dragged_module(
        &self,
        end_mount: Self,
        extra_height_steps: i32,
    ) -> Option<(Vec3, Vec3, crate::ModuleSpec)> {
        let crate::SpaceModule::Scalable(kind) = crate::SpaceModule::from_medium(self.mold.medium())? else {
            return None;
        };
        let (u0, v0, z0) = self.site.cell_coordinates();
        let (u1, v1, z1) = end_mount.site.cell_coordinates();
        let (anchor, spec) = if u0 == u1 && v0 == v1 && extra_height_steps == 0 {
            (crate::CellCoord::from_floor(u0, v0, z0, 0.0), crate::ModuleSpec::from_mold(self.mold)?)
        } else {
            let (anc, span) = crate::ModuleSpan::dragged_between(
                crate::CellCoord::from_floor(u0, v0, z0, 0.0),
                crate::CellCoord::from_floor(u1, v1, z1, 0.0),
                extra_height_steps,
            );
            let turn = crate::CardinalTurn::from_code(self.mold.chronicle_bytes()[1]);
            (anc, crate::ModuleSpec::scalable(kind, span, turn))
        };
        let (au, av, _) = anchor.components();
        let az = anchor.floor_elev(0.0);
        let (uc, vc) = GridCell::new(au, av).center();
        let eye = self.site.frame.point_at(uc, vc, az + 0.45) + self.site.shift;
        let dir = -self.site.frame.normal();
        Some((eye, dir, spec))
    }

    fn emit_docking_brackets(&self, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.site.cell.u_bounds(), self.site.cell.v_bounds());
        let (z0, z1) = (self.site.floor_elev, self.site.floor_elev + BLOCK_STEP);
        let (arm, thick) = (0.105, 0.014);
        let col = self.domain.bracket_color();
        let mut rod = |au0: f32, au1: f32, av0: f32, av1: f32, az0: f32, az1: f32| {
            let mut shifted = BracketShiftTarget { shift: self.site.shift, col, inner: target };
            BlockMedium::emit_box(au0, au1, av0, av1, az0, az1, &self.site.frame, col, col, false, &mut shifted);
        };
        for &(cu, su) in &[(u0, 1.0f32), (u1, -1.0f32)] {
            let (bu0, bu1) = if su > 0.0 { (cu, cu + arm) } else { (cu - arm, cu) };
            let (tu0, tu1) = if su > 0.0 { (cu, cu + thick) } else { (cu - thick, cu) };
            for &(cv, sv) in &[(v0, 1.0f32), (v1, -1.0f32)] {
                let (bv0, bv1) = if sv > 0.0 { (cv, cv + arm) } else { (cv - arm, cv) };
                let (tv0, tv1) = if sv > 0.0 { (cv, cv + thick) } else { (cv - thick, cv) };
                for &(cz, sz) in &[(z0, 1.0f32), (z1, -1.0f32)] {
                    let (bz0, bz1) = if sz > 0.0 { (cz, cz + arm) } else { (cz - arm, cz) };
                    let (tz0, tz1) = if sz > 0.0 { (cz, cz + thick) } else { (cz - thick, cz) };
                    rod(bu0, bu1, tv0, tv1, tz0, tz1);
                    rod(tu0, tu1, bv0, bv1, tz0, tz1);
                    rod(tu0, tu1, tv0, tv1, bz0, bz1);
                }
            }
        }
    }
}

struct BracketShiftTarget<'a> {
    shift: Vec3,
    col: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl MeshTarget for BracketShiftTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let (s, col) = (self.shift, self.col);
        self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, norm, _| {
            Quad::glowing(v0 + s, v1 + s, v2 + s, v3 + s, norm, col, 0.94)
        }));
    }
}
