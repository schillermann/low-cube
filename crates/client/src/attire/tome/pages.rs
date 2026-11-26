use glam::Vec3;
use world::{BlockMold, MeshTarget};
use crate::menu::PixelStamp;
use super::super::burden::SlotBurden;
use super::super::compartment::{
    CompartmentBackplate, ShowcasedCompartment, SlotPlaque, WeightedCompartment,
};
use super::super::limb::LimbFrame;
use super::super::pouch::BeltSatchel;
use super::super::YouthAttire;
use super::bookmark::TomeBookmark;
use super::domain::{FolioSelection, GRID_COLS, GRID_SLOTS};
use super::polaroid::{
    IngredientFormula, PinnedPolaroid, PolaroidCard, ReadinessPipCompartment, VacantScopePlaque,
};
use super::spread::FolioSpread;

/// Decorator rendering the left-page `[X] READY TO CRAFT` scope plaque and `3x4` `SlotPlaque` compartment grid.
#[derive(Clone, Copy)]
pub struct LeftGridFolio<'a, S> {
    origin: S,
    bookmark: TomeBookmark,
    satchel: &'a BeltSatchel,
    attire: YouthAttire,
}

impl<'a, S> LeftGridFolio<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        bookmark: TomeBookmark,
        satchel: &'a BeltSatchel,
        attire: YouthAttire,
    ) -> Self {
        Self {
            origin,
            bookmark,
            satchel,
            attire,
        }
    }
}

impl<'a, S: FolioSpread> FolioSpread for LeftGridFolio<'a, S> {
    fn emit_spread(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_spread(frame, target);
        let left = frame.shifted(-0.162, 0.006, -0.004);

        let ready_scope = self.bookmark.ready_only();
        let venue = self.bookmark.venue();
        let top_bar = left.shifted(0.0, 0.164, -0.003);
        let (bar_bg, bar_rim, bar_txt, mark) = if ready_scope {
            (
                Vec3::new(0.11, 0.24, 0.17),
                Vec3::new(0.28, 0.86, 0.48),
                Vec3::new(0.82, 1.58, 0.96),
                if venue.anvil_station() { "[X] ANVIL READY (F)" } else { "[X] READY TO CRAFT (F)" },
            )
        } else if venue.anvil_station() {
            (
                Vec3::new(0.22, 0.16, 0.11),
                Vec3::new(0.92, 0.64, 0.24),
                Vec3::new(1.58, 1.38, 0.86),
                "[ ] SMITHY ANVIL (F)",
            )
        } else {
            (
                Vec3::new(0.12, 0.16, 0.23),
                Vec3::new(0.30, 0.38, 0.50),
                Vec3::new(1.42, 1.46, 1.54),
                "[ ] READY TO CRAFT (F)",
            )
        };
        top_bar.emit_framed_plaque(0.128, 0.016, 0.0018, bar_bg, bar_rim, target);
        PixelStamp::new(0.0017, 0.0019, bar_txt).emit_on_frame(mark, 0.0, 0.0, 0.0155, &top_bar, target);

        let visible = self.bookmark.visible_recipes(self.satchel, self.attire);
        let selection = self.bookmark.resolve_selection(self.satchel, self.attire);
        let mold = BlockMold::cube();

        for slot_idx in 0..GRID_SLOTS {
            let (col, row) = (slot_idx % GRID_COLS, slot_idx / GRID_COLS);
            let sx = ((col as f32) - 1.0) * 0.088;
            let sy = 0.104 - (row as f32) * 0.082;
            if let Some(&(canonical_idx, entry)) = visible.get(slot_idx) {
                let selected = selection.highlights_recipe(canonical_idx);
                let ready = entry.craftable_at(self.satchel, self.attire, venue);
                let sz = if selected { -0.012 } else { 0.0 };
                let rim = if selected {
                    Vec3::new(0.96, 0.80, 0.26)
                } else if ready {
                    Vec3::new(0.26, 0.68, 0.38)
                } else {
                    Vec3::new(0.72, 0.28, 0.24)
                };
                let slot_frame = left.shifted(sx, sy, sz).scaled(0.82);
                let plaque = ReadinessPipCompartment::new(
                    ShowcasedCompartment::new(
                        WeightedCompartment::new(
                            CompartmentBackplate::new(selected, true, rim),
                            SlotBurden::new(entry.product(), entry.batch_yield()),
                        ),
                        entry.product().figure(mold),
                    ),
                    ready,
                    entry.batch_yield(),
                );
                plaque.emit_slot(&slot_frame, target);
            } else {
                let empty_frame = left.shifted(sx, sy, 0.0).scaled(0.82);
                CompartmentBackplate::new(false, false, Vec3::new(0.18, 0.22, 0.30))
                    .emit_slot(&empty_frame, target);
            }
        }
    }
}

/// Decorator rendering the right-page tilted Polaroid showcase, ingredient list, action badges, and legend strip.
#[derive(Clone, Copy)]
pub struct RightPolaroidFolio<'a, S> {
    origin: S,
    selection: FolioSelection,
    satchel: &'a BeltSatchel,
    attire: YouthAttire,
}

impl<'a, S> RightPolaroidFolio<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        selection: FolioSelection,
        satchel: &'a BeltSatchel,
        attire: YouthAttire,
    ) -> Self {
        Self {
            origin,
            selection,
            satchel,
            attire,
        }
    }
}

impl<'a, S: FolioSpread> FolioSpread for RightPolaroidFolio<'a, S> {
    fn emit_spread(&self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.origin.emit_spread(frame, target);
        let right = frame.shifted(0.162, 0.006, -0.004);

        match self.selection {
            FolioSelection::Highlighted((_, venue), entry) => {
                let polaroid_frame = right.shifted(0.0, 0.074, -0.006).rolled(-0.048);
                PinnedPolaroid::new(
                    PolaroidCard::new(SlotBurden::new(entry.product(), entry.batch_yield())),
                    entry.product().figure(BlockMold::cube()),
                )
                .emit_slot(&polaroid_frame, target);

                IngredientFormula::at_venue(entry, self.satchel, self.attire, venue)
                    .emit_slot(&right, target);
            }
            FolioSelection::VacantTab(tab) => {
                VacantScopePlaque::new(tab).emit_slot(&right, target);
            }
        }

        PixelStamp::new(0.0016, 0.0017, Vec3::new(1.52, 1.30, 0.50)).emit_on_frame(
            "WASD/WHEEL: GRID   TAB/1-4: SECTION   F: READY   CLICK/E/Q: CRAFT   C/ESC: CLOSE",
            0.0,
            -0.203,
            0.0155,
            frame,
            target,
        );
    }
}
