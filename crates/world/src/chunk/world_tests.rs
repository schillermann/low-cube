use glam::Vec3;
use crate::{CountedMeshTarget, GravityField, MeshTarget, PlanetaryWorld, Quad};
use super::{VolumePlanet, VOLUME_EDGE_LENGTH};

struct SilentCollector;

impl MeshTarget for SilentCollector {
    fn push_quad(&mut self, _quad: Quad) {}
}

#[test]
fn test_volume_planet_scale_verification_128_to_1024() {
    // 128^3 Pocket Moonlet
    let moonlet = VolumePlanet::of_size(128);
    moonlet.inspect_grid(|grid| {
        assert_eq!(grid.total_volume_count(), 512);
        assert_eq!(grid.hollow_core_volume_count(), 216);
    });

    // 512^3 Dwarf World
    let dwarf = VolumePlanet::of_size(512);
    let side_volumes_dwarf = 512 / VOLUME_EDGE_LENGTH;
    dwarf.inspect_grid(|grid| {
        assert_eq!(grid.total_volume_count(), side_volumes_dwarf * side_volumes_dwarf * side_volumes_dwarf);
        assert!(grid.hollow_core_volume_count() > 20_000);
    });

    // 1024^3 Standard Terrestrial World (64x64x64 volumes, 2-volume crust -> 60x60x60 core)
    let terrestrial = VolumePlanet::of_size(1024);
    terrestrial.inspect_grid(|grid| {
        assert_eq!(grid.total_volume_count(), 262_144);
        assert_eq!(grid.hollow_core_volume_count(), 216_000);
        let void_ratio = 216_000.0 / 262_144.0;
        assert!(void_ratio > 0.82, "Hollow core void must exceed 82%");
    });
}

#[test]
fn test_volume_planet_gravity_and_surface_constraint() {
    let world = VolumePlanet::of_size(128);

    // Gravity pull & surface up across faces
    assert_eq!(world.surface_up_at(Vec3::new(0.0, 70.0, 0.0)), Vec3::Y);
    assert_eq!(world.pull_direction_at(Vec3::new(0.0, 70.0, 0.0)), Vec3::NEG_Y);

    assert_eq!(world.surface_up_at(Vec3::new(70.0, 0.0, 0.0)), Vec3::X);
    assert_eq!(world.pull_direction_at(Vec3::new(70.0, 0.0, 0.0)), Vec3::NEG_X);

    assert_eq!(world.surface_up_at(Vec3::new(0.0, 0.0, -70.0)), Vec3::NEG_Z);
    assert_eq!(world.pull_direction_at(Vec3::new(0.0, 0.0, -70.0)), Vec3::Z);

    // Surface constraint maintains avatar above terrain
    let constrained = world.constrain_to_surface(Vec3::new(0.0, 60.0, 0.0), 1.0);
    assert_eq!(constrained.y, 65.0); // half_size (64.0) + player_height (1.0)
}

#[test]
fn test_volume_planet_mesh_emission() {
    // 32-block test world
    let world = VolumePlanet::of_size(32);
    let mut collector = SilentCollector;
    let mut counted = CountedMeshTarget::new(&mut collector);
    world.emit_surface(&mut counted);
    assert!(counted.count() > 0, "Emitted surface quads must be positive");
}
