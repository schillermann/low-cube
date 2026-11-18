use glam::Vec3;
use crate::relief::{MeadowRing, MeadowStamp};
use crate::{Face, MeshTarget};

/// Address of a single `16×16`-cell sector patch on one planetary cube face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatchCoord {
    face: u8,
    sector_u: i16,
    sector_v: i16,
}

impl PatchCoord {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(face: u8, sector_u: i16, sector_v: i16) -> Self {
        Self { face, sector_u, sector_v }
    }

    pub const fn face(self) -> u8 { self.face }
    pub const fn sector_u(self) -> i16 { self.sector_u }
    pub const fn sector_v(self) -> i16 { self.sector_v }
}

/// Toroidal `9×9` meadow sector evaluator scanning stale VRAM slots around a 3D viewpoint.
pub struct MeadowScan<'a> {
    face_idx: usize,
    face: &'a dyn Face,
    center: Vec3,
}

impl<'a> MeadowScan<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(face_idx: usize, face: &'a dyn Face, center: Vec3) -> Self {
        Self { face_idx, face, center }
    }

    pub fn drain_one(&self, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        let (csu, csv) = self.face.meadow_center_sector(self.center);
        for dsv in -4i32..=4 {
            for dsu in -4i32..=4 {
                let (gsu, gsv) = (csu + dsu, csv + dsv);
                let cheb = dsu.abs().max(dsv.abs());
                let lod = if cheb > 3 { 3 } else if cheb <= 1 { 2 } else { 0 };
                let rev = if cheb > 3 { 0 } else { self.face.meadow_sector_revision(gsu, gsv) };
                let slot = MeadowRing::ring_slot(gsu, gsv);
                let stamp = MeadowStamp::new(self.face_idx as u8, (gsu, gsv), (lod, rev));
                if ring.accept_if_stale(slot, stamp) {
                    if cheb <= 3 {
                        self.face.emit_meadow_sector(gsu, gsv, lod, target);
                    }
                    return Some(slot);
                }
            }
        }
        None
    }
}
