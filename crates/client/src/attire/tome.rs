use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BlockMold, MeshTarget};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
use crate::menu::PixelStamp;
use super::pose::BodyPose;
use super::pouch::BeltSatchel;
use super::relic::HeldRelic;
use super::YouthAttire;

/// Single craftable recipe entry inside the medieval Crafting Book.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftingEntry {
    product: HeldRelic, category: &'static str, ingredients: [(HeldRelic, u32); 2],
}

impl CraftingEntry {
    pub const fn new(product: HeldRelic, category: &'static str, ingredients: [(HeldRelic, u32); 2]) -> Self {
        Self { product, category, ingredients }
    }
    pub const fn product(&self) -> HeldRelic { self.product }
    pub const fn ingredients(&self) -> [(HeldRelic, u32); 2] { self.ingredients }

    pub fn craftable(&self, satchel: &BeltSatchel, attire: YouthAttire) -> bool {
        self.ingredients.iter().all(|&(relic, needed)| needed == 0 || satchel.resource_tally(relic, attire) >= needed)
    }

    pub fn requirement_summary(&self, satchel: &BeltSatchel, attire: YouthAttire) -> String {
        let mut parts = Vec::with_capacity(2);
        for &(relic, needed) in &self.ingredients {
            if needed > 0 { parts.push(format!("{}: {}/{}", relic.title(), satchel.resource_tally(relic, attire), needed)); }
        }
        parts.join("   ")
    }
}

/// Interactive medieval Crafting Book listing craftable tools, weapons, and items with live resource counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtisanTome {
    cursor: usize, open: bool,
}

impl ArtisanTome {
    pub const fn new(cursor: usize, open: bool) -> Self { Self { cursor, open } }
    pub const fn closed() -> Self { Self::new(0, false) }
    pub const fn open(&self) -> bool { self.open }

    pub const fn entries() -> [CraftingEntry; 7] {
        [
            CraftingEntry::new(HeldRelic::WoodmanAxe, "TOOL", [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 1)]),
            CraftingEntry::new(HeldRelic::PlankBlock, "BLOCK", [(HeldRelic::TimberBlock, 1), (HeldRelic::EmptyPalm, 0)]),
            CraftingEntry::new(HeldRelic::WoodenMallet, "TOOL", [(HeldRelic::DeadwoodBough, 2), (HeldRelic::FlintStone, 1)]),
            CraftingEntry::new(HeldRelic::IronSpade, "TOOL", [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 2)]),
            CraftingEntry::new(HeldRelic::KnightlySword, "WEAPON", [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 3)]),
            CraftingEntry::new(HeldRelic::TimberTorch, "ITEM", [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 1)]),
            CraftingEntry::new(HeldRelic::QuarriedStone, "BLOCK", [(HeldRelic::FlintStone, 2), (HeldRelic::EmptyPalm, 0)]),
        ]
    }

    pub const fn selected_entry(&self) -> CraftingEntry {
        let list = Self::entries();
        list[self.cursor % list.len()]
    }

    pub const fn with_open(self, open: bool) -> Self { Self::new(self.cursor, open) }
    pub const fn toggle(self) -> Self { Self::new(self.cursor, !self.open) }

    pub const fn navigate(self, delta: isize) -> Self {
        let len = Self::entries().len() as isize;
        Self::new((self.cursor as isize + delta).rem_euclid(len) as usize, self.open)
    }

    pub fn craft_selected(self, side: f32, satchel: BeltSatchel, attire: YouthAttire) -> (Self, BeltSatchel, YouthAttire, bool) {
        let (state, entry) = (satchel.reconcile_attire(attire), self.selected_entry());
        if !entry.craftable(&state, attire) { return (self, state, attire, false); }
        let (next_satchel, next_attire) = state.consume_and_equip(entry.ingredients(), entry.product(), side, attire);
        (self.with_open(false), next_satchel, next_attire, true)
    }

    pub fn dispatch_trigger(self, trigger: InputTrigger, bindings: &ActionBindings, satchel: BeltSatchel, attire: YouthAttire) -> (Self, BeltSatchel, YouthAttire, bool) {
        if bindings.matches(ControlAction::MoveForward, trigger) || bindings.matches(ControlAction::MoveLeft, trigger) || bindings.matches(ControlAction::MoldLeft, trigger) {
            return (self.navigate(-1), satchel, attire, true);
        }
        if bindings.matches(ControlAction::MoveBackward, trigger) || bindings.matches(ControlAction::MoveRight, trigger) || bindings.matches(ControlAction::MoldRight, trigger) {
            return (self.navigate(1), satchel, attire, true);
        }
        if bindings.matches(ControlAction::SatchelRight, trigger) || bindings.matches(ControlAction::SatchelLeft, trigger)
            || bindings.matches(ControlAction::Jump, trigger) || bindings.matches(ControlAction::MoldForm, trigger)
            || trigger == InputTrigger::Key(KeyCode::Enter)
        {
            let side = if bindings.matches(ControlAction::SatchelLeft, trigger) { -1.0 } else { 1.0 };
            let (next_tome, next_satchel, next_attire, _) = self.craft_selected(side, satchel, attire);
            return (next_tome, next_satchel, next_attire, true);
        }
        (self, satchel, attire, false)
    }

    pub fn emit_book(
        &self,
        satchel: &BeltSatchel,
        attire: YouthAttire,
        pose: &BodyPose,
        target: &mut dyn MeshTarget,
    ) {
        if !self.open {
            return;
        }
        let entries = Self::entries();
        let tray = pose.view_frame().shifted(0.0, 0.01, 0.44);
        tray.emit_framed_plaque(0.285, 0.235, 0.0035, Vec3::new(0.07, 0.09, 0.14), Vec3::new(0.54, 0.36, 0.19), target);
        PixelStamp::new(0.0028, 0.0028, Vec3::new(1.58, 1.32, 0.48))
            .emit_on_frame("CRAFTING BOOK", 0.0, 0.205, 0.016, &tray, target);

        for (i, entry) in entries.iter().enumerate() {
            let selected = i == (self.cursor % entries.len());
            let ready = entry.craftable(satchel, attire);
            let row_y = 0.148 - (i as f32) * 0.048;
            let row_frame = tray.shifted(0.0, row_y, if selected { -0.012 } else { 0.0 });
            let row_bg = if selected { Vec3::new(0.16, 0.21, 0.30) } else { Vec3::new(0.10, 0.13, 0.19) };
            let row_rim = if selected {
                Vec3::new(0.96, 0.80, 0.26)
            } else if ready {
                Vec3::new(0.28, 0.58, 0.34)
            } else {
                Vec3::new(0.32, 0.24, 0.18)
            };
            row_frame.emit_framed_plaque(0.262, 0.021, 0.0020, row_bg, row_rim, target);

            let icon_frame = row_frame.shifted(-0.225, 0.0, -0.010).scaled(0.42);
            entry.product.emit_in_satchel_slot(BlockMold::cube(), row_rim, &icon_frame, target);

            let title_col = if ready { Vec3::new(1.10, 1.55, 0.95) } else { Vec3::new(1.48, 1.36, 1.12) };
            let status = if ready { "READY" } else { "NEED RESOURCES" };
            let heading = format!("{} [{}] - {}", entry.product.title(), entry.category, status);
            PixelStamp::new(0.0019, 0.0019, title_col)
                .emit_on_frame(&heading, 0.025, 0.007, 0.016, &row_frame, target);

            let req_col = if ready { Vec3::new(0.88, 1.48, 0.88) } else { Vec3::new(1.52, 0.96, 0.68) };
            let req_text = entry.requirement_summary(satchel, attire);
            PixelStamp::new(0.0018, 0.0018, req_col)
                .emit_on_frame(&req_text, 0.025, -0.008, 0.016, &row_frame, target);
        }

        let timber = satchel.resource_tally(HeldRelic::TimberBlock, attire);
        let planks = satchel.resource_tally(HeldRelic::PlankBlock, attire);
        let wood = satchel.resource_tally(HeldRelic::DeadwoodBough, attire);
        let flint = satchel.resource_tally(HeldRelic::FlintStone, attire);
        let stone = satchel.resource_tally(HeldRelic::QuarriedStone, attire);
        let earth = satchel.resource_tally(HeldRelic::ExcavatedEarth, attire);
        let tally_line = format!("OWNED: WOOD {}  PLANKS {}  DEADWOOD {}  FLINT {}  STONE {}  EARTH {}", timber, planks, wood, flint, stone, earth);
        PixelStamp::new(0.0020, 0.0020, Vec3::new(1.45, 1.48, 1.55))
            .emit_on_frame(&tally_line, 0.0, -0.192, 0.016, &tray, target);
        PixelStamp::new(0.0019, 0.0019, Vec3::new(1.55, 1.30, 0.48))
            .emit_on_frame("WASD/WHEEL: SELECT   CLICK/E/Q: CRAFT   C/ESC: CLOSE", 0.0, -0.214, 0.016, &tray, target);
    }
}
