use crate::block::BlockMedium;
use crate::brick_arena::{BrickCell, BrickStructure, CellCoord, ModuleSpec, RawAtom};
use super::super::natural_strata::NaturalStrata;

/// Raycast-free `4 m × 4 m` sector snapshot capture over a `BrickStructure` (`1` field).
pub struct SectorVault<'a> {
    grid: &'a BrickStructure,
}

impl<'a> SectorVault<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(grid: &'a BrickStructure) -> Self {
        Self { grid }
    }

    pub fn capture(&self, gsu: i32, gsv: i32) -> Vec<crate::SnapshotBlock> {
        let (u0, v0) = (gsu * 8, gsv * 8);
        let (u1, v1) = (u0 + 8, v0 + 8);
        let mut out = Vec::new();
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            let (u, v) = coord.planar();
            if u < u0 || u >= u1 || v < v0 || v >= v1 {
                return;
            }
            let uv = crate::SectorSnapshot::pack_uv(u, v, gsu, gsv);
            let fq = (coord.floor_elev(0.0) * 4.0).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            match cell {
                BrickCell::RawAtom(atom) if !atom.medium().indestructible() => {
                    out.push(crate::SnapshotBlock::new(uv, fq, atom.stamp(), atom.verdant_turf()));
                }
                BrickCell::ModuleAnchor(spec) => {
                    let bytes = spec.wire_bytes();
                    out.push(crate::SnapshotBlock::new(
                        uv,
                        fq,
                        [bytes[1], (bytes[2] << 4) | (bytes[3] & 0x0F), spec.medium().code()],
                        (bytes[4] & 1) == 1,
                    ));
                }
                _ => {}
            }
        });
        out.sort_unstable_by_key(|b| (b.local_uv(), (b.floor_elevation() * 4.0).round() as i16));
        out
    }

    pub fn collect_modified(&self, out: &mut Vec<(i32, i32)>) {
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            let indestructible = matches!(cell, BrickCell::RawAtom(a) if a.medium().indestructible());
            if !indestructible {
                let (u, v) = coord.planar();
                out.push((u.div_euclid(8), v.div_euclid(8)));
            }
        });
    }
}

/// Raycast-free `4 m × 4 m` sector snapshot restoration into a `BrickStructure` (`1` field).
pub struct SectorCanvas<'a> {
    grid: &'a mut BrickStructure,
}

impl<'a> SectorCanvas<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(grid: &'a mut BrickStructure) -> Self {
        Self { grid }
    }

    pub fn apply(&mut self, strata: &NaturalStrata, gsu: i32, gsv: i32, blocks: &[crate::SnapshotBlock]) {
        let (u0, v0) = (gsu * 8, gsv * 8);
        self.grid.clear_in((u0, u0 + 8), (v0, v0 + 8));
        for b in blocks {
            let (u, v) = crate::SectorSnapshot::unpack_uv(b.local_uv(), gsu, gsv);
            let stamp = b.stamp();
            let med = BlockMedium::from_code(stamp[2]);
            if med.indestructible() {
                continue;
            }
            if let Some(space_mod) = crate::SpaceModule::from_medium(med) {
                let turn = crate::CardinalTurn::from_code(u8::from(b.verdant()));
                let span = crate::ModuleSpan::clamped(stamp[0], stamp[1] >> 4, stamp[1] & 0x0F);
                let spec = ModuleSpec::new(space_mod, span, turn);
                let anchor = CellCoord::from_floor(u, v, b.floor_elevation(), 0.0);
                let _ = self.grid.place_module_at(anchor, spec);
                continue;
            }
            let atom = if stamp[0] == crate::NATURAL_OVERHANG_STAMP {
                RawAtom::natural_crown(strata.crown_at(u, v), b.verdant())
            } else {
                RawAtom::from_stamp(stamp, b.verdant())
            };
            self.grid.insert_atom(u, v, b.floor_elevation(), atom);
        }
    }
}
