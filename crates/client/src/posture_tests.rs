use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{GravityField, MeshTarget, PlanetaryWorld};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::basis::OrientationBasis;
use crate::controls::{KeyboardControls, KeyMotion};
use crate::gait::{LocomotionGait, SneakGait, WalkGait};
use crate::orientation::MouseLook;
use crate::posture::AvatarPosture;

struct FlatGroundWorld;

impl GravityField for FlatGroundWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 {
        Vec3::NEG_Y
    }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 {
        Vec3::Y
    }
}

impl PlanetaryWorld for FlatGroundWorld {
    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        Vec3::new(position.x, player_height, position.z)
    }
}

#[test]
fn test_posture_initial_standing() {
    let standing = AvatarPosture::standing();
    assert!(!standing.is_crouching());

    let crouching = AvatarPosture::crouching();
    assert!(crouching.is_crouching());
}

#[test]
fn test_posture_transitions_to_crouch_and_back() {
    let initial = AvatarPosture::standing();
    let sneak_gait = SneakGait::crouching();
    let walk_gait = WalkGait::upright();

    // Transitioning into crouch across multiple frames
    let mut posture = initial;
    for _ in 0..30 {
        posture = sneak_gait.shape_posture(posture, 1.0 / 60.0);
    }
    assert!(posture.is_crouching());

    // Transitioning back to upright standing
    for _ in 0..30 {
        posture = walk_gait.shape_posture(posture, 1.0 / 60.0);
    }
    assert!(!posture.is_crouching());
}

#[test]
fn test_posture_computes_distinct_view_matrices() {
    let look = MouseLook::idle();
    let basis = OrientationBasis::aligned(Vec3::Y, &look);
    let position = Vec3::new(0.0, 1.0, 0.0);

    let standing_view = AvatarPosture::standing().view_matrix(&basis, position);
    let crouching_view = AvatarPosture::crouching().view_matrix(&basis, position);

    assert_ne!(standing_view, crouching_view);
}

#[test]
fn test_avatar_lowers_viewpoint_when_sneaking() {
    let world = FlatGroundWorld;
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 1.0, 0.0));
    let mut controls = KeyMotion::idle();

    // Standing view projection matrix
    let standing_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    // Sneaking with Shift key lowers viewpoint into a crouch
    controls.register_key(KeyCode::ShiftLeft, true);
    for _ in 0..20 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let crouching_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    // View matrix changed due to lowered eye position
    assert_ne!(standing_matrix, crouching_matrix);

    // Releasing Shift restores upright viewpoint
    controls.register_key(KeyCode::ShiftLeft, false);
    for _ in 0..30 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let restored_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    let diff = (standing_matrix - restored_matrix).abs();
    for col in 0..4 {
        for row in 0..4 {
            assert!(diff.col(col)[row] < 1e-3);
        }
    }
}

#[test]
fn test_avatar_expands_fov_when_sprinting() {
    let world = FlatGroundWorld;
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 1.0, 0.0));
    let mut controls = KeyMotion::idle();

    let normal_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    // Double-tap W to initiate sprint
    controls.register_key(KeyCode::KeyW, true);
    controls.register_key(KeyCode::KeyW, false);
    controls.register_key(KeyCode::KeyW, true);

    for _ in 0..30 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let sprint_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    // Projection matrix changed due to expanded sprint FOV
    assert_ne!(normal_matrix, sprint_matrix);
    assert!((normal_matrix.col(0)[0] - sprint_matrix.col(0)[0]).abs() > 0.05);

    // Releasing W restores normal walking FOV
    controls.register_key(KeyCode::KeyW, false);
    for _ in 0..40 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let restored_matrix = avatar.view_projection_matrix(16.0 / 9.0);
    assert!((normal_matrix.col(0)[0] - restored_matrix.col(0)[0]).abs() < 1e-3);
}
