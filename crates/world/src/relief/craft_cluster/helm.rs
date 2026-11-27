use crate::block::{BlockMedium, GridCell, WedgeHeading};
use crate::brick_arena::{BrickCell, BrickStructure, BrickVolume};
use crate::frame::SurfaceFrame;
use super::super::sight_ray::SurfaceSightRay;

/// Focused cockpit helm inspector distinguishing rocket consoles (`FlightConsole`/`PhosphorConsole`) from car wheels (`SteeringWheel`).
pub(super) struct ClusterHelm<'a> {
    grid: &'a BrickStructure,
}

impl<'a> ClusterHelm<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(grid: &'a BrickStructure) -> Self {
        Self { grid }
    }

    fn helm_medium(&self) -> Option<BlockMedium> {
        let (mut console, mut phos, mut wheel) = (false, false, false);
        self.grid.bricks().inner().visit_occupied_cells(|_, cell| match cell {
            BrickCell::RawAtom(a) => match a.medium() {
                BlockMedium::FlightConsole => console = true,
                BlockMedium::SteeringWheel => wheel = true,
                _ => {}
            },
            BrickCell::ModuleAnchor(s) if s.medium() == BlockMedium::PhosphorConsole => phos = true,
            _ => {}
        });
        if console {
            Some(BlockMedium::FlightConsole)
        } else if phos {
            Some(BlockMedium::PhosphorConsole)
        } else if wheel {
            Some(BlockMedium::SteeringWheel)
        } else {
            None
        }
    }

    pub(super) fn unit(&self) -> Option<(i32, i32, f32, WedgeHeading)> {
        let target_med = self.helm_medium()?;
        let datum = self.grid.datum();
        let mut found = None;
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            let (cu, cv) = coord.planar();
            let floor = coord.floor_elev(datum);
            match cell {
                BrickCell::RawAtom(a) if a.medium() == target_med => {
                    found = Some((cu, cv, floor, a.heading()));
                }
                BrickCell::ModuleAnchor(s) if s.medium() == target_med => {
                    found = Some((cu, cv, floor, WedgeHeading::from_code(s.turn().code())));
                }
                _ => {}
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
        if super::super::molded_structure::MoldedStructure::medium_at(self.grid, u, v, layer_elev) == Some(target_med) {
            return true;
        }
        let datum = self.grid.datum();
        (1..=28).any(|step| {
            let sample = ray.sample_point((step as f32) * 0.06);
            let (su, sv) = frame.decompose(sample);
            let (cu, cv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            let ray_elev = frame.elevation_of(sample);
            self.grid.bricks().inner().any_in_column(cu, cv, |coord, cell| {
                let med = match cell {
                    BrickCell::Empty => None,
                    BrickCell::RawAtom(a) => Some(a.medium()),
                    BrickCell::ModuleAnchor(s) => Some(s.medium()),
                    BrickCell::ModuleProxy(_) => self.grid.bricks().resolve_module(coord).map(|(_, s)| s.medium()),
                };
                let floor = coord.floor_elev(datum);
                let ceil = coord.ceiling_elev(datum);
                med == Some(target_med) && ray_elev >= floor - 0.15 && ray_elev <= ceil + 1.15
            })
        })
    }

    pub(super) fn drivable(&self) -> bool {
        let (mut seat, mut wheel, mut steering, mut console, mut nozzle) = (false, false, false, false, false);
        self.grid.bricks().inner().visit_occupied_cells(|_, cell| {
            let med = match cell {
                BrickCell::RawAtom(a) => Some(a.medium()),
                BrickCell::ModuleAnchor(s) => Some(s.medium()),
                _ => None,
            };
            match med {
                Some(BlockMedium::CabinSeat | BlockMedium::PilotSeat) => seat = true,
                Some(BlockMedium::CarWheel) => wheel = true,
                Some(BlockMedium::SteeringWheel) => steering = true,
                Some(BlockMedium::FlightConsole | BlockMedium::PhosphorConsole) => console = true,
                Some(BlockMedium::RocketNozzle | BlockMedium::FusionTorch | BlockMedium::PropulsionBell) => nozzle = true,
                _ => {}
            }
        });
        if console { seat && nozzle } else { seat && steering && wheel }
    }
}
