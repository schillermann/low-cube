mod earthworks;
mod facets;
mod patches;
mod shards;
mod slices;

use glam::Vec3;
use crate::{
    BeveledBoundary, CubicBoundary, Face, GravityField, OpenBoundary, PlanetaryWorld, SeamBoundary,
    SurfaceContact,
};
pub use self::patches::{MeadowScan, PatchCoord};
pub use self::shards::ZoneShards;
pub use self::slices::ModifiedSlices;

/// Cubic planetary world composed of polymorphic `Face` instances, modified-slice tracking, and seam boundary.
pub struct CubePlanet {
    faces: Vec<Box<dyn Face>>,
    fallback: Box<dyn Face>,
    slices: ModifiedSlices,
    boundary: Box<dyn SeamBoundary>,
}

impl CubePlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_boundary(
        faces: Vec<Box<dyn Face>>,
        fallback: Box<dyn Face>,
        slices: ModifiedSlices,
        boundary: Box<dyn SeamBoundary>,
    ) -> Self {
        Self { faces, fallback, slices, boundary }
    }

    /// Secondary constructor initializing a planar/open boundary with clean slice bitmask.
    pub fn new(faces: Vec<Box<dyn Face>>, fallback: Box<dyn Face>) -> Self {
        Self::with_boundary(faces, fallback, ModifiedSlices::clean(), Box::new(OpenBoundary::new()))
    }

    /// Secondary constructor initializing a sharp 6-face cubic boundary.
    pub fn cubic(faces: Vec<Box<dyn Face>>, fallback: Box<dyn Face>, half_size: f32) -> Self {
        Self::with_boundary(
            faces,
            fallback,
            ModifiedSlices::clean(),
            Box::new(CubicBoundary::sharp(half_size)),
        )
    }

    /// Secondary constructor initializing a beveled 6-face cubic boundary.
    pub fn beveled_cubic(
        faces: Vec<Box<dyn Face>>,
        fallback: Box<dyn Face>,
        half_size: f32,
        margin: f32,
    ) -> Self {
        Self::with_boundary(
            faces,
            fallback,
            ModifiedSlices::clean(),
            Box::new(BeveledBoundary::decorating(CubicBoundary::sharp(half_size), half_size, margin)),
        )
    }

    pub(super) fn matching_index(&self, position: Vec3) -> usize {
        for (idx, face) in self.faces.iter().enumerate() {
            if face.matches(position) {
                return idx;
            }
        }
        0
    }

    pub(super) fn aimed_indices(&self, eye: Vec3, direction: Vec3) -> (usize, usize) {
        let primary = self.matching_index(eye);
        let secondary = self.matching_index(eye + direction.normalize_or_zero() * 1.25);
        (primary, secondary)
    }

    pub(super) fn active_face(&self, position: Vec3) -> &dyn Face {
        for face in &self.faces {
            if face.matches(position) {
                return face.as_ref();
            }
        }
        self.fallback.as_ref()
    }

    pub(super) fn inspect_aimed_faces<T>(
        &self,
        eye: Vec3,
        direction: Vec3,
        op: impl Fn(&dyn Face) -> Option<T>,
    ) -> Option<T> {
        let (primary, secondary) = self.aimed_indices(eye, direction);
        for idx in [primary, secondary] {
            if let Some(face) = self.faces.get(idx) {
                if let Some(out) = op(face.as_ref()) {
                    return Some(out);
                }
            }
            if secondary == primary {
                break;
            }
        }
        None
    }

    pub(super) fn try_aimed_faces<T>(
        &mut self,
        eye: Vec3,
        direction: Vec3,
        flora_offset: bool,
        mut op: impl FnMut(&mut dyn Face) -> Option<T>,
    ) -> Option<T> {
        let len = self.faces.len();
        let (primary, secondary) = self.aimed_indices(eye, direction);
        for idx in [primary, secondary] {
            if let Some(face) = self.faces.get_mut(idx) {
                if let Some(out) = op(face.as_mut()) {
                    self.slices.record(if flora_offset { len + idx } else { idx });
                    return Some(out);
                }
            }
            if secondary == primary {
                break;
            }
        }
        None
    }
}

impl GravityField for CubePlanet {
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.boundary.blend_normal(position, self.active_face(position).surface_up())
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}

impl PlanetaryWorld for CubePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        for (idx, face) in self.faces.iter().enumerate() {
            if face.matches(position) {
                let constrained = face.constrain(position, player_height);
                return self.boundary.fold_constrained(idx, position, constrained);
            }
        }
        self.fallback.constrain(position, player_height)
    }

    fn surface_contact(&self, probe: Vec3, clearance: f32) -> SurfaceContact {
        if !self.boundary.within_approach(probe) {
            return SurfaceContact::Vacuum;
        }
        SurfaceContact::SolidCrust {
            ground: self.constrain_to_surface(probe, clearance),
            normal: self.surface_up_at(probe),
        }
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.active_face(position).canopy_drag(position)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.faces.iter().find_map(|f| f.placed_camera_tripod())
    }

    fn collect_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        for face in &self.faces {
            face.collect_medium_sites(medium, out);
        }
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        ZoneShards::new(dt, 0b0011_1111).step_faces(&mut self.faces, &mut self.slices)
    }

    fn step_active_zones(
        &mut self,
        dt: f32,
        earth_zone_mask: u8,
        _active_bodies: &[crate::BodyId],
    ) -> bool {
        ZoneShards::new(dt, earth_zone_mask).step_faces(&mut self.faces, &mut self.slices)
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        self.active_face(position).lethal_at(position)
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.active_face(position).confine_beneath_crust(position)
    }

    fn celestial_atlas(&self) -> crate::CelestialAtlas {
        let half = self.constrain_to_surface(Vec3::new(0.25, 2048.0, 0.25), 0.0).y.round().max(4.0);
        crate::CelestialAtlas::solitary(half)
    }
}
