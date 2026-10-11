use glam::Vec3;

mod face;
mod planet;

#[cfg(test)]
mod tests;

pub use face::PlanarFace;
pub use planet::{CubePlanet, StandardPlanetFactory};

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
