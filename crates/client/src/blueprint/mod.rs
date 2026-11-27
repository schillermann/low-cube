mod archive;
mod plaque;
mod saved;
mod sheet;

use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{CraftBlueprint, MeshTarget, PlanetaryWorld};
use crate::attire::{BeltSatchel, BodyPose, EvaAttire};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
pub use self::archive::{BlueprintArchive, DirectoryBlueprints};
#[cfg(test)]
pub use self::archive::MemoryBlueprints;
use self::plaque::FolioPlaque;
pub use self::saved::SavedBlueprint;

const MAX_NAME_CHARS: usize = 24;

/// Polymorphic UI phase of `BlueprintFolio` without nullable fields.
pub enum FolioView {
    Closed,
    NamingDraft {
        draft: String,
        captured: CraftBlueprint,
    },
    RigCatalog {
        entries: Vec<SavedBlueprint>,
        cursor: usize,
        aim: (Vec3, Vec3),
    },
}

/// Interactive blueprint folio handling custom vehicle naming, persistent storage, and crane blueprint construction.
pub struct BlueprintFolio {
    view: FolioView,
    archive: Box<dyn BlueprintArchive>,
}

impl BlueprintFolio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(view: FolioView, archive: Box<dyn BlueprintArchive>) -> Self {
        Self { view, archive }
    }

    pub fn with_archive(archive: Box<dyn BlueprintArchive>) -> Self {
        Self::new(FolioView::Closed, archive)
    }

    pub fn profile() -> Self {
        Self::with_archive(Box::new(DirectoryBlueprints::profile()))
    }

    pub const fn open(&self) -> bool {
        !matches!(self.view, FolioView::Closed)
    }

    pub const fn typing_name(&self) -> bool {
        matches!(self.view, FolioView::NamingDraft { .. })
    }

    pub fn close(&mut self) {
        self.view = FolioView::Closed;
    }

    pub fn open_naming(&mut self, captured: CraftBlueprint) {
        self.view = FolioView::NamingDraft {
            draft: String::new(),
            captured,
        };
    }

    pub fn open_rig_catalog(&mut self, eye: Vec3, view_dir: Vec3) {
        let entries = self.archive.load_saved();
        self.view = FolioView::RigCatalog {
            entries,
            cursor: 0,
            aim: (eye, view_dir),
        };
    }

    pub fn append_text(&mut self, typed: &str) -> bool {
        let FolioView::NamingDraft { draft, .. } = &mut self.view else {
            return false;
        };
        let mut changed = false;
        for ch in typed.chars() {
            let valid = ch.is_ascii_alphanumeric()
                || matches!(ch, ' ' | '-' | '_' | '#' | '.' | '+' | '(' | ')' | '/' | ':');
            if valid && draft.chars().count() < MAX_NAME_CHARS {
                if ch == ' ' && (draft.is_empty() || draft.ends_with(' ')) {
                    continue;
                }
                draft.push(ch.to_ascii_uppercase());
                changed = true;
            }
        }
        changed
    }

    fn next_untitled_name(&self) -> String {
        let existing = self.archive.load_saved();
        let mut index = 1usize;
        loop {
            let candidate = format!("VEHICLE {index}");
            if !existing.iter().any(|e| e.title_text().eq_ignore_ascii_case(&candidate)) {
                return candidate;
            }
            index += 1;
        }
    }

    fn commit_draft(&mut self) -> bool {
        let prev = std::mem::replace(&mut self.view, FolioView::Closed);
        let FolioView::NamingDraft { draft, captured } = prev else {
            self.view = prev;
            return false;
        };
        let trimmed = draft.trim();
        let final_name = if trimmed.is_empty() {
            self.next_untitled_name()
        } else {
            trimmed.to_string()
        };
        let entry = SavedBlueprint::new(final_name, captured);
        self.archive.store(&entry);
        true
    }

    pub fn handle_trigger(&mut self, trigger: InputTrigger, bindings: &ActionBindings) -> bool {
        match &mut self.view {
            FolioView::Closed => false,
            FolioView::NamingDraft { draft, .. } => match trigger {
                InputTrigger::Key(KeyCode::Escape) => {
                    self.close();
                    true
                }
                InputTrigger::Key(KeyCode::Enter | KeyCode::NumpadEnter) => self.commit_draft(),
                InputTrigger::Key(KeyCode::Backspace) => {
                    let _ = draft.pop();
                    true
                }
                _ => true,
            },
            FolioView::RigCatalog { entries, cursor, .. } => {
                if trigger == InputTrigger::Key(KeyCode::Escape) {
                    self.close();
                    return true;
                }
                if bindings.matches(ControlAction::Satchel, trigger) {
                    self.close();
                    return false;
                }
                if matches!(trigger, InputTrigger::Key(KeyCode::Delete | KeyCode::Backspace)) {
                    if !entries.is_empty() {
                        let idx = *cursor % entries.len();
                        let removed = entries.remove(idx);
                        *cursor = if entries.is_empty() { 0 } else { idx.min(entries.len() - 1) };
                        self.archive.expunge(removed.title_text());
                    }
                    return true;
                }
                let step = if bindings.matches(ControlAction::MoveForward, trigger)
                    || bindings.matches(ControlAction::MoveLeft, trigger)
                    || bindings.matches(ControlAction::MoldLeft, trigger)
                    || trigger == InputTrigger::Key(KeyCode::ArrowUp)
                {
                    -1isize
                } else if bindings.matches(ControlAction::MoveBackward, trigger)
                    || bindings.matches(ControlAction::MoveRight, trigger)
                    || bindings.matches(ControlAction::MoldRight, trigger)
                    || trigger == InputTrigger::Key(KeyCode::ArrowDown)
                {
                    1isize
                } else {
                    0
                };
                if step != 0 {
                    if !entries.is_empty() {
                        let len = entries.len() as isize;
                        *cursor = (*cursor as isize + step).rem_euclid(len) as usize;
                    }
                    return true;
                }
                false
            }
        }
    }

    pub fn try_build_trigger(
        &mut self,
        trigger: InputTrigger,
        bindings: &ActionBindings,
        satchel: BeltSatchel,
        attire: EvaAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> (BeltSatchel, EvaAttire, bool) {
        if !matches!(self.view, FolioView::RigCatalog { .. }) {
            return (satchel, attire, false);
        }
        let confirm = matches!(trigger, InputTrigger::Key(KeyCode::Enter | KeyCode::NumpadEnter))
            || bindings.matches(ControlAction::Jump, trigger)
            || bindings.matches(ControlAction::MoldForm, trigger)
            || bindings.matches(ControlAction::SatchelRight, trigger)
            || bindings.matches(ControlAction::SatchelLeft, trigger);
        if !confirm {
            return (satchel, attire, false);
        }
        self.enact_rig_build(satchel, attire, world)
    }

    pub fn act_open(
        &mut self,
        satchel: BeltSatchel,
        attire: EvaAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> (BeltSatchel, EvaAttire, bool, bool) {
        match &self.view {
            FolioView::Closed => (satchel, attire, false, false),
            FolioView::NamingDraft { .. } => {
                let _ = self.commit_draft();
                (satchel, attire, false, true)
            }
            FolioView::RigCatalog { entries, .. } => {
                if entries.is_empty() {
                    self.close();
                    return (satchel, attire, false, true);
                }
                let (ns, na, built) = self.enact_rig_build(satchel, attire, world);
                (ns, na, built, true)
            }
        }
    }

    fn enact_rig_build(
        &mut self,
        satchel: BeltSatchel,
        attire: EvaAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> (BeltSatchel, EvaAttire, bool) {
        let FolioView::RigCatalog { entries, cursor, aim } = &self.view else {
            return (satchel, attire, false);
        };
        if entries.is_empty() {
            return (satchel, attire, false);
        };
        let selected = &entries[*cursor % entries.len()];
        if !selected.buildable(&satchel, attire) {
            return (satchel, attire, false);
        }
        let manifest = selected.manifest();
        if !world.build_rig_blueprint(aim.0, aim.1, selected.blueprint()) {
            return (satchel, attire, false);
        }
        let (ns, na) = satchel
            .consume_manifest(&manifest, attire)
            .expect("verified buildable blueprint manifest");
        self.close();
        (ns, na, true)
    }

    pub fn emit(&self, satchel: &BeltSatchel, attire: EvaAttire, pose: &BodyPose, target: &mut dyn MeshTarget) {
        FolioPlaque::new(&self.view, (satchel, attire)).emit(pose, target);
    }
}
