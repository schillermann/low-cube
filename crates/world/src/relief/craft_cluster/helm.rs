use crate::block::{BlockMedium, GridCell, WedgeHeading};
use crate::frame::SurfaceFrame;
use super::super::molded_structure::VoxelCells;
use super::super::sight_ray::SurfaceSightRay;

/// Focused cockpit helm inspector distinguishing rocket consoles (`FlightConsole`) from car wheels (`SteeringWheel`).
pub(super) struct ClusterHelm<'a> {
    grid: &'a VoxelCells,
}

impl<'a> ClusterHelm<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(grid: &'a VoxelCells) -> Self {
        Self { grid }
    }

    fn helm_medium(&self) -> Option<BlockMedium> {
        let (mut console, mut wheel) = (false, false);
        self.grid.visit_cells(|_, list| {
            for b in list {
                match b.block_medium() {
                    BlockMedium::FlightConsole => console = true,
                    BlockMedium::SteeringWheel => wheel = true,
                    _ => {}
                }
            }
        });
        if console {
            Some(BlockMedium::FlightConsole)
        } else if wheel {
            Some(BlockMedium::SteeringWheel)
        } else {
            None
        }
    }

    pub(super) fn unit(&self) -> Option<(i32, i32, f32, WedgeHeading)> {
        let target_med = self.helm_medium()?;
        let mut found = None;
        self.grid.visit_cells(|(cu, cv), list| {
            if found.is_none() {
                if let Some(eng) = list.iter().rfind(|b| b.block_medium() == target_med) {
                    found = Some((cu, cv, eng.floor(), eng.block_heading()));
                }
            }
        });
        found
    }

    pub(super) fn targets_helm(
        &self,
        u: i32,
        v: i32,
        layer_elev: f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> bool {
        let Some(target_med) = self.helm_medium() else {
            return false;
        };
        if self.grid.any_at(u, v, |b| b.occupies_layer(layer_elev) && b.block_medium() == target_med) {
            return true;
        }
        (1..=28).any(|step| {
            let sample = ray.sample_point((step as f32) * 0.06);
            let (su, sv) = frame.decompose(sample);
            let (cu, cv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            let ray_elev = frame.elevation_of(sample);
            self.grid.any_at(cu, cv, |b| {
                b.block_medium() == target_med && ray_elev >= b.floor() - 0.15 && ray_elev <= b.ceiling() + 1.15
            })
        })
    }

    pub(super) fn drivable(&self) -> bool {
        let (mut seat, mut wheel, mut steering, mut console) = (false, false, false, false);
        let mut nozzle = false;
        self.grid.visit_cells(|_, list| {
            for block in list {
                match block.block_medium() {
                    BlockMedium::CabinSeat => seat = true,
                    BlockMedium::CarWheel => wheel = true,
                    BlockMedium::SteeringWheel => steering = true,
                    BlockMedium::FlightConsole => console = true,
                    BlockMedium::RocketNozzle | BlockMedium::FusionTorch => nozzle = true,
                    _ => {}
                }
            }
        });
        if console {
            seat && nozzle
        } else {
            seat && steering && wheel
        }
    }
}
