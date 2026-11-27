use glam::Vec3;
use world::MeshTarget;
use crate::attire::{BodyRegion, ComplexionSwatch, EvaAttire};
use super::super::archetype::AvatarArchetype;
use super::super::glyph::PixelStamp;
use super::super::view::MenuView;
use super::super::widget::{ScreenBounds, ScreenPoint};

/// Stateless screen bounds layout for the 3D Character Studio.
pub struct StudioBounds;

impl StudioBounds {
    /// Primary constructor initializing the character studio button layout.
    pub const fn new() -> Self {
        Self
    }

    pub fn rotate_left_button(&self) -> ScreenBounds { ScreenBounds::new(-0.42, 0.72, -0.22, 0.81) }
    pub fn front_button(&self) -> ScreenBounds { ScreenBounds::new(-0.20, 0.72, -0.02, 0.81) }
    pub fn rotate_right_button(&self) -> ScreenBounds { ScreenBounds::new(0.00, 0.72, 0.20, 0.81) }
    pub fn animation_button(&self) -> ScreenBounds { ScreenBounds::new(0.22, 0.72, 0.44, 0.81) }
    pub fn male_button(&self) -> ScreenBounds { ScreenBounds::new(-0.94, -0.74, -0.72, -0.66) }
    pub fn female_button(&self) -> ScreenBounds { ScreenBounds::new(-0.70, -0.74, -0.48, -0.66) }
    pub fn reset_button(&self) -> ScreenBounds { ScreenBounds::new(0.48, -0.78, 0.94, -0.68) }
    pub fn back_button(&self) -> ScreenBounds { ScreenBounds::new(-0.18, -0.95, 0.18, -0.84) }
    pub fn turntable_viewport(&self) -> ScreenBounds { ScreenBounds::new(-0.43, -0.80, 0.43, 0.70) }

    fn grid_slot(&self, idx: usize, top_y: f32, row_step: f32, height: f32) -> ScreenBounds {
        let (x0, y1) = (-0.94 + ((idx % 2) as f32) * 0.24, top_y - ((idx / 2) as f32) * row_step);
        ScreenBounds::new(x0, y1 - height, x0 + 0.22, y1)
    }

    pub fn archetype_slot(&self, idx: usize) -> ScreenBounds { self.grid_slot(idx, 0.68, 0.10, 0.08) }
    pub fn swatch_slot(&self, idx: usize) -> ScreenBounds { self.grid_slot(idx, 0.26, 0.095, 0.075) }
    pub fn clothing_slot(&self, idx: usize) -> ScreenBounds { self.grid_slot(idx, -0.24, 0.10, 0.08) }

    pub fn region_rows(&self, idx: usize) -> (f32, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds) {
        let top = 0.68 - (idx as f32) * 0.33;
        let (r1, r2) = ((top - 0.13, top - 0.05), (top - 0.24, top - 0.16));
        (
            top,
            ScreenBounds::new(0.48, r1.0, 0.56, r1.1),
            ScreenBounds::new(0.58, r1.0, 0.84, r1.1),
            ScreenBounds::new(0.86, r1.0, 0.94, r1.1),
            ScreenBounds::new(0.48, r2.0, 0.56, r2.1),
            ScreenBounds::new(0.58, r2.0, 0.84, r2.1),
            ScreenBounds::new(0.86, r2.0, 0.94, r2.1),
        )
    }
}

/// 2D overlay view for the 3D Character Studio implementing `MenuView` without getters.
#[derive(Clone, Copy)]
pub struct StudioLayout<'a> {
    attire: &'a EvaAttire,
    archetype: AvatarArchetype,
    animation_title: &'static str,
    pointer: ScreenPoint,
}

impl<'a> StudioLayout<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        attire: &'a EvaAttire,
        archetype: AvatarArchetype,
        animation_title: &'static str,
        pointer: ScreenPoint,
    ) -> Self {
        Self { attire, archetype, animation_title, pointer }
    }
}

impl MenuView for StudioLayout<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let bounds = StudioBounds::new();
        let pt = self.pointer;
        let attire = self.attire;
        let (panel_bg, panel_rim, btn_col, gold) = (
            Vec3::new(0.08, 0.10, 0.14),
            Vec3::new(0.28, 0.34, 0.44),
            Vec3::new(0.16, 0.21, 0.29),
            Vec3::new(0.95, 0.80, 0.32),
        );
        ScreenBounds::new(-0.97, 0.84, 0.97, 0.96).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::title(gold).emit_centered("CHARACTER STUDIO - SELECTION AND CUSTOMIZATION", 0.0, 0.90, target);
        bounds.rotate_left_button().emit_button("< ROTATE", pt, btn_col, false, target);
        bounds.front_button().emit_button("FRONT", pt, btn_col, false, target);
        bounds.rotate_right_button().emit_button("ROTATE >", pt, btn_col, false, target);
        bounds.animation_button().emit_button(self.animation_title, pt, btn_col, false, target);

        ScreenBounds::new(-0.97, -0.81, -0.45, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("CHARACTER BODY", -0.71, 0.75, target);
        for (i, arch) in AvatarArchetype::all().iter().enumerate() {
            bounds.archetype_slot(i).emit_button(
                arch.gender_title(attire.feminine()),
                pt,
                btn_col,
                self.archetype == *arch,
                target,
            );
        }
        PixelStamp::caption(gold).emit_centered("SKIN COLOR", -0.71, 0.32, target);
        for (i, swatch) in ComplexionSwatch::all().iter().enumerate() {
            bounds.swatch_slot(i).emit_swatch(
                swatch.label(),
                pt,
                swatch.skin(),
                attire.selected_complexion(*swatch),
                target,
            );
        }
        PixelStamp::caption(gold).emit_centered("PRESSURE SUIT", -0.71, -0.18, target);
        let clothing_labels = ["SUIT SHELL", "GAUNTLETS", "PLSS PACK", "MAG BOOTS", "EVA HELM", "SUIT TINT"];
        for (i, lbl) in clothing_labels.iter().enumerate() {
            bounds.clothing_slot(i).emit_button(lbl, pt, btn_col, false, target);
        }
        PixelStamp::caption(gold).emit_centered("GENDER", -0.71, -0.60, target);
        bounds.male_button().emit_button("MALE", pt, btn_col, !attire.feminine(), target);
        bounds.female_button().emit_button("FEMALE", pt, btn_col, attire.feminine(), target);

        ScreenBounds::new(0.45, -0.81, 0.97, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("BODY PARTS", 0.71, 0.75, target);
        for (idx, region) in BodyRegion::all().iter().enumerate() {
            let (top, s_dec, s_bar, s_inc, g_dec, g_bar, g_inc) = bounds.region_rows(idx);
            let prop = attire.region_proportion(*region);
            PixelStamp::caption(Vec3::new(0.85, 0.90, 0.96)).emit_centered(region.title(), 0.71, top - 0.01, target);
            s_dec.emit_button("-", pt, btn_col, false, target);
            s_bar.emit_gauge("SIZE", prop.size_ratio(), Vec3::new(0.24, 0.56, 0.78), target);
            s_inc.emit_button("+", pt, btn_col, false, target);
            g_dec.emit_button("-", pt, btn_col, false, target);
            g_bar.emit_gauge("THICKNESS", prop.girth_ratio(), Vec3::new(0.68, 0.44, 0.22), target);
            g_inc.emit_button("+", pt, btn_col, false, target);
        }
        bounds.reset_button().emit_button("RESET PROPORTIONS", pt, btn_col, false, target);
        bounds.back_button().emit_button("BACK", pt, btn_col, false, target);
    }
}
