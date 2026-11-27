mod occlusion;
mod slice_merge;

use glam::Vec3;
use world::{BlockMedium, Brick8, BrickCell, BrickCoord, LocalSlot, RawAtom};
use crate::instanced_module::ModuleInstance;
use crate::packed_quad::{PackedQuad, Rgb565Swatch};
use self::occlusion::{CornerOcclusion, SolidCubeMask};
use self::slice_merge::SliceBitMerge;

pub use self::occlusion::{ArenaNeighbors, BrickNeighborhood, VacantNeighbors};

const NOT_COL7: u64 = 0x7F7F_7F7F_7F7F_7F7F;
const NOT_COL0: u64 = 0xFEFE_FEFE_FEFE_FEFE;

/// CPU `u64` Binary Greedy Brick Quad Extractor (`~10–20 µs` per `8×8×8` `Brick8`, `3` private fields).
pub struct GreedyBrickQuads<'a> {
    brick: &'a Brick8,
    coord: BrickCoord,
    datum: f32,
}

impl<'a> GreedyBrickQuads<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(brick: &'a Brick8, coord: BrickCoord, datum: f32) -> Self {
        Self { brick, coord, datum }
    }

    /// Metric `[origin_x, origin_y, origin_z, step=0.5]` uniform matching `PackedQuad` coordinates emitted for `coord`.
    pub fn patch_origin(&self) -> [f32; 4] {
        let base = self.coord.origin_cell();
        let (u, v, _) = base.components();
        [(u as f32) * 0.5, base.floor_elev(self.datum), (v as f32) * 0.5, 0.5]
    }

    /// Extracts merged `8-Byte` `PackedQuad`s + `B × H × T` `ModuleInstance`s with `0` per-quad heap allocations.
    pub fn extract(
        &self,
        neighbors: &dyn BrickNeighborhood,
        quads_out: &mut Vec<PackedQuad>,
        modules_out: &mut Vec<ModuleInstance>,
    ) {
        if self.brick.vacant() {
            return;
        }
        self.collect_module_anchors(modules_out);
        let solid = SolidCubeMask::new(self.brick).opaque_planes();
        if (solid[0] | solid[1] | solid[2] | solid[3] | solid[4] | solid[5] | solid[6] | solid[7]) == 0 {
            return;
        }
        let nb_px = self.opaque_neighbor(neighbors, [1, 0, 0]);
        let nb_mx = self.opaque_neighbor(neighbors, [-1, 0, 0]);
        let nb_py = self.opaque_neighbor(neighbors, [0, 1, 0]);
        let nb_my = self.opaque_neighbor(neighbors, [0, -1, 0]);
        let nb_pz = self.opaque_neighbor(neighbors, [0, 0, 1]);
        let nb_mz = self.opaque_neighbor(neighbors, [0, 0, -1]);

        self.emit_y_faces(&solid, &nb_py, true, quads_out);
        self.emit_y_faces(&solid, &nb_my, false, quads_out);
        self.emit_x_faces(&solid, &nb_px, true, quads_out);
        self.emit_x_faces(&solid, &nb_mx, false, quads_out);
        self.emit_z_faces(&solid, &nb_pz, true, quads_out);
        self.emit_z_faces(&solid, &nb_mz, false, quads_out);
    }

    fn opaque_neighbor(&self, neighbors: &dyn BrickNeighborhood, offset: [i32; 3]) -> [u64; 8] {
        let _ = self;
        match neighbors.neighbor_brick(offset) {
            Some(nb) if !nb.vacant() => SolidCubeMask::new(nb).opaque_planes(),
            _ => [0u64; 8],
        }
    }

    fn collect_module_anchors(&self, modules_out: &mut Vec<ModuleInstance>) {
        let coord = self.coord;
        let datum = self.datum;
        self.brick.visit_occupied(|slot, cell| {
            if let BrickCell::ModuleAnchor(spec) = cell {
                modules_out.push(ModuleInstance::from_grid_anchor(coord.cell_at(slot), spec, datum));
            }
        });
    }

    fn emit_y_faces(&self, solid: &[u64; 8], nb_y: &[u64; 8], top: bool, out: &mut Vec<PackedQuad>) {
        let normal_idx = if top { 2 } else { 3 };
        for lw in 0..8usize {
            let ahead = if top {
                if lw < 7 { solid[lw + 1] } else { nb_y[0] }
            } else if lw > 0 {
                solid[lw - 1]
            } else {
                nb_y[7]
            };
            let vis = solid[lw] & !ahead;
            if vis == 0 {
                continue;
            }
            let ao = CornerOcclusion::from_y_plane(ahead, top);
            let mut active = [0u8; 8];
            let mut sigs = [[0u32; 8]; 8];
            let mut m = vis;
            while m != 0 {
                let tz = m.trailing_zeros() as usize;
                let (lu, lv) = (tz & 7, tz >> 3);
                let (su, sv) = if top { (lv, lu) } else { (lu, lv) };
                active[sv] |= 1u8 << su;
                sigs[sv][su] = self.cell_signature(lu, lv, lw, normal_idx, ao.evaluate_at(su, sv));
                m &= m - 1;
            }
            SliceBitMerge::new(active, sigs, normal_idx).emit_merged(lw as u8, out);
        }
    }

    fn emit_x_faces(&self, solid: &[u64; 8], nb_x: &[u64; 8], east: bool, out: &mut Vec<PackedQuad>) {
        let normal_idx = if east { 0 } else { 1 };
        let mut vis_planes = [0u64; 8];
        let mut any = 0u64;
        for lw in 0..8usize {
            let ahead = if east {
                let seam = (nb_x[lw] & 0x0101_0101_0101_0101) << 7;
                ((solid[lw] >> 1) & NOT_COL7) | seam
            } else {
                let seam = (nb_x[lw] & 0x8080_8080_8080_8080) >> 7;
                ((solid[lw] << 1) & NOT_COL0) | seam
            };
            vis_planes[lw] = solid[lw] & !ahead;
            any |= vis_planes[lw];
        }
        if any == 0 {
            return;
        }
        for lu in 0..8usize {
            let ahead_src = if east {
                if lu < 7 { (solid, lu + 1) } else { (nb_x, 0) }
            } else if lu > 0 {
                (solid, lu - 1)
            } else {
                (nb_x, 7)
            };
            let ao = CornerOcclusion::from_x_plane(ahead_src.0, ahead_src.1, east);
            let mut active = [0u8; 8];
            let mut sigs = [[0u32; 8]; 8];
            let mut has_any = false;
            for lw in 0..8usize {
                let col_bits = (vis_planes[lw] >> lu) & 0x0101_0101_0101_0101;
                if col_bits == 0 {
                    continue;
                }
                for lv in 0..8usize {
                    if ((col_bits >> (lv * 8)) & 1) != 0 {
                        has_any = true;
                        let (su, sv) = if east { (lw, lv) } else { (lv, lw) };
                        active[sv] |= 1u8 << su;
                        sigs[sv][su] = self.cell_signature(lu, lv, lw, normal_idx, ao.evaluate_at(su, sv));
                    }
                }
            }
            if has_any {
                SliceBitMerge::new(active, sigs, normal_idx).emit_merged(lu as u8, out);
            }
        }
    }

    fn emit_z_faces(&self, solid: &[u64; 8], nb_z: &[u64; 8], south: bool, out: &mut Vec<PackedQuad>) {
        let normal_idx = if south { 4 } else { 5 };
        let mut vis_planes = [0u64; 8];
        let mut any = 0u64;
        for lw in 0..8usize {
            let ahead = if south {
                (solid[lw] >> 8) | ((nb_z[lw] & 0xFF) << 56)
            } else {
                (solid[lw] << 8) | ((nb_z[lw] >> 56) & 0xFF)
            };
            vis_planes[lw] = solid[lw] & !ahead;
            any |= vis_planes[lw];
        }
        if any == 0 {
            return;
        }
        for lv in 0..8usize {
            let ahead_src = if south {
                if lv < 7 { (solid, lv + 1) } else { (nb_z, 0) }
            } else if lv > 0 {
                (solid, lv - 1)
            } else {
                (nb_z, 7)
            };
            let ao = CornerOcclusion::from_z_plane(ahead_src.0, ahead_src.1, south);
            let mut active = [0u8; 8];
            let mut sigs = [[0u32; 8]; 8];
            let mut has_any = false;
            for lw in 0..8usize {
                let mut row = ((vis_planes[lw] >> (lv * 8)) & 0xFF) as u8;
                if row == 0 {
                    continue;
                }
                has_any = true;
                while row != 0 {
                    let lu = row.trailing_zeros() as usize;
                    let (su, sv) = if south { (lu, lw) } else { (lw, lu) };
                    active[sv] |= 1u8 << su;
                    sigs[sv][su] = self.cell_signature(lu, lv, lw, normal_idx, ao.evaluate_at(su, sv));
                    row &= row - 1;
                }
            }
            if has_any {
                SliceBitMerge::new(active, sigs, normal_idx).emit_merged(lv as u8, out);
            }
        }
    }

    fn cell_signature(&self, lu: usize, lv: usize, lw: usize, normal_idx: u8, ao_pack: u8) -> u32 {
        let slot = LocalSlot::from_axes(lu as u8, lv as u8, lw as u8);
        let (color, tag) = match self.brick.cell_at(slot) {
            BrickCell::RawAtom(atom) => self.atom_appearance(atom, normal_idx),
            _ => (Vec3::splat(0.7), 0u8),
        };
        let swatch = Rgb565Swatch::from_linear(color).code();
        (ao_pack as u32) | ((tag as u32) << 8) | ((swatch as u32) << 16)
    }

    fn atom_appearance(&self, atom: RawAtom, normal_idx: u8) -> (Vec3, u8) {
        let _ = self;
        match atom.medium() {
            BlockMedium::TurfEarth if atom.verdant_turf() && normal_idx == 2 => (Vec3::new(0.29, 0.59, 0.30), 1),
            BlockMedium::TurfEarth => (Vec3::new(0.46, 0.34, 0.22), 0),
            BlockMedium::Sand => (Vec3::new(0.84, 0.74, 0.46), 0),
            BlockMedium::CrustRock | BlockMedium::QuarriedStone => (Vec3::new(0.52, 0.53, 0.56), 0),
            BlockMedium::TitaniumHull => (Vec3::new(0.72, 0.77, 0.84), 0),
            BlockMedium::CarbonSilicate => (Vec3::new(0.24, 0.27, 0.33), 0),
            BlockMedium::VitrifiedBasalt => (Vec3::new(0.18, 0.19, 0.22), 0),
            BlockMedium::IronOre => (Vec3::new(0.58, 0.44, 0.38), 0),
            BlockMedium::CopperOre => (Vec3::new(0.78, 0.48, 0.26), 0),
            BlockMedium::GoldOre => (Vec3::new(0.92, 0.78, 0.24), 2),
            BlockMedium::LaserBeacon => (Vec3::new(0.22, 0.94, 0.88), 3),
            BlockMedium::RawTimber | BlockMedium::PlanedPlanks => (Vec3::new(0.62, 0.45, 0.28), 0),
            BlockMedium::OriginStone | BlockMedium::SpawnStone => (Vec3::new(0.42, 0.78, 0.88), 2),
            _ => (Vec3::new(0.64, 0.67, 0.72), 0),
        }
    }
}
