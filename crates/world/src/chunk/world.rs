use glam::Vec3;
use crate::{
    ClimateZone, FacePerimeter, GravityField, MeshTarget, PerimeterCrossing,
    PlanetaryTopology, PlanetaryWorld, Topology,
};
use super::{
    PlanetaryCrust, SparsePlanetaryGrid, StratifiedVolume, VolumeCoord,
    VoxelCoord, VOLUME_EDGE_LENGTH,
};

/// 3D Volume-partitioned cubic planetary world with an encapsulated hollow core void and bedrock shell.
pub struct VolumePlanet {
    half_size: f32,
    crust: PlanetaryCrust,
    grid: SparsePlanetaryGrid,
    topology: PlanetaryTopology,
}

impl VolumePlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        half_size: f32,
        crust: PlanetaryCrust,
        grid: SparsePlanetaryGrid,
        topology: PlanetaryTopology,
    ) -> Self {
        Self {
            half_size,
            crust,
            grid,
            topology,
        }
    }

    /// Secondary constructor assembling a volume-partitioned planet for given block dimensions.
    pub fn of_size(side_blocks: u32) -> Self {
        let side_volumes = (side_blocks as usize + VOLUME_EDGE_LENGTH - 1) / VOLUME_EDGE_LENGTH;
        let crust = PlanetaryCrust::calibrated(side_blocks as usize);
        let mut grid = SparsePlanetaryGrid::planetary(side_volumes, crust.crust_depth_volumes());

        // Populate surface crust layer across all 6 faces
        for cz in 0..side_volumes {
            for cy in 0..side_volumes {
                for cx in 0..side_volumes {
                    let coord = VolumeCoord::new(cx as i32, cy as i32, cz as i32);
                    if !grid.in_hollow_core(coord) {
                        let mut volume = StratifiedVolume::planetary();
                        let touches_pos_y = cy == side_volumes - 1;
                        let touches_neg_y = cy == 0;
                        let touches_pos_x = cx == side_volumes - 1;
                        let touches_neg_x = cx == 0;
                        let touches_pos_z = cz == side_volumes - 1;
                        let touches_neg_z = cz == 0;

                        if touches_pos_y || touches_neg_y || touches_pos_x || touches_neg_x || touches_pos_z || touches_neg_z {
                            for v1 in 0..VOLUME_EDGE_LENGTH {
                                for v2 in 0..VOLUME_EDGE_LENGTH {
                                    if touches_pos_y {
                                        volume.place(VoxelCoord::new(v1, VOLUME_EDGE_LENGTH - 1, v2), 1);
                                    }
                                    if touches_neg_y {
                                        volume.place(VoxelCoord::new(v1, 0, v2), 3);
                                    }
                                    if touches_pos_x {
                                        volume.place(VoxelCoord::new(VOLUME_EDGE_LENGTH - 1, v1, v2), 2);
                                    }
                                    if touches_neg_x {
                                        volume.place(VoxelCoord::new(0, v1, v2), 4);
                                    }
                                    if touches_pos_z {
                                        volume.place(VoxelCoord::new(v1, v2, VOLUME_EDGE_LENGTH - 1), 1);
                                    }
                                    if touches_neg_z {
                                        volume.place(VoxelCoord::new(v1, v2, 0), 5);
                                    }
                                }
                            }
                        }
                        grid.install_volume(coord, Box::new(volume));
                    }
                }
            }
        }

        let half_size = (side_blocks as f32) * 0.5;
        let topology = PlanetaryTopology::cubic(half_size);
        Self::new(half_size, crust, grid, topology)
    }

    /// Delivers the sparse grid to a consumer closure.
    pub fn inspect_grid<R>(&self, consumer: impl FnOnce(&SparsePlanetaryGrid) -> R) -> R {
        consumer(&self.grid)
    }
}

impl GravityField for VolumePlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        ClimateZone::dominant(position).normal()
    }
}

impl PlanetaryWorld for VolumePlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.grid.emit_planetary_surface(self.half_size, target);
    }

    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        let zone = ClimateZone::dominant(position);
        let frame = zone.surface_frame(self.half_size);
        let perimeter = FacePerimeter::square(self.half_size);
        let crossing = perimeter.crossing_point(&frame, position);

        let (u, v) = frame.decompose(position);
        let planar_point = frame.point_at(u, v, player_height);
        let contained = self.crust.bedrock_barrier().arrest_penetration(planar_point, planar_point);

        match crossing {
            PerimeterCrossing::Interior => contained,
            PerimeterCrossing::Crossing { edge, .. } => {
                let seam = self.topology.seam(zone, edge);
                seam.fold_point(contained)
            }
        }
    }
}
