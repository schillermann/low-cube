use std::sync::OnceLock;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget, Quad};
use super::arboreal::Arboreal;
use super::full_cube_ground::FullCubeGround;
use super::meridian_ground::MeridianGround;
use super::planting_ground::PlantingGround;
use super::spawn_ground::SpawnSanctuaryGround;
use super::surface_ground::SurfaceGround;
use super::woodland_ground::WoodlandGround;
use super::zoned_forest::ZonedForest;

struct FloraQuadTarget<'a>(&'a mut Vec<Quad>);

impl MeshTarget for FloraQuadTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.0.push(quad);
    }
}

/// Composite decorator populating a planetary face with climate-adapted arboreal specimens.
pub struct ForestedFace {
    origin: Box<dyn Face>,
    frame: SurfaceFrame,
    trees: Vec<Box<dyn Arboreal>>,
    cached_flora: OnceLock<Vec<Quad>>,
}

impl ForestedFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Box<dyn Face>,
        frame: SurfaceFrame,
        trees: Vec<Box<dyn Arboreal>>,
        cached_flora: OnceLock<Vec<Quad>>,
    ) -> Self {
        Self {
            origin,
            frame,
            trees,
            cached_flora,
        }
    }

    /// Secondary constructor populating the face with native trees distributed across terrain.
    pub fn in_zone(origin: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let frame = zone.surface_frame(half_size);
        let forest = ZonedForest::new(zone);

        let mut trees = Vec::new();
        {
            let surface_ground = Box::new(SurfaceGround::new(origin.as_ref(), &frame, half_size));
            let full_cube_ground = Box::new(FullCubeGround::new(surface_ground, origin.as_ref(), &frame, half_size));
            let woodland_ground = Box::new(WoodlandGround::in_zone(full_cube_ground, zone));
            let meridian_ground = Box::new(MeridianGround::new(woodland_ground, 1.2));
            let ground = SpawnSanctuaryGround::new(meridian_ground, zone, 4.5);

            let border_margin = 4.0;
            let min_coord = (-(half_size - border_margin) * 2.0).floor() as i32;
            let max_coord = ((half_size - border_margin) * 2.0).floor() as i32;

            let step = if half_size >= 48.0 { 8 } else { 6 };

            let mut cell_u = min_coord;
            while cell_u <= max_coord {
                let mut cell_v = min_coord;
                while cell_v <= max_coord {
                    let hash = cell_u.wrapping_mul(73856093) ^ cell_v.wrapping_mul(19349663);
                    if ground.receptive(cell_u, cell_v, hash) {
                        let tree_u = (cell_u as f32) * 0.5 + 0.25;
                        let tree_v = (cell_v as f32) * 0.5 + 0.25;
                        let elevation = ground.ground_elevation(cell_u, cell_v);
                        trees.push(forest.plant(frame, tree_u, tree_v, elevation));
                    }

                    cell_v += step;
                }
                cell_u += step;
            }
        }

        Self::new(origin, frame, trees, OnceLock::new())
    }
}

impl Face for ForestedFace {
    fn matches(&self, point: Vec3) -> bool {
        self.origin.matches(point)
    }

    fn surface_up(&self) -> Vec3 {
        self.origin.surface_up()
    }

    fn pull_direction(&self) -> Vec3 {
        self.origin.pull_direction()
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let mut position = self.origin.constrain(point, distance);
        let (u, v) = self.frame.decompose(point);
        for tree in &self.trees {
            position = tree.constrain_surface(position, u, v, &self.frame, distance);
        }
        position
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_surface(target);
        let flora = self.cached_flora.get_or_init(|| {
            let mut quads = Vec::with_capacity(self.trees.len() * 24);
            let mut collector = FloraQuadTarget(&mut quads);
            for tree in &self.trees {
                tree.emit_surface(&mut collector);
            }
            quads
        });
        for &quad in flora {
            target.push_quad(quad);
        }
    }

    fn canopy_drag(&self, point: Vec3) -> f32 {
        let (u, v) = self.frame.decompose(point);
        let height = self.frame.height_of(point);
        let mut peak = self.origin.canopy_drag(point);
        for tree in &self.trees {
            let drag = tree.canopy_drag(u, v, height);
            if drag > peak {
                peak = drag;
            }
        }
        peak
    }

    fn excavate(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.origin.excavate(eye, direction)
    }

    fn place_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.origin.place_block(eye, direction, avatar_pos)
    }
}
