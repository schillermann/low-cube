use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{
    ClimateZone, Face, GravityField, MeshTarget, PlanarFace, PlanetaryWorld, Quad, StoneBlock,
    SurfaceFrame, VoxelCluster,
};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use super::{
    AttireProfile, BodyPose, FileAttire, SightMode, StrideRhythm, Wardrobe, YouthAttire,
    YouthPhysique,
};

struct QuadLedger {
    quads: Vec<Quad>,
}

impl QuadLedger {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }

    fn populated(&self) -> bool {
        !self.quads.is_empty()
    }

    fn exceeds(&self, other: &Self) -> bool {
        self.quads.len() > other.quads.len()
    }

    fn visible_in_frustum(&self, vp: glam::Mat4) -> usize {
        self.quads
            .iter()
            .filter(|q| {
                q.transfer(|v0, _, _, _, _, _| {
                    let clip = vp * v0.extend(1.0);
                    clip.w > 0.03
                        && (clip.x / clip.w).abs() < 1.0
                        && (clip.y / clip.w).abs() < 1.0
                        && (clip.z / clip.w).abs() <= 1.0
                })
            })
            .count()
    }
}

impl MeshTarget for QuadLedger {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

struct WallTestWorld {
    cluster: VoxelCluster,
}

impl WallTestWorld {
    fn with_cube_at(u: i32, v: i32) -> Self {
        let face = Box::new(PlanarFace::in_zone(ClimateZone::Temperate, Vec3::ONE, 8.0, 16));
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);
        let block = Box::new(StoneBlock::cube(u, v, 0.0));
        Self {
            cluster: VoxelCluster::new(face, frame, vec![block]),
        }
    }
}

impl GravityField for WallTestWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 {
        Vec3::NEG_Y
    }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 {
        Vec3::Y
    }
}

impl PlanetaryWorld for WallTestWorld {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.cluster.emit_surface(target);
    }
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.cluster.constrain(position, player_height)
    }
}

struct WalkEastControls;

impl KeyboardControls for WalkEastControls {
    fn register_key(&mut self, _code: KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, _forward: Vec3, right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(right);
    }
}

#[test]
fn test_body_envelope_prevents_camera_from_clipping_into_block_wall() {
    // Solid 1x1x1 stone cube occupies x in [1.0, 2.0], z in [0.0, 1.0]
    let world = WallTestWorld::with_cube_at(1, 0);
    let start = world.constrain_to_surface(Vec3::new(0.2, 5.0, 0.5), 1.0);
    let mut avatar = PlanetaryAvatar::at(start);
    let controls = WalkEastControls;

    // Walk continuously towards the block wall at x = 1.0 for 60 frames
    for _ in 0..60 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }

    let view_proj = avatar.view_projection_matrix(16.0 / 9.0);
    let cam_pos = view_proj.inverse().project_point3(Vec3::ZERO);

    // Camera must stop outside the block wall (x = 1.0) with safe clearance > 0.15m
    let wall_clearance = 1.0 - cam_pos.x;
    assert!(wall_clearance >= 0.15, "Avatar camera must maintain safe body clearance, got {}", wall_clearance);
}

#[test]
fn test_first_person_omits_head_while_multiplayer_and_mirror_include_crown() {
    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, 0.8,
        StrideRhythm::new(0.0, 1.0), YouthPhysique::SlenderSquire,
    );
    let attire = YouthAttire::squire();

    let mut fp_ledger = QuadLedger::empty();
    attire.emit(&pose, SightMode::FirstPersonSight, &mut fp_ledger);

    let mut mp_ledger = QuadLedger::empty();
    attire.emit_peer(&pose, &mut mp_ledger);

    assert!(fp_ledger.populated(), "First-person view must emit torso, legs, boots, and hands");
    assert!(mp_ledger.exceeds(&fp_ledger), "Multiplayer and Looking-Glass view must emit additional crown quads");
}

#[test]
fn test_hands_raised_only_when_holding_item_and_body_visible_when_looking_down() {
    let start = Vec3::new(0.0, 9.0, 0.0);
    let mut avatar = PlanetaryAvatar::at(start);

    let mut empty_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut empty_ledger);
    let vp_level = avatar.view_projection_matrix(16.0 / 9.0);
    let visible_empty_level = empty_ledger.visible_in_frustum(vp_level);
    assert_eq!(visible_empty_level, 0, "Empty hands must hang at the sides and not obstruct level forward view");

    // Equip a held item (KeyH): right hand must now be held up in level view
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyH));
    let mut held_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut held_ledger);
    let visible_held_level = held_ledger.visible_in_frustum(vp_level);
    assert!(visible_held_level > 0, "Holding an item must raise the hand into first-person level view");

    // Look down towards feet (positive dy rotates pitch downwards)
    avatar.rotate_view(0.0, 350.0);
    let vp_down = avatar.view_projection_matrix(16.0 / 9.0);
    let mut down_ledger = QuadLedger::empty();
    avatar.emit_appearance(&mut down_ledger);
    let visible_down = down_ledger.visible_in_frustum(vp_down);
    assert!(
        visible_down > visible_held_level,
        "Looking down must bring torso, belt, legs, and boots into the camera view frustum"
    );
}

#[test]
fn test_wardrobe_customization_keys_and_file_persistence() {
    let temp_path = std::env::temp_dir().join("test_medieval_character_profile.toml");
    let store = FileAttire::new(temp_path.to_str().unwrap().to_string());

    let squire = YouthAttire::squire();
    let initial_bytes = store.encode_multiplayer(&squire);

    let customized = squire
        .with_next_tunic()
        .with_next_vambrace()
        .with_next_girdle()
        .with_next_footwear()
        .with_next_crown()
        .with_next_relic()
        .with_next_pigment()
        .with_next_complexion();

    store.save(&customized);
    let loaded = store.load_or_squire();
    std::fs::remove_file(&temp_path).expect("Failed to clean up temporary profile");

    assert_eq!(customized, loaded);
    assert_ne!(initial_bytes, store.encode_multiplayer(&loaded));

    // Verify Avatar key dispatch for LookingGlass mirror toggle and relic holding
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let fp_matrix = avatar.view_projection_matrix(16.0 / 9.0);
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyM));
    let mirror_matrix = avatar.view_projection_matrix(16.0 / 9.0);
    assert_ne!(fp_matrix, mirror_matrix, "Looking-Glass mirror mode must shift camera in front of avatar");
}
