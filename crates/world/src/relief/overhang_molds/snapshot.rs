use crate::grid_body::GridBody;
use super::super::cantilever_rig::RigRegistry;
use super::super::natural_strata::NaturalStrata;
use super::super::turf_scars::TurfScars;

/// Read-only sector snapshot view over `GridBody` crust, `RigRegistry`, and `TurfScars` (`3` fields).
pub(super) struct SectorMoldsView<'a> {
    crust_body: &'a GridBody,
    rigs: &'a RigRegistry,
    scars: &'a TurfScars,
}

impl<'a> SectorMoldsView<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        crust_body: &'a GridBody,
        rigs: &'a RigRegistry,
        scars: &'a TurfScars,
    ) -> Self {
        Self { crust_body, rigs, scars }
    }

    pub(super) fn capture(
        &self,
        gsu: i32,
        gsv: i32,
    ) -> (u64, Vec<crate::SnapshotBlock>, Vec<crate::RigSnapshot>) {
        (
            self.scars.capture_bare_mask(gsu, gsv),
            self.crust_body.crust().capture_sector(gsu, gsv),
            self.rigs.capture_sector(gsu, gsv),
        )
    }

    pub(super) fn collect_modified_sectors(&self, out: &mut Vec<(i32, i32)>) {
        self.crust_body.crust().collect_modified_sectors(out);
        self.rigs.collect_modified_sectors(out);
        self.scars.collect_modified_sectors(out);
    }
}

/// Mutable sector snapshot target updating `GridBody` crust, `RigRegistry`, and `TurfScars` (`3` fields).
pub(super) struct SectorMoldsTarget<'a> {
    crust_body: &'a mut GridBody,
    rigs: &'a mut RigRegistry,
    scars: &'a mut TurfScars,
}

impl<'a> SectorMoldsTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        crust_body: &'a mut GridBody,
        rigs: &'a mut RigRegistry,
        scars: &'a mut TurfScars,
    ) -> Self {
        Self { crust_body, rigs, scars }
    }

    pub(super) fn apply(
        self,
        strata: &NaturalStrata,
        gsu: i32,
        gsv: i32,
        bare_mask: u64,
        cols: &crate::SectorColumnState,
    ) {
        self.scars.apply_bare_mask(gsu, gsv, bare_mask);
        self.crust_body.crust_mut().apply_sector(strata, gsu, gsv, cols.blocks());
        self.rigs.apply_sector(gsu, gsv, cols.rigs());
    }
}
