use glam::Vec3;
use super::climate_zone::ClimateZone;

/// Encapsulated 3D spatial position folded onto a destination planetary climate zone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldedPoint {
    zone: ClimateZone,
    point: Vec3,
}

impl FoldedPoint {
    /// Primary constructor assigning destination climate zone and folded spatial position directly.
    pub fn new(zone: ClimateZone, point: Vec3) -> Self {
        Self { zone, point }
    }

    /// Destination climate zone of the folded point.
    pub fn zone(&self) -> ClimateZone {
        self.zone
    }

    /// 3D spatial position in world coordinates.
    pub fn point(&self) -> Vec3 {
        self.point
    }

    /// Surface normal of the destination climate zone.
    pub fn surface_up(&self) -> Vec3 {
        self.zone.normal()
    }

    /// Pull direction of planetary gravity in the destination climate zone.
    pub fn pull_direction(&self) -> Vec3 {
        -self.zone.normal()
    }

    /// Predicate verifying whether this folded point resides in the given climate zone.
    pub fn in_zone(&self, zone: ClimateZone) -> bool {
        self.zone == zone
    }

    /// Euclidean distance from this point to an arbitrary spatial target.
    pub fn distance_to(&self, target: Vec3) -> f32 {
        (self.point - target).length()
    }

    /// Feeds encapsulated climate zone and point into a consumer without extracting raw data ("Tell, Don't Ask").
    pub fn transfer<T>(&self, consumer: impl FnOnce(ClimateZone, Vec3) -> T) -> T {
        consumer(self.zone, self.point)
    }
}
