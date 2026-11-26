use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;

/// Walkable architectural stone plinth height (`0.25 m`, below `0.35 m` human stride threshold).
pub const PLINTH_HEIGHT: f32 = GridCell::SPAN * 0.5;

/// Focused 3D stone plinth sculpture contract ("Tell, Don't Ask").
trait PlinthSculpture {
    fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget);
}

/// Immutable 6-quad architectural prism with distinct cap/wall tones and top emissive glow (`3` fields).
#[derive(Clone, Copy, Debug)]
struct PlinthSlab {
    bounds: [f32; 6],
    tones: (Vec3, Vec3),
    glow: f32,
}

impl PlinthSlab {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(bounds: [f32; 6], tones: (Vec3, Vec3), glow: f32) -> Self {
        Self { bounds, tones, glow }
    }
}

impl PlinthSculpture for PlinthSlab {
    fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let [u0, u1, v0, v1, h0, h1] = self.bounds;
        let (top_col, wall_col) = self.tones;
        let [t, b, n] = [frame.tangent(), frame.bitangent(), frame.normal()];
        let [b0, b1, b2, b3] = [
            frame.point_at(u0, v0, h0),
            frame.point_at(u1, v0, h0),
            frame.point_at(u1, v1, h0),
            frame.point_at(u0, v1, h0),
        ];
        let [t0, t1, t2, t3] = [
            frame.point_at(u0, v0, h1),
            frame.point_at(u1, v0, h1),
            frame.point_at(u1, v1, h1),
            frame.point_at(u0, v1, h1),
        ];
        target.push_quad(Quad::glowing(t0, t1, t2, t3, n, top_col, self.glow));
        target.push_quad(Quad::new(b0, b3, b2, b1, -n, wall_col * 0.72));
        target.push_quad(Quad::new(b0, b1, t1, t0, -b, wall_col));
        target.push_quad(Quad::new(b2, b3, t3, t2, b, wall_col));
        target.push_quad(Quad::new(b1, b2, t2, t1, t, wall_col * 0.92));
        target.push_quad(Quad::new(b3, b0, t0, t3, -t, wall_col * 0.92));
    }
}

/// Decorator crowning an inner `PlinthSculpture` with an upper architectural collar or luminous inlay (`2` fields).
#[derive(Clone, Copy, Debug)]
struct CrownedPlinth<F, C> {
    foundation: F,
    crown: C,
}

impl<F, C> CrownedPlinth<F, C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(foundation: F, crown: C) -> Self {
        Self { foundation, crown }
    }
}

impl<F: PlinthSculpture, C: PlinthSculpture> PlinthSculpture for CrownedPlinth<F, C> {
    fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        self.foundation.emit(frame, target);
        self.crown.emit(frame, target);
    }
}

/// Sculpted stone arrival plinth with a luminous cyan-gold compass inlay marking an admin spawn point.
#[derive(Clone, Copy, Debug)]
pub struct SpawnPlinthMesh {
    cell: GridCell,
    floor_elevation: f32,
}

impl SpawnPlinthMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32) -> Self {
        Self { cell, floor_elevation }
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        PlinthSculpture::emit(self, frame, target);
    }
}

impl PlinthSculpture for SpawnPlinthMesh {
    fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (cu, cv) = self.cell.center();
        let h = GridCell::SPAN * 0.5;
        let (z0, z1) = (self.floor_elevation, self.floor_elevation + PLINTH_HEIGHT);
        let basalt = PlinthSlab::new(
            [cu - h, cu + h, cv - h, cv + h, z0, z1 - 0.03],
            (Vec3::new(0.30, 0.34, 0.42), Vec3::new(0.22, 0.25, 0.32)),
            0.0,
        );
        let brass = Vec3::new(0.92, 0.76, 0.28);
        let collar = PlinthSlab::new(
            [cu - h * 0.82, cu + h * 0.82, cv - h * 0.82, cv + h * 0.82, z1 - 0.03, z1],
            (brass, brass * 0.82),
            0.25,
        );
        let cyan = Vec3::new(0.22, 0.94, 0.98);
        let beacon = PlinthSlab::new(
            [cu - h * 0.46, cu + h * 0.46, cv - h * 0.46, cv + h * 0.46, z1, z1 + 0.012],
            (cyan, cyan),
            0.88,
        );
        CrownedPlinth::new(CrownedPlinth::new(basalt, collar), beacon).emit(frame, target);
    }
}

/// Indestructible primordial sanctuary origin plinth marking the planet's initial and fallback arrival site.
#[derive(Clone, Copy, Debug)]
pub struct OriginPlinthMesh {
    cell: GridCell,
    floor_elevation: f32,
}

impl OriginPlinthMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32) -> Self {
        Self { cell, floor_elevation }
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        PlinthSculpture::emit(self, frame, target);
    }
}

impl PlinthSculpture for OriginPlinthMesh {
    fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (cu, cv) = self.cell.center();
        let h = GridCell::SPAN * 0.5;
        let (z0, z1) = (self.floor_elevation, self.floor_elevation + PLINTH_HEIGHT);
        let alabaster = PlinthSlab::new(
            [cu - h, cu + h, cv - h, cv + h, z0, z1 - 0.035],
            (Vec3::new(0.76, 0.78, 0.82), Vec3::new(0.58, 0.61, 0.66)),
            0.0,
        );
        let gold = Vec3::new(0.95, 0.80, 0.32);
        let collar = PlinthSlab::new(
            [cu - h * 0.86, cu + h * 0.86, cv - h * 0.86, cv + h * 0.86, z1 - 0.035, z1],
            (gold, gold * 0.84),
            0.30,
        );
        let rose = Vec3::new(0.98, 0.88, 0.48);
        let east_west = PlinthSlab::new(
            [cu - h * 0.72, cu + h * 0.72, cv - h * 0.16, cv + h * 0.16, z1, z1 + 0.010],
            (rose, rose),
            0.55,
        );
        let north_south = PlinthSlab::new(
            [cu - h * 0.16, cu + h * 0.16, cv - h * 0.72, cv + h * 0.72, z1, z1 + 0.010],
            (rose, rose),
            0.55,
        );
        let emerald = Vec3::new(0.32, 0.96, 0.68);
        let heart = PlinthSlab::new(
            [cu - h * 0.34, cu + h * 0.34, cv - h * 0.34, cv + h * 0.34, z1, z1 + 0.018],
            (emerald, emerald),
            0.92,
        );
        CrownedPlinth::new(
            CrownedPlinth::new(CrownedPlinth::new(CrownedPlinth::new(alabaster, collar), east_west), north_south),
            heart,
        )
        .emit(frame, target);
    }
}
