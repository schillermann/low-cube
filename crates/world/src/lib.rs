use glam::Vec3;

/// Interface for any consumer that collects generated surface quads.
/// Implements "Tell, Don't Ask" - the world pushes geometry rather than exposing internal arrays.
pub trait SurfaceMeshReceiver {
    fn push_quad(
        &mut self,
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        normal: Vec3,
        color: Vec3,
    );
}

/// Interface for gravity and orientation calculation around planetary bodies.
pub trait GravityField {
    fn pull_direction_at(&self, position: Vec3) -> Vec3;
    fn surface_up_at(&self, position: Vec3) -> Vec3;
}

/// Core interface for a planetary voxel world.
pub trait PlanetaryWorld: GravityField {
    fn emit_surface(&self, receiver: &mut dyn SurfaceMeshReceiver);
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3;
}

/// Represents the six distinct faces of a cubic world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubeFace {
    PosY, // Top (Temperate Forest / Grass)
    NegY, // Bottom (Volcanic Crag)
    PosX, // Right (Scorched Desert)
    NegX, // Left (Frozen Tundra / Arctic)
    PosZ, // Front (Dense Jungle)
    NegZ, // Back (Arid Steppe)
}

impl CubeFace {
    pub fn normal(&self) -> Vec3 {
        match self {
            Self::PosY => Vec3::Y,
            Self::NegY => Vec3::NEG_Y,
            Self::PosX => Vec3::X,
            Self::NegX => Vec3::NEG_X,
            Self::PosZ => Vec3::Z,
            Self::NegZ => Vec3::NEG_Z,
        }
    }

    pub fn base_color(&self) -> Vec3 {
        match self {
            Self::PosY => Vec3::new(0.28, 0.65, 0.25), // Green
            Self::NegY => Vec3::new(0.35, 0.18, 0.18), // Dark Volcanic Red
            Self::PosX => Vec3::new(0.85, 0.75, 0.35), // Desert Sand
            Self::NegX => Vec3::new(0.75, 0.88, 0.95), // Arctic Ice
            Self::PosZ => Vec3::new(0.12, 0.45, 0.22), // Jungle Emerald
            Self::NegZ => Vec3::new(0.68, 0.48, 0.32), // Arid Steppe Brown
        }
    }
}

/// A cubic planetary world body with 90-degree edge gravity.
pub struct CubePlanet {
    size_blocks: u32,
    half_size: f32,
}

impl CubePlanet {
    pub fn new(size_blocks: u32) -> Self {
        let half_size = (size_blocks as f32) * 0.5;
        Self {
            size_blocks,
            half_size,
        }
    }

    /// Determines the closest face based on position relative to cube center.
    pub fn dominant_face(&self, position: Vec3) -> CubeFace {
        let ax = position.x.abs();
        let ay = position.y.abs();
        let az = position.z.abs();

        if ay >= ax && ay >= az {
            if position.y >= 0.0 { CubeFace::PosY } else { CubeFace::NegY }
        } else if ax >= ay && ax >= az {
            if position.x >= 0.0 { CubeFace::PosX } else { CubeFace::NegX }
        } else {
            if position.z >= 0.0 { CubeFace::PosZ } else { CubeFace::NegZ }
        }
    }
}

impl GravityField for CubePlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.dominant_face(position).normal()
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.dominant_face(position).normal()
    }
}

impl PlanetaryWorld for CubePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        let face = self.dominant_face(position);
        let half = self.half_size;
        let target_dist = half + player_height;

        let mut constrained = position;
        match face {
            CubeFace::PosY => {
                constrained.y = target_dist;
                constrained.x = constrained.x.clamp(-half, half);
                constrained.z = constrained.z.clamp(-half, half);
            }
            CubeFace::NegY => {
                constrained.y = -target_dist;
                constrained.x = constrained.x.clamp(-half, half);
                constrained.z = constrained.z.clamp(-half, half);
            }
            CubeFace::PosX => {
                constrained.x = target_dist;
                constrained.y = constrained.y.clamp(-half, half);
                constrained.z = constrained.z.clamp(-half, half);
            }
            CubeFace::NegX => {
                constrained.x = -target_dist;
                constrained.y = constrained.y.clamp(-half, half);
                constrained.z = constrained.z.clamp(-half, half);
            }
            CubeFace::PosZ => {
                constrained.z = target_dist;
                constrained.x = constrained.x.clamp(-half, half);
                constrained.y = constrained.y.clamp(-half, half);
            }
            CubeFace::NegZ => {
                constrained.z = -target_dist;
                constrained.x = constrained.x.clamp(-half, half);
                constrained.y = constrained.y.clamp(-half, half);
            }
        }
        constrained
    }

    fn emit_surface(&self, receiver: &mut dyn SurfaceMeshReceiver) {
        let half = self.half_size;
        let count = self.size_blocks;
        let block_step = 1.0f32;

        let faces = [
            CubeFace::PosY,
            CubeFace::NegY,
            CubeFace::PosX,
            CubeFace::NegX,
            CubeFace::PosZ,
            CubeFace::NegZ,
        ];

        for face in &faces {
            let norm = face.normal();
            let base_col = face.base_color();

            for i in 0..count {
                for j in 0..count {
                    let u0 = -half + (i as f32) * block_step;
                    let u1 = u0 + block_step;
                    let v0 = -half + (j as f32) * block_step;
                    let v1 = v0 + block_step;

                    // Checker pattern modulation for voxel block grid visibility
                    let pattern = ((i + j) % 2 == 0) as i32 as f32 * 0.08;
                    let col = Vec3::new(
                        (base_col.x + pattern).clamp(0.0, 1.0),
                        (base_col.y + pattern).clamp(0.0, 1.0),
                        (base_col.z + pattern).clamp(0.0, 1.0),
                    );

                    let (p0, p1, p2, p3) = match face {
                        CubeFace::PosY => (
                            Vec3::new(u0, half, v0),
                            Vec3::new(u1, half, v0),
                            Vec3::new(u1, half, v1),
                            Vec3::new(u0, half, v1),
                        ),
                        CubeFace::NegY => (
                            Vec3::new(u0, -half, v1),
                            Vec3::new(u1, -half, v1),
                            Vec3::new(u1, -half, v0),
                            Vec3::new(u0, -half, v0),
                        ),
                        CubeFace::PosX => (
                            Vec3::new(half, v0, u0),
                            Vec3::new(half, v0, u1),
                            Vec3::new(half, v1, u1),
                            Vec3::new(half, v1, u0),
                        ),
                        CubeFace::NegX => (
                            Vec3::new(-half, v0, u1),
                            Vec3::new(-half, v0, u0),
                            Vec3::new(-half, v1, u0),
                            Vec3::new(-half, v1, u1),
                        ),
                        CubeFace::PosZ => (
                            Vec3::new(u1, v0, half),
                            Vec3::new(u0, v0, half),
                            Vec3::new(u0, v1, half),
                            Vec3::new(u1, v1, half),
                        ),
                        CubeFace::NegZ => (
                            Vec3::new(u0, v0, -half),
                            Vec3::new(u1, v0, -half),
                            Vec3::new(u1, v1, -half),
                            Vec3::new(u0, v1, -half),
                        ),
                    };

                    receiver.push_quad(p0, p1, p2, p3, norm, col);
                }
            }
        }
    }
}
