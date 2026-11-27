use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::brick_arena::{BrickStructure, CellCoord, ModuleSpec, RawAtom};
use crate::frame::SurfaceFrame;
use crate::relief::{
    CraftBlueprint, CraftCluster, MoldedStructure, NaturalStrata, RigArm, ShaftExcavation,
    SurfaceSightRay, TurfScars, VehicleAssembly,
};

const BLOCK_STEP: f32 = GridCell::SPAN;
const MAX_HOIST_STEPS: i32 = 8;

/// Read-only magnetic assembly flange survey over a `MagneticFlange` `GridBody` (`2` fields).
pub struct FlangeSurvey<'a> {
    grid: &'a BrickStructure,
    arm: RigArm,
}

impl<'a> FlangeSurvey<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(grid: &'a BrickStructure, arm: RigArm) -> Self {
        Self { grid, arm }
    }

    pub fn snapshot_at(&self, mast_u: i32, mast_v: i32, gsu: i32, gsv: i32) -> crate::RigSnapshot {
        let uv = crate::SectorSnapshot::pack_uv(mast_u, mast_v, gsu, gsv);
        let craft = CraftCluster::new(self.grid.clone());
        crate::RigSnapshot::new(uv, self.arm.posture_tuple(), craft.snapshot_relative(mast_u, mast_v))
    }

    pub fn occupies_mast_layer(&self, layer_elev: f32) -> bool {
        self.arm.occupies_mast_layer(layer_elev)
    }

    pub fn boom_or_craft_at(&self, mast_u: i32, mast_v: i32, u: i32, v: i32) -> bool {
        self.arm.occupies_boom_cell(mast_u, mast_v, u, v) || self.grid.present_at(u, v)
    }

    pub fn boom_slot_at(&self, mast_u: i32, mast_v: i32, u: i32, v: i32, layer_elev: f32) -> bool {
        let hoist_z = self.arm.hoist_elevation();
        self.arm.hoisted()
            && (u, v) != (mast_u, mast_v)
            && !self.grid.layer_occupied_at(u, v, hoist_z + 0.25)
            && self.arm.occupies_boom_cell(mast_u, mast_v, u, v)
            && (layer_elev - (hoist_z + 0.25)).abs() <= 0.30
    }

    pub fn aimed_empty_boom(&self, mast_u: i32, mast_v: i32, u: i32, v: i32) -> bool {
        self.grid.vacant() && self.arm.occupies_boom_cell(mast_u, mast_v, u, v)
    }

    pub fn mount_first_target(&self, mast_u: i32, mast_v: i32, u: i32, v: i32) -> Option<(i32, i32, f32)> {
        (self.grid.vacant() && self.arm.hoisted() && self.arm.near_boom(mast_u, mast_v, u, v)).then(|| {
            let (eu, ev) = self.arm.gripped_cell(mast_u, mast_v);
            (eu, ev, self.arm.hoist_elevation())
        })
    }

    pub fn resolve_craft_attach(
        &self,
        mast_u: i32,
        mast_v: i32,
        u: i32,
        v: i32,
        ground_top: f32,
        sub: bool,
        raw_floor: f32,
    ) -> Option<f32> {
        if self.grid.vacant() || ((u, v) == (mast_u, mast_v) && self.arm.claw_slot(mast_u, mast_v) != (mast_u, mast_v)) {
            return None;
        }
        let cluster = CraftCluster::new(self.grid.clone());
        let mut floor_elev = if sub { raw_floor } else { raw_floor.max(ground_top) };
        if !cluster.touches_placement(u, v, floor_elev) {
            if !sub && (floor_elev - ground_top).abs() <= 0.05 {
                floor_elev = cluster.adjacent_hoisted_floor(u, v, |min_z| self.arm.above_base(min_z))?;
            } else {
                return None;
            }
        }
        while self.grid.layer_occupied_at(u, v, floor_elev + 0.25) {
            floor_elev += if sub { -BLOCK_STEP } else { BLOCK_STEP };
        }
        Some(floor_elev)
    }

    pub fn constrain_at(
        &self,
        mast_u: i32,
        mast_v: i32,
        cell: GridCell,
        planar: (f32, f32),
        mut h: f32,
        span: (f32, f32),
    ) -> f32 {
        let (cu, cv) = cell.feed_coordinates(|gu, gv| (gu, gv));
        if (cu, cv) == (mast_u, mast_v) {
            h = self.arm.constrain_mast_support(h, span.0);
        }
        self.grid.constrain_support(cell, planar, h, span)
    }

    pub fn layer_occupied_at(&self, mast_u: i32, mast_v: i32, u: i32, v: i32, layer_elev: f32) -> bool {
        ((u, v) == (mast_u, mast_v) && self.arm.occupies_mast_layer(layer_elev))
            || self.grid.layer_occupied_at(u, v, layer_elev)
    }

    pub fn hit_boom_or_craft(
        &self,
        mast_u: i32,
        mast_v: i32,
        (cu, cv): (i32, i32),
        (u, v): (f32, f32),
        ray_elev: f32,
    ) -> Option<f32> {
        if let Some(craft_top) = self.grid.hit_top_at(cu, cv, u, v, ray_elev) {
            return Some(craft_top);
        }
        if (cu, cv) == (mast_u, mast_v) && self.arm.occupies_mast_layer(ray_elev) {
            return Some(self.arm.mast_ceiling());
        }
        self.boom_slot_at(mast_u, mast_v, cu, cv, ray_elev)
            .then_some(self.arm.hoist_elevation() + BLOCK_STEP)
    }

    pub fn emit_mast_and_craft(
        &self,
        mast_u: i32,
        mast_v: i32,
        cell: GridCell,
        frame: &SurfaceFrame,
        colors: (Vec3, Vec3),
        scars: &TurfScars,
        target: &mut dyn MeshTarget,
    ) -> bool {
        let (u, v) = cell.feed_coordinates(|gu, gv| (gu, gv));
        let mast_emitted = if (u, v) == (mast_u, mast_v) {
            self.arm.geometry().emit(cell, frame, target);
            true
        } else {
            false
        };
        let craft_emitted = self.grid.emit_at(cell, frame, colors, scars, target);
        mast_emitted || craft_emitted
    }

    pub fn capture_blueprint(&self, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<CraftBlueprint> {
        CraftCluster::new(self.grid.clone()).capture_blueprint(eu, ev, frame)
    }
}

/// Mutable magnetic assembly flange mason over a `MagneticFlange` `GridBody` (`2` fields).
pub struct FlangeMason<'a> {
    grid: &'a mut BrickStructure,
    arm: &'a mut RigArm,
}

impl<'a> FlangeMason<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(grid: &'a mut BrickStructure, arm: &'a mut RigArm) -> Self {
        Self { grid, arm }
    }

    pub fn place_block(&mut self, mast: (i32, i32), cell: (i32, i32), floor_elev: f32, atom: RawAtom) -> Option<(i32, i32)> {
        let relocated = self.arm.retract_for_placed_block(mast.0, mast.1, cell.0, cell.1, floor_elev);
        let mut cluster = CraftCluster::new(std::mem::replace(self.grid, BrickStructure::untouched()));
        cluster.insert(cell.0, cell.1, floor_elev, atom);
        *self.grid = cluster.into_structure();
        relocated
    }

    pub fn place_module(
        &mut self,
        mast: (i32, i32),
        cell: (i32, i32),
        floor_elev: f32,
        spec: ModuleSpec,
    ) -> Option<(bool, Option<(i32, i32)>)> {
        let anchor = CellCoord::from_floor(cell.0, cell.1, floor_elev, self.grid.datum());
        if !self.grid.place_module_at(anchor, spec) {
            return Some((false, None));
        }
        let relocated = self.arm.retract_for_placed_block(mast.0, mast.1, cell.0, cell.1, floor_elev);
        Some((true, relocated))
    }

    pub fn remove_layer(&mut self, mast: (i32, i32), u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        let removed = self.grid.remove_at(u, v, layer_elev).and_then(|(s, _)| s.medium())?;
        if self.grid.vacant() {
            self.arm.reset_reach();
        } else {
            let hoist_z = self.arm.hoist_elevation();
            let cluster = CraftCluster::new(self.grid.clone());
            self.arm.advance_toward_craft(mast.0, mast.1, |cu, cv| {
                cluster.occupies_claw((cu, cv), hoist_z) || cluster.present_at(cu, cv)
            });
        }
        Some(removed)
    }

    pub fn adjust(
        &mut self,
        mast: (i32, i32),
        delta_steps: i32,
        rotate_quarter: bool,
        ground_fn: impl Fn(i32, i32) -> f32,
    ) -> bool {
        if self.grid.vacant() {
            return false;
        }
        if delta_steps != 0 {
            let raw_dz = self.arm.clamped_step_delta(delta_steps);
            let cluster = CraftCluster::new(self.grid.clone());
            let dz = cluster.clamped_vertical_delta(raw_dz, ground_fn);
            if dz.abs() < 1e-4 {
                return false;
            }
            self.arm.apply_vertical_shift(dz);
            self.grid.shift_elevation(dz);
            return true;
        }
        if rotate_quarter {
            let (piv_u, piv_v) = self.arm.gripped_cell(mast.0, mast.1);
            self.grid.rotate_around(piv_u, piv_v);
            return true;
        }
        false
    }

    pub fn release_drivable(
        &mut self,
        mast: (i32, i32),
        (u, v, layer_elev): (i32, i32, f32),
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
        ground_fn: impl Fn(i32, i32) -> f32,
    ) -> Option<VehicleAssembly> {
        let probe = CraftCluster::new(self.grid.clone());
        if !probe.targets_steering(u, v, layer_elev, ray, frame) || !probe.drivable() {
            return None;
        }
        let _ = self.adjust(mast, -MAX_HOIST_STEPS, false, &ground_fn);
        let taken = CraftCluster::new(std::mem::replace(self.grid, BrickStructure::untouched()));
        match taken.into_vehicle(&ground_fn, ray, frame) {
            Ok(assembly) => {
                self.arm.reset();
                Some(assembly)
            }
            Err(unmoved) => {
                *self.grid = unmoved.into_structure();
                None
            }
        }
    }

    /// Unlocks ANY non-empty magnetic-flange assembly (even unpiloted stations/modules!) directly into a `VehicleAssembly`.
    pub fn unlock_any_assembly(
        &mut self,
        mast: (i32, i32),
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
        ground_fn: impl Fn(i32, i32) -> f32,
    ) -> Option<VehicleAssembly> {
        if self.grid.vacant() {
            return None;
        }
        let _ = self.adjust(mast, -MAX_HOIST_STEPS, false, &ground_fn);
        let taken = CraftCluster::new(std::mem::replace(self.grid, BrickStructure::untouched()));
        let assembly = taken.into_any_assembly(&ground_fn, ray, frame)?;
        self.arm.reset();
        Some(assembly)
    }

    pub fn pack_up_into(
        &mut self,
        mast: (i32, i32),
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        crust_body: &mut super::GridBody,
        scars: &mut TurfScars,
    ) {
        let _ = self.adjust(mast, -MAX_HOIST_STEPS, false, |cu, cv| {
            (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
        });
        let taken = CraftCluster::new(std::mem::replace(self.grid, BrickStructure::untouched()));
        taken.dismantle_into(strata, shafts, crust_body, scars);
    }
}
