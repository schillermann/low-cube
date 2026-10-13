use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::noise::{CoherentNoise, DomainWarp, FractalHarmonics};
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget};
use super::cavity::{NullCavity, SubterraneanCavity};
use super::elevation::{ReliefElevation, ZonedRelief};
use super::palette::ReliefPalette;
use super::strata::ReliefStrata;

/// Planetary relief face generating undulating climbable terrain and subterranean cavities ("Tell, Don't Ask").
pub struct FacetedReliefFace {
    frame: SurfaceFrame,
    relief: Box<dyn ReliefElevation>,
    cavity: Box<dyn SubterraneanCavity>,
    palette: ReliefPalette,
    strata: ReliefStrata,
    divisions: u32,
}

impl FacetedReliefFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        frame: SurfaceFrame,
        relief: Box<dyn ReliefElevation>,
        cavity: Box<dyn SubterraneanCavity>,
        palette: ReliefPalette,
        strata: ReliefStrata,
        divisions: u32,
    ) -> Self {
        Self {
            frame,
            relief,
            cavity,
            palette,
            strata,
            divisions,
        }
    }

    /// Secondary constructor assembling a relief face calibrated for a specific planetary climate zone.
    pub fn in_zone(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        let frame = zone.surface_frame(half_size);
        let palette = ReliefPalette::for_zone(zone);
        let strata = ReliefStrata::new();

        let seed_offset = match zone {
            ClimateZone::Temperate => 101,
            ClimateZone::Volcanic => 202,
            ClimateZone::Desert => 303,
            ClimateZone::Arctic => 404,
            ClimateZone::Jungle => 505,
            ClimateZone::Steppe => 606,
        };

        let freq = if half_size >= 48.0 { 0.025 } else { 0.04 };
        let base_noise = Box::new(CoherentNoise::new(seed_offset, freq));
        let harmonics = Box::new(FractalHarmonics::new(base_noise, 3, 0.5, 2.0));
        let warp_x = Box::new(CoherentNoise::new(seed_offset + 10, freq * 0.5));
        let warp_z = Box::new(CoherentNoise::new(seed_offset + 20, freq * 0.5));
        let warped = Box::new(DomainWarp::new(harmonics, warp_x, warp_z, 2.5));

        let (min_elev, max_elev) = if half_size >= 48.0 {
            match zone {
                ClimateZone::Volcanic => (0, 16),
                ClimateZone::Arctic => (0, 14),
                ClimateZone::Steppe => (0, 10),
                ClimateZone::Temperate => (0, 9),
                ClimateZone::Jungle => (-1, 8),
                ClimateZone::Desert => (0, 7),
            }
        } else if half_size >= 24.0 {
            match zone {
                ClimateZone::Volcanic => (0, 12),
                ClimateZone::Arctic => (0, 10),
                ClimateZone::Steppe => (0, 8),
                ClimateZone::Temperate => (0, 7),
                ClimateZone::Jungle => (-1, 6),
                ClimateZone::Desert => (0, 5),
            }
        } else {
            match zone {
                ClimateZone::Volcanic | ClimateZone::Arctic => (0, 3),
                ClimateZone::Jungle => (-1, 2),
                _ => (0, 2),
            }
        };


        let relief = Box::new(ZonedRelief::new(
            warped,
            frame.normal(),
            frame.tangent(),
            frame.bitangent(),
            half_size,
            min_elev,
            max_elev,
        ));

        let cavity = Box::new(NullCavity);

        Self::new(frame, relief, cavity, palette, strata, divisions)
    }
}

impl Face for FacetedReliefFace {
    fn matches(&self, point: Vec3) -> bool {
        let height = point.dot(self.frame.normal());
        if height <= 0.0 {
            return false;
        }

        let u = point.dot(self.frame.tangent()).abs();
        let v = point.dot(self.frame.bitangent()).abs();

        height >= u && height >= v
    }

    fn surface_up(&self) -> Vec3 {
        self.frame.normal()
    }

    fn pull_direction(&self) -> Vec3 {
        -self.frame.normal()
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let (u, v) = self.frame.decompose(point);
        let surface_h = self.relief.surface_height_at(u, v);
        let current_h = self.frame.height_of(point);
        let depth_h = self.cavity.constrain_depth(u, v, current_h, surface_h);
        self.frame.point_at(u, v, depth_h + distance)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let half = (self.divisions as i32) / 2;
        for u in -half..half {
            for v in -half..half {
                let cell = GridCell::new(u, v);
                self.strata.emit_cell(
                    cell,
                    &self.frame,
                    self.relief.as_ref(),
                    self.cavity.as_ref(),
                    &self.palette,
                    half,
                    target,
                );
            }

        }
    }
}
