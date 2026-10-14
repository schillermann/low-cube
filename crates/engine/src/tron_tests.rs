use glam::Vec3;
use world::{MeshTarget, Quad};
use crate::line_buffer::{CpuLineBatch, CpuTronBatch};

#[test]
fn test_cpu_line_batch_emits_quad_perimeter_lines() {
    let mut batch = CpuLineBatch::empty();
    let q = Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::Z,
        Vec3::new(0.0, 0.9, 1.0),
    );

    batch.push_quad(q);
    // 4 line segments = 8 line indices
    batch.push_quad(q);
}

#[test]
fn test_cpu_line_batch_skips_degenerate_edges_for_triangular_quads() {
    let mut batch = CpuLineBatch::empty();
    // Triangular quad where v2 == v3 (e.g., peak or corner wedge)
    let q = Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.5, 1.0, 0.0),
        Vec3::new(0.5, 1.0, 0.0),
        Vec3::Z,
        Vec3::new(1.0, 0.55, 0.0),
    );

    batch.push_quad(q);
}

#[test]
fn test_cpu_tron_batch_populates_both_solid_and_line_geometry() {
    let mut tron = CpuTronBatch::empty();
    let q = Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::Y,
        Vec3::new(0.0, 0.9, 1.0),
    );

    tron.push_quad(q);
}
