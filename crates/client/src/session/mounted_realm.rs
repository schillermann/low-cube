use world::PlanetaryWorld;
use crate::menu::PortalInvite;
use crate::world_vault::{RealmPort, WorldSlug, WorldSpec};

/// Polymorphic local world binding avoiding nullable `Option<WorldSpec>` fields.
enum ActiveRealm {
    Unbound,
    Local(WorldSpec),
}

/// Encapsulated local `RealmPort` and currently mounted player `WorldSpec` (`2` fields, zero `Rc<RefCell>`).
pub(super) struct MountedRealm {
    port: Box<dyn RealmPort>,
    active: ActiveRealm,
}

impl MountedRealm {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(port: Box<dyn RealmPort>, active: ActiveRealm) -> Self {
        Self { port, active }
    }

    pub(super) fn unbound(port: Box<dyn RealmPort>) -> Self {
        Self::new(port, ActiveRealm::Unbound)
    }

    pub(super) fn mount_solitary(&mut self, spec: WorldSpec) -> Box<dyn PlanetaryWorld> {
        let world = self.port.mount_solitary(&spec);
        self.active = ActiveRealm::Local(spec);
        world
    }

    pub(super) fn mount_hosted_if_local(&self) -> Option<(Box<dyn PlanetaryWorld>, PortalInvite)> {
        match self.active {
            ActiveRealm::Unbound => None,
            ActiveRealm::Local(ref spec) => Some(self.port.mount_hosted(spec)),
        }
    }

    pub(super) fn unbind(&mut self) {
        self.active = ActiveRealm::Unbound;
    }

    pub(super) fn evict(&mut self, slug: &WorldSlug) {
        self.active = ActiveRealm::Unbound;
        self.port.purge_evicted(slug);
    }
}
