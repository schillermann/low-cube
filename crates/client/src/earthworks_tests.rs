use std::path::Path;
use glam::Vec3;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use crate::attire::{HeldRelic, YouthAttire};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{InputTrigger, KeyMotion};

struct SurfaceLedger { quads: Vec<Quad> }
impl SurfaceLedger {
    fn empty() -> Self { Self { quads: Vec::new() } }
    fn count(&self) -> usize { self.quads.len() }
}
impl MeshTarget for SurfaceLedger {
    fn push_quad(&mut self, quad: Quad) { self.quads.push(quad); }
}

fn check_rs_files_in_dir(dir: &Path) {
    for entry in std::fs::read_dir(dir).expect("read_dir failed").flatten() {
        let path = entry.path();
        if path.is_dir() {
            check_rs_files_in_dir(&path);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = std::fs::read_to_string(&path).expect("read_to_string failed");
            let line_count = content.lines().count();
            assert!(line_count <= 300, "{} exceeds 300 lines ({} lines)", path.display(), line_count);
            assert!(!content.contains(concat!("#[allow(", "dead_code)]")), "{} contains dead_code suppression", path.display());
            let mut in_struct = false;
            let mut struct_header = String::new();
            let mut field_count = 0usize;
            for line in content.lines() {
                let clean = line.split("//").next().unwrap_or("");
                if !in_struct && clean.contains("struct ") && clean.contains('{') {
                    let after_brace = clean.split('{').nth(1).unwrap_or("");
                    if let Some(inside) = after_brace.split('}').next() {
                        if clean.contains('}') {
                            let c = inside.replace("::", "").matches(':').count();
                            assert!(c <= 4, "{} inline struct exceeds 4 fields ({}): {}", path.display(), c, clean.trim());
                            continue;
                        }
                    }
                    in_struct = true;
                    struct_header = clean.trim().to_string();
                    field_count = 0;
                } else if in_struct {
                    if clean.contains('}') {
                        assert!(field_count <= 4, "{} exceeds 4 fields ({}): {}", path.display(), field_count, struct_header);
                        in_struct = false;
                    } else if clean.replace("::", "").contains(':') {
                        field_count += 1;
                    }
                }
            }
        }
    }
}

#[test]
fn test_workspace_architecture_guard_enforces_line_and_field_limits() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = manifest_dir.parent().expect("crates dir");
    check_rs_files_in_dir(&crates_dir.join("world/src"));
    check_rs_files_in_dir(&crates_dir.join("engine/src"));
    check_rs_files_in_dir(&crates_dir.join("client/src"));
}

#[test]
fn test_digging_speed_one_hand_vs_two_hands_vs_spade_and_abort_on_release() {
    let mut world_one_hand = BiomedCubeWorld::beveled(16, 1.0);
    let mut world_two_hands = BiomedCubeWorld::beveled(16, 1.0);
    let mut world_spade = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world_one_hand.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);
    let mut avatar_one = PlanetaryAvatar::at(spawn);
    let mut avatar_two = PlanetaryAvatar::at(spawn);
    let mut avatar_spade = PlanetaryAvatar::at(spawn);
    avatar_one.rotate_degrees(0.0, 65.3);
    avatar_one.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    for _ in 0..7 {
        assert!(!avatar_one.use_left_hand(&mut world_one_hand));
    }
    assert!(avatar_one.use_left_hand(&mut world_one_hand));

    avatar_two.rotate_degrees(0.0, 65.3);
    for _ in 0..4 {
        assert!(!avatar_two.use_right_hand(&mut world_two_hands));
        let _ = avatar_two.release_hand(&mut world_two_hands);
    }
    assert!(!avatar_two.use_left_hand(&mut world_two_hands) && !avatar_two.use_right_hand(&mut world_two_hands));
    for _ in 0..2 {
        assert!(!avatar_two.use_right_hand(&mut world_two_hands));
    }
    assert!(avatar_two.use_right_hand(&mut world_two_hands));

    avatar_spade.rotate_degrees(0.0, 70.5);
    avatar_spade.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    assert!(!avatar_spade.use_right_hand(&mut world_spade));
    assert!(avatar_spade.release_hand(&mut world_spade));
    assert!(!avatar_spade.use_right_hand(&mut world_spade));
    for _ in 0..40 {
        avatar_spade.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world_spade);
        let _ = avatar_spade.step_digging(&mut world_spade);
    }
    assert!(!avatar_spade.release_hand(&mut world_spade));
    let target_probe = Vec3::new(0.0, 9.0, -0.75);
    let height_after_spade = world_spade.constrain_to_surface(target_probe, 0.0).y;
    assert!(!avatar_spade.use_left_hand(&mut world_spade));
    for _ in 0..270 {
        avatar_spade.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world_spade);
        let _ = avatar_spade.step_digging(&mut world_spade);
    }
    assert!(!avatar_spade.release_hand(&mut world_spade));
    assert!((height_after_spade - world_spade.constrain_to_surface(target_probe, 0.0).y - 0.5).abs() < 1e-4);
    assert!(!avatar_spade.use_left_hand(&mut world_spade) && avatar_spade.release_hand(&mut world_spade));
}

#[test]
fn test_bare_hand_excavates_and_places_only_with_holding_hand() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let target_probe = Vec3::new(0.0, 9.0, -0.75);
    let initial_height = world.constrain_to_surface(target_probe, 0.0).y;
    let mut avatar = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.0, 16.5, 0.0), 1.0));
    avatar.rotate_degrees(0.0, 70.5);
    let (mut empty_hands_mesh, mut held_block_mesh, mut digging_with_block_mesh) =
        (SurfaceLedger::empty(), SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut empty_hands_mesh);

    assert!(!avatar.use_right_hand(&mut world) && !avatar.use_left_hand(&mut world));
    for _ in 0..2 {
        assert!(!avatar.use_left_hand(&mut world));
    }
    assert!(avatar.use_left_hand(&mut world));
    assert!((initial_height - world.constrain_to_surface(target_probe, 0.0).y - 0.5).abs() < 1e-4);
    avatar.emit_appearance(&mut held_block_mesh);
    assert!(held_block_mesh.count() > empty_hands_mesh.count() && !avatar.use_right_hand(&mut world));
    let _ = avatar.release_hand(&mut world);
    assert!(!avatar.use_left_hand(&mut world));
    avatar.emit_appearance(&mut digging_with_block_mesh);
    assert_eq!(digging_with_block_mesh.count(), empty_hands_mesh.count());
    assert!(avatar.release_hand(&mut world));
    assert!((world.constrain_to_surface(target_probe, 0.0).y - initial_height).abs() < 1e-4);
}

#[test]
fn test_mouse_wheel_molds_rotates_and_places_sloped_and_slab_blocks() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let target_probe = Vec3::new(0.0, 9.0, -0.75);
    let mut avatar = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.0, 16.5, 0.0), 1.0));
    avatar.rotate_degrees(0.0, 70.5);
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    assert!(!avatar.use_right_hand(&mut world) && avatar.use_right_hand(&mut world));
    assert!(avatar.dispatch_trigger(InputTrigger::WheelDown) && avatar.dispatch_trigger(InputTrigger::WheelDown));

    let (mut wedge_fwd_mesh, mut wedge_right_mesh) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut wedge_fwd_mesh);
    assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    avatar.emit_appearance(&mut wedge_right_mesh);
    let vertices = |ledger: &SurfaceLedger| -> Vec<Vec3> {
        ledger.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };
    assert_ne!(vertices(&wedge_fwd_mesh), vertices(&wedge_right_mesh));
    for _ in 0..3 {
        assert!(avatar.dispatch_trigger(InputTrigger::WheelClick));
    }
    assert!(!avatar.use_left_hand(&mut world) && avatar.release_hand(&mut world));
    assert!(world.constrain_to_surface(Vec3::new(0.25, 9.0, -0.95), 0.0).y > world.constrain_to_surface(Vec3::new(0.25, 9.0, -0.55), 0.0).y + 0.3);

    let mut slab_world = BiomedCubeWorld::beveled(16, 1.0);
    let slab_initial = slab_world.constrain_to_surface(target_probe, 0.0).y;
    let mut slab_avatar = PlanetaryAvatar::at(slab_world.constrain_to_surface(Vec3::new(0.0, 16.5, 0.0), 1.0));
    slab_avatar.rotate_degrees(0.0, 70.5);
    slab_avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::QuarriedStone, HeldRelic::EmptyPalm));
    assert!(slab_avatar.dispatch_trigger(InputTrigger::WheelDown));
    assert!(!slab_avatar.use_right_hand(&mut slab_world) && slab_avatar.release_hand(&mut slab_world));
    assert!((slab_world.constrain_to_surface(target_probe, 0.0).y - (slab_initial + 0.25)).abs() < 1e-4);
}

#[test]
fn test_satchel_stows_and_retrieves_blocks_and_tools_invisibly_when_closed() {
    let world = BiomedCubeWorld::beveled(16, 1.0);
    let mut avatar = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.0, 16.5, 0.0), 1.0));
    avatar.rotate_degrees(0.0, 70.5);
    let (mut empty_hands_mesh, mut empty_satchel_mesh, mut closed_mesh, mut open_mesh) = (
        SurfaceLedger::empty(), SurfaceLedger::empty(), SurfaceLedger::empty(), SurfaceLedger::empty(),
    );
    avatar.emit_appearance(&mut empty_hands_mesh);
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab));
    avatar.emit_appearance(&mut empty_satchel_mesh);
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab));

    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::ExcavatedEarth));
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyQ) && avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyE));
    avatar.emit_appearance(&mut closed_mesh);
    assert_eq!(closed_mesh.count(), empty_hands_mesh.count());
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab));
    avatar.emit_appearance(&mut open_mesh);
    assert!(empty_satchel_mesh.count() > empty_hands_mesh.count() + 100 && open_mesh.count() > empty_satchel_mesh.count());
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyD) && avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyQ));
}

#[test]
fn test_two_free_hands_swing_both_arms_and_troupe_avatar_excavates_and_places() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);
    let controls = KeyMotion::idle();
    let mut avatar = PlanetaryAvatar::at(spawn);
    avatar.rotate_degrees(0.0, 65.3);

    let (mut idle_mesh, mut digging_mesh) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut idle_mesh);
    assert!(!avatar.use_left_hand(&mut world) && !avatar.use_right_hand(&mut world));
    for _ in 0..6 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    avatar.emit_appearance(&mut digging_mesh);
    let first_v0 = |ledger: &SurfaceLedger| -> Vec<Vec3> {
        ledger.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };
    assert_ne!(first_v0(&idle_mesh), first_v0(&digging_mesh));
    for _ in 0..2 {
        assert!(!avatar.use_right_hand(&mut world));
    }
    assert!(avatar.use_right_hand(&mut world) && !avatar.release_hand(&mut world));
    assert!(!avatar.use_right_hand(&mut world) && avatar.release_hand(&mut world));
}

#[test]
fn test_spade_in_left_hand_with_empty_right_hand_excavates_sloped_block_cleanly() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let mut initial_batch = engine::CpuMeshBatch::empty();
    world.emit_surface(&mut initial_batch);

    let (mut slope_u, mut slope_z, mut slope_mid_y) = (0.0f32, -4.25f32, 0.0f32);
    'search: for cu in -12..12 {
        for cv in -12..-5 {
            let (u, v) = ((cu as f32) * 0.5 + 0.25, (cv as f32) * 0.5 + 0.25);
            let sample_y = world.constrain_to_surface(Vec3::new(u, 18.0, v), 0.0).y - 16.0;
            if (0.10..0.40).contains(&sample_y.rem_euclid(0.5)) {
                (slope_u, slope_z, slope_mid_y) = (u, v, sample_y);
                break 'search;
            }
        }
    }
    assert!(slope_mid_y > 0.0);
    let stand_pos = world.constrain_to_surface(Vec3::new(slope_u, 18.0, slope_z + 0.75), 1.0);
    let mut avatar = PlanetaryAvatar::at(stand_pos);
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::IronSpade));
    avatar.rotate_degrees(0.0, 65.3);
    assert!(!avatar.use_left_hand(&mut world) && avatar.use_left_hand(&mut world));
    assert!(world.constrain_to_surface(Vec3::new(slope_u, 18.0, slope_z), 0.0).y - 16.0 < slope_mid_y);
    assert_eq!(world.face_count(), 6);
    let (mut full_planet_ledger, mut modified_face_ledger) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    world.emit_surface(&mut full_planet_ledger);
    assert_eq!(world.emit_modified_face(&mut modified_face_ledger), 0);
    assert!(modified_face_ledger.count() > 0 && modified_face_ledger.count() * 4 < full_planet_ledger.count());
}

#[test]
fn test_crosshair_aim_excavates_within_three_blocks_and_rejects_beyond_three_blocks() {
    let mut world = BiomedCubeWorld::beveled(16, 1.0);
    let (spawn, far_probe) = (Vec3::new(0.0, 9.0, 0.0), Vec3::new(0.0, 9.0, -2.25));
    let mut avatar = PlanetaryAvatar::at(world.constrain_to_surface(spawn, 1.0));
    let (mut idle_avatar, mut air_swing) = (SurfaceLedger::empty(), SurfaceLedger::empty());
    avatar.emit_appearance(&mut idle_avatar);

    avatar.rotate_view(0.0, 234.0);
    let initial_far = world.constrain_to_surface(far_probe, 0.0).y;
    assert!(!avatar.use_left_hand(&mut world));
    avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    avatar.emit_appearance(&mut air_swing);
    assert_eq!(air_swing.count(), idle_avatar.count());
    assert!((world.constrain_to_surface(far_probe, 0.0).y - initial_far).abs() < 1e-4);

    avatar.rotate_view(0.0, 41.0);
    let (closer_probe, aimed_probe) = (Vec3::new(0.0, 9.0, -1.25), Vec3::new(0.0, 9.0, -1.75));
    let (initial_closer, initial_aimed) = (world.constrain_to_surface(closer_probe, 0.0).y, world.constrain_to_surface(aimed_probe, 0.0).y);
    let (mut untouched_face, mut loosened_face, mut swing_mesh) =
        (SurfaceLedger::empty(), SurfaceLedger::empty(), SurfaceLedger::empty());
    world.emit_modified_face(&mut untouched_face);
    assert!(!avatar.use_right_hand(&mut world) && !avatar.use_left_hand(&mut world));
    world.emit_modified_face(&mut loosened_face);
    assert!(loosened_face.count() > untouched_face.count());
    avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    avatar.emit_appearance(&mut swing_mesh);
    assert!(swing_mesh.count() > air_swing.count());
    for _ in 0..2 {
        assert!(!avatar.use_left_hand(&mut world));
    }
    assert!(avatar.use_left_hand(&mut world));
    assert!((world.constrain_to_surface(closer_probe, 0.0).y - initial_closer).abs() < 1e-4);
    assert!((initial_aimed - world.constrain_to_surface(aimed_probe, 0.0).y - 0.5).abs() < 1e-4);
}
