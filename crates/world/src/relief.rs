use glam::Vec3;
use crate::{Face, MeshTarget, Quad};

/// Interface representing topological elevation above the planar base of a face.
pub trait TerrainElevation: Send + Sync {
    fn elevation_at(&self, u: f32, v: f32) -> f32;
}

/// Pyramidal mountain ridge composed of walkable 45° slopes (Wedges) and a terraced rock shelf (Slab).
pub struct RidgeElevation {
    center_u: f32,
    center_v: f32,
    base_radius: f32,
    terrace_radius: f32,
    terrace_height: f32,
    summit_radius: f32,
    shelf_height: f32,
}

impl RidgeElevation {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        center_u: f32,
        center_v: f32,
        base_radius: f32,
        terrace_radius: f32,
        terrace_height: f32,
        summit_radius: f32,
        shelf_height: f32,
    ) -> Self {
        Self {
            center_u,
            center_v,
            base_radius,
            terrace_radius,
            terrace_height,
            summit_radius,
            shelf_height,
        }
    }

    /// Secondary constructor creating an alpine mountain massif ahead of the spawn point.
    pub fn alpine(center_u: f32, center_v: f32) -> Self {
        Self::new(center_u, center_v, 3.0, 1.5, 2.0, 0.8, 0.5)
    }
}

impl TerrainElevation for RidgeElevation {
    fn elevation_at(&self, u: f32, v: f32) -> f32 {
        let du = (u - self.center_u).abs();
        let dv = (v - self.center_v).abs();
        let d = du.max(dv);

        if d >= self.base_radius {
            0.0
        } else if d >= self.terrace_radius {
            // Walkable 45° slope rising continuously up to terrace height
            let fraction = (self.base_radius - d) / (self.base_radius - self.terrace_radius);
            fraction * self.terrace_height
        } else if d >= self.summit_radius {
            // Terrace plateau
            self.terrace_height
        } else {
            // Summit rock shelf accessible via realistic 0.5m human jump
            self.terrace_height + self.shelf_height
        }
    }
}

/// Planetary face featuring faceted 3D terrain with slopes, ramps, and rock shelves.
pub struct ReliefFace {
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
    color: Vec3,
    half_size: f32,
    divisions: u32,
    elevation: Box<dyn TerrainElevation>,
}

impl ReliefFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        color: Vec3,
        half_size: f32,
        divisions: u32,
        elevation: Box<dyn TerrainElevation>,
    ) -> Self {
        Self {
            normal,
            tangent,
            bitangent,
            color,
            half_size,
            divisions,
            elevation,
        }
    }
}

impl Face for ReliefFace {
    fn matches(&self, point: Vec3) -> bool {
        let height = point.dot(self.normal);
        if height <= 0.0 {
            return false;
        }

        let u = point.dot(self.tangent).abs();
        let v = point.dot(self.bitangent).abs();

        height >= u && height >= v
    }

    fn surface_up(&self) -> Vec3 {
        self.normal
    }

    fn pull_direction(&self) -> Vec3 {
        -self.normal
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let u = point.dot(self.tangent);
        let v = point.dot(self.bitangent);
        let elev = self.elevation.elevation_at(u, v);
        let h = self.half_size + elev + distance;

        self.normal * h + self.tangent * u + self.bitangent * v
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let step = (self.half_size * 2.0) / (self.divisions as f32);
        let base_center = self.normal * self.half_size;

        for i in 0..self.divisions {
            for j in 0..self.divisions {
                let u0 = -self.half_size + (i as f32) * step;
                let u1 = u0 + step;
                let v0 = -self.half_size + (j as f32) * step;
                let v1 = v0 + step;

                let h0 = self.elevation.elevation_at(u0, v0);
                let h1 = self.elevation.elevation_at(u1, v0);
                let h2 = self.elevation.elevation_at(u1, v1);
                let h3 = self.elevation.elevation_at(u0, v1);

                let p0 = base_center + self.tangent * u0 + self.bitangent * v0 + self.normal * h0;
                let p1 = base_center + self.tangent * u1 + self.bitangent * v0 + self.normal * h1;
                let p2 = base_center + self.tangent * u1 + self.bitangent * v1 + self.normal * h2;
                let p3 = base_center + self.tangent * u0 + self.bitangent * v1 + self.normal * h3;

                let edge_a = p1 - p0;
                let edge_b = p3 - p0;
                let cross = edge_a.cross(edge_b);
                let facet_normal = if cross.length_squared() > 1e-6 {
                    cross.normalize()
                } else {
                    self.normal
                };

                let pattern = ((i + j) % 2 == 0) as i32 as f32 * 0.08;
                let height_tint = ((h0 + h1 + h2 + h3) * 0.25) * 0.06;
                let quad_color = Vec3::new(
                    (self.color.x + pattern + height_tint).clamp(0.0, 1.0),
                    (self.color.y + pattern + height_tint * 0.8).clamp(0.0, 1.0),
                    (self.color.z + pattern + height_tint * 1.2).clamp(0.0, 1.0),
                );

                target.push_quad(Quad::new(p0, p1, p2, p3, facet_normal, quad_color));
            }
        }
    }
}
