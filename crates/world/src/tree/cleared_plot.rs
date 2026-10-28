use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::depository::TimberDepository;
use super::floor_litter::FloorLitter;
use super::hewn_target::HewnTimberTarget;
use super::timber_yield::SurfaceFind;

#[derive(Clone, Copy, Debug)]
struct PlotAnchor {
    frame: SurfaceFrame,
    u: f32,
    v: f32,
    elevation: f32,
}

#[derive(Clone, Copy, Debug)]
struct FelledAftermath {
    burst: f32,
    cleave_u: f32,
    cleave_v: f32,
    salvage: SurfaceFind,
}

/// Vacant ground plot where vegetation has been harvested, optionally emitting final falling leaves, woodchips, and felled timber.
pub struct ClearedPlot {
    anchor: PlotAnchor,
    aftermath: FelledAftermath,
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
            anchor: PlotAnchor { frame, u, v, elevation },
            aftermath: FelledAftermath { burst, cleave_u, cleave_v, salvage },
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
        let (a, f) = (self.anchor, self.aftermath);
        Box::new(Self::with_salvage(a.frame, a.u, a.v, a.elevation, 0.0, f.cleave_u, f.cleave_v, f.salvage))
    }

    fn forage_at(&self, eye: Vec3, direction: Vec3) -> (Box<dyn Arboreal>, SurfaceFind) {
        let (a, f) = (self.anchor, self.aftermath);
        if !f.salvage.gathered() {
            return (Box::new(Self::new(a.u, a.v, a.elevation)), SurfaceFind::EmptyHanded);
        }
        let litter = FloorLitter::new(a.frame, a.u, a.v, a.elevation, f.salvage, 0.45);
        let (_, find) = litter.forage_at(eye, direction);
        if find.gathered() {
            (
                Box::new(Self::with_salvage(a.frame, a.u, a.v, a.elevation, f.burst, f.cleave_u, f.cleave_v, SurfaceFind::EmptyHanded)),
                find,
            )
        } else {
            (
                Box::new(Self::with_salvage(a.frame, a.u, a.v, a.elevation, f.burst, f.cleave_u, f.cleave_v, f.salvage)),
                SurfaceFind::EmptyHanded,
            )
        }
    }

    fn step_timber(&mut self, dt: f32) -> bool {
        if self.aftermath.burst > 0.0 {
            self.aftermath.burst = (self.aftermath.burst - dt * 2.2).max(0.0);
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
        let (a, f) = (self.anchor, self.aftermath);
        if f.salvage.gathered() {
            FloorLitter::new(a.frame, a.u, a.v, a.elevation, f.salvage, 0.45).emit_surface(target);
        }
        if f.burst > 0.0 {
            let mut hewn = HewnTimberTarget::new(
                target,
                a.frame,
                a.u,
                a.v,
                a.elevation,
                1.0,
                f.burst,
                f.cleave_u,
                f.cleave_v,
                Vec3::new(0.24, 0.56, 0.18),
            );
            hewn.emit_cleave_burst();
        }
    }

    fn canopy_drag(&self, _u: f32, _v: f32, _height: f32) -> f32 {
        0.0
    }
}
