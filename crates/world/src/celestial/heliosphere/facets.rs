use glam::Vec3;
use crate::{
    BlockMold, ClimateZone, CraftBlueprint, ExposedStratum, GravityField, MeadowRing, MeshTarget,
    PlannedMount, SightCone, TerrainMesh, VesselDeck, VesselSink,
};
use super::super::belt::AsteroidBelt;
use super::super::catalog::SurfaceNature;
use super::super::spin::ApexSpin;
use super::cosmic_sky::{CosmicBodyCube, GiantCloudShell, EARTH_VOXEL_HANDOFF};
use super::HeliosphereWorld;

impl TerrainMesh for HeliosphereWorld {
    fn allocated_brick_bytes(&self) -> usize {
        let vic = self.vicinity.borrow();
        let offworld: usize = [0usize, 1]
            .into_iter()
            .filter_map(|i| vic.slot_body(i))
            .map(|b| b.planet().allocated_brick_bytes())
            .sum();
        self.earth.allocated_brick_bytes() + offworld
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.earth.emit_surface(target);
        let vic = self.vicinity.borrow();
        for slot in 0..2 {
            if let Some(body) = vic.slot_body(slot) {
                body.planet().emit_surface(target);
            }
        }
    }

    fn face_count(&self) -> usize { 12 }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        match index {
            0..=5 => self.earth.emit_face(index, target),
            6..=11 => {
                if let Some(body) = self.vicinity.borrow().slot_body(0) {
                    body.planet().emit_face(index - 6, target);
                }
            }
            12..=17 => self.earth.emit_face(index - 6, target),
            18..=23 => {
                if let Some(body) = self.vicinity.borrow().slot_body(1) {
                    body.planet().emit_face(index - 18, target);
                }
            }
            _ => {}
        }
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        let mut vic = self.vicinity.borrow_mut();
        for slot in 0..2 {
            if let Some(body) = vic.slot_body_mut(slot) {
                if body.planet().pending_slices() {
                    let f = body.planet().emit_modified_face(target);
                    if let Some(mapped) = Self::map_slot_slice(slot, f) {
                        return mapped;
                    }
                }
            }
        }
        drop(vic);
        Self::map_earth_slice(self.earth.emit_modified_face(target))
    }

    fn pending_slices(&self) -> bool {
        let vic = self.vicinity.borrow();
        let v_pending = [0usize, 1].into_iter().any(|s| {
            vic.slot_body(s).is_some_and(|b| b.pending_upload() || b.planet().pending_slices())
        });
        self.earth.pending_slices() || v_pending
    }

    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        if let Some(idx) = self.earth.drain_modified_slice(target) {
            return Some(Self::map_earth_slice(idx));
        }
        let mut vic = self.vicinity.borrow_mut();
        for slot in 0..2 {
            let Some(body) = vic.slot_body_mut(slot) else { continue };
            if let Some(face_idx) = body.take_dirty_slice() {
                body.planet().emit_face(face_idx, target);
                return Self::map_slot_slice(slot, face_idx);
            }
            while let Some(raw_idx) = body.planet_mut().drain_modified_slice(target) {
                if let Some(mapped) = Self::map_slot_slice(slot, raw_idx) {
                    return Some(mapped);
                }
            }
        }
        None
    }

    fn face_patch_count(&self, slice: usize) -> usize {
        if slice <= 5 { self.earth.face_patch_count(slice) } else { 0 }
    }

    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) {
        if slice <= 5 {
            self.earth.emit_face_patch(slice, patch_slot, target);
        }
    }

    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> {
        self.earth.drain_modified_patch(target)
    }

    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        if self.near_earth(center) {
            self.earth.drain_stale_meadow_patch(center, ring, target)
        } else {
            None
        }
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        if self.near_earth(center) {
            self.earth.emit_meadow_around(center, view_dir, target);
        }
    }

    fn emit_cosmic_field(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        let viewer = cone.observe_from(|eye| eye);
        let snaps = self.ephemeris.snapshot_all(0.0);
        let epoch = self.ephemeris.epoch_secs() as f32;
        let spin = ApexSpin::from_ephemeris(self.ephemeris);
        let (active_ids, uploaded_ids) = {
            let vic = self.vicinity.borrow();
            (vic.active_ids(), vic.uploaded_ids())
        };
        let earth_voxel_active = self.ephemeris.frame().earth_centered();
        if earth_voxel_active {
            use crate::PlanetaryWeather as _;
            crate::TroposphereDeck::new(self.ephemeris.proportions().earth_half(), epoch)
                .emit_around(cone, target);
        }
        for snap in snaps {
            if matches!(snap.nature(), SurfaceNature::GasGiant { .. }) {
                GiantCloudShell::new(snap, epoch).emit_near(viewer, target);
            } else {
                CosmicBodyCube::new(snap, spin).emit_for_viewer(viewer, uploaded_ids, earth_voxel_active, target);
            }
        }
        let sun = snaps[0];
        AsteroidBelt::new(self.ephemeris.proportions(), self.ephemeris.epoch_secs())
            .emit_local_field(cone, sun.center(), sun.gm(), active_ids, target);
        if !self.vicinity.borrow().authoritative() {
            return;
        }
        let mut beacons = Vec::new();
        if earth_voxel_active {
            crate::PlanetaryWorld::collect_medium_sites(&self.earth, crate::BlockMedium::LaserBeacon, &mut beacons);
        }
        {
            let vic = self.vicinity.borrow();
            for slot in 0..2 {
                if let Some(body) = vic.slot_body(slot) {
                    crate::PlanetaryWorld::collect_medium_sites(body.planet(), crate::BlockMedium::LaserBeacon, &mut beacons);
                }
            }
        }
        for (b_pos, b_normal) in beacons.into_iter().take(16) {
            let delta = viewer - b_pos;
            let dist = delta.length();
            if dist > 160.0 && delta.dot(b_normal) > 0.0 {
                let dir = -delta / dist;
                let elev = (delta.dot(b_normal) / dist).clamp(0.08, 1.0);
                crate::SkySignal::optical_beacon(dir, dist * 0.001, elev).emit(cone, target);
            }
        }
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        let mut mask = 0u32;
        {
            let mut vic = self.vicinity.borrow_mut();
            vic.awaken_visible(position, self.ephemeris);
            if let Some(b0) = vic.slot_body(0) {
                mask |= b0.planet().visible_faces_from(position, b0.half_size()) << 6;
            }
            if let Some(b1) = vic.slot_body(1) {
                mask |= b1.planet().visible_faces_from(position, b1.half_size()) << 18;
            }
        }
        let home_half = self.ephemeris.proportions().earth_half();
        let earth_dist = position.length();
        if self.ephemeris.frame().earth_centered() && earth_dist <= EARTH_VOXEL_HANDOFF {
            let continental = home_half > 128.0;
            let orbital = earth_dist > home_half + 96.0;
            let show_flora = !continental && earth_dist <= home_half + 420.0;
            let up = self.earth.surface_up_at(position);
            for (idx, zone) in ClimateZone::all().into_iter().enumerate() {
                let n = zone.normal();
                let visible = if continental {
                    n.dot(up) > 0.2
                        || position.dot(n) >= home_half - 180.0
                        || (orbital && position.dot(n) > -home_half * 0.25)
                } else {
                    position.dot(n) > -home_half * 0.45 || n.dot(up) > -0.5
                };
                if visible {
                    mask |= 1u32 << idx;
                    if show_flora {
                        mask |= 1u32 << (12 + idx);
                    }
                }
            }
        }
        if self.near_earth(position) {
            mask |= 1u32 << 24;
        }
        mask
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        self.earth.drain_disclosures(out);
        let mut vic = self.vicinity.borrow_mut();
        for (slot, offset) in [(0usize, 6u8), (1usize, 18u8)] {
            if let Some(body) = vic.slot_body_mut(slot) {
                let mut v_buf = Vec::new();
                body.planet_mut().drain_disclosures(&mut v_buf);
                for (face, s) in v_buf {
                    out.push((face + offset, s));
                }
            }
        }
    }
}

impl VesselDeck for HeliosphereWorld {
    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        self.earth.present_vessels(sink);
        let vic = self.vicinity.borrow();
        for slot in 0..2 {
            if let Some(body) = vic.slot_body(slot) {
                body.planet().present_vessels(sink);
            }
        }
    }

    fn present_vessels_near(&self, center: Vec3, radius: f32, sink: &mut dyn VesselSink) {
        if self.ephemeris.frame().earth_centered()
            && (self.near_earth(center) || center.length() <= self.ephemeris.proportions().earth_half() + radius)
        {
            self.earth.present_vessels_near(center, radius, sink);
        }
        let vic = self.vicinity.borrow();
        for slot in 0..2 {
            if let Some(body) = vic.slot_body(slot) {
                if body.near(center, radius) {
                    body.planet().present_vessels_near(center, radius, sink);
                }
            }
        }
    }

    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(eye, self.ephemeris) {
            if body.near(eye, 32.0) {
                return body.planet().plan_molded_block(eye, direction, avatar_pos, mold);
            }
        }
        self.earth.plan_molded_block(eye, direction, avatar_pos, mold)
    }

    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(eye, self.ephemeris) {
            if body.near(eye, 32.0) {
                return body.planet().capture_vehicle_blueprint(eye, direction);
            }
        }
        self.earth.capture_vehicle_blueprint(eye, direction)
    }

    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(eye, self.ephemeris) {
            if body.near(eye, 32.0) {
                return body.planet().aimed_empty_rig(eye, direction, avatar_pos);
            }
        }
        self.earth.aimed_empty_rig(eye, direction, avatar_pos)
    }
}
