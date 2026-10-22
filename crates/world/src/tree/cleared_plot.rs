use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::depository::TimberDepository;
use super::floor_litter::FloorLitter;
use super::hewn_target::HewnTimberTarget;
use super::timber_yield::SurfaceFind;

/// Vacant ground plot where vegetation has been harvested, optionally emitting final falling leaves, woodchips, and felled timber.
pub struct ClearedPlot {
    frame: SurfaceFrame,
    u: f32,
    v: f32,
    elevation: f32,
    burst: f32,
    cleave_u: f32,
    cleave_v: f32,
    salvage: SurfaceFind,
}

impl ClearedPlot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn with_salvage(
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        burst: f32,
        cleave_u: f32,
        cleave_v: f32,
        salvage: SurfaceFind,
    ) -> Self {
        Self {
            frame,
            u,
            v,
            elevation,
            burst,
            cleave_u,
            cleave_v,
            salvage,
        }
    }

    /// Secondary constructor for a freshly felled plot emitting final woodchips, canopy leaves, and fallen deadwood.
    pub fn felled(
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        cleave_u: f32,
        cleave_v: f32,
    ) -> Self {
        Self::with_salvage(frame, u, v, elevation, 0.92, cleave_u, cleave_v, SurfaceFind::DeadwoodBough)
    }

    /// Secondary constructor for a calm vacant plot with zero residual particles.
    pub fn new(u: f32, v: f32, elevation: f32) -> Self {
        Self::with_salvage(
            SurfaceFrame::new(Vec3::Y, Vec3::X, Vec3::Z, 0.0),
            u,
            v,
            elevation,
            0.0,
            0.0,
            1.0,
            SurfaceFind::EmptyHanded,
        )
    }
}

impl Arboreal for ClearedPlot {
    fn strike(&self, _cleave: u32, _recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        Box::new(Self::with_salvage(
            self.frame,
            self.u,
            self.v,
            self.elevation,
            0.0,
            self.cleave_u,
            self.cleave_v,
            self.salvage,
        ))
    }

    fn forage_at(&self, eye: Vec3, direction: Vec3) -> (Box<dyn Arboreal>, SurfaceFind) {
        if !self.salvage.gathered() {
            return (Box::new(Self::new(self.u, self.v, self.elevation)), SurfaceFind::EmptyHanded);
        }
        let litter = FloorLitter::new(self.frame, self.u, self.v, self.elevation, self.salvage, 0.45);
        let (_, find) = litter.forage_at(eye, direction);
        if find.gathered() {
            (
                Box::new(Self::with_salvage(self.frame, self.u, self.v, self.elevation, self.burst, self.cleave_u, self.cleave_v, SurfaceFind::EmptyHanded)),
                find,
            )
        } else {
            (
                Box::new(Self::with_salvage(self.frame, self.u, self.v, self.elevation, self.burst, self.cleave_u, self.cleave_v, self.salvage)),
                SurfaceFind::EmptyHanded,
            )
        }
    }

    fn step_timber(&mut self, dt: f32) -> bool {
        if self.burst > 0.0 {
            self.burst = (self.burst - dt * 2.2).max(0.0);
            true
        } else {
            false
        }
    }

    fn solid(&self) -> bool {
        false
    }

    fn standing(&self) -> bool {
        false
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
        if self.salvage.gathered() {
            FloorLitter::new(self.frame, self.u, self.v, self.elevation, self.salvage, 0.45).emit_surface(target);
        }
        if self.burst > 0.0 {
            let mut hewn = HewnTimberTarget::new(
                target,
                self.frame,
                self.u,
                self.v,
                self.elevation,
                1.0,
                self.burst,
                self.cleave_u,
                self.cleave_v,
                Vec3::new(0.24, 0.56, 0.18),
            );
            hewn.emit_cleave_burst();
        }
    }

    fn canopy_drag(&self, _u: f32, _v: f32, _height: f32) -> f32 {
        0.0
    }
}
