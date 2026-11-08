use glam::Vec3;
use world::{BlockMold, MeshTarget};
use super::pose::BodyPose;
use super::relic::HeldRelic;

/// Encapsulates stowed satchel items, WASD inspection cursor, and waist-grid rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SatchelTray {
    stowed: [u32; 21],
    active_side: f32,
    cursor: usize,
    open: bool,
}

impl SatchelTray {
    pub const fn new(stowed: [u32; 21], active_side: f32, cursor: usize, open: bool) -> Self {
        Self {
            stowed,
            active_side,
            cursor,
            open,
        }
    }

    pub const fn open(&self) -> bool {
        self.open
    }

    pub const fn active_side(&self) -> f32 {
        self.active_side
    }

    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub const fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn focus_side(&mut self, side: f32) {
        self.active_side = if side > 0.0 { 1.0 } else { -1.0 };
    }

    pub fn open_for_side(&mut self, side: f32, len: usize) {
        self.open = true;
        self.cursor = self.cursor.min(len.saturating_sub(1));
        self.focus_side(side);
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
        ]
        .iter()
        .any(|&relic| self.count(relic) > 0)
    }

    pub fn packed_slots(&self) -> ([HeldRelic; 12], usize) {
        let (mut slots, mut len) = ([HeldRelic::EmptyPalm; 12], 0usize);
        let materials = [
            HeldRelic::CabinSeat,
            HeldRelic::PedalCluster,
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
            if self.count(mat) > 0 && len < 12 {
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
                if len < 12 {
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
        let (_, len) = self.packed_slots();
        self.open = !self.open;
        if self.open {
            self.cursor = self.cursor.min(len.saturating_sub(1));
        }
    }

    pub fn navigate(&mut self, dx: i32, dy: i32) -> bool {
        if !self.open {
            return false;
        }
        let (_, len) = self.packed_slots();
        if len > 0 {
            let step = if dy > 0 {
                3
            } else if dy < 0 {
                len.saturating_sub(3).max(1)
            } else if dx > 0 {
                1
            } else {
                len - 1
            };
            self.cursor = (self.cursor + step) % len;
        }
        true
    }

    pub fn emit_on_waist(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        if !self.open {
            return;
        }
        let ((slots, len), tray, mold) = (
            self.packed_slots(),
            pose.view_frame().shifted(0.0, -0.02, 0.40),
            BlockMold::cube(),
        );
        let (cols, rows, sel) = (
            len.clamp(1, 3),
            len.max(1).div_ceil(3),
            self.cursor.min(len.saturating_sub(1)),
        );
        if len == 0 {
            HeldRelic::EmptyPalm.figure(mold).emit_in_slot(false, Vec3::new(0.42, 0.27, 0.16), &tray, target);
        }
        for (i, &item) in slots[..len].iter().enumerate() {
            let (col, row) = (i % 3, i / 3);
            let r_cols = if row + 1 == rows && len % 3 != 0 { len % 3 } else { cols };
            let sx = ((col as f32) - (r_cols as f32 - 1.0) * 0.5) * 0.115;
            let sy = ((rows as f32 - 1.0) * 0.5 - (row as f32)) * 0.115;
            let selected = i == sel;
            let sz = if selected { -0.022 } else { 0.0 };
            let rim_col = if selected {
                Vec3::new(0.95, 0.78, 0.24)
            } else {
                Vec3::new(0.42, 0.27, 0.16)
            };
            item.figure(mold).emit_in_slot(selected, rim_col, &tray.shifted(sx, sy, sz), target);
        }
        let legend = if len > 0 { slots[sel] } else { HeldRelic::EmptyPalm };
        super::relic::RelicFigure::emit_legend(
            legend,
            &tray.shifted(0.0, -((rows as f32 - 1.0) * 0.5) * 0.115 - 0.090, 0.0),
            target,
        );
    }
}
