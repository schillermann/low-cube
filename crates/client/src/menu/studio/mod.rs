mod layout;

use glam::Mat4;
use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::attire::{AttireProfile, BodyRegion, ComplexionSwatch, FileAttire, HeldRelic, YouthAttire, YouthPhysique};
use super::archetype::AvatarArchetype;
use super::stage::TurntableStage;
use super::widget::ScreenPoint;

/// Outcome returned from interactive studio actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioDirective {
    StayInStudio,
    ReturnToPauseMenu,
}

#[derive(Clone, Copy, Debug)]
struct StudioPointer {
    point: ScreenPoint,
    dragging: bool,
}

impl StudioPointer {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(point: ScreenPoint, dragging: bool) -> Self {
        Self { point, dragging }
    }
}

/// Interactive 3D character selection, turntable inspection, skin color, and body-part proportion studio.
#[derive(Clone, Copy, Debug)]
pub struct WardrobeStudio {
    attire: YouthAttire,
    archetype: AvatarArchetype,
    stage: TurntableStage,
    pointer: StudioPointer,
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
            pointer: StudioPointer::new(point, dragging),
        }
    }

    pub fn from_profile() -> Self {
        let loaded = FileAttire::profile().load_or_squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm);
        Self::new(loaded, AvatarArchetype::Knight, TurntableStage::front_showcase(), ScreenPoint::center(), false)
    }

    pub fn step(&mut self, dt: f32) {
        self.stage = self.stage.step(dt);
    }

    pub fn current_attire(&self) -> YouthAttire {
        self.attire
    }

    pub(super) const fn pointer_point(&self) -> ScreenPoint {
        self.pointer.point
    }

    pub(super) fn selected_archetype(&self, candidate: AvatarArchetype) -> bool {
        self.archetype == candidate
    }

    pub fn stage_projection(&self, aspect: f32) -> Mat4 {
        self.stage.view_projection(aspect)
    }

    pub fn emit_stage(&self, target: &mut dyn MeshTarget) {
        self.stage.emit_stage(&self.attire, target);
    }

    pub fn release_pointer(&mut self) {
        self.pointer.dragging = false;
    }

    pub fn move_pointer(&mut self, next: ScreenPoint) {
        if self.pointer.dragging {
            self.stage = self.stage.rotated(next.horizontal_delta(self.pointer.point) * 3.8);
        }
        self.pointer.point = next;
    }

    pub fn handle_key(&mut self, code: KeyCode) -> bool {
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
                    self.attire = updated;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn update_attire(&mut self, updated: YouthAttire) {
        self.attire = updated;
        FileAttire::profile().save(&self.attire);
    }

    pub fn press_pointer(&mut self) -> StudioDirective {
        let pt = self.pointer.point;
        if Self::rotate_left_button().contains(pt) {
            self.stage = self.stage.rotated(-0.35);
            return StudioDirective::StayInStudio;
        }
        if Self::front_button().contains(pt) {
            self.stage = self.stage.reset_angle();
            return StudioDirective::StayInStudio;
        }
        if Self::rotate_right_button().contains(pt) {
            self.stage = self.stage.rotated(0.35);
            return StudioDirective::StayInStudio;
        }

        let (held_r, held_l) = self.attire.held_pair();
        for (i, arch) in AvatarArchetype::all().iter().enumerate() {
            if Self::archetype_slot(i).contains(pt) {
                self.archetype = *arch;
                self.update_attire(arch.attire_for(self.attire.feminine()).with_relics(held_r, held_l));
                return StudioDirective::StayInStudio;
            }
        }
        for (i, swatch) in ComplexionSwatch::all().iter().enumerate() {
            if Self::swatch_slot(i).contains(pt) {
                self.update_attire(self.attire.with_complexion(*swatch));
                return StudioDirective::StayInStudio;
            }
        }
        let clothing_keys = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6];
        for (i, key) in clothing_keys.iter().enumerate() {
            if Self::clothing_slot(i).contains(pt) {
                if let Some(next) = self.attire.with_wardrobe_key(*key) {
                    self.update_attire(next);
                }
                return StudioDirective::StayInStudio;
            }
        }
        if Self::male_button().contains(pt) {
            self.update_attire(self.archetype.attire_for(false).with_relics(held_r, held_l));
            return StudioDirective::StayInStudio;
        }
        if Self::female_button().contains(pt) {
            self.update_attire(self.archetype.attire_for(true).with_relics(held_r, held_l));
            return StudioDirective::StayInStudio;
        }
        for (idx, region) in BodyRegion::all().iter().enumerate() {
            let (_, s_dec, _, s_inc, g_dec, _, g_inc) = Self::region_rows(idx);
            if s_dec.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, -1, 0));
                return StudioDirective::StayInStudio;
            }
            if s_inc.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, 1, 0));
                return StudioDirective::StayInStudio;
            }
            if g_dec.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, 0, -1));
                return StudioDirective::StayInStudio;
            }
            if g_inc.contains(pt) {
                self.update_attire(self.attire.with_adjusted_region(*region, 0, 1));
                return StudioDirective::StayInStudio;
            }
        }
        if Self::reset_button().contains(pt) {
            self.update_attire(self.attire.with_physique(YouthPhysique::SlenderSquire.as_feminine(self.attire.feminine())));
            return StudioDirective::StayInStudio;
        }
        if Self::back_button().contains(pt) {
            return StudioDirective::ReturnToPauseMenu;
        }
        if Self::turntable_viewport().contains(pt) {
            self.pointer.dragging = true;
        }
        StudioDirective::StayInStudio
    }
}
