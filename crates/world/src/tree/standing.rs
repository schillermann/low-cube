use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::cleared_plot::ClearedPlot;
use super::depository::TimberDepository;
use super::species::TreeSpecies;

/// Living standing tree anchored into the surface frame.
pub struct StandingTree {
    species: Box<dyn TreeSpecies>,
    frame: SurfaceFrame,
    u: f32,
    v: f32,
    elevation: f32,
    vitality: u32,
}

impl StandingTree {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        species: Box<dyn TreeSpecies>,
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        vitality: u32,
    ) -> Self {
        Self {
            species,
            frame,
            u,
            v,
            elevation,
            vitality,
        }
    }

    /// Secondary constructor initializing full species vitality.
    pub fn of_species(
        species: Box<dyn TreeSpecies>,
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
    ) -> Self {
        let initial = species.initial_vitality();
        Self::new(species, frame, u, v, elevation, initial)
    }
}

impl Arboreal for StandingTree {
    fn strike(&self, cleave: u32, recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        if cleave >= self.vitality {
            self.species.yield_harvest(recipient);
            Box::new(ClearedPlot::new(self.u, self.v, self.elevation))
        } else {
            Box::new(Self::new(
                self.species.clone_species(),
                self.frame,
                self.u,
                self.v,
                self.elevation,
                self.vitality - cleave,
            ))
        }
    }

    fn solid(&self) -> bool {
        true
    }

    fn standing(&self) -> bool {
        true
    }

    fn constrain_surface(
        &self,
        base: Vec3,
        _u: f32,
        _v: f32,
        _frame: &SurfaceFrame,
        _distance: f32,
    ) -> Vec3 {
        base
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.species
            .emit_tree(&self.frame, self.u, self.v, self.elevation, target);
    }

    fn canopy_drag(&self, u: f32, v: f32, height: f32) -> f32 {
        let dist = (u - self.u).hypot(v - self.v);
        let relative_h = height - self.elevation;
        self.species.canopy_drag(dist, relative_h)
    }
}
