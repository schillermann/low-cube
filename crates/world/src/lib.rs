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

/// Pure geometric planar face operating strictly on coordinate bases without integer switches.
pub struct PlanarFace {
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
    color: Vec3,
}

impl PlanarFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(normal: Vec3, tangent: Vec3, bitangent: Vec3, color: Vec3) -> Self {
        Self {
            normal,
            tangent,
            bitangent,
            color,
        }
    }
}

impl Face for PlanarFace {
    fn matches(&self, point: Vec3) -> bool {
        let height = point.dot(self.normal);
        if height <= 0.0 {
            return false;
        }

        let u = point.dot(self.tangent).abs();
        let v = point.dot(self.bitangent).abs();

        height >= u && height >= v
    }

    fn normal(&self) -> Vec3 {
        self.normal
    }

    fn constrain(&self, point: Vec3, half_size: f32, distance: f32) -> Vec3 {
        let u = point.dot(self.tangent).clamp(-half_size, half_size);
        let v = point.dot(self.bitangent).clamp(-half_size, half_size);
        let h = half_size + distance;

        self.normal * h + self.tangent * u + self.bitangent * v
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

/// A cubic planetary world composed of polymorphic Face instances with Dependency Injection.
pub struct CubePlanet {
    half_size: f32,
    divisions: u32,
    faces: Vec<Box<dyn Face>>,
}

impl CubePlanet {
    /// Primary constructor: code-free, assigns fields directly.
    pub fn new(half_size: f32, divisions: u32, faces: Vec<Box<dyn Face>>) -> Self {
        Self {
            half_size,
            divisions,
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

/// Factory constructing standard cubic planetary worlds.
pub struct StandardPlanetFactory;

impl StandardPlanetFactory {
    pub fn create_cube(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let faces: Vec<Box<dyn Face>> = vec![
            // Top (+Y): Temperate Green
            Box::new(PlanarFace::new(
                Vec3::Y,
                Vec3::X,
                Vec3::Z,
                Vec3::new(0.28, 0.65, 0.25),
            )),
            // Bottom (-Y): Volcanic Dark Red
            Box::new(PlanarFace::new(
                Vec3::NEG_Y,
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::new(0.35, 0.18, 0.18),
            )),
            // Right (+X): Desert Sand
            Box::new(PlanarFace::new(
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::Y,
                Vec3::new(0.85, 0.75, 0.35),
            )),
            // Left (-X): Arctic Ice
            Box::new(PlanarFace::new(
                Vec3::NEG_X,
                Vec3::Z,
                Vec3::Y,
                Vec3::new(0.75, 0.88, 0.95),
            )),
            // Front (+Z): Jungle Emerald
            Box::new(PlanarFace::new(
                Vec3::Z,
                Vec3::X,
                Vec3::Y,
                Vec3::new(0.12, 0.45, 0.22),
            )),
            // Back (-Z): Arid Steppe Brown
            Box::new(PlanarFace::new(
                Vec3::NEG_Z,
                Vec3::NEG_X,
                Vec3::Y,
                Vec3::new(0.68, 0.48, 0.32),
            )),
        ];

        CubePlanet::new(half, size_blocks, faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_orientation_across_faces() {
        let planet = StandardPlanetFactory::create_cube(16);

        // Top face (+Y)
        assert_eq!(planet.surface_up_at(Vec3::new(0.0, 10.0, 0.0)), Vec3::Y);
        assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 10.0, 0.0)), Vec3::NEG_Y);

        // Right face (+X)
        assert_eq!(planet.surface_up_at(Vec3::new(10.0, 0.0, 0.0)), Vec3::X);
        assert_eq!(planet.pull_direction_at(Vec3::new(10.0, 0.0, 0.0)), Vec3::NEG_X);

        // Front face (+Z)
        assert_eq!(planet.surface_up_at(Vec3::new(0.0, 0.0, 10.0)), Vec3::Z);
        assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 0.0, 10.0)), Vec3::NEG_Z);

        // Bottom face (-Y)
        assert_eq!(planet.surface_up_at(Vec3::new(0.0, -10.0, 0.0)), Vec3::NEG_Y);
        assert_eq!(planet.pull_direction_at(Vec3::new(0.0, -10.0, 0.0)), Vec3::Y);

        // Left face (-X)
        assert_eq!(planet.surface_up_at(Vec3::new(-10.0, 0.0, 0.0)), Vec3::NEG_X);
        assert_eq!(planet.pull_direction_at(Vec3::new(-10.0, 0.0, 0.0)), Vec3::X);

        // Back face (-Z)
        assert_eq!(planet.surface_up_at(Vec3::new(0.0, 0.0, -10.0)), Vec3::NEG_Z);
        assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 0.0, -10.0)), Vec3::Z);
    }

    #[test]
    fn test_edge_transition_switch() {
        let planet = StandardPlanetFactory::create_cube(16);

        // On top face near edge: Y = 9.0, Z = 7.9 -> Dominant is Y
        assert_eq!(planet.surface_up_at(Vec3::new(0.0, 9.0, 7.9)), Vec3::Y);

        // Over the edge on front face: Y = 7.9, Z = 9.0 -> Dominant is Z
        assert_eq!(planet.surface_up_at(Vec3::new(0.0, 7.9, 9.0)), Vec3::Z);
    }
}
