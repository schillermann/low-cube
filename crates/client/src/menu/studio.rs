use glam::{Mat4, Vec3};
use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::attire::{AttireProfile, BodyRegion, ComplexionSwatch, FileAttire, YouthAttire, YouthPhysique};
use super::archetype::AvatarArchetype;
use super::glyph::PixelStamp;
use super::stage::TurntableStage;
use super::widget::{ScreenBounds, ScreenPoint};

/// Outcome returned from interactive studio actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioDirective {
    StayInStudio,
    ReturnToPauseMenu,
}

/// Interactive 3D character selection, turntable inspection, skin color, and body-part proportion studio.
#[derive(Clone, Copy, Debug)]
pub struct WardrobeStudio {
    attire: YouthAttire,
    archetype: AvatarArchetype,
    stage: TurntableStage,
    pointer: ScreenPoint,
    dragging: bool,
}

impl WardrobeStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(attire: YouthAttire, archetype: AvatarArchetype, stage: TurntableStage, pointer: ScreenPoint, dragging: bool) -> Self {
        Self { attire, archetype, stage, pointer, dragging }
    }

    pub fn from_profile() -> Self {
        Self::new(FileAttire::profile().load_or_squire(), AvatarArchetype::Knight, TurntableStage::front_showcase(), ScreenPoint::center(), false)
    }

    pub fn step(&mut self, dt: f32) { self.stage = self.stage.step(dt); }
    pub fn current_attire(&self) -> YouthAttire { self.attire }
    pub fn stage_projection(&self, aspect: f32) -> Mat4 { self.stage.view_projection(aspect) }
    pub fn emit_stage(&self, target: &mut dyn MeshTarget) { self.stage.emit_stage(&self.attire, target); }
    pub fn release_pointer(&mut self) { self.dragging = false; }

    pub fn move_pointer(&mut self, next: ScreenPoint) {
        if self.dragging { self.stage = self.stage.rotated(next.horizontal_delta(self.pointer) * 3.8); }
        self.pointer = next;
    }

    pub fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::ArrowLeft | KeyCode::KeyA => { self.stage = self.stage.rotated(-0.22); true }
            KeyCode::ArrowRight | KeyCode::KeyD => { self.stage = self.stage.rotated(0.22); true }
            KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4 | KeyCode::Digit5 | KeyCode::Digit6 | KeyCode::KeyF => {
                if let Some(updated) = self.attire.cycled_by_key(code) { self.attire = updated; true } else { false }
            }
            _ => false,
        }
    }

    fn update_attire(&mut self, updated: YouthAttire) {
        self.attire = updated;
        FileAttire::profile().save(&self.attire);
    }

    fn region_rows(idx: usize) -> (f32, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds, ScreenBounds) {
        let top = 0.68 - (idx as f32) * 0.33;
        let (r1, r2) = ((top - 0.13, top - 0.05), (top - 0.24, top - 0.16));
        (
            top,
            ScreenBounds::new(0.48, r1.0, 0.56, r1.1), ScreenBounds::new(0.58, r1.0, 0.84, r1.1), ScreenBounds::new(0.86, r1.0, 0.94, r1.1),
            ScreenBounds::new(0.48, r2.0, 0.56, r2.1), ScreenBounds::new(0.58, r2.0, 0.84, r2.1), ScreenBounds::new(0.86, r2.0, 0.94, r2.1),
        )
    }

    pub fn press_pointer(&mut self) -> StudioDirective {
        let pt = self.pointer;
        if ScreenBounds::new(-0.36, 0.72, -0.14, 0.81).contains(pt) { self.stage = self.stage.rotated(-0.35); return StudioDirective::StayInStudio; }
        if ScreenBounds::new(-0.11, 0.72, 0.11, 0.81).contains(pt) { self.stage = self.stage.reset_angle(); return StudioDirective::StayInStudio; }
        if ScreenBounds::new(0.14, 0.72, 0.36, 0.81).contains(pt) { self.stage = self.stage.rotated(0.35); return StudioDirective::StayInStudio; }

        for (i, arch) in AvatarArchetype::all().iter().enumerate() {
            let (x0, y1) = (-0.94 + ((i % 2) as f32) * 0.24, 0.68 - ((i / 2) as f32) * 0.10);
            if ScreenBounds::new(x0, y1 - 0.08, x0 + 0.22, y1).contains(pt) {
                self.archetype = *arch;
                self.update_attire(arch.attire_for(self.attire.feminine()));
                return StudioDirective::StayInStudio;
            }
        }
        for (i, swatch) in ComplexionSwatch::all().iter().enumerate() {
            let (x0, y1) = (-0.94 + ((i % 2) as f32) * 0.24, 0.26 - ((i / 2) as f32) * 0.095);
            if ScreenBounds::new(x0, y1 - 0.075, x0 + 0.22, y1).contains(pt) {
                self.update_attire(self.attire.with_complexion(*swatch));
                return StudioDirective::StayInStudio;
            }
        }
        let clothing_keys = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6];
        for (i, key) in clothing_keys.iter().enumerate() {
            let (x0, y1) = (-0.94 + ((i % 2) as f32) * 0.24, -0.24 - ((i / 2) as f32) * 0.10);
            if ScreenBounds::new(x0, y1 - 0.08, x0 + 0.22, y1).contains(pt) {
                if let Some(next) = self.attire.with_wardrobe_key(*key) { self.update_attire(next); }
                return StudioDirective::StayInStudio;
            }
        }
        if ScreenBounds::new(-0.94, -0.74, -0.72, -0.66).contains(pt) { self.update_attire(self.archetype.attire_for(false)); return StudioDirective::StayInStudio; }
        if ScreenBounds::new(-0.70, -0.74, -0.48, -0.66).contains(pt) { self.update_attire(self.archetype.attire_for(true)); return StudioDirective::StayInStudio; }
        for (idx, region) in BodyRegion::all().iter().enumerate() {
            let (_, s_dec, _, s_inc, g_dec, _, g_inc) = Self::region_rows(idx);
            if s_dec.contains(pt) { self.update_attire(self.attire.with_adjusted_region(*region, -1, 0)); return StudioDirective::StayInStudio; }
            if s_inc.contains(pt) { self.update_attire(self.attire.with_adjusted_region(*region, 1, 0)); return StudioDirective::StayInStudio; }
            if g_dec.contains(pt) { self.update_attire(self.attire.with_adjusted_region(*region, 0, -1)); return StudioDirective::StayInStudio; }
            if g_inc.contains(pt) { self.update_attire(self.attire.with_adjusted_region(*region, 0, 1)); return StudioDirective::StayInStudio; }
        }
        if ScreenBounds::new(0.48, -0.78, 0.94, -0.68).contains(pt) {
            self.update_attire(self.attire.with_physique(YouthPhysique::SlenderSquire.as_feminine(self.attire.feminine())));
            return StudioDirective::StayInStudio;
        }
        if ScreenBounds::new(-0.18, -0.95, 0.18, -0.84).contains(pt) {
            return StudioDirective::ReturnToPauseMenu;
        }
        if ScreenBounds::new(-0.43, -0.80, 0.43, 0.70).contains(pt) { self.dragging = true; }
        StudioDirective::StayInStudio
    }

    pub fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        let (panel_bg, panel_rim, btn_col, gold) = (Vec3::new(0.08, 0.10, 0.14), Vec3::new(0.28, 0.34, 0.44), Vec3::new(0.16, 0.21, 0.29), Vec3::new(0.95, 0.80, 0.32));
        ScreenBounds::new(-0.97, 0.84, 0.97, 0.96).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::title(gold).emit_centered("CHARACTER STUDIO - SELECTION AND CUSTOMIZATION", 0.0, 0.90, target);
        ScreenBounds::new(-0.36, 0.72, -0.14, 0.81).emit_button("< ROTATE", self.pointer, btn_col, false, target);
        ScreenBounds::new(-0.11, 0.72, 0.11, 0.81).emit_button("FRONT", self.pointer, btn_col, false, target);
        ScreenBounds::new(0.14, 0.72, 0.36, 0.81).emit_button("ROTATE >", self.pointer, btn_col, false, target);

        ScreenBounds::new(-0.97, -0.81, -0.45, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("CHARACTER BODY", -0.71, 0.75, target);
        for (i, arch) in AvatarArchetype::all().iter().enumerate() {
            let (x0, y1) = (-0.94 + ((i % 2) as f32) * 0.24, 0.68 - ((i / 2) as f32) * 0.10);
            ScreenBounds::new(x0, y1 - 0.08, x0 + 0.22, y1).emit_button(arch.gender_title(self.attire.feminine()), self.pointer, btn_col, self.archetype == *arch, target);
        }
        PixelStamp::caption(gold).emit_centered("SKIN COLOR", -0.71, 0.32, target);
        for (i, swatch) in ComplexionSwatch::all().iter().enumerate() {
            let (x0, y1) = (-0.94 + ((i % 2) as f32) * 0.24, 0.26 - ((i / 2) as f32) * 0.095);
            ScreenBounds::new(x0, y1 - 0.075, x0 + 0.22, y1).emit_swatch(swatch.label(), self.pointer, swatch.skin(), self.attire.selected_complexion(*swatch), target);
        }
        PixelStamp::caption(gold).emit_centered("CLOTHING", -0.71, -0.18, target);
        let clothing_labels = ["TUNIC", "SLEEVES", "BELT", "BOOTS", "HEADGEAR", "DYE COLOR"];
        for (i, lbl) in clothing_labels.iter().enumerate() {
            let (x0, y1) = (-0.94 + ((i % 2) as f32) * 0.24, -0.24 - ((i / 2) as f32) * 0.10);
            ScreenBounds::new(x0, y1 - 0.08, x0 + 0.22, y1).emit_button(lbl, self.pointer, btn_col, false, target);
        }
        PixelStamp::caption(gold).emit_centered("GENDER", -0.71, -0.60, target);
        ScreenBounds::new(-0.94, -0.74, -0.72, -0.66).emit_button("MALE", self.pointer, btn_col, !self.attire.feminine(), target);
        ScreenBounds::new(-0.70, -0.74, -0.48, -0.66).emit_button("FEMALE", self.pointer, btn_col, self.attire.feminine(), target);

        ScreenBounds::new(0.45, -0.81, 0.97, 0.81).emit_framed(panel_bg, panel_rim, 0.006, target);
        PixelStamp::caption(gold).emit_centered("BODY PARTS", 0.71, 0.75, target);
        for (idx, region) in BodyRegion::all().iter().enumerate() {
            let (top, s_dec, s_bar, s_inc, g_dec, g_bar, g_inc) = Self::region_rows(idx);
            let prop = self.attire.region_proportion(*region);
            PixelStamp::caption(Vec3::new(0.85, 0.90, 0.96)).emit_centered(region.title(), 0.71, top - 0.01, target);
            s_dec.emit_button("-", self.pointer, btn_col, false, target);
            s_bar.emit_gauge("SIZE", prop.size_ratio(), Vec3::new(0.24, 0.56, 0.78), target);
            s_inc.emit_button("+", self.pointer, btn_col, false, target);
            g_dec.emit_button("-", self.pointer, btn_col, false, target);
            g_bar.emit_gauge("THICKNESS", prop.girth_ratio(), Vec3::new(0.68, 0.44, 0.22), target);
            g_inc.emit_button("+", self.pointer, btn_col, false, target);
        }
        ScreenBounds::new(0.48, -0.78, 0.94, -0.68).emit_button("RESET PROPORTIONS", self.pointer, btn_col, false, target);

        ScreenBounds::new(-0.18, -0.95, 0.18, -0.84).emit_button("BACK", self.pointer, btn_col, false, target);
    }
}
