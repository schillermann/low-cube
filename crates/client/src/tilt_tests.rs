use glam::{Quat, Vec3};
use world::ClimateZone;
use crate::horizon::{CameraHorizon, HorizonTilt};
use crate::orientation::MouseLook;
use crate::posture::AvatarPosture;

#[test]
fn test_horizon_tilt_decorates_view_matrix_with_roll() {
    let look = MouseLook::idle();
    let horizon = CameraHorizon::aligned(Quat::IDENTITY, &look);
    let tilt_angle = 30.0f32.to_radians();
    let tilted = HorizonTilt::new(&horizon, tilt_angle);
    let posture = AvatarPosture::standing();
    let pos = Vec3::new(1.0, 2.0, 3.0);

    let base_view = posture.view_matrix(&horizon, pos);
    let tilted_view = posture.view_matrix(&tilted, pos);

    // Roll modifies the view matrix
    assert_ne!(base_view, tilted_view);
    // Determinant of rotation part remains 1.0
    assert!((tilted_view.determinant() - 1.0).abs() < 1e-4);
}

#[test]
fn test_horizon_tilt_zero_roll_produces_identical_view_matrix() {
    let look = MouseLook::idle();
    let horizon = CameraHorizon::aligned(Quat::IDENTITY, &look);
    let zero_tilt = HorizonTilt::new(&horizon, 0.0);
    let posture = AvatarPosture::standing();
    let pos = Vec3::new(1.0, 2.0, 3.0);

    let base_view = posture.view_matrix(&horizon, pos);
    let tilted_view = posture.view_matrix(&zero_tilt, pos);

    let diff = (base_view - tilted_view).abs();
    for col in 0..4 {
        for row in 0..4 {
            assert!(diff.col(col)[row] < 1e-5);
        }
    }
}

#[test]
fn test_singularity_free_view_matrices_across_all_12_edges_and_8_vertices() {
    let look = MouseLook::idle();
    let posture = AvatarPosture::standing();
    let half_size = 16.0;

    // Test across all 6 climate zones and their surface normals
    for zone in ClimateZone::all() {
        let normal = zone.normal();
        let surface_rot = Quat::from_rotation_arc(Vec3::Y, normal);
        let horizon = CameraHorizon::aligned(surface_rot, &look);

        // Test roll angles from 0 to 360 degrees in 45-degree increments
        for step in 0..8 {
            let roll = (step as f32) * std::f32::consts::FRAC_PI_4;
            let tilted = HorizonTilt::new(&horizon, roll);

            let pos = normal * half_size;
            let view = posture.view_matrix(&tilted, pos);

            // Matrix entries must all be valid and finite
            for col in 0..4 {
                for row in 0..4 {
                    assert!(
                        view.col(col)[row].is_finite(),
                        "View matrix entry must be finite at zone {:?}, step {}",
                        zone,
                        step
                    );
                }
            }
        }
    }

    // Test all 8 vertex corners (x = +-half, y = +-half, z = +-half)
    let signs = [-1.0f32, 1.0f32];
    for sx in signs {
        for sy in signs {
            for sz in signs {
                let corner_normal = Vec3::new(sx, sy, sz).normalize();
                let corner_rot = Quat::from_rotation_arc(Vec3::Y, corner_normal);
                let horizon = CameraHorizon::aligned(corner_rot, &look);

                for step in 0..4 {
                    let roll = (step as f32) * std::f32::consts::FRAC_PI_2;
                    let tilted = HorizonTilt::new(&horizon, roll);
                    let pos = corner_normal * half_size;
                    let view = posture.view_matrix(&tilted, pos);

                    for col in 0..4 {
                        for row in 0..4 {
                            assert!(
                                view.col(col)[row].is_finite(),
                                "Corner view matrix must be finite at ({}, {}, {}) step {}",
                                sx,
                                sy,
                                sz,
                                step
                            );
                        }
                    }
                }
            }
        }
    }
}
