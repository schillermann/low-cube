use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::cleared_plot::ClearedPlot;
use super::depository::TimberDepository;
use super::hewn_target::HewnTimberTarget;
use super::species::TreeSpecies;

/// Living standing tree anchored into the surface frame with progressive axe notch and strike shudder.
pub struct StandingTree {
    species: Box<dyn TreeSpecies>,
    frame: SurfaceFrame,
    u: f32,
    v: f32,
    elevation: f32,
    vitality: u32,
    shudder: f32,
    cleave_u: f32,
    cleave_v: f32,
}

impl StandingTree {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn with_cleave(
        species: Box<dyn TreeSpecies>,
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        vitality: u32,
        shudder: f32,
        cleave_u: f32,
        cleave_v: f32,
    ) -> Self {
        Self {
            species,
            frame,
            u,
            v,
            elevation,
            vitality,
            shudder,
            cleave_u,
            cleave_v,
        }
    }

    /// Secondary constructor with calm initial state at given vitality.
    pub fn new(
        species: Box<dyn TreeSpecies>,
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        vitality: u32,
    ) -> Self {
        Self::with_cleave(species, frame, u, v, elevation, vitality, 0.0, 0.0, 1.0)
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

    fn apply_cleave(
        &self,
        cleave: u32,
        cu: f32,
        cv: f32,
        recipient: &mut dyn TimberDepository,
    ) -> Box<dyn Arboreal> {
        if cleave == 0 {
            return Box::new(Self::with_cleave(
                self.species.clone_species(),
                self.frame,
                self.u,
                self.v,
                self.elevation,
                self.vitality,
                self.shudder,
                self.cleave_u,
                self.cleave_v,
            ));
        }
        if cleave >= self.vitality {
            self.species.yield_harvest(recipient);
            Box::new(ClearedPlot::felled(self.frame, self.u, self.v, self.elevation, cu, cv))
        } else {
            Box::new(Self::with_cleave(
                self.species.clone_species(),
                self.frame,
                self.u,
                self.v,
                self.elevation,
                self.vitality - cleave,
                0.96,
                cu,
                cv,
            ))
        }
    }
}

impl Arboreal for StandingTree {
    fn strike(&self, cleave: u32, recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        self.apply_cleave(cleave, self.cleave_u, self.cleave_v, recipient)
    }

    fn cleave_at(
        &self,
        eye: Vec3,
        direction: Vec3,
        recipient: &mut dyn TimberDepository,
    ) -> (Box<dyn Arboreal>, bool) {
        let dir = direction.normalize_or_zero();
        let (eu, ev) = self.frame.decompose(eye);
        let (du, dv) = self.frame.decompose(dir);
        let (tu, tv) = (self.u - eu, self.v - ev);
        let dist = tu.hypot(tv);
        let len_2d = du.hypot(dv);
        if dist <= 2.45 && len_2d > 1e-4 {
            let (ndu, ndv) = (du / len_2d, dv / len_2d);
            let proj = tu * ndu + tv * ndv;
            let perp = (tu - proj * ndu).hypot(tv - proj * ndv);
            let ray_h = self.frame.elevation_of(eye) + dir.dot(self.frame.normal()) * (proj / len_2d) - self.elevation;
            if (0.05..=2.45).contains(&proj) && perp <= 0.78 && (-0.5..=3.8).contains(&ray_h) {
                return (self.apply_cleave(1, ndu, ndv, recipient), true);
            }
        }
        (self.strike(0, recipient), false)
    }

    fn step_timber(&mut self, dt: f32) -> bool {
        if self.shudder > 0.0 {
            self.shudder = (self.shudder - dt * 2.2).max(0.0);
            true
        } else {
            false
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
        let initial = self.species.initial_vitality().max(1);
        let cuts = initial.saturating_sub(self.vitality);
        let notch_ratio = if cuts == 0 {
            0.0
        } else {
            let swing_in = (1.0 - self.shudder * self.shudder).clamp(0.20, 1.0);
            (((cuts - 1) as f32 + swing_in) / (initial as f32)).clamp(0.0, 0.95)
        };
        let mut hewn = HewnTimberTarget::new(
            target,
            self.frame,
            self.u,
            self.v,
            self.elevation,
            notch_ratio,
            self.shudder,
            self.cleave_u,
            self.cleave_v,
            Vec3::new(0.24, 0.56, 0.18),
        );
        self.species.emit_tree(&self.frame, self.u, self.v, self.elevation, &mut hewn);
        hewn.emit_cleave_burst();
    }

    fn canopy_drag(&self, u: f32, v: f32, height: f32) -> f32 {
        let dist = (u - self.u).hypot(v - self.v);
        let relative_h = height - self.elevation;
        self.species.canopy_drag(dist, relative_h)
    }
}
