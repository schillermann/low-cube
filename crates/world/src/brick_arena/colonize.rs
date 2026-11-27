use glam::Vec3;
use crate::MeshTarget;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::relief::TurfScars;
use super::atom::RawAtom;
use super::brick::BrickCell;
use super::coord::CellCoord;
use super::inertia::BrickVolume;
use super::module::{MachineKind, SpaceModule};
use super::module_mesh::ModuleSculpture;
use super::niche::NicheConfinement;
use super::structure::BrickStructure;

/// Read-only vertical column view over `BrickStructure` for sod exposure, niche confinement, and terraced emission (`2` fields).
pub(super) struct BrickColumnView<'a> {
    structure: &'a BrickStructure,
    uv: (i32, i32),
}

impl<'a> BrickColumnView<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(structure: &'a BrickStructure, uv: (i32, i32)) -> Self {
        Self { structure, uv }
    }

    pub(super) fn covers_ground(&self, ground_top: f32) -> bool {
        let w = ((ground_top - self.structure.datum()) / GridCell::SPAN).round() as i32;
        let coord = CellCoord::new(self.uv.0, self.uv.1, w);
        match self.structure.bricks().cell_at(coord) {
            BrickCell::Empty => false,
            BrickCell::RawAtom(atom) => atom.at_floor(coord.floor_elev(self.structure.datum())).smothers_turf(),
            BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_) => true,
        }
    }

    pub(super) fn bare_surface(&self, ground_bare: bool) -> bool {
        self.structure
            .bricks()
            .inner()
            .find_in_column_rev(self.uv.0, self.uv.1, |_, cell| match cell {
                BrickCell::RawAtom(atom) => Some(atom.bare_surface(ground_bare)),
                _ => Some(ground_bare),
            })
            .unwrap_or(ground_bare)
    }

    pub(super) fn covered_above(&self, coord: CellCoord) -> bool {
        let above = coord.shifted(0, 0, 1);
        match self.structure.bricks().cell_at(above) {
            BrickCell::Empty => false,
            BrickCell::RawAtom(atom) => atom.at_floor(above.floor_elev(self.structure.datum())).smothers_turf(),
            BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_) => true,
        }
    }

    pub(super) fn verdant_near_step(&self, target_step: i32) -> bool {
        self.structure.bricks().inner().any_in_column(self.uv.0, self.uv.1, |coord, cell| match cell {
            BrickCell::RawAtom(atom) => {
                !self.covered_above(coord)
                    && atom.verdant_turf()
                    && (target_step - (coord.ceiling_elev(self.structure.datum()) / GridCell::SPAN).round() as i32).abs() <= 1
            }
            _ => false,
        })
    }

    pub(super) fn confine_niche(&self, ground_h: f32, elev: f32) -> Option<f32> {
        if !self.structure.bricks().inner().column_present(self.uv.0, self.uv.1) {
            return None;
        }
        let datum = self.structure.datum();
        let (mut spans, mut len) = ([(0.0f32, 0.0f32); 32], 0usize);
        self.structure.bricks().inner().visit_column(self.uv.0, self.uv.1, |coord, cell| {
            if self.airlock_open_passage(coord, cell) != Some(true) && len < spans.len() {
                spans[len] = (coord.floor_elev(datum), coord.ceiling_elev(datum));
                len += 1;
            }
        });
        NicheConfinement::new(&spans[..len], ground_h).confine(elev)
    }

    pub(super) fn airlock_open_passage(&self, coord: CellCoord, cell: BrickCell) -> Option<bool> {
        let (anchor, spec) = match cell {
            BrickCell::ModuleAnchor(s) => (coord, s),
            BrickCell::ModuleProxy(_) => self.structure.bricks().resolve_module(coord)?,
            _ => return None,
        };
        if spec.module() != SpaceModule::Machine(MachineKind::AirlockChamber) {
            return Some(false);
        }
        let dw = coord.layer_w() - anchor.layer_w();
        Some(dw == 1 || dw == 2)
    }

    pub(super) fn emit(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        colors: (Vec3, Vec3),
        scars: &TurfScars,
        target: &mut dyn MeshTarget,
    ) -> bool {
        if !self.structure.bricks().inner().column_present(self.uv.0, self.uv.1) {
            return false;
        }
        let datum = self.structure.datum();
        let (top_col, wall_col) = colors;
        let mut total_count = 0usize;
        self.structure.bricks().inner().visit_column(self.uv.0, self.uv.1, |_, _| total_count += 1);
        self.structure.bricks().inner().visit_column(self.uv.0, self.uv.1, |coord, bcell| {
            match bcell {
                BrickCell::Empty | BrickCell::ModuleProxy(_) => {}
                BrickCell::ModuleAnchor(spec) => {
                    ModuleSculpture::new(coord, spec, datum).emit(frame, target);
                }
                BrickCell::RawAtom(atom) => {
                    let floor = coord.floor_elev(datum);
                    let stage = scars.cell_stage(self.uv.0, self.uv.1, floor);
                    let covered = self.covered_above(coord);
                    let flank_only = total_count > 2
                        && self.full_cube_at(coord)
                        && self.full_cube_at(coord.shifted(0, 0, -1))
                        && self.full_cube_at(coord.shifted(0, 0, 1));
                    atom.at_floor(floor).emit_terraced(
                        cell, frame, top_col, wall_col, covered, flank_only, stage, target,
                    );
                }
            }
        });
        true
    }

    fn full_cube_at(&self, coord: CellCoord) -> bool {
        match self.structure.bricks().cell_at(coord) {
            BrickCell::RawAtom(a) => a.at_floor(coord.floor_elev(self.structure.datum())).full_cube_envelope(),
            _ => false,
        }
    }
}

/// Collaborator stepping 3D turf-grass colonization across bare surface `RawAtom`s (`1` field).
pub(super) struct TurfColonization<'a> {
    structure: &'a mut BrickStructure,
}

impl<'a> TurfColonization<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(structure: &'a mut BrickStructure) -> Self {
        Self { structure }
    }

    pub(super) fn advance(
        self,
        dt: f32,
        verdant_neighbors: impl Fn(i32, i32, i32) -> u32,
    ) -> (Vec<(i32, i32)>, bool) {
        let mut active = Vec::new();
        let datum = self.structure.datum();
        self.structure.bricks().inner().visit_occupied_cells(|coord, cell| {
            if let BrickCell::RawAtom(atom) = cell {
                let (u, v) = coord.planar();
                let view = BrickColumnView::new(self.structure, (u, v));
                if !view.covered_above(coord) && atom.bare_turf() {
                    let h = (coord.ceiling_elev(datum) / GridCell::SPAN).round() as i32;
                    let count = verdant_neighbors(u, v, h);
                    if count > 0 {
                        active.push((coord, count));
                    }
                }
            }
        });
        let any = !active.is_empty();
        let mut regrown = Vec::new();
        for (coord, neighbors) in active {
            let mut grown = false;
            self.structure.mutate_atom_at(coord, |atom: RawAtom| {
                let (next, g) = atom.colonized(coord.grid_cell(), dt, neighbors);
                grown = g;
                next
            });
            if grown {
                regrown.push(coord.planar());
            }
        }
        (regrown, any)
    }
}
