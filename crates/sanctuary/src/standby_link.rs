use std::collections::VecDeque;
use world::PlanetaryWorld;
use crate::envelope::SanctuaryEnvelope;
use crate::fleet_ledger::FleetLedger;
use crate::mailbox::PortalBeat;
use crate::remote_link::RemoteLink;

/// Client UDP wire wrapper implementing `PortalBeat` without locking `PlanetaryWorld`.
pub struct StandbyLink {
    link: RemoteLink,
    deferred: VecDeque<SanctuaryEnvelope>,
}

impl StandbyLink {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(link: RemoteLink, deferred: VecDeque<SanctuaryEnvelope>) -> Self {
        Self { link, deferred }
    }

    pub fn wrapping(link: RemoteLink) -> Self {
        Self::new(link, VecDeque::new())
    }

    pub const fn link(&self) -> &RemoteLink {
        &self.link
    }

    pub fn link_mut(&mut self) -> &mut RemoteLink {
        &mut self.link
    }

    pub fn pump(
        &mut self,
        dt: f32,
        world: &mut dyn PlanetaryWorld,
        fleet: &mut FleetLedger,
        predicted: &mut VecDeque<world::WorldFact>,
    ) -> (bool, usize) {
        self.link.pump(dt, &mut self.deferred, world, fleet, predicted)
    }
}

impl PortalBeat for StandbyLink {
    fn step_standby(&mut self, dt: f32) -> usize {
        self.link.pump_standby(dt, &mut self.deferred)
    }
}
