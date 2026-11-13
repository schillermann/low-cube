use glam::Vec3;
use world::{BlockMold, MeshTarget};
use super::burden::{CarryLoad, SatchelBurden, SlotBurden};
use super::compartment::{
    CompartmentBackplate, LegendCompartment, ShowcasedCompartment, SlotPlaque, TalliedCompartment,
    WeightedCompartment,
};
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::sack::HangingSack;

const TRAY_SLOTS: usize = 15;

/// Visibility state of the waist satchel inspection tray.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayState {
    Closed,
    Open,
}

/// Encapsulates stowed satchel items, WASD inspection cursor, and waist-grid rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SatchelTray {
    stowed: [u32; 25],
    active_side: f32,
    cursor: usize,
    state: TrayState,
}

impl SatchelTray {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(stowed: [u32; 25], active_side: f32, cursor: usize, state: TrayState) -> Self {
        Self { stowed, active_side, cursor, state }
    }

    pub const fn open(&self) -> bool {
        matches!(self.state, TrayState::Open)
    }

    pub const fn focused_sides(&self) -> (bool, bool) {
        (self.active_side > 0.0, self.active_side < 0.0)
    }

    pub const fn with_open(mut self, open: bool) -> Self {
        self.state = if open { TrayState::Open } else { TrayState::Closed };
        self
    }

    pub fn close(&mut self) {
        self.state = TrayState::Closed;
    }

    pub fn focus_side(&mut self, side: f32) {
        self.active_side = if side > 0.0 { 1.0 } else { -1.0 };
    }

    pub fn open_if_populated(&mut self, side: f32) -> bool {
        let (slots, len) = self.packed_slots();
        if len == 0 {
            return false;
        }
        self.state = TrayState::Open;
        self.cursor = self.cursor.min(TRAY_SLOTS - 1);
        if slots[self.cursor].free_palm() {
            self.cursor = 0;
        }
        self.focus_side(side);
        true
    }

    pub fn selected_relic(&self) -> HeldRelic {
        let (slots, _) = self.packed_slots();
        slots[self.cursor.min(TRAY_SLOTS - 1)]
    }

    pub const fn count(&self, relic: HeldRelic) -> u32 {
        self.stowed[relic.code() as usize]
    }

    pub fn any_stowed_blocks(&self) -> bool {
        [
            HeldRelic::ExcavatedEarth,
            HeldRelic::CrustBlock,
            HeldRelic::QuarriedStone,
            HeldRelic::TimberBlock,
            HeldRelic::PlankBlock,
            HeldRelic::SteeringWheel,
            HeldRelic::AircraftEngine,
            HeldRelic::RocketEngine,
            HeldRelic::CabinSeat,
            HeldRelic::CarWheel,
            HeldRelic::PedalCluster,
            HeldRelic::CrossAxle,
            HeldRelic::CopperOre,
            HeldRelic::IronOre,
            HeldRelic::GoldOre,
        ]
        .iter()
        .any(|&relic| self.count(relic) > 0)
    }

    fn packed_slots(&self) -> ([HeldRelic; TRAY_SLOTS], usize) {
        let (mut slots, mut len) = ([HeldRelic::EmptyPalm; TRAY_SLOTS], 0usize);
        let materials = [
            HeldRelic::GoldOre,
            HeldRelic::IronOre,
            HeldRelic::CopperOre,
            HeldRelic::CabinSeat,
            HeldRelic::PedalCluster,
            HeldRelic::CrossAxle,
            HeldRelic::CarWheel,
            HeldRelic::SteeringWheel,
            HeldRelic::AircraftEngine,
            HeldRelic::RocketEngine,
            HeldRelic::ExcavatedEarth,
            HeldRelic::CrustBlock,
            HeldRelic::QuarriedStone,
            HeldRelic::TimberBlock,
            HeldRelic::PlankBlock,
            HeldRelic::DeadwoodBough,
            HeldRelic::FlintStone,
        ];
        for mat in materials {
            if self.count(mat) > 0 && len < TRAY_SLOTS {
                slots[len] = mat;
                len += 1;
            }
        }
        let tools = [
            HeldRelic::FieldCamera,
            HeldRelic::CantileverCrane,
            HeldRelic::IronSpade,
            HeldRelic::WoodmanAxe,
            HeldRelic::KnightlySword,
            HeldRelic::WoodenMallet,
            HeldRelic::TimberTorch,
        ];
        for relic in tools {
            for _ in 0..self.count(relic) {
                if len < TRAY_SLOTS {
                    slots[len] = relic;
                    len += 1;
                }
            }
        }
        (slots, len)
    }

    pub fn stow_relic(&mut self, relic: HeldRelic, count: u32) {
        if relic.gripped() {
            let idx_code = relic.code() as usize;
            self.stowed[idx_code] = self.stowed[idx_code].saturating_add(count.max(1));
            let (slots, len) = self.packed_slots();
            if let Some(idx) = slots[..len].iter().position(|&r| r == relic) {
                self.cursor = idx;
            }
        }
    }

    pub fn take_stowed(&mut self, relic: HeldRelic) -> bool {
        let slot = &mut self.stowed[relic.code() as usize];
        if *slot > 0 {
            *slot -= 1;
            true
        } else {
            false
        }
    }

    pub fn toggle_open(&mut self) {
        let (slots, len) = self.packed_slots();
        self.state = if self.open() { TrayState::Closed } else { TrayState::Open };
        if self.open() {
            self.cursor = self.cursor.min(TRAY_SLOTS - 1);
            if slots[self.cursor].free_palm() && len > 0 {
                self.cursor = 0;
            }
        }
    }

    pub fn navigate(&mut self, dx: i32, dy: i32) -> bool {
        if !self.open() {
            return false;
        }
        let cur = self.cursor.min(TRAY_SLOTS - 1);
        let (col, row) = (cur % 3, cur / 3);
        if dy > 0 {
            self.cursor = ((row + 1) % 5) * 3 + col;
        } else if dy < 0 {
            self.cursor = ((row + 4) % 5) * 3 + col;
        } else if dx > 0 {
            self.cursor = (cur + 1) % TRAY_SLOTS;
        } else if dx < 0 {
            self.cursor = (cur + TRAY_SLOTS - 1) % TRAY_SLOTS;
        }
        true
    }

    pub fn emit_with_burden(&self, pose: &BodyPose, burden: SatchelBurden, target: &mut dyn MeshTarget) {
        if !self.open() {
            return;
        }
        let ((slots, _), eye, tray, mold, sel) = (
            self.packed_slots(),
            pose.eye_origin(),
            pose.view_frame().shifted(0.0, 0.034, 0.60),
            BlockMold::cube(),
            self.cursor.min(TRAY_SLOTS - 1),
        );
        for (i, &item) in slots.iter().enumerate() {
            let (col, row) = (i % 3, i / 3);
            let sx = ((col as f32) - 1.0) * 0.104;
            let sy = (2.0 - (row as f32)) * 0.098;
            let selected = i == sel;
            let sz = if selected { -0.022 } else { 0.0 };
            let rim_col = if selected {
                Vec3::new(0.95, 0.78, 0.24)
            } else if item.gripped() {
                Vec3::new(0.42, 0.27, 0.16)
            } else {
                Vec3::new(0.28, 0.22, 0.17)
            };
            let qty = if item.gripped() { self.count(item) } else { 0 };
            let compartment = TalliedCompartment::new(
                ShowcasedCompartment::new(
                    WeightedCompartment::new(
                        CompartmentBackplate::new(selected, item.gripped(), rim_col),
                        SlotBurden::new(item, qty),
                    ),
                    item.figure(mold),
                ),
                eye,
                rim_col,
                qty,
            );
            compartment.emit_slot(&tray.shifted(sx, sy, sz), target);
        }
        HangingSack::new(burden).emit_slot(&tray, target);
        let chosen = slots[sel];
        let qty = if chosen.gripped() { self.count(chosen) } else { 0 };
        LegendCompartment::new(SlotBurden::new(chosen, qty)).emit_slot(&tray.shifted(0.0, -0.284, 0.0), target);
    }
}

impl CarryLoad for SatchelTray {
    fn fold_burden(&self, mut acc: SatchelBurden) -> SatchelBurden {
        let mut code = 1u8;
        while code < 25 {
            let qty = self.stowed[code as usize];
            if qty > 0 {
                acc = SlotBurden::new(HeldRelic::from_code(code), qty).fold_burden(acc);
            }
            code += 1;
        }
        acc
    }
}
