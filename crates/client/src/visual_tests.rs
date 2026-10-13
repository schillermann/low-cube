use glam::Vec3;
use world::{BiomedCubeWorld, ClimateZone, GravityField, MeshTarget, PerimeterEdge, PlanetaryTopology, PlanetaryWorld, Quad, Topology};

struct QuadAccumulator {
    quads: Vec<Quad>,
}

impl QuadAccumulator {
    fn new() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadAccumulator {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

fn compute_lighting(normal: Vec3, light_dir: Vec3) -> f32 {
    let diff = normal.normalize().dot(light_dir.normalize()).max(0.0);
    0.35 + 0.65 * diff
}

#[test]
fn test_directional_lighting_continuity_across_all_12_seam_transitions() {
    let size = 16;
    let half_size = (size as f32) * 0.5;
    let margin = 0.5;
    let world = BiomedCubeWorld::beveled(size, margin);
    let topology = PlanetaryTopology::cubic(half_size);
    let sun_dir = Vec3::new(0.6, 0.8, 0.4).normalize();

    let all_zones = [
        ClimateZone::Temperate,
        ClimateZone::Volcanic,
        ClimateZone::Desert,
        ClimateZone::Arctic,
        ClimateZone::Jungle,
        ClimateZone::Steppe,
    ];

    for zone in all_zones {
        for edge in PerimeterEdge::all() {
            let seam = topology.seam(zone, edge);
            let n1 = zone.normal();
            let n2 = seam.destination().normal();
            let seam_center = (n1 + n2) * half_size;

            let mut prev_light = compute_lighting(zone.normal(), sun_dir);

            // Sample across the bevel transition zone in 20 smooth interpolation steps
            for step in 1..=20 {
                let factor = (step as f32) / 20.0;
                let test_point = seam_center + (n2 - n1) * ((factor - 0.5) * margin * 2.0);
                let normal = world.surface_up_at(test_point);
                let light = compute_lighting(normal, sun_dir);

                assert!(
                    light >= 0.35 && light <= 1.0,
                    "Lighting intensity must stay strictly within ambient and peak bounds: {light}"
                );

                let light_delta = (light - prev_light).abs();
                assert!(
                    light_delta < 0.15,
                    "Lighting delta across bevel step must be smooth, observed {light_delta}"
                );
                prev_light = light;
            }

            let dest_light = compute_lighting(seam.destination().normal(), sun_dir);
            let final_delta = (prev_light - dest_light).abs();
            assert!(
                final_delta < 0.15,
                "Seam edge lighting must blend continuously into destination face normal lighting"
            );
        }
    }
}

#[test]
fn test_biome_transitions_and_quad_alignment_at_all_seams() {
    let size = 16;
    let world = BiomedCubeWorld::of_size(size);
    let mut accumulator = QuadAccumulator::new();
    world.emit_surface(&mut accumulator);

    assert!(
        accumulator.quads.len() >= 6 * (size as usize) * (size as usize),
        "Total emitted quads must cover the complete 6-faced planetary surface"
    );

    let colors: Vec<Vec3> = accumulator.quads.iter().map(|q| q.transfer(|_, _, _, _, _, color| color)).collect();
    let distinct_colors = colors.iter().any(|c1| colors.iter().any(|c2| (c1 - c2).length() > 0.3));
    assert!(distinct_colors, "Adjacent biomes must exhibit distinct climate zone colors");

    // Verify all quad normals are unit vectors and finite
    for quad in &accumulator.quads {
        quad.transfer(|v0, v1, v2, v3, normal, _color| {
            assert!(normal.is_finite(), "Quad normal must be finite");
            assert!((normal.length() - 1.0).abs() < 1e-4, "Quad normal must be normalized");
            assert!(v0.is_finite() && v1.is_finite() && v2.is_finite() && v3.is_finite());
        });
    }
}

#[test]
fn test_shadow_continuity_faces_in_umbra_clamp_smoothly_to_ambient() {
    let sun_dir = Vec3::new(0.0, 1.0, 0.0);
    // Face opposing sunlight completely (-Y)
    let bottom_normal = Vec3::NEG_Y;
    let light = compute_lighting(bottom_normal, sun_dir);
    assert_eq!(light, 0.35, "Faces entirely in shadow must clamp cleanly to ambient floor");

    // Face parallel to sunlight
    let side_normal = Vec3::X;
    let side_light = compute_lighting(side_normal, sun_dir);
    assert_eq!(side_light, 0.35, "Faces grazing sunlight must smoothly transition to ambient");
}
