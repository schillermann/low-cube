use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, CraftBlueprint, ExposedStratum, GravityField,
    MiningEdge, SurfaceFind, VehicleAssembly, WorldEarthworks,
};
use super::BinaryPlanet;

impl<G: GravityField + Send> WorldEarthworks for BinaryPlanet<G> {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        if self.near_moon(eye) {
            self.moon.harvest_surface(eye, direction).or_else(|| self.home.harvest_surface(eye, direction))
        } else {
            self.home.harvest_surface(eye, direction).or_else(|| self.moon.harvest_surface(eye, direction))
        }
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        if self.near_moon(eye) {
            self.moon.loosen_surface(eye, direction, stage).or_else(|| self.home.loosen_surface(eye, direction, stage))
        } else {
            self.home.loosen_surface(eye, direction, stage).or_else(|| self.moon.loosen_surface(eye, direction, stage))
        }
    }

    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool {
        if self.near_moon(eye) {
            self.moon.surface_yields_to(eye, direction, edge)
        } else {
            self.home.surface_yields_to(eye, direction, edge)
        }
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        if self.near_moon(eye) { CleaveOutcome::Missed } else { self.home.strike_timber(eye, direction) }
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        if self.near_moon(eye) { SurfaceFind::EmptyHanded } else { self.home.forage_surface(eye, direction) }
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        if self.near_moon(eye) {
            self.moon.cast_surface_find(eye, direction, find)
        } else {
            self.home.cast_surface_find(eye, direction, find)
        }
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        if self.near_moon(eye) {
            self.moon.place_molded_surface_block(eye, direction, avatar_pos, mold)
        } else {
            self.home.place_molded_surface_block(eye, direction, avatar_pos, mold)
        }
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        if self.near_moon(eye) {
            self.moon.adjust_rig(eye, direction, delta_steps, rotate_quarter)
        } else {
            self.home.adjust_rig(eye, direction, delta_steps, rotate_quarter)
        }
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        if self.near_moon(eye) {
            self.moon.lower_rig(eye, direction)
        } else {
            self.home.lower_rig(eye, direction)
        }
    }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        if self.near_moon(eye) {
            self.moon.build_rig_blueprint(eye, direction, blueprint)
        } else {
            self.home.build_rig_blueprint(eye, direction, blueprint)
        }
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        if self.near_moon(eye) {
            self.moon.board_vehicle(eye, direction, holding_block)
        } else {
            self.home.board_vehicle(eye, direction, holding_block)
        }
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        if self.near_moon(assembly.origin()) {
            self.moon.park_vehicle(assembly)
        } else {
            self.home.park_vehicle(assembly)
        }
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        if face_index < 6 {
            self.home.disclose_stratum(face_index, stratum);
        } else {
            self.moon.disclose_stratum(face_index - 6, stratum);
        }
    }
}
