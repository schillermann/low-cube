use glam::Vec3;
use crate::{CubeFace, ElevatedFace, Face, MeshTarget, PlanarFace, Quad, TintedFace};

struct QuadReceiver {
    quads: Vec<Quad>,
}

impl MeshTarget for QuadReceiver {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_elevated_face_decorates_elevation_and_constraint() {
    let base_face = Box::new(PlanarFace::on_cube(
        CubeFace::Top,
        Vec3::new(0.5, 0.5, 0.5),
        8.0,
        2,
    ));
    let elevated = ElevatedFace::new(base_face, 2.5);

    // Baseline top face at y=8.0; player height 1.0; elevated by 2.5 -> y = 8.0 + 1.0 + 2.5 = 11.5
    let constrained = elevated.constrain(Vec3::new(0.0, 5.0, 0.0), 1.0);
    assert_eq!(constrained.y, 11.5);

    let mut receiver = QuadReceiver { quads: Vec::new() };
    elevated.emit_surface(&mut receiver);
    assert!(!receiver.quads.is_empty());
    receiver.quads[0].transfer(|v0, _, _, _, normal, _| {
        assert_eq!(normal, Vec3::Y);
        // Base quad y was 8.0, elevated by 2.5 -> y = 10.5
        assert_eq!(v0.y, 10.5);
    });
}

#[test]
fn test_tinted_face_decorates_emitted_quad_color() {
    let base_face = Box::new(PlanarFace::on_cube(
        CubeFace::Top,
        Vec3::new(1.0, 1.0, 1.0),
        8.0,
        2,
    ));
    let tinted = TintedFace::new(base_face, Vec3::new(0.5, 0.25, 0.1));

    let mut receiver = QuadReceiver { quads: Vec::new() };
    tinted.emit_surface(&mut receiver);
    assert!(!receiver.quads.is_empty());
    receiver.quads[0].transfer(|_, _, _, _, _, color| {
        // Multiplied by tint (0.5, 0.25, 0.1)
        assert!(color.x <= 0.55 && color.x >= 0.45);
        assert!(color.y <= 0.30 && color.y >= 0.20);
        assert!(color.z <= 0.15 && color.z >= 0.05);
    });
}
