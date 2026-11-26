use crate::world_vault::{ScaleTier, SeedPhrase, WorldSpec, WorldTitle};

/// Active keyboard input target on the `CREATE WORLD` configurator screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreationFocus {
    NameField,
    SeedField,
}

/// Pure domain form state for configuring a new player world (`4` fields).
#[derive(Clone, Debug)]
pub struct WorldDraft {
    title: WorldTitle,
    tier: ScaleTier,
    phrase: SeedPhrase,
    focus: CreationFocus,
}

impl WorldDraft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        title: WorldTitle,
        tier: ScaleTier,
        phrase: SeedPhrase,
        focus: CreationFocus,
    ) -> Self {
        Self { title, tier, phrase, focus }
    }

    pub fn fresh(ordinal: usize, entropy: u64) -> Self {
        Self::new(
            WorldTitle::numbered(ordinal),
            ScaleTier::moonlet(),
            SeedPhrase::rolled(entropy),
            CreationFocus::NameField,
        )
    }

    pub fn focus_name(&mut self) {
        self.focus = CreationFocus::NameField;
    }

    pub fn focus_seed(&mut self) {
        self.focus = CreationFocus::SeedField;
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            CreationFocus::NameField => CreationFocus::SeedField,
            CreationFocus::SeedField => CreationFocus::NameField,
        };
    }

    pub const fn name_focused(&self) -> bool {
        matches!(self.focus, CreationFocus::NameField)
    }

    pub const fn seed_focused(&self) -> bool {
        matches!(self.focus, CreationFocus::SeedField)
    }

    pub fn clear_name(&mut self) {
        self.title = WorldTitle::new(String::new());
        self.focus = CreationFocus::NameField;
    }

    pub fn reroll_seed(&mut self, entropy: u64) {
        self.phrase = SeedPhrase::rolled(entropy);
        self.focus = CreationFocus::SeedField;
    }

    pub fn prev_scale(&mut self) {
        self.tier = self.tier.prev();
    }

    pub fn next_scale(&mut self) {
        self.tier = self.tier.next();
    }

    pub fn size_banner(&self) -> String {
        self.tier.banner()
    }

    pub fn name_caption(&self) -> String {
        self.title.field_caption(self.name_focused())
    }

    pub fn seed_caption(&self) -> String {
        self.phrase.field_caption(self.seed_focused())
    }

    pub fn backspace(&mut self) {
        match self.focus {
            CreationFocus::NameField => self.title = self.title.popped(),
            CreationFocus::SeedField => self.phrase = self.phrase.popped(),
        }
    }

    pub fn append_text(&mut self, raw: &str) {
        match self.focus {
            CreationFocus::NameField => self.title = self.title.appended(raw),
            CreationFocus::SeedField => self.phrase = self.phrase.appended(raw),
        }
    }

    pub fn mint_spec(&self, fallback_ordinal: usize, fallback_entropy: u64) -> WorldSpec {
        let title = self.title.resolved(fallback_ordinal);
        let slug = title.derive_slug();
        let phrase = self.phrase.resolved(fallback_entropy);
        WorldSpec::new(slug, title, self.tier, phrase)
    }
}
