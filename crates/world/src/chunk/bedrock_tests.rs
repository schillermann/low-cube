use glam::Vec3;
use super::{BedrockShell, PlanetaryCrust, VoxelBarrier};

#[test]
fn test_bedrock_shell_containment_and_penetration_arrest() {
    // Core void half extent: 48.0m (e.g. 128m moonlet with 16m crust: 64 - 16 = 48m)
    // 1-block bedrock shell thickness: 1.0m -> boundary at 49.0m
    let shell = BedrockShell::boundary(48.0);

    // Point in crust at y = 55.0m: contained
    let crust_pos = Vec3::new(0.0, 55.0, 0.0);
    assert!(shell.contained(crust_pos));
    assert!(!shell.indestructible_at(crust_pos));

    // Point on top surface of bedrock shell at y = 49.0m: contained
    let surface_pos = Vec3::new(0.0, 49.0, 0.0);
    assert!(shell.contained(surface_pos));

    // Point inside bedrock block at y = 48.5m: indestructible bedrock!
    let bedrock_pos = Vec3::new(0.0, 48.5, 0.0);
    assert!(!shell.contained(bedrock_pos));
    assert!(shell.indestructible_at(bedrock_pos));

    // Point fallen into the hollow core void at y = 30.0m
    let void_pos = Vec3::new(0.0, 30.0, 0.0);
    assert!(!shell.contained(void_pos));
    assert!(!shell.indestructible_at(void_pos));

    // Arrest penetration from crust downward toward core
    let arrested = shell.arrest_penetration(crust_pos, void_pos);
    assert_eq!(arrested.y, 49.0);
    assert!(shell.contained(arrested));
}

#[test]
fn test_planetary_crust_scale_calibration() {
    let moonlet = PlanetaryCrust::calibrated(128);
    assert_eq!(moonlet.crust_depth_volumes(), 1);
    assert_eq!(moonlet.inner_void_half_extent(), 48.0);

    let terrestrial = PlanetaryCrust::calibrated(1024);
    assert_eq!(terrestrial.crust_depth_volumes(), 2);
    assert_eq!(terrestrial.inner_void_half_extent(), 480.0);

    let mega = PlanetaryCrust::calibrated(2048);
    assert_eq!(mega.crust_depth_volumes(), 3);
    assert_eq!(mega.inner_void_half_extent(), 976.0);

    let barrier = terrestrial.bedrock_barrier();
    assert!(barrier.contained(Vec3::new(0.0, 500.0, 0.0)));
    assert!(!barrier.contained(Vec3::new(0.0, 470.0, 0.0)));
}
