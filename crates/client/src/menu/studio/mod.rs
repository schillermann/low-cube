mod layout;
mod saved;

use glam::Mat4;
use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::attire::{
    AttireProfile, BodyRegion, ComplexionSwatch, HeldRelic, YouthAttire, YouthPhysique,
};
use crate::avatar::Avatar;
use super::archetype::AvatarArchetype;
use super::stage::TurntableStage;
use super::view::MenuView;
use super::widget::ScreenPoint;
use layout::{StudioBounds, StudioLayout};

pub use saved::SavedWardrobe;

/// Outcome returned from interactive studio actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioDirective {
    StayInStudio,
    ReturnToPauseMenu,
}

/// Narrow Tell-Don't-Ask recipient for updated avatar attire.
pub trait AttireSink {
    fn apply_attire(&mut self, attire: YouthAttire);
}

impl AttireSink for &mut dyn Avatar {
    fn apply_attire(&mut self, attire: YouthAttire) {
        Avatar::apply_attire(*self, attire);
    }
}

/// Focused interface for 3D character wardrobe studios and persistence decorators.
pub trait WardrobePanel {
    fn step(&mut self, dt: f32);
    fn stage_projection(&self, aspect: f32) -> Mat4;
    fn emit_stage(&self, target: &mut dyn MeshTarget);
    fn emit_overlay(&self, target: &mut dyn MeshTarget);
    fn release_grip(&mut self);
    fn move_pointer(&mut self, next: ScreenPoint);
    fn handle_key(&mut self, code: KeyCode, sink: &mut dyn AttireSink) -> bool;
    fn press_pointer(&mut self, sink: &mut dyn AttireSink) -> StudioDirective;
}

impl MenuView for &dyn WardrobePanel {
    fn emit(&self, target: &mut dyn MeshTarget) {
        self.emit_overlay(target);
    }
}

#[derive(Clone, Copy, Debug)]
struct StageGrip {
    point: ScreenPoint,
    dragging: bool,
}

impl StageGrip {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(point: ScreenPoint, dragging: bool) -> Self {
        Self { point, dragging }
    }
}

/// Pure in-memory 3D character selection, turntable inspection, skin color, and body-part proportion studio.
#[derive(Clone, Copy, Debug)]
pub struct WardrobeStudio {
    attire: YouthAttire,
    archetype: AvatarArchetype,
    stage: TurntableStage,
    grip: StageGrip,
}

impl WardrobeStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        attire: YouthAttire,
        archetype: AvatarArchetype,
        stage: TurntableStage,
        point: ScreenPoint,
        dragging: bool,
    ) -> Self {
        Self {
            attire,
            archetype,
            stage,
            grip: StageGrip::new(point, dragging),
        }
    }

    #[cfg(test)]
    pub fn seeded() -> Self {
        Self::new(
            YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm),
            AvatarArchetype::Knight,
            TurntableStage::front_showcase(),
            ScreenPoint::center(),
            false,
        )
    }

    pub fn from_profile(profile: &dyn AttireProfile) -> Self {
        let loaded = profile.load_or_squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm);
        Self::new(loaded, AvatarArchetype::Knight, TurntableStage::front_showcase(), ScreenPoint::center(), false)
    }

    fn update_attire(&mut self, updated: YouthAttire, sink: &mut dyn AttireSink) {
        self.attire = updated;
        sink.apply_attire(self.attire);
    }
}

impl WardrobePanel for WardrobeStudio {
    fn step(&mut self, dt: f32) {
        self.stage = self.stage.step(dt);
    }

    fn stage_projection(&self, aspect: f32) -> Mat4 {
        self.stage.view_projection(aspect)
    }

    fn emit_stage(&self, target: &mut dyn MeshTarget) {
        self.stage.emit_stage(&self.attire, target);
    }

    fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        StudioLayout::new(
            &self.attire,
            self.archetype,
            self.stage.active_animation().title(),
            self.grip.point,
        )
        .emit(target);
    }

    fn release_grip(&mut self) {
        self.grip.dragging = false;
    }

    fn move_pointer(&mut self, next: ScreenPoint) {
        if self.grip.dragging {
            self.stage = self.stage.rotated(next.horizontal_delta(self.grip.point) * 3.8);
        }
        self.grip.point = next;
    }

    fn handle_key(&mut self, code: KeyCode, sink: &mut dyn AttireSink) -> bool {
        match code {
            KeyCode::ArrowLeft => {
                self.stage = self.stage.rotated(-0.22);
                true
            }
            KeyCode::ArrowRight => {
                self.stage = self.stage.rotated(0.22);
                true
            }
            KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4 | KeyCode::Digit5 | KeyCode::Digit6 | KeyCode::KeyF => {
                if let Some(updated) = self.attire.cycled_by_key(code) {
                    self.update_attire(updated, sink);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn press_pointer(&mut self, sink: &mut dyn AttireSink) -> StudioDirective {
        let bounds = StudioBounds::new();
        let pt = self.grip.point;
        if bounds.rotate_left_button().contains(pt) {
            self.stage = self.stage.rotated(-0.35);
            return StudioDirective::StayInStudio;
        }
        if bounds.front_button().contains(pt) {
            self.stage = self.stage.reset_angle();
            return StudioDirective::StayInStudio;
        }
        if bounds.rotate_right_button().contains(pt) {
            self.stage = self.stage.rotated(0.35);
            return StudioDirective::StayInStudio;
        }
        if bounds.animation_button().contains(pt) {
            self.stage = self.stage.next_animation();
            return StudioDirective::StayInStudio;
        }

        let (held_r, held_l) = self.attire.held_pair();
        for (i, arch) in AvatarArchetype::all().iter().enumerate() {
            if bounds.archetype_slot(i).contains(pt) {
                self.archetype = *arch;
                self.update_attire(arch.attire_for(self.attire.feminine()).with_relics(held_r, held_l), sink);
                return StudioDirective::StayInStudio;
            }
        }
        for (i, swatch) in ComplexionSwatch::all().iter().enumerate() {
            if bounds.swatch_slot(i).contains(pt) {
                self.update_attire(self.attire.with_complexion(*swatch), sink);
                return StudioDirective::StayInStudio;
            }
        }
        let clothing_keys = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6];
        for (i, key) in clothing_keys.iter().enumerate() {
            if bounds.clothing_slot(i).contains(pt) {
                if let Some(next) = self.attire.with_wardrobe_key(*key) {
                    self.update_attire(next, sink);
                }
                return StudioDirective::StayInStudio;
            }
        }
        if bounds.male_button().contains(pt) {
            self.update_attire(self.archetype.attire_for(false).with_relics(held_r, held_l), sink);
            return StudioDirective::StayInStudio;
        }
        if bounds.female_button().contains(pt) {
            self.update_attire(self.archetype.attire_for(true).with_relics(held_r, held_l), sink);
            return StudioDirective::StayInStudio;
        }
        for (idx, region) in BodyRegion::all().iter().enumerate() {
            let (_, s_dec, _, s_inc, g_dec, _, g_inc) = bounds.region_rows(idx);
            if s_dec.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, -1, 0), sink);
                return StudioDirective::StayInStudio;
            }
            if s_inc.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, 1, 0), sink);
                return StudioDirective::StayInStudio;
            }
            if g_dec.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, 0, -1), sink);
                return StudioDirective::StayInStudio;
            }
            if g_inc.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, 0, 1), sink);
                return StudioDirective::StayInStudio;
            }
        }
        if bounds.reset_button().contains(pt) {
            self.update_attire(
                self.attire.with_physique(YouthPhysique::SlenderSquire.as_feminine(self.attire.feminine())),
                sink,
            );
            return StudioDirective::StayInStudio;
        }
        if bounds.back_button().contains(pt) {
            return StudioDirective::ReturnToPauseMenu;
        }
        if bounds.turntable_viewport().contains(pt) {
            self.grip.dragging = true;
        }
        StudioDirective::StayInStudio
    }
}
