use glam::Vec3;
use world::MeshTarget;
use crate::menu::glyph::PixelStamp;
use crate::menu::view::{MenuView, TitledCard};
use crate::menu::widget::ScreenPoint;
use super::draft::WorldDraft;
use super::layout::WorldLayout;

/// Body view for the `CREATE WORLD` configurator screen (`2` fields).
pub struct CreateWorldBody<'a> {
    draft: &'a WorldDraft,
    pointer: ScreenPoint,
}

impl<'a> CreateWorldBody<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(draft: &'a WorldDraft, pointer: ScreenPoint) -> Self {
        Self { draft, pointer }
    }

    pub const fn card(self) -> TitledCard<'static, Self> {
        let layout = WorldLayout::new();
        TitledCard::new(layout.card(), "CREATE WORLD", 0.54, self)
    }
}

impl MenuView for CreateWorldBody<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let layout = WorldLayout::new();
        let label_col = Vec3::new(0.76, 0.84, 0.96);

        PixelStamp::caption(label_col).emit_centered("WORLD NAME", -0.13, 0.42, target);
        let name_col = if self.draft.name_focused() {
            Vec3::new(0.18, 0.34, 0.50)
        } else {
            Vec3::new(0.14, 0.20, 0.30)
        };
        layout.name_field().emit_button(
            &self.draft.name_caption(),
            self.pointer,
            name_col,
            self.draft.name_focused(),
            target,
        );
        layout.clear_name_button().emit_button("CLEAR", self.pointer, Vec3::new(0.32, 0.24, 0.36), false, target);

        PixelStamp::caption(label_col).emit_centered("WORLD SIZE", 0.0, 0.16, target);
        layout.size_prev_button().emit_button("<", self.pointer, Vec3::new(0.22, 0.32, 0.52), false, target);
        layout.size_readout().emit_button(&self.draft.size_banner(), self.pointer, Vec3::new(0.16, 0.28, 0.40), true, target);
        layout.size_next_button().emit_button(">", self.pointer, Vec3::new(0.22, 0.32, 0.52), false, target);

        PixelStamp::caption(label_col).emit_centered("WORLD SEED", -0.17, -0.10, target);
        let seed_col = if self.draft.seed_focused() {
            Vec3::new(0.18, 0.34, 0.50)
        } else {
            Vec3::new(0.14, 0.20, 0.30)
        };
        layout.seed_field().emit_button(
            &self.draft.seed_caption(),
            self.pointer,
            seed_col,
            self.draft.seed_focused(),
            target,
        );
        layout.random_seed_button().emit_button("RANDOM", self.pointer, Vec3::new(0.24, 0.38, 0.48), false, target);

        layout.create_button().emit_button(
            "CREATE WORLD",
            self.pointer,
            Vec3::new(0.18, 0.48, 0.28),
            true,
            target,
        );
        layout.create_back_button().emit_button("BACK", self.pointer, Vec3::new(0.26, 0.26, 0.38), false, target);
    }
}
