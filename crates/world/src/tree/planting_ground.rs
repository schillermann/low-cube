/// Environmental ground interface evaluating arboreal habitability and surface elevation ("Tell, Don't Ask").
pub trait PlantingGround: Send + Sync {
    /// Tells whether this ground location is receptive to hosting a tree specimen.
    fn receptive(&self, cell_u: i32, cell_v: i32, hash: i32) -> bool;

    /// Evaluates the ground elevation in meters at the given cell coordinates.
    fn ground_elevation(&self, cell_u: i32, cell_v: i32) -> f32;
}
