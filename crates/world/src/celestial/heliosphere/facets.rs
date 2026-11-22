use glam::{Quat, Vec3};
use crate::{
    BlockMold, ClimateZone, CraftBlueprint, ExposedStratum, GravityField, MeadowRing, MeshTarget,
    PlannedMount, Quad, SightCone, TerrainMesh, VesselDeck, VesselSink, WoodlandLife,
};
use super::super::belt::AsteroidBelt;
use super::super::catalog::SurfaceNature;
use super::super::ephemeris::BodySnapshot;
use super::HeliosphereWorld;

/// Gas-giant nested cloud-cube and concentric cubic Saturn-ring emitter (`<= 4` fields).
struct GiantCloudShell {
    snap: BodySnapshot,
    epoch: f32,
}

impl GiantCloudShell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(snap: BodySnapshot, epoch: f32) -> Self {
        Self { snap, epoch }
    }

    fn emit_near(self, viewer: Vec3, target: &mut dyn MeshTarget) {
        let rel = self.snap.center().as_vec3() - viewer;
        let h = self.snap.half_size() as f32;
        if rel.length_squared() > (h * 42.0).powi(2) {
            return;
        }
        let c = self.snap.center().as_vec3();
        let (c_primary, c_band) = self.snap.palette();
        let faces: [(Vec3, Vec3, Vec3); 6] = [
            (Vec3::Y, Vec3::X, Vec3::NEG_Z),
            (Vec3::NEG_Y, Vec3::X, Vec3::Z),
            (Vec3::X, Vec3::NEG_Z, Vec3::NEG_Y),
            (Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y),
            (Vec3::Z, Vec3::X, Vec3::Y),
            (Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y),
        ];
        let spin_inner = Quat::from_rotation_y(self.epoch * 0.03);
        for (nl, ul, vl) in faces {
            let (n, u, v) = (spin_inner * nl, spin_inner * ul * (h * 0.94), spin_inner * vl * (h * 0.94));
            let fc = c + n * (h * 0.94);
            target.push_quad(Quad::new(fc - u - v, fc + u - v, fc + u + v, fc - u + v, n, c_band));
        }
        let spin_outer = Quat::from_rotation_y(-self.epoch * 0.018);
        for (nl, ul, vl) in faces {
            let (n, u, v) = (spin_outer * nl, spin_outer * ul * h, spin_outer * vl * h);
            let fc = c + n * h;
            target.push_quad(Quad::glowing(fc - u - v, fc + u - v, fc + u + v, fc - u + v, n, c_primary, -2.55));
        }
        if self.snap.nature().ringed_giant() {
            let (ri, ro) = (h * 1.45, h * 2.45);
            let ring_col = Vec3::new(0.86, 0.80, 0.64);
            for sign in [-1.0_f32, 1.0_f32] {
                let y = c + Vec3::Y * (sign * 0.4);
                let n = Vec3::Y * sign;
                target.push_quad(Quad::new(
                    y + Vec3::new(-ro, 0.0, -ro), y + Vec3::new(ro, 0.0, -ro),
                    y + Vec3::new(ri, 0.0, -ri), y + Vec3::new(-ri, 0.0, -ri), n, ring_col,
                ));
                target.push_quad(Quad::new(
                    y + Vec3::new(ro, 0.0, -ro), y + Vec3::new(ro, 0.0, ro),
                    y + Vec3::new(ri, 0.0, ri), y + Vec3::new(ri, 0.0, -ri), n, ring_col * 0.94,
                ));
                target.push_quad(Quad::new(
                    y + Vec3::new(ro, 0.0, ro), y + Vec3::new(-ro, 0.0, ro),
                    y + Vec3::new(-ri, 0.0, ri), y + Vec3::new(ri, 0.0, ri), n, ring_col,
                ));
                target.push_quad(Quad::new(
                    y + Vec3::new(-ro, 0.0, ro), y + Vec3::new(-ro, 0.0, -ro),
                    y + Vec3::new(-ri, 0.0, -ri), y + Vec3::new(-ri, 0.0, ri), n, ring_col * 0.94,
                ));
            }
        }
    }
}

impl TerrainMesh for HeliosphereWorld {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.earth.emit_surface(target);
        if let Some(body) = self.vicinity.borrow().active_body() {
            body.planet().emit_surface(target);
        }
    }

    fn face_count(&self) -> usize { 12 }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        match index {
            0..=5 => self.earth.emit_face(index, target),
            6..=11 => {
                if let Some(body) = self.vicinity.borrow().active_body() {
                    body.planet().emit_face(index - 6, target);
                }
            }
            12..=17 => self.earth.emit_face(index - 6, target),
            _ => {}
        }
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        if let Some(body) = self.vicinity.borrow_mut().active_body_mut() {
            if body.planet().pending_slices() {
                return Self::map_vicinity_slice(body.planet().emit_modified_face(target));
            }
        }
        Self::map_earth_slice(self.earth.emit_modified_face(target))
    }

    fn pending_slices(&self) -> bool {
        let v_pending = self
            .vicinity
            .borrow()
            .active_body()
            .is_some_and(|b| b.pending_upload() || b.planet().pending_slices());
        self.earth.pending_slices() || v_pending
    }

    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        if let Some(idx) = self.earth.drain_modified_slice(target) {
            return Some(Self::map_earth_slice(idx));
        }
        let mut vic = self.vicinity.borrow_mut();
        let body = vic.active_body_mut()?;
        if let Some(face_idx) = body.take_dirty_slice() {
            body.planet().emit_face(face_idx, target);
            return Some(face_idx + 6);
        }
        body.planet_mut().drain_modified_slice(target).map(Self::map_vicinity_slice)
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
        for snap in snaps {
            if matches!(snap.nature(), SurfaceNature::GasGiant { .. }) {
                GiantCloudShell::new(snap, epoch).emit_near(viewer, target);
            }
        }
        let sun = snaps[0];
        let active_id = self.vicinity.borrow().active_id();
        AsteroidBelt::new(self.ephemeris.proportions(), self.ephemeris.epoch_secs())
            .emit_local_field(cone, sun.center(), sun.gm(), active_id, target);
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        let mut mask = 0u32;
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(position, self.ephemeris) {
            mask |= body.planet().visible_faces_from(position, body.half_size()) << 6;
        }
        let home_half = self.ephemeris.proportions().earth_half();
        let show_flora = position.length() <= home_half + 420.0;
        let up = self.earth.surface_up_at(position);
        for (idx, zone) in ClimateZone::all().into_iter().enumerate() {
            let n = zone.normal();
            if position.dot(n) > -home_half * 0.45 || n.dot(up) > -0.5 {
                mask |= 1u32 << idx;
                if show_flora {
                    mask |= 1u32 << (12 + idx);
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
        if let Some(body) = self.vicinity.borrow_mut().active_body_mut() {
            let mut v_buf = Vec::new();
            body.planet_mut().drain_disclosures(&mut v_buf);
            for (face, s) in v_buf {
                out.push((face + 6, s));
            }
        }
    }
}

impl WoodlandLife for HeliosphereWorld {
    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        if self.near_earth(center) { self.earth.arboreal_perch_near(center, selector) } else { None }
    }

    fn woodland_habitat(&self, position: Vec3) -> bool {
        self.near_earth(position)
    }
}

impl VesselDeck for HeliosphereWorld {
    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        self.earth.present_vessels(sink);
        if let Some(body) = self.vicinity.borrow().active_body() {
            body.planet().present_vessels(sink);
        }
    }

    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(eye, self.ephemeris) {
            return body.planet().plan_molded_block(eye, direction, avatar_pos, mold);
        }
        self.earth.plan_molded_block(eye, direction, avatar_pos, mold)
    }

    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(eye, self.ephemeris) {
            return body.planet().capture_vehicle_blueprint(eye, direction);
        }
        self.earth.capture_vehicle_blueprint(eye, direction)
    }

    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        if let Some(body) = self.vicinity.borrow_mut().ensure_near(eye, self.ephemeris) {
            return body.planet().aimed_empty_rig(eye, direction, avatar_pos);
        }
        self.earth.aimed_empty_rig(eye, direction, avatar_pos)
    }
}
