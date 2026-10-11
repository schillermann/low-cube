use glam::Vec3;

/// Interface for collecting surface quads ("Tell, Don't Ask").
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

/// Interface for planetary gravity fields.
pub trait GravityField {
    fn pull_direction_at(&self, position: Vec3) -> Vec3;
    fn surface_up_at(&self, position: Vec3) -> Vec3;
}

/// Core interface for a planetary voxel body.
pub trait PlanetaryWorld: GravityField {
    fn emit_surface(&self, receiver: &mut dyn SurfaceMeshReceiver);
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3;
}

/// Interface representing an individual face of a planetary body.
pub trait Face: Send + Sync {
    fn matches(&self, point: Vec3) -> bool;
    fn normal(&self) -> Vec3;
    fn constrain(&self, point: Vec3, half_size: f32, distance: f32) -> Vec3;
    fn emit_surface(&self, half_size: f32, divisions: u32, receiver: &mut dyn SurfaceMeshReceiver);
}

/// Encapsulated planar face implementing Face.
pub struct PlanarFace {
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
    color: Vec3,
    primary_axis: usize,
    positive_side: bool,
}

impl PlanarFace {
    pub fn new(
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        color: Vec3,
        primary_axis: usize,
        positive_side: bool,
    ) -> Self {
        Self {
            normal,
            tangent,
            bitangent,
            color,
            primary_axis,
            positive_side,
        }
    }
}

impl Face for PlanarFace {
    fn matches(&self, point: Vec3) -> bool {
        let abs_x = point.x.abs();
        let abs_y = point.y.abs();
        let abs_z = point.z.abs();

        let is_dominant = match self.primary_axis {
            0 => abs_x >= abs_y && abs_x >= abs_z,
            1 => abs_y >= abs_x && abs_y >= abs_z,
            _ => abs_z >= abs_x && abs_z >= abs_y,
        };

        if !is_dominant {
            return false;
        }

        let coord = match self.primary_axis {
            0 => point.x,
            1 => point.y,
            _ => point.z,
        };

        (coord >= 0.0) == self.positive_side
    }

    fn normal(&self) -> Vec3 {
        self.normal
    }

    fn constrain(&self, point: Vec3, half_size: f32, distance: f32) -> Vec3 {
        let mut result = point;
        let target_dist = if self.positive_side {
            half_size + distance
        } else {
            -(half_size + distance)
        };

        match self.primary_axis {
            0 => {
                result.x = target_dist;
                result.y = result.y.clamp(-half_size, half_size);
                result.z = result.z.clamp(-half_size, half_size);
            }
            1 => {
                result.y = target_dist;
                result.x = result.x.clamp(-half_size, half_size);
                result.z = result.z.clamp(-half_size, half_size);
            }
            _ => {
                result.z = target_dist;
                result.x = result.x.clamp(-half_size, half_size);
                result.y = result.y.clamp(-half_size, half_size);
            }
        }
        result
    }

    fn emit_surface(&self, half_size: f32, divisions: u32, receiver: &mut dyn SurfaceMeshReceiver) {
        let step = (half_size * 2.0) / (divisions as f32);
        let base_center = self.normal * half_size;

        for i in 0..divisions {
            for j in 0..divisions {
                let u0 = -half_size + (i as f32) * step;
                let u1 = u0 + step;
                let v0 = -half_size + (j as f32) * step;
                let v1 = v0 + step;

                let pattern = ((i + j) % 2 == 0) as i32 as f32 * 0.08;
                let quad_color = Vec3::new(
                    (self.color.x + pattern).clamp(0.0, 1.0),
                    (self.color.y + pattern).clamp(0.0, 1.0),
                    (self.color.z + pattern).clamp(0.0, 1.0),
                );

                let p0 = base_center + self.tangent * u0 + self.bitangent * v0;
                let p1 = base_center + self.tangent * u1 + self.bitangent * v0;
                let p2 = base_center + self.tangent * u1 + self.bitangent * v1;
                let p3 = base_center + self.tangent * u0 + self.bitangent * v1;

                receiver.push_quad(p0, p1, p2, p3, self.normal, quad_color);
            }
        }
    }
}

/// A cubic planetary world composed of polymorphic Face instances.
pub struct CubePlanet {
    half_size: f32,
    divisions: u32,
    faces: Vec<Box<dyn Face>>,
}

impl CubePlanet {
    pub fn new(size_blocks: u32) -> Self {
        let half = (size_blocks as f32) * 0.5;
        let faces: Vec<Box<dyn Face>> = vec![
            // Top (+Y): Temperate Green
            Box::new(PlanarFace::new(
                Vec3::Y,
                Vec3::X,
                Vec3::Z,
                Vec3::new(0.28, 0.65, 0.25),
                1,
                true,
            )),
            // Bottom (-Y): Volcanic Dark Red
            Box::new(PlanarFace::new(
                Vec3::NEG_Y,
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::new(0.35, 0.18, 0.18),
                1,
                false,
            )),
            // Right (+X): Desert Sand
            Box::new(PlanarFace::new(
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::Y,
                Vec3::new(0.85, 0.75, 0.35),
                0,
                true,
            )),
            // Left (-X): Arctic Ice
            Box::new(PlanarFace::new(
                Vec3::NEG_X,
                Vec3::Z,
                Vec3::Y,
                Vec3::new(0.75, 0.88, 0.95),
                0,
                false,
            )),
            // Front (+Z): Jungle Emerald
            Box::new(PlanarFace::new(
                Vec3::Z,
                Vec3::X,
                Vec3::Y,
                Vec3::new(0.12, 0.45, 0.22),
                2,
                true,
            )),
            // Back (-Z): Arid Steppe Brown
            Box::new(PlanarFace::new(
                Vec3::NEG_Z,
                Vec3::NEG_X,
                Vec3::Y,
                Vec3::new(0.68, 0.48, 0.32),
                2,
                false,
            )),
        ];

        Self {
            half_size: half,
            divisions: size_blocks,
            faces,
        }
    }

    fn active_face(&self, position: Vec3) -> &dyn Face {
        for face in &self.faces {
            if face.matches(position) {
                return face.as_ref();
            }
        }
        self.faces[0].as_ref()
    }
}

impl GravityField for CubePlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.active_face(position).normal()
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.active_face(position).normal()
    }
}

impl PlanetaryWorld for CubePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.active_face(position).constrain(position, self.half_size, player_height)
    }

    fn emit_surface(&self, receiver: &mut dyn SurfaceMeshReceiver) {
        for face in &self.faces {
            face.emit_surface(self.half_size, self.divisions, receiver);
        }
    }
}
