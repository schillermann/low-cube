use glam::Vec3;
use world::MeshTarget;
use crate::menu::glyph::PixelStamp;
use crate::menu::view::{MenuView, TitledCard};
use crate::menu::widget::ScreenPoint;
use crate::world_vault::{WorldSlug, WorldSpec};
use super::layout::WorldLayout;

/// Body view for the `MY WORLDS` selection, launch, and deletion panel (`4` fields).
pub struct WorldRosterBody<'a> {
    entries: &'a [WorldSpec],
    selection: (usize, usize),
    active_slug: &'a WorldSlug,
    pointer: ScreenPoint,
}

impl<'a> WorldRosterBody<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        entries: &'a [WorldSpec],
        selection: (usize, usize),
        active_slug: &'a WorldSlug,
        pointer: ScreenPoint,
    ) -> Self {
        Self { entries, selection, active_slug, pointer }
    }

    pub const fn card(self) -> TitledCard<'static, Self> {
        let layout = WorldLayout::new();
        TitledCard::new(layout.card(), "MY WORLDS", 0.54, self)
    }
}

impl MenuView for WorldRosterBody<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let layout = WorldLayout::new();
        let (selected_idx, page) = self.selection;
        let total = self.entries.len();
        let page_count = total.max(1).div_ceil(WorldLayout::PAGE_SIZE);
        let safe_page = page.min(page_count - 1);
        let start = safe_page * WorldLayout::PAGE_SIZE;

        if total == 0 {
            PixelStamp::caption(Vec3::new(0.72, 0.78, 0.88)).emit_centered(
                "NO SAVED WORLDS YET - CLICK NEW WORLD BELOW",
                0.0,
                0.16,
                target,
            );
        } else {
            for slot in 0..WorldLayout::PAGE_SIZE {
                let idx = start + slot;
                let Some(spec) = self.entries.get(idx) else {
                    break;
                };
                let chosen = idx == selected_idx;
                let live = spec.same_slug(self.active_slug);
                let row_col = if chosen {
                    Vec3::new(0.22, 0.46, 0.34)
                } else if live {
                    Vec3::new(0.18, 0.34, 0.42)
                } else {
                    Vec3::new(0.16, 0.20, 0.30)
                };
                let label = spec.roster_button_label(live);
                layout.slot_row(slot).emit_button(&label, self.pointer, row_col, chosen, target);
            }
            if page_count > 1 {
                layout.prev_page_button().emit_button("< PREV", self.pointer, Vec3::new(0.20, 0.28, 0.44), false, target);
                PixelStamp::caption(Vec3::new(0.78, 0.84, 0.94)).emit_centered(
                    &format!("PAGE {} / {}", safe_page + 1, page_count),
                    0.0,
                    -0.26,
                    target,
                );
                layout.next_page_button().emit_button("NEXT >", self.pointer, Vec3::new(0.20, 0.28, 0.44), false, target);
            }
        }

        let play_ready = total > 0 && selected_idx < total;
        let play_col = if play_ready { Vec3::new(0.18, 0.46, 0.26) } else { Vec3::new(0.18, 0.20, 0.24) };
        let del_col = if play_ready { Vec3::new(0.50, 0.18, 0.20) } else { Vec3::new(0.22, 0.18, 0.20) };
        layout.play_button().emit_button("PLAY WORLD", self.pointer, play_col, play_ready, target);
        layout.new_world_button().emit_button("NEW WORLD", self.pointer, Vec3::new(0.20, 0.38, 0.56), false, target);
        layout.delete_button().emit_button("DELETE WORLD", self.pointer, del_col, false, target);
        layout.roster_back_button().emit_button("MAIN MENU", self.pointer, Vec3::new(0.26, 0.26, 0.38), false, target);
    }
}
