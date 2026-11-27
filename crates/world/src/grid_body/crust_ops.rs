use glam::Vec3;
use crate::block::{BlockMedium, GridCell};
use crate::brick_arena::{
    BrickCell, BrickStructure, BrickVolume, CellCoord, DismantledSpan, ModuleSpec, RawAtom,
    NICHE_CONTACT_TOLERANCE,
};
use crate::frame::SurfaceFrame;
use crate::relief::{MoldedStructure, 
    CanopyHabitat, NaturalHabitat, NaturalStrata, SanctuaryPlinth, SectorCanvas,
    SectorVault, ShaftExcavation, TurfHabitat, TurfScars,
};

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Read-only planetary crust survey over a `CrustAnchored` `GridBody` (`2` private fields).
pub struct CrustSurvey<'a> {
    grid: &'a BrickStructure,
    plinth: SanctuaryPlinth,
}

impl<'a> CrustSurvey<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(grid: &'a BrickStructure, plinth: SanctuaryPlinth) -> Self {
        Self { grid, plinth }
    }

    pub fn sanctuary_at(&self, u: i32, v: i32) -> bool {
        self.plinth.active_at(u, v)
            || self.grid.bricks().inner().any_in_column(u, v, |_, cell| {
                matches!(cell, BrickCell::RawAtom(a) if a.medium().indestructible())
            })
    }

    pub fn collect_medium_sites(&self, medium: BlockMedium, frame: &SurfaceFrame, out: &mut Vec<(Vec3, Vec3)>) {
        self.plinth.collect_site(medium, frame, out);
        self.grid.collect_medium_sites(medium, frame, out);
    }

    pub fn capture_sector(&self, gsu: i32, gsv: i32) -> Vec<crate::SnapshotBlock> {
        SectorVault::new(self.grid).capture(gsu, gsv)
    }

    pub fn collect_modified_sectors(&self, out: &mut Vec<(i32, i32)>) {
        SectorVault::new(self.grid).collect_modified(out);
    }
}

/// Mutable planetary crust mason over a `CrustAnchored` `GridBody` (`1` private field).
pub struct CrustMason<'a> {
    grid: &'a mut BrickStructure,
}

impl<'a> CrustMason<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(grid: &'a mut BrickStructure) -> Self {
        Self { grid }
    }

    pub fn stack_bare(
        self,
        (u, v): (i32, i32),
        (floor_elev, ground_top): (f32, f32),
        sub: bool,
        atom: RawAtom,
        scars: &mut TurfScars,
    ) {
        let bare = atom.smothered();
        if !sub
            && (floor_elev - ground_top).abs() < NICHE_CONTACT_TOLERANCE
            && bare.at_floor(floor_elev).smothers_turf()
        {
            scars.expose_dirt(u, v);
        }
        self.grid.insert_atom(u, v, floor_elev, bare);
    }

    pub fn place_module(
        self,
        anchor: CellCoord,
        spec: ModuleSpec,
        ground_top: f32,
        scars: &mut TurfScars,
    ) -> bool {
        if !self.grid.place_module_at(anchor, spec) {
            return false;
        }
        let (du, dv, _) = spec.span().dimensions();
        if (anchor.floor_elev(0.0) - ground_top).abs() < NICHE_CONTACT_TOLERANCE {
            for ov in 0..dv {
                for ou in 0..du {
                    let (u, v) = anchor.shifted(ou as i32, ov as i32, 0).planar();
                    scars.expose_dirt(u, v);
                    scars.clear_stage(u, v);
                }
            }
        }
        true
    }

    pub fn remove_layer(
        self,
        (u, v): (i32, i32),
        (layer_elev, ground_top): (f32, f32),
        scars: &mut TurfScars,
    ) -> Option<BlockMedium> {
        let (removed, column_vacated) = self.grid.remove_at(u, v, layer_elev)?;
        match removed {
            DismantledSpan::Vacant => None,
            DismantledSpan::Atom { coord, atom } => {
                let floor = coord.floor_elev(0.0);
                if column_vacated
                    && (floor - ground_top).abs() < NICHE_CONTACT_TOLERANCE
                    && atom.at_floor(floor).smothers_turf()
                {
                    scars.expose_dirt(u, v);
                }
                scars.clear_stage(u, v);
                Some(atom.medium())
            }
            DismantledSpan::Module { anchor, spec } => {
                let (du, dv, _) = spec.span().dimensions();
                for ov in 0..dv {
                    for ou in 0..du {
                        let (cu, cv) = anchor.shifted(ou as i32, ov as i32, 0).planar();
                        scars.clear_stage(cu, cv);
                    }
                }
                Some(spec.medium())
            }
        }
    }

    pub fn retain_overhang(
        self,
        strata: &NaturalStrata,
        (u, v): (i32, i32),
        (next_elev, elev): (i32, i32),
        ground_verdant: bool,
    ) {
        let (nat, vault) = (strata.elevation_at(u, v), strata.vault());
        let crown_verdant = ground_verdant && !self.grid.covers_ground_at(u, v, (nat as f32) * BLOCK_STEP);
        for step in (next_elev + 1)..elev {
            let floor = (step as f32) * BLOCK_STEP;
            let atom = if vault.crust_layer_at(u, v, step) {
                RawAtom::full_cube(BlockMedium::CrustRock)
            } else if step == nat - 1 {
                RawAtom::natural_crown(strata.crown_at(u, v), crown_verdant)
            } else {
                RawAtom::full_cube(BlockMedium::TurfEarth)
            };
            self.grid.insert_atom(u, v, floor, atom);
        }
    }

    pub fn regrow_sod(
        self,
        plinth: SanctuaryPlinth,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        scars: &TurfScars,
        dt: f32,
    ) -> (Vec<(i32, i32)>, bool) {
        if matches!(plinth, SanctuaryPlinth::Absent) && self.grid.vacant() {
            return (Vec::new(), false);
        }
        let active = {
            let habitat = CanopyHabitat::new(NaturalHabitat::new(strata, shafts, scars), &*self.grid);
            let mut list = Vec::new();
            self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
                if let BrickCell::RawAtom(atom) = cell {
                    let (u, v) = coord.planar();
                    let above = self.grid.bricks().cell_at(coord.shifted(0, 0, 1)).occupied();
                    if !above && atom.bare_turf() {
                        let n = habitat.verdant_neighbors(u, v, coord.layer_w() + 1);
                        if n > 0 {
                            list.push((coord, n));
                        }
                    }
                }
            });
            list
        };
        if active.is_empty() {
            return (Vec::new(), false);
        }
        let mut regrown = Vec::new();
        for (coord, n) in active {
            let BrickCell::RawAtom(atom) = self.grid.bricks().cell_at(coord) else { continue };
            let (next, grown) = atom.colonized(coord.grid_cell(), dt, n);
            if next != atom {
                self.grid.insert_atom(coord.planar().0, coord.planar().1, coord.floor_elev(0.0), next);
            }
            if grown {
                regrown.push(coord.planar());
            }
        }
        (regrown, true)
    }

    pub fn apply_sector(self, strata: &NaturalStrata, gsu: i32, gsv: i32, blocks: &[crate::SnapshotBlock]) {
        SectorCanvas::new(self.grid).apply(strata, gsu, gsv, blocks);
    }
}
