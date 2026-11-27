use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::brick_arena::{
    BrickCell, BrickStructure, BrickVolume, CellCoord, DismantledSpan, ModuleSpec, RawAtom,
    NICHE_CONTACT_TOLERANCE,
};
use crate::frame::SurfaceFrame;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::turf_habitat::{CanopyHabitat, NaturalHabitat, TurfHabitat};
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::world_snapshots::{SectorCanvas, SectorVault};
use super::{MoldedStructure, SanctuaryPlinth};

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Static planetary blocks and multi-block modules backed by `InertialBricks<BrickArena>` (`2` fields, `0` untouched `Brick8`s).
pub struct WorldBlocks {
    grid: BrickStructure,
    plinth: SanctuaryPlinth,
}

impl WorldBlocks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_plinth(grid: BrickStructure, plinth: SanctuaryPlinth) -> Self {
        Self { grid, plinth }
    }

    pub const fn new(grid: BrickStructure) -> Self {
        Self::with_plinth(grid, SanctuaryPlinth::absent())
    }

    pub fn untouched() -> Self {
        Self::new(BrickStructure::untouched())
    }

    pub fn crown_origin_plinth(&mut self) {
        self.plinth = SanctuaryPlinth::origin_crown();
    }

    pub const fn structure(&self) -> &BrickStructure {
        &self.grid
    }

    pub fn allocated_brick_bytes(&self) -> usize {
        self.grid.allocated_brick_bytes()
    }

    pub fn sanctuary_at(&self, u: i32, v: i32) -> bool {
        self.plinth.active_at(u, v)
            || self.grid.bricks().inner().any_in_column(u, v, |_, cell| {
                matches!(cell, BrickCell::RawAtom(a) if a.medium().indestructible())
            })
    }

    pub fn stack_bare(
        &mut self,
        u: i32,
        v: i32,
        floor_elev: f32,
        ground_top: f32,
        sub: bool,
        atom: RawAtom,
        scars: &mut TurfScars,
    ) {
        let bare = atom.smothered();
        if !sub && (floor_elev - ground_top).abs() < NICHE_CONTACT_TOLERANCE && bare.at_floor(floor_elev).smothers_turf() {
            scars.expose_dirt(u, v);
        }
        self.grid.insert_atom(u, v, floor_elev, bare);
    }

    pub fn place_module(
        &mut self,
        anchor: CellCoord,
        spec: ModuleSpec,
        ground_top: f32,
        scars: &mut TurfScars,
    ) -> bool {
        if !self.grid.place_module_at(anchor, spec) {
            return false;
        }
        let (du, dv, _) = spec.span().dimensions();
        if (anchor.floor_elev(0.0) - ground_top).abs() < NICHE_CONTACT_TOLERANCE {
            for ov in 0..dv {
                for ou in 0..du {
                    let (u, v) = anchor.shifted(ou as i32, ov as i32, 0).planar();
                    scars.expose_dirt(u, v);
                    scars.clear_stage(u, v);
                }
            }
        }
        true
    }

    pub fn covers_ground_at(&self, u: i32, v: i32, ground_top: f32) -> bool {
        self.grid.covers_ground_at(u, v, ground_top)
    }

    pub fn bare_surface_at(&self, u: i32, v: i32, ground_bare: bool) -> bool {
        self.grid.bare_surface_at(u, v, ground_bare)
    }

    pub fn verdant_near_step(&self, u: i32, v: i32, target_step: i32) -> bool {
        self.grid.verdant_near_step(u, v, target_step)
    }

    pub fn regrow_sod(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        scars: &TurfScars,
        dt: f32,
    ) -> (Vec<(i32, i32)>, bool) {
        if self.grid.vacant() {
            return (Vec::new(), false);
        }
        let active = {
            let habitat = CanopyHabitat::new(NaturalHabitat::new(strata, shafts, scars), self);
            let mut list = Vec::new();
            self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
                if let BrickCell::RawAtom(atom) = cell {
                    let (u, v) = coord.planar();
                    let above = self.grid.bricks().cell_at(coord.shifted(0, 0, 1)).occupied();
                    if !above && atom.bare_turf() {
                        let n = habitat.verdant_neighbors(u, v, coord.layer_w() + 1);
                        if n > 0 {
                            list.push((coord, n));
                        }
                    }
                }
            });
            list
        };
        if active.is_empty() {
            return (Vec::new(), false);
        }
        let mut regrown = Vec::new();
        for (coord, n) in active {
            let BrickCell::RawAtom(atom) = self.grid.bricks().cell_at(coord) else { continue };
            let (next, grown) = atom.colonized(coord.grid_cell(), dt, n);
            if next != atom {
                self.grid.insert_atom(coord.planar().0, coord.planar().1, coord.floor_elev(0.0), next);
            }
            if grown {
                regrown.push(coord.planar());
            }
        }
        (regrown, true)
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta: i32) -> Option<(i32, i32)> {
        self.grid.step_camera_turn(u, v, delta)
    }

    pub fn confine_niche_elevation(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        self.grid.confine_niche(u, v, ground_h, elev)
    }

    pub fn remove_layer(&mut self, u: i32, v: i32, layer_elev: f32, ground_top: f32, scars: &mut TurfScars) -> Option<BlockMedium> {
        let (removed, column_vacated) = self.grid.remove_at(u, v, layer_elev)?;
        match removed {
            DismantledSpan::Vacant => None,
            DismantledSpan::Atom { coord, atom } => {
                let floor = coord.floor_elev(0.0);
                if column_vacated && (floor - ground_top).abs() < NICHE_CONTACT_TOLERANCE && atom.at_floor(floor).smothers_turf() {
                    scars.expose_dirt(u, v);
                }
                scars.clear_stage(u, v);
                Some(atom.medium())
            }
            DismantledSpan::Module { anchor, spec } => {
                let (du, dv, _) = spec.span().dimensions();
                for ov in 0..dv {
                    for ou in 0..du {
                        let (cu, cv) = anchor.shifted(ou as i32, ov as i32, 0).planar();
                        scars.clear_stage(cu, cv);
                    }
                }
                Some(spec.medium())
            }
        }
    }

    pub fn retain_overhang(&mut self, strata: &NaturalStrata, u: i32, v: i32, next_elev: i32, elev: i32, ground_verdant: bool) {
        let (nat, vault) = (strata.elevation_at(u, v), strata.vault());
        let crown_verdant = ground_verdant && !self.covers_ground_at(u, v, (nat as f32) * BLOCK_STEP);
        for step in (next_elev + 1)..elev {
            let floor = (step as f32) * BLOCK_STEP;
            let atom = if vault.crust_layer_at(u, v, step) {
                RawAtom::full_cube(BlockMedium::CrustRock)
            } else if step == nat - 1 {
                RawAtom::natural_crown(strata.crown_at(u, v), crown_verdant)
            } else {
                RawAtom::full_cube(BlockMedium::TurfEarth)
            };
            self.grid.insert_atom(u, v, floor, atom);
        }
    }

    pub fn settle_gravity(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, scars: &mut TurfScars) -> Vec<(i32, i32)> {
        self.grid.settle_gravity(|u, v| (shafts.elevation_at(strata, u, v) as f32) * BLOCK_STEP, scars)
    }

    pub fn collect_medium_sites(&self, medium: BlockMedium, frame: &SurfaceFrame, out: &mut Vec<(Vec3, Vec3)>) {
        self.plinth.collect_site(medium, frame, out);
        self.grid.collect_medium_sites(medium, frame, out);
    }

    pub fn capture_sector(&self, gsu: i32, gsv: i32) -> Vec<crate::SnapshotBlock> {
        SectorVault::new(&self.grid).capture(gsu, gsv)
    }

    pub fn apply_sector(&mut self, strata: &NaturalStrata, gsu: i32, gsv: i32, blocks: &[crate::SnapshotBlock]) {
        SectorCanvas::new(&mut self.grid).apply(strata, gsu, gsv, blocks);
    }

    pub fn collect_modified_sectors(&self, out: &mut Vec<(i32, i32)>) {
        SectorVault::new(&self.grid).collect_modified(out);
    }
}

impl MoldedStructure for WorldBlocks {
    fn vacant(&self) -> bool { matches!(self.plinth, SanctuaryPlinth::Absent) && self.grid.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.plinth.active_at(u, v) || self.grid.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        let base = if self.plinth.active_at(u, v) { floor_steps.max(0) } else { floor_steps };
        self.grid.top_steps_at(u, v, base)
    }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        let (u, v) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let base = if self.plinth.active_at(u, v) && span.0 >= -0.55 { h.max(0.0) } else { h };
        self.grid.constrain_support(cell, planar, base, span)
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.plinth.in_layer(u, v, layer_elev) || self.grid.layer_occupied_at(u, v, layer_elev)
    }
    fn locked_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> { self.grid.seat_backrest_cell(u, v, layer_elev) }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.grid.medium_at(u, v, layer_elev).or_else(|| self.plinth.medium_at(u, v, layer_elev))
    }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.grid.hit_top_at(cu, cv, u, v, ray_elev).or_else(|| self.plinth.hit_top(cu, cv, ray_elev))
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.grid.perch_at(u, v, layer_elev, eu, ev, frame)
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.grid.tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let p = self.plinth.emit_at(cell, frame, target);
        let g = self.grid.emit_at(cell, frame, colors, scars, target);
        p || g
    }
}
