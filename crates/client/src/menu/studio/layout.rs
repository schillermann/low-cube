use glam::Vec3;
use world::MeshTarget;
use crate::attire::{BodyRegion, ComplexionSwatch};
use super::super::archetype::AvatarArchetype;
use super::super::glyph::PixelStamp;
use super::super::widget::ScreenBounds;
use super::WardrobeStudio;

/// Decorator over `WardrobeStudio` providing UI button layout bounds and 2D overlay emission.
#[derive(Clone, Copy)]
pub struct StudioLayout<'a> {
    studio: &'a WardrobeStudio,
}

impl<'a> StudioLayout<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(studio: &'a WardrobeStudio) -> Self {
        Self { studio }
    }

    pub fn rotate_left_button() -> ScreenBounds { ScreenBounds::new(-0.36, 0.72, -0.14, 0.81) }
    pub fn front_button() -> ScreenBounds { ScreenBounds::new(-0.11, 0.72, 0.11, 0.81) }
    pub fn rotate_right_button() -> ScreenBounds { ScreenBounds::new(0.14, 0.72, 0.36, 0.81) }
    pub fn male_button() -> ScreenBounds { ScreenBounds::new(-0.94, -0.74, -0.72, -0.66) }
    pub fn female_button() -> ScreenBounds { ScreenBounds::new(-0.70, -0.74, -0.48, -0.66) }
    pub fn reset_button() -> ScreenBounds { ScreenBounds::new(0.48, -0.78, 0.94, -0.68) }
    pub fn back_button() -> ScreenBounds { ScreenBounds::new(-0.18, -0.95, 0.18, -0.84) }
    pub fn turntable_viewport() -> ScreenBounds { ScreenBounds::new(-0.43, -0.80, 0.43, 0.70) }

    fn grid_slot(idx: usize, top_y: f32, row_step: f32, height: f32) -> ScreenBounds {
        let (x0, y1) = (-0.94 + ((idx % 2) as f32) * 0.24, top_y - ((idx / 2) as f32) * row_step);
        ScreenBounds::new(x0, y1 - height, x0 + 0.22, y1)
    }

    pub fn archetype_slot(idx: usize) -> ScreenBounds { Self::grid_slot(idx, 0.68, 0.10, 0.08) }
    pub fn swatch_slot(idx: usize) -> ScreenBounds { Self::grid_slot(idx, 0.26, 0.095, 0.075) }
    pub fn clothing_slot(idx: usize) -> ScreenBounds { Self::grid_slot(idx, -0.24, 0.10, 0.08) }

    pub fn region_rows(idx: usize) -> (f32, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds) {
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

    pub fn emit_overlay(self, target: &mut dyn MeshTarget) {
        let pt = self.studio.pointer_point();
        let attire = self.studio.current_attire();
        let (panel_bg, panel_rim, btn_col, gold) = (
            Vec3::new(0.08, 0.10, 0.14),
            Vec3::new(0.28, 0.34, 0.44),
            Vec3::new(0.16, 0.21, 0.29),
            Vec3::new(0.95, 0.80, 0.32),
        );
        ScreenBounds::new(-0.97, 0.84, 0.97, 0.96).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::title(gold).emit_centered("CHARACTER STUDIO - SELECTION AND CUSTOMIZATION", 0.0, 0.90, target);
        Self::rotate_left_button().emit_button("< ROTATE", pt, btn_col, false, target);
        Self::front_button().emit_button("FRONT", pt, btn_col, false, target);
        Self::rotate_right_button().emit_button("ROTATE >", pt, btn_col, false, target);

        ScreenBounds::new(-0.97, -0.81, -0.45, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("CHARACTER BODY", -0.71, 0.75, target);
        for (i, arch) in AvatarArchetype::all().iter().enumerate() {
            Self::archetype_slot(i).emit_button(
                arch.gender_title(attire.feminine()),
                pt,
                btn_col,
                self.studio.selected_archetype(*arch),
                target,
            );
        }
        PixelStamp::caption(gold).emit_centered("SKIN COLOR", -0.71, 0.32, target);
        for (i, swatch) in ComplexionSwatch::all().iter().enumerate() {
            Self::swatch_slot(i).emit_swatch(
                swatch.label(),
                pt,
                swatch.skin(),
                attire.selected_complexion(*swatch),
                target,
            );
        }
        PixelStamp::caption(gold).emit_centered("CLOTHING", -0.71, -0.18, target);
        let clothing_labels = ["TUNIC", "SLEEVES", "BELT", "BOOTS", "HEADGEAR", "DYE COLOR"];
        for (i, lbl) in clothing_labels.iter().enumerate() {
            Self::clothing_slot(i).emit_button(lbl, pt, btn_col, false, target);
        }
        PixelStamp::caption(gold).emit_centered("GENDER", -0.71, -0.60, target);
        Self::male_button().emit_button("MALE", pt, btn_col, !attire.feminine(), target);
        Self::female_button().emit_button("FEMALE", pt, btn_col, attire.feminine(), target);

        ScreenBounds::new(0.45, -0.81, 0.97, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("BODY PARTS", 0.71, 0.75, target);
        for (idx, region) in BodyRegion::all().iter().enumerate() {
            let (top, s_dec, s_bar, s_inc, g_dec, g_bar, g_inc) = Self::region_rows(idx);
            let prop = attire.region_proportion(*region);
            PixelStamp::caption(Vec3::new(0.85, 0.90, 0.96)).emit_centered(region.title(), 0.71, top - 0.01, target);
            s_dec.emit_button("-", pt, btn_col, false, target);
            s_bar.emit_gauge("SIZE", prop.size_ratio(), Vec3::new(0.24, 0.56, 0.78), target);
            s_inc.emit_button("+", pt, btn_col, false, target);
            g_dec.emit_button("-", pt, btn_col, false, target);
            g_bar.emit_gauge("THICKNESS", prop.girth_ratio(), Vec3::new(0.68, 0.44, 0.22), target);
            g_inc.emit_button("+", pt, btn_col, false, target);
        }
        Self::reset_button().emit_button("RESET PROPORTIONS", pt, btn_col, false, target);
        Self::back_button().emit_button("BACK", pt, btn_col, false, target);
    }
}
