use glam::Vec3;
use crate::{CubePlanet, TranslatedWorld};
use super::super::catalog::BodyId;

/// Live off-world rocky planet, moon, or landed belt asteroid occupying a 6-slice GPU bank.
pub struct AwakenedBody {
    id_and_half: (BodyId, f32),
    center: Vec3,
    planet: TranslatedWorld<CubePlanet>,
    dirty_slices: u8,
}

impl AwakenedBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        id_and_half: (BodyId, f32),
        center: Vec3,
        planet: TranslatedWorld<CubePlanet>,
        dirty_slices: u8,
    ) -> Self {
        Self { id_and_half, center, planet, dirty_slices }
    }

    pub const fn body_id(&self) -> BodyId {
        self.id_and_half.0
    }

    pub const fn half_size(&self) -> f32 {
        self.id_and_half.1
    }

    pub const fn center(&self) -> Vec3 {
        self.center
    }

    pub const fn planet(&self) -> &TranslatedWorld<CubePlanet> {
        &self.planet
    }

    pub fn planet_mut(&mut self) -> &mut TranslatedWorld<CubePlanet> {
        &mut self.planet
    }

    pub fn into_parts(self) -> (BodyId, f32, CubePlanet) {
        (self.id_and_half.0, self.id_and_half.1, self.planet.into_inner())
    }

    pub fn relocate(&mut self, next_center: Vec3) {
        if (self.center - next_center).length_squared() > 1e-6 {
            self.center = next_center;
            self.planet.relocate_center(next_center);
            self.mark_all_dirty();
        }
    }

    pub fn near(&self, pos: Vec3, pad: f32) -> bool {
        (pos - self.center).abs().max_element() <= self.id_and_half.1 + pad
    }

    pub fn within_vicinity(&self, probe: Vec3) -> bool {
        let pad = if self.id_and_half.0.belt_slot().is_some() {
            96.0
        } else {
            (self.id_and_half.1 * 26.0).clamp(144.0, 1_050.0)
        };
        self.near(probe, pad)
    }

    pub fn recenter_to(&mut self, frame: crate::CosmicFrame, snaps: &[super::super::ephemeris::BodySnapshot]) {
        let id = self.id_and_half.0;
        if frame.anchored_body() == Some(id) {
            self.relocate(Vec3::ZERO);
        } else if let Some(snap) = snaps.iter().find(|s| s.id() == id) {
            self.relocate(snap.center().as_vec3());
        }
    }

    pub fn surface_gap(&self, pos: Vec3) -> f32 {
        (pos - self.center).abs().max_element() - self.id_and_half.1
    }

    pub const fn pending_upload(&self) -> bool {
        self.dirty_slices != 0
    }

    pub fn mark_all_dirty(&mut self) {
        self.dirty_slices = 0b0011_1111;
    }

    pub fn take_dirty_slice(&mut self) -> Option<usize> {
        for bit in 0..6u8 {
            if (self.dirty_slices & (1 << bit)) != 0 {
                self.dirty_slices &= !(1 << bit);
                return Some(usize::from(bit));
            }
        }
        None
    }
}

/// Polymorphic off-world terrain slot avoiding `Option<AwakenedBody>`.
pub enum ActiveVicinity {
    Dormant,
    Awakened(AwakenedBody),
}

impl ActiveVicinity {
    pub(super) const fn active_id(&self) -> Option<BodyId> {
        match self {
            Self::Dormant => None,
            Self::Awakened(b) => Some(b.body_id()),
        }
    }

    pub(super) const fn uploaded_id(&self) -> Option<BodyId> {
        match self {
            Self::Dormant => None,
            Self::Awakened(b) => {
                if b.pending_upload() {
                    None
                } else {
                    Some(b.body_id())
                }
            }
        }
    }

    pub(super) const fn body(&self) -> Option<&AwakenedBody> {
        match self {
            Self::Dormant => None,
            Self::Awakened(b) => Some(b),
        }
    }

    pub(super) fn body_mut(&mut self) -> Option<&mut AwakenedBody> {
        match self {
            Self::Dormant => None,
            Self::Awakened(b) => Some(b),
        }
    }
}
