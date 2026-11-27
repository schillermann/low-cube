use glam::Vec3;
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use crate::relief::TurfScars;
use super::arena::BrickArena;
use super::atom::RawAtom;
use super::brick::BrickCell;
use super::coord::CellCoord;
use super::inertia::{BrickVolume, InertialBricks};
use super::module_spec::ModuleSpec;
use super::structure::BrickStructure;

/// Spatial rigid transforms, gravitational settling, and sector queries over `BrickStructure` (`1` field).
pub(super) struct BrickSweep<'a> {
    structure: &'a mut BrickStructure,
}

impl<'a> BrickSweep<'a> {
    pub(super) fn new(structure: &'a mut BrickStructure) -> Self {
        Self { structure }
    }

    pub(super) fn rotate_around(self, piv_u: i32, piv_v: i32) {
        let mut atoms: Vec<(CellCoord, RawAtom)> = Vec::new();
        let mut modules: Vec<(CellCoord, ModuleSpec)> = Vec::new();
        self.structure.bricks().inner().visit_occupied_cells(|coord, cell| match cell {
            BrickCell::RawAtom(atom) => atoms.push((coord, atom)),
            BrickCell::ModuleAnchor(spec) => modules.push((coord, spec)),
            _ => {}
        });
        let datum = self.structure.datum();
        let mut rebuilt = BrickStructure::new(InertialBricks::wrapping(BrickArena::untouched()), datum);
        for (coord, atom) in atoms {
            let (cu, cv, cw) = coord.components();
            let (du, dv) = (cu - piv_u, cv - piv_v);
            let _ = rebuilt.place_raw(CellCoord::new(piv_u - dv, piv_v + du, cw), atom.rotated_quarter());
        }
        for (anchor, spec) in modules {
            let (cu, cv, cw) = anchor.components();
            let (du, dv) = (cu - piv_u, cv - piv_v);
            let (b, _, _) = spec.span().dimensions();
            let next_anchor = CellCoord::new(piv_u - dv - (i32::from(b) - 1), piv_v + du, cw);
            let _ = rebuilt.place_module_at(next_anchor, spec.rotated_quarter());
        }
        *self.structure = rebuilt;
    }

    pub(super) fn translate_by(self, du: i32, dv: i32, dz: f32) -> Vec<(i32, i32, f32)> {
        let mut atoms: Vec<(CellCoord, RawAtom)> = Vec::new();
        let mut modules: Vec<(CellCoord, ModuleSpec)> = Vec::new();
        self.structure.bricks().inner().visit_occupied_cells(|coord, cell| match cell {
            BrickCell::RawAtom(atom) => atoms.push((coord, atom)),
            BrickCell::ModuleAnchor(spec) => modules.push((coord, spec)),
            _ => {}
        });
        let next_datum = self.structure.datum() + dz;
        let dw = (next_datum / GridCell::SPAN).round() as i32;
        let rem_datum = next_datum - (dw as f32) * GridCell::SPAN;
        let mut rebuilt = BrickStructure::new(InertialBricks::wrapping(BrickArena::untouched()), rem_datum);
        let mut moved = Vec::new();
        for (coord, atom) in atoms {
            let dest = coord.shifted(du, dv, dw);
            let _ = rebuilt.place_raw(dest, atom);
            moved.push((dest.planar().0, dest.planar().1, dest.floor_elev(rem_datum)));
        }
        for (anchor, spec) in modules {
            let dest = anchor.shifted(du, dv, dw);
            let _ = rebuilt.place_module_at(dest, spec);
            moved.push((dest.planar().0, dest.planar().1, dest.floor_elev(rem_datum)));
        }
        *self.structure = rebuilt;
        moved
    }

    pub(super) fn settle_gravity(
        self,
        ground_fn: impl Fn(i32, i32) -> f32,
        scars: &mut TurfScars,
    ) -> Vec<(i32, i32)> {
        let datum = self.structure.datum();
        let mut candidates: Vec<(CellCoord, RawAtom)> = Vec::new();
        self.structure.bricks().inner().visit_occupied_cells(|coord, cell| {
            if let BrickCell::RawAtom(atom) = cell {
                if atom.settles_under_gravity() {
                    candidates.push((coord, atom));
                }
            }
        });
        candidates.sort_unstable_by_key(|(c, _)| (c.planar(), c.layer_w()));
        let mut settled = Vec::new();
        for (coord, atom) in candidates {
            let (u, v) = coord.planar();
            let below = coord.shifted(0, 0, -1);
            let min_w = ((ground_fn(u, v) - datum) / GridCell::SPAN).round() as i32;
            if below.layer_w() >= min_w && !self.structure.bricks().cell_at(below).occupied() {
                let _ = self.structure.dismantle_coord(coord);
                self.structure.insert_atom(u, v, below.floor_elev(datum), atom);
                if self.structure.covers_ground_at(u, v, ground_fn(u, v)) {
                    scars.expose_dirt(u, v);
                }
                if !settled.contains(&(u, v)) {
                    settled.push((u, v));
                }
            }
        }
        settled
    }

    pub(super) fn clear_sector_rect(self, (u0, u1): (i32, i32), (v0, v1): (i32, i32)) {
        let mut targets = Vec::new();
        self.structure.bricks().inner().visit_occupied_cells(|coord, cell| {
            let (u, v) = coord.planar();
            if u >= u0 && u < u1 && v >= v0 && v < v1 {
                let indestructible = match cell {
                    BrickCell::RawAtom(atom) => atom.medium().indestructible(),
                    _ => false,
                };
                if !indestructible {
                    targets.push(coord);
                }
            }
        });
        for coord in targets {
            let _ = self.structure.dismantle_coord(coord);
        }
    }
}

/// Read-only scan collecting world-space crown positions for a target `BlockMedium` (`1` field).
pub(super) struct MediumSiteScan<'a> {
    structure: &'a BrickStructure,
}

impl<'a> MediumSiteScan<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(structure: &'a BrickStructure) -> Self {
        Self { structure }
    }

    pub(super) fn collect(self, medium: BlockMedium, frame: &SurfaceFrame, out: &mut Vec<(Vec3, Vec3)>) {
        let normal = frame.normal();
        let datum = self.structure.datum();
        self.structure.bricks().inner().visit_occupied_cells(|coord, cell| match cell {
            BrickCell::RawAtom(atom) if atom.medium() == medium => {
                let (cx, cy) = coord.grid_cell().center();
                let crown = atom.at_floor(coord.floor_elev(datum)).crown_elevation();
                out.push((frame.point_at(cx, cy, crown), normal));
            }
            BrickCell::ModuleAnchor(spec) if spec.medium() == medium => {
                let (cx, cy) = coord.grid_cell().center();
                let (_, _, dw) = spec.span().dimensions();
                let top = coord.floor_elev(datum) + (dw as f32) * GridCell::SPAN;
                out.push((frame.point_at(cx, cy, top), normal));
            }
            _ => {}
        });
    }
}
