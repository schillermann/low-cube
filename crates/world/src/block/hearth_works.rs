use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::wedge::WedgeHeading;

struct HearthPrism<'a> {
    cell: GridCell,
    floor: f32,
    heading: WedgeHeading,
    frame: &'a SurfaceFrame,
}

impl<'a> HearthPrism<'a> {
    const fn new(cell: GridCell, floor: f32, heading: WedgeHeading, frame: &'a SurfaceFrame) -> Self {
        Self { cell, floor, heading, frame }
    }

    fn local_point(&self, lateral: f32, forward: f32, height: f32) -> Vec3 {
        let (cu, cv) = self.cell.center();
        let (du, dv) = match self.heading {
            WedgeHeading::North => (lateral, -forward),
            WedgeHeading::South => (-lateral, forward),
            WedgeHeading::East => (forward, lateral),
            WedgeHeading::West => (-forward, -lateral),
        };
        self.frame.point_at(cu + du, cv + dv, self.floor + height)
    }

    fn local_axes(&self) -> (Vec3, Vec3, Vec3) {
        let n = self.frame.normal();
        let t = self.frame.tangent();
        let b = self.frame.bitangent();
        let (right, front) = match self.heading {
            WedgeHeading::North => (t, -b),
            WedgeHeading::South => (-t, b),
            WedgeHeading::East => (b, t),
            WedgeHeading::West => (-b, -t),
        };
        (right, front, n)
    }

    fn emit_box(&self, bounds: [f32; 6], top_col: Vec3, wall_col: Vec3, glow: f32, target: &mut dyn MeshTarget) {
        let [x0, x1, y0, y1, z0, z1] = bounds;
        let (right, front, up) = self.local_axes();
        let [b0, b1, b2, b3] = [
            self.local_point(x0, y0, z0),
            self.local_point(x1, y0, z0),
            self.local_point(x1, y1, z0),
            self.local_point(x0, y1, z0),
        ];
        let [t0, t1, t2, t3] = [
            self.local_point(x0, y0, z1),
            self.local_point(x1, y0, z1),
            self.local_point(x1, y1, z1),
            self.local_point(x0, y1, z1),
        ];
        target.push_quad(Quad::glowing(t0, t1, t2, t3, up, top_col, glow));
        target.push_quad(Quad::glowing(b0, b3, b2, b1, -up, wall_col, glow));
        target.push_quad(Quad::glowing(b0, b1, t1, t0, -front, wall_col, glow));
        target.push_quad(Quad::glowing(t3, t2, b2, b3, front, wall_col, glow));
        target.push_quad(Quad::glowing(b3, b0, t0, t3, -right, wall_col, glow));
        target.push_quad(Quad::glowing(b1, b2, t2, t1, right, wall_col, glow));
    }
}

/// Domed stone culinary bread-and-roast oven (`~250 °C`) with glowing amber hearth mouth and flue (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct BakingOvenMesh {
    cell: GridCell,
    floor_elevation: f32,
    heading: WedgeHeading,
}

impl BakingOvenMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32, heading: WedgeHeading) -> Self {
        Self { cell, floor_elevation, heading }
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let prism = HearthPrism::new(self.cell, self.floor_elevation, self.heading, frame);
        let stone_base = Vec3::new(0.56, 0.57, 0.59);
        let stone_flank = Vec3::new(0.45, 0.46, 0.48);
        let clay_dome = Vec3::new(0.74, 0.44, 0.28);
        let clay_wall = Vec3::new(0.62, 0.35, 0.21);
        let soot_arch = Vec3::new(0.18, 0.16, 0.15);
        let bake_embers = Vec3::new(0.96, 0.46, 0.12);
        prism.emit_box([-0.23, 0.23, -0.23, 0.23, 0.0, 0.16], stone_base, stone_flank, 0.0, target);
        prism.emit_box([-0.21, 0.21, -0.20, 0.18, 0.16, 0.36], clay_dome, clay_wall, 0.0, target);
        prism.emit_box([-0.16, 0.16, -0.17, 0.14, 0.36, 0.44], clay_dome, clay_wall, 0.0, target);
        prism.emit_box([-0.06, 0.06, -0.19, -0.09, 0.44, 0.50], stone_base, soot_arch, 0.0, target);
        prism.emit_box([-0.11, 0.11, 0.16, 0.22, 0.16, 0.31], soot_arch, soot_arch, 0.0, target);
        prism.emit_box([-0.08, 0.08, 0.15, 0.225, 0.17, 0.25], bake_embers, bake_embers, 0.42, target);
    }
}

/// Refractory shaft smelting bloomery (`~1,300 °C`) with iron bands and incandescent tuyere blast port (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct BloomeryForgeMesh {
    cell: GridCell,
    floor_elevation: f32,
    heading: WedgeHeading,
}

impl BloomeryForgeMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32, heading: WedgeHeading) -> Self {
        Self { cell, floor_elevation, heading }
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let prism = HearthPrism::new(self.cell, self.floor_elevation, self.heading, frame);
        let basalt_plinth = Vec3::new(0.28, 0.29, 0.33);
        let shaft_stone = Vec3::new(0.38, 0.36, 0.35);
        let iron_hoop = Vec3::new(0.18, 0.20, 0.24);
        let white_heat = Vec3::new(1.00, 0.74, 0.26);
        let throat_slag = Vec3::new(0.94, 0.38, 0.10);
        prism.emit_box([-0.23, 0.23, -0.23, 0.23, 0.0, 0.12], basalt_plinth, basalt_plinth, 0.0, target);
        prism.emit_box([-0.19, 0.19, -0.19, 0.19, 0.12, 0.34], shaft_stone, basalt_plinth, 0.0, target);
        prism.emit_box([-0.205, 0.205, -0.205, 0.205, 0.21, 0.25], iron_hoop, iron_hoop, 0.0, target);
        prism.emit_box([-0.15, 0.15, -0.15, 0.15, 0.34, 0.49], shaft_stone, basalt_plinth, 0.0, target);
        prism.emit_box([-0.165, 0.165, -0.165, 0.165, 0.41, 0.44], iron_hoop, iron_hoop, 0.0, target);
        prism.emit_box([-0.09, 0.09, -0.09, 0.09, 0.48, 0.50], throat_slag, throat_slag, 0.56, target);
        prism.emit_box([-0.065, 0.065, 0.18, 0.235, 0.14, 0.24], white_heat, white_heat, 0.82, target);
    }
}

/// Horned steel smithy anvil mounted atop a heavy hewn oak timber block (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct SmithyAnvilMesh {
    cell: GridCell,
    floor_elevation: f32,
    heading: WedgeHeading,
}

impl SmithyAnvilMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32, heading: WedgeHeading) -> Self {
        Self { cell, floor_elevation, heading }
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let prism = HearthPrism::new(self.cell, self.floor_elevation, self.heading, frame);
        let stump_end = Vec3::new(0.76, 0.60, 0.38);
        let stump_bark = Vec3::new(0.34, 0.22, 0.12);
        let anvil_waist = Vec3::new(0.22, 0.24, 0.28);
        let anvil_face = Vec3::new(0.56, 0.60, 0.66);
        prism.emit_box([-0.18, 0.18, -0.18, 0.18, 0.0, 0.22], stump_end, stump_bark, 0.0, target);
        prism.emit_box([-0.19, 0.19, -0.19, 0.19, 0.16, 0.19], anvil_waist, anvil_waist, 0.0, target);
        prism.emit_box([-0.13, 0.13, -0.10, 0.10, 0.22, 0.28], anvil_waist, anvil_waist, 0.0, target);
        prism.emit_box([-0.08, 0.08, -0.08, 0.08, 0.28, 0.37], anvil_waist, anvil_waist, 0.0, target);
        prism.emit_box([-0.10, 0.10, -0.16, 0.14, 0.37, 0.46], anvil_face, anvil_waist, 0.0, target);
        prism.emit_box([-0.06, 0.06, 0.14, 0.23, 0.39, 0.45], anvil_face, anvil_waist, 0.0, target);
        prism.emit_box([-0.035, 0.035, 0.23, 0.28, 0.41, 0.445], anvil_face, anvil_waist, 0.0, target);
        prism.emit_box([-0.08, 0.08, -0.23, -0.16, 0.40, 0.46], anvil_face, anvil_waist, 0.0, target);
    }
}
