use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::cleared_plot::ClearedPlot;
use super::depository::TimberDepository;
use super::hewn_target::HewnTimberTarget;
use super::species::TreeSpecies;

#[derive(Clone, Copy, Debug)]
struct TreeRoot {
    frame: SurfaceFrame,
    u: f32,
    v: f32,
    elevation: f32,
}

#[derive(Clone, Copy, Debug)]
struct CleaveState {
    vitality: u32,
    shudder: f32,
    cleave_u: f32,
    cleave_v: f32,
}

/// Living standing tree anchored into the surface frame with progressive axe notch and strike shudder.
pub struct StandingTree {
    species: Box<dyn TreeSpecies>,
    root: TreeRoot,
    cleave: CleaveState,
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
            root: TreeRoot { frame, u, v, elevation },
            cleave: CleaveState { vitality, shudder, cleave_u, cleave_v },
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
        let (r, c) = (self.root, self.cleave);
        if cleave == 0 {
            return Box::new(Self::with_cleave(
                self.species.clone_species(),
                r.frame,
                r.u,
                r.v,
                r.elevation,
                c.vitality,
                c.shudder,
                c.cleave_u,
                c.cleave_v,
            ));
        }
        if cleave >= c.vitality {
            self.species.yield_harvest(recipient);
            Box::new(ClearedPlot::felled(r.frame, r.u, r.v, r.elevation, cu, cv))
        } else {
            Box::new(Self::with_cleave(
                self.species.clone_species(),
                r.frame,
                r.u,
                r.v,
                r.elevation,
                c.vitality - cleave,
                0.96,
                cu,
                cv,
            ))
        }
    }
}

impl Arboreal for StandingTree {
    fn strike(&self, cleave: u32, recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        self.apply_cleave(cleave, self.cleave.cleave_u, self.cleave.cleave_v, recipient)
    }

    fn cleave_at(
        &self,
        eye: Vec3,
        direction: Vec3,
        recipient: &mut dyn TimberDepository,
    ) -> (Box<dyn Arboreal>, bool) {
        let r = self.root;
        let dir = direction.normalize_or_zero();
        let (eu, ev) = r.frame.decompose(eye);
        let (du, dv) = r.frame.decompose(dir);
        let (tu, tv) = (r.u - eu, r.v - ev);
        let dist = tu.hypot(tv);
        let len_2d = du.hypot(dv);
        if dist <= 2.45 && len_2d > 1e-4 {
            let (ndu, ndv) = (du / len_2d, dv / len_2d);
            let proj = tu * ndu + tv * ndv;
            let perp = (tu - proj * ndu).hypot(tv - proj * ndv);
            let ray_h = r.frame.elevation_of(eye) + dir.dot(r.frame.normal()) * (proj / len_2d) - r.elevation;
            if (0.05..=2.45).contains(&proj) && perp <= 0.78 && (-0.5..=3.8).contains(&ray_h) {
                return (self.apply_cleave(1, ndu, ndv, recipient), true);
            }
        }
        (self.strike(0, recipient), false)
    }

    fn step_timber(&mut self, dt: f32) -> bool {
        if self.cleave.shudder > 0.0 {
            self.cleave.shudder = (self.cleave.shudder - dt * 2.2).max(0.0);
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
        let (r, c) = (self.root, self.cleave);
        let initial = self.species.initial_vitality().max(1);
        let cuts = initial.saturating_sub(c.vitality);
        let notch_ratio = if cuts == 0 {
            0.0
        } else {
            let swing_in = (1.0 - c.shudder * c.shudder).clamp(0.20, 1.0);
            (((cuts - 1) as f32 + swing_in) / (initial as f32)).clamp(0.0, 0.95)
        };
        let mut hewn = HewnTimberTarget::new(
            target,
            r.frame,
            r.u,
            r.v,
            r.elevation,
            notch_ratio,
            c.shudder,
            c.cleave_u,
            c.cleave_v,
            Vec3::new(0.24, 0.56, 0.18),
        );
        self.species.emit_tree(&r.frame, r.u, r.v, r.elevation, &mut hewn);
        hewn.emit_cleave_burst();
    }

    fn canopy_drag(&self, u: f32, v: f32, height: f32) -> f32 {
        let r = self.root;
        let dist = (u - r.u).hypot(v - r.v);
        let relative_h = height - r.elevation;
        self.species.canopy_drag(dist, relative_h)
    }
}
