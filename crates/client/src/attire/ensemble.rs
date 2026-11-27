use super::{
    ComplexionSwatch, CrewPhysique, HeldRelic, HelmStyle, MagBootStyle, PartProportion,
    PigmentSwatch, PlssHarness, SuitGauntlet, SuitShell, BodyRegion,
};

/// Encapsulated four-piece Space-Age EVA pressure-suit ensemble (`4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SuitEnsemble {
    shell: SuitShell,
    gauntlet: SuitGauntlet,
    harness: PlssHarness,
    boots: MagBootStyle,
}

impl SuitEnsemble {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        shell: SuitShell,
        gauntlet: SuitGauntlet,
        harness: PlssHarness,
        boots: MagBootStyle,
    ) -> Self {
        Self { shell, gauntlet, harness, boots }
    }

    pub(super) const fn shell(self) -> SuitShell { self.shell }
    pub(super) const fn gauntlet(self) -> SuitGauntlet { self.gauntlet }
    pub(super) const fn harness(self) -> PlssHarness { self.harness }
    pub(super) const fn boots(self) -> MagBootStyle { self.boots }
    pub(super) fn next_shell(mut self) -> Self { self.shell = self.shell.next(); self }
    pub(super) fn next_gauntlet(mut self) -> Self { self.gauntlet = self.gauntlet.next(); self }
    pub(super) fn next_harness(mut self) -> Self { self.harness = self.harness.next(); self }
    pub(super) fn next_boots(mut self) -> Self { self.boots = self.boots.next(); self }
}

/// Encapsulated EVA helmet style, suit pigment, skin complexion, and crew physique (`4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrewIdentity {
    helm: HelmStyle,
    pigment: PigmentSwatch,
    complexion: ComplexionSwatch,
    physique: CrewPhysique,
}

impl CrewIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        helm: HelmStyle,
        pigment: PigmentSwatch,
        complexion: ComplexionSwatch,
        physique: CrewPhysique,
    ) -> Self {
        Self { helm, pigment, complexion, physique }
    }

    pub(super) const fn helm(self) -> HelmStyle { self.helm }
    pub(super) const fn pigment(self) -> PigmentSwatch { self.pigment }
    pub(super) const fn complexion(self) -> ComplexionSwatch { self.complexion }
    pub(super) const fn physique(self) -> CrewPhysique { self.physique }
    pub(super) fn next_helm(mut self) -> Self { self.helm = self.helm.next(); self }
    pub(super) fn next_pigment(mut self) -> Self { self.pigment = self.pigment.next(); self }
    pub(super) fn next_complexion(mut self) -> Self {
        self.complexion = self.complexion.next();
        self.physique = self.physique.next();
        self
    }
    pub(super) const fn with_complexion(mut self, complexion: ComplexionSwatch) -> Self {
        self.complexion = complexion;
        self
    }
    pub(super) const fn with_physique(mut self, physique: CrewPhysique) -> Self {
        self.physique = physique;
        self
    }
    pub(super) fn with_feminine(mut self, feminine: bool) -> Self {
        self.physique = self.physique.as_feminine(feminine);
        self
    }
    pub(super) fn adjusted_region(mut self, region: BodyRegion, delta_size: i8, delta_girth: i8) -> Self {
        self.physique = self.physique.adjusted(region, delta_size, delta_girth);
        self
    }
    pub(super) fn region_proportion(self, region: BodyRegion) -> PartProportion {
        self.physique.region_proportion(region)
    }
}

/// Encapsulated right-palm and left-palm held Space-Age tools or blocks (`2` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldHands {
    relic: HeldRelic,
    offhand: HeldRelic,
}

impl HeldHands {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic, offhand: HeldRelic) -> Self {
        Self { relic, offhand }
    }

    /// Secondary constructor initializing two empty gloved palms.
    pub const fn empty() -> Self {
        Self::new(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm)
    }

    pub(super) const fn pair(self) -> (HeldRelic, HeldRelic) { (self.relic, self.offhand) }
    #[cfg(test)]
    pub(super) fn next_relic(mut self) -> Self { self.relic = self.relic.next(); self }
}
