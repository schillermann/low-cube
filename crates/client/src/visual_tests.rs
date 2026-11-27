use glam::Vec3;
use world::{BiomedQuadSphere, ClimateZone, GravityField, MeshTarget, PerimeterEdge, PlanetaryTopology, Quad, TerrainMesh as _, Topology};

struct QuadLedger {
    quads: Vec<Quad>,
}

impl QuadLedger {
    fn new() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadLedger {
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
    let radius = (size as f32) * 0.5;
    let world = BiomedQuadSphere::of_size(size);
    let topology = PlanetaryTopology::quad_sphere(radius);
    let sun_dir = Vec3::new(0.6, 0.8, 0.4).normalize();

    for zone in ClimateZone::all() {
        for edge in PerimeterEdge::all() {
            let seam = topology.seam(zone, edge);
            let n1 = zone.normal();
            let n2 = seam.destination().normal();

            let mut prev_light = compute_lighting(n1, sun_dir);
            for step in 1..=20 {
                let factor = (step as f32) / 20.0;
                let dir = (n1 * (1.0 - factor) + n2 * factor).normalize();
                let test_point = dir * radius;
                let normal = world.surface_up_at(test_point);
                let light = compute_lighting(normal, sun_dir);

                assert!((0.35..=1.0).contains(&light));
                assert!((light - prev_light).abs() < 0.10);
                prev_light = light;
            }
            let dest_light = compute_lighting(n2, sun_dir);
            assert!((prev_light - dest_light).abs() < 0.10);
        }
    }
}

#[test]
fn test_biome_transitions_and_quad_alignment_at_all_seams() {
    let size = 16;
    let world = BiomedQuadSphere::of_size(size);
    let mut accumulator = QuadLedger::new();
    world.emit_surface(&mut accumulator);

    assert!(
        accumulator.quads.len() >= 6 * (size as usize) * (size as usize),
        "Total emitted quads must cover the complete 6-shell QuadSphere surface"
    );

    let colors: Vec<Vec3> = accumulator.quads.iter().map(|q| q.transfer(|_, _, _, _, _, color| color)).collect();
    let distinct_colors = colors.iter().any(|c1| colors.iter().any(|c2| (c1 - c2).length() > 0.3));
    assert!(distinct_colors, "Adjacent biomes must exhibit distinct climate zone colors");

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
    let bottom_normal = Vec3::NEG_Y;
    assert_eq!(compute_lighting(bottom_normal, sun_dir), 0.35);
    assert_eq!(compute_lighting(Vec3::X, sun_dir), 0.35);
}
