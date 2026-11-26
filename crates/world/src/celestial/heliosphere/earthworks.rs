use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, CraftBlueprint, CraftIdentity, CraftPose,
    ExposedStratum, MiningEdge, SurfaceFind, VehicleAssembly, WorldEarthworks, WorldFact,
};
use super::HeliosphereWorld;

impl WorldEarthworks for HeliosphereWorld {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            let (id, center) = (body.body_id(), body.center());
            let out = body.planet_mut().harvest_surface(eye, direction);
            if out.is_some() {
                vic.record_local_fact(id, WorldFact::Harvested { eye: eye - center, direction });
            }
            return out;
        }
        drop(vic);
        self.earth.harvest_surface(eye, direction)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            let (id, center) = (body.body_id(), body.center());
            let out = body.planet_mut().loosen_surface(eye, direction, stage);
            if out.is_some() && stage >= 4 {
                vic.record_local_fact(id, WorldFact::Harvested { eye: eye - center, direction });
            }
            return out;
        }
        drop(vic);
        self.earth.loosen_surface(eye, direction, stage)
    }

    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            return body.planet_mut().surface_yields_to(eye, direction, edge);
        }
        drop(vic);
        self.earth.surface_yields_to(eye, direction, edge)
    }

    fn aimed_surface_medium(&self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            return body.planet_mut().aimed_surface_medium(eye, direction);
        }
        drop(vic);
        self.earth.aimed_surface_medium(eye, direction)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        if self.near_earth(eye) {
            self.earth.strike_timber(eye, direction)
        } else {
            CleaveOutcome::Missed
        }
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        if self.near_earth(eye) {
            self.earth.forage_surface(eye, direction)
        } else {
            SurfaceFind::EmptyHanded
        }
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            return body.planet_mut().cast_surface_find(eye, direction, find);
        }
        drop(vic);
        self.earth.cast_surface_find(eye, direction, find)
    }

    fn place_molded_surface_block(
        &mut self,
        eye: Vec3,
        direction: Vec3,
        avatar_pos: Vec3,
        mold: BlockMold,
    ) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            let (id, c) = (body.body_id(), body.center());
            let ok = body.planet_mut().place_molded_surface_block(eye, direction, avatar_pos, mold);
            if ok {
                vic.record_local_fact(
                    id,
                    WorldFact::MoldPlaced { eye: eye - c, direction, avatar_pos: avatar_pos - c, mold },
                );
            }
            return ok;
        }
        drop(vic);
        self.earth.place_molded_surface_block(eye, direction, avatar_pos, mold)
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            let (id, c) = (body.body_id(), body.center());
            let ok = body.planet_mut().adjust_rig(eye, direction, delta_steps, rotate_quarter);
            if ok {
                vic.record_local_fact(
                    id,
                    WorldFact::RigAdjusted { eye: eye - c, direction, delta_steps, rotate_quarter },
                );
            }
            return ok;
        }
        drop(vic);
        self.earth.adjust_rig(eye, direction, delta_steps, rotate_quarter)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            let (id, c) = (body.body_id(), body.center());
            let ok = body.planet_mut().lower_rig(eye, direction);
            if ok {
                vic.record_local_fact(id, WorldFact::RigLowered { eye: eye - c, direction });
            }
            return ok;
        }
        drop(vic);
        self.earth.lower_rig(eye, direction)
    }

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            return body.planet_mut().build_rig_blueprint(eye, direction, blueprint);
        }
        drop(vic);
        self.earth.build_rig_blueprint(eye, direction, blueprint)
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            return body.planet_mut().board_vehicle(eye, direction, holding_block);
        }
        drop(vic);
        self.earth.board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        let eph = self.ephemeris;
        let pos = assembly.origin();
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(pos, eph) {
            return body.planet_mut().park_vehicle(assembly);
        }
        drop(vic);
        self.earth.park_vehicle(assembly)
    }

    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        let mut ok = self.earth.register_craft_blueprint(blueprint.clone());
        let mut vic = self.vicinity.borrow_mut();
        for slot in 0..2 {
            if let Some(body) = vic.slot_body_mut(slot) {
                ok |= body.planet_mut().register_craft_blueprint(blueprint.clone());
            }
        }
        ok
    }

    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(eye, eph) {
            return body.planet_mut().claim_vehicle_helm(craft, pilot_slot, eye, direction);
        }
        drop(vic);
        self.earth.claim_vehicle_helm(craft, pilot_slot, eye, direction)
    }

    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        let eph = self.ephemeris;
        let mut vic = self.vicinity.borrow_mut();
        if let Some(body) = vic.ensure_near_mut(pose.origin(), eph) {
            return body.planet_mut().park_craft_by_id(craft, pose);
        }
        drop(vic);
        self.earth.park_craft_by_id(craft, pose)
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        if face_index < 6 {
            self.earth.disclose_stratum(face_index, stratum);
        } else if face_index < 12 {
            if let Some(body) = self.vicinity.borrow_mut().slot_body_mut(0) {
                body.planet_mut().disclose_stratum(face_index - 6, stratum);
            }
        } else if (18..24).contains(&face_index) {
            if let Some(body) = self.vicinity.borrow_mut().slot_body_mut(1) {
                body.planet_mut().disclose_stratum(face_index - 18, stratum);
            }
        }
    }

    fn offworld_frame(&self, probe: Vec3) -> Option<(crate::BodyId, Vec3)> {
        if self.near_earth(probe) {
            return None;
        }
        self.vicinity
            .borrow_mut()
            .ensure_near(probe, self.ephemeris)
            .map(|b| (b.body_id(), b.center()))
    }

    fn body_center(&self, body: crate::BodyId) -> Option<Vec3> {
        if body == crate::BodyId::EARTH && self.ephemeris.frame().earth_centered() {
            return Some(Vec3::ZERO);
        }
        if self.ephemeris.frame().anchored_body() == Some(body) {
            return Some(Vec3::ZERO);
        }
        if let Some(active) = self.vicinity.borrow().find_body(body) {
            return Some(active.center());
        }
        self.ephemeris
            .snapshot_all(0.0)
            .into_iter()
            .find(|s| s.id() == body)
            .map(|s| s.center().as_vec3())
    }

    fn harvest_offworld(&mut self, body: crate::BodyId, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        if body == crate::BodyId::EARTH {
            return self.earth.harvest_surface(eye, direction);
        }
        self.vicinity
            .borrow_mut()
            .harvest_on_body(body, eye, direction, self.ephemeris.proportions())
    }

    fn enact_offworld(&mut self, body: crate::BodyId, fact: &WorldFact) -> bool {
        if body == crate::BodyId::EARTH {
            return fact.enact_on(&mut self.earth);
        }
        self.vicinity
            .borrow_mut()
            .enact_on_body(body, fact, self.ephemeris.proportions())
    }

    fn capture_sector_snapshot(&self, face: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        self.earth.capture_sector_snapshot(face, gsu, gsv)
    }

    fn apply_sector_snapshot(&mut self, snapshot: &crate::SectorSnapshot) -> bool {
        self.earth.apply_sector_snapshot(snapshot)
    }

    fn collect_sector_snapshots(&self, out: &mut Vec<crate::SectorSnapshot>) {
        self.earth.collect_sector_snapshots(out);
    }

    fn capture_offworld_snapshot(
        &mut self,
        body: crate::BodyId,
        face: u8,
        gsu: i32,
        gsv: i32,
    ) -> crate::SectorSnapshot {
        if body == crate::BodyId::EARTH {
            return self.earth.capture_sector_snapshot(face, gsu, gsv);
        }
        self.vicinity
            .borrow_mut()
            .capture_body_sector(body, (face, gsu, gsv), self.ephemeris.proportions())
    }

    fn collect_offworld_snapshots(&mut self, out: &mut Vec<(crate::BodyId, crate::SectorSnapshot)>) {
        self.vicinity
            .borrow_mut()
            .collect_offworld_snapshots(self.ephemeris.proportions(), out);
    }
}
