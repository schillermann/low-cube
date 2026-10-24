use crate::{MeshTarget, Quad};
use crate::block::GridCell;
use crate::topology::ClimateZone;
use super::crust_vault::CrustVault;

struct ShellPatches {
    quads: Vec<Quad>,
}

impl MeshTarget for ShellPatches {
    fn push_quad(&mut self, quad: Quad) { self.quads.push(quad); }
}

const ZONES: [ClimateZone; 6] = [ClimateZone::Temperate, ClimateZone::Volcanic, ClimateZone::Desert, ClimateZone::Arctic, ClimateZone::Jungle, ClimateZone::Steppe];

fn quad_area(quad: &Quad) -> f32 {
    quad.transfer(|v0, v1, v2, v3, _, _| ((v1 - v0).cross(v2 - v0).length() + (v2 - v0).cross(v3 - v0).length()) * 0.5)
}

#[test]
fn test_magma_core_renders_as_glowing_sphere_tiling_every_face() {
    let core = CrustVault::beneath(128).core();
    let mut patches = ShellPatches { quads: Vec::new() };
    for zone in ZONES {
        let frame = zone.surface_frame(64.0);
        for u in -128..128 {
            for v in -128..128 { core.emit_patch(GridCell::new(u, v), &frame, &mut patches); }
        }
    }
    assert_eq!(patches.quads.len(), 6 * 32 * 32, "Each face must tile its sixth of the sphere with 32x32 strided patches");
    for quad in &patches.quads {
        assert!(quad.radiate(|glow| glow) > 0.5, "Magma patches must be self-luminous");
        quad.transfer(|v0, v1, v2, v3, normal, color| {
            for corner in [v0, v1, v2, v3] { assert!((corner.length() - 40.0).abs() < 1e-3, "Patch corner must lie on the radius-40 sphere (12 blocks inside radius-46 vault), got {}", corner.length()); }
            assert!(normal.dot((v0 + v2).normalize()) > 0.99, "Patch normal must face outward");
            assert!(color.x >= 0.99 && color.z < 0.4, "Magma must glow in molten orange hues, got {color:?}");
        });
    }
}

#[test]
fn test_crust_vault_scales_circumference_and_magma_core_with_world_cube_size() {
    let vault = CrustVault::beneath(128);
    let vault2 = CrustVault::beneath(256);
    let (crust_ceil, core_crest) = (vault.ceiling_height_at(0.0, 0.0), vault.core().crest_at(0.0, 0.0));
    assert_eq!(vault.floor_steps_at(0, 0), -32, "Vault at factor 1 breaches below -16.0 m ceiling");
    assert!((vault.circumference() - std::f32::consts::TAU * 48.0).abs() < 0.01);
    assert!((vault2.circumference() - std::f32::consts::TAU * 96.0).abs() < 0.01);
    assert!((crust_ceil - core_crest - 8.0).abs() < 0.01, "Crust ceiling must sit 16 blocks (8 m) above the core, got {crust_ceil} vs {core_crest}");
    assert!(vault.core().engulfs(0.0, 0.0, -24.0, 0.01) && !vault.core().engulfs(0.0, 0.0, -22.9, 1.05));
}

#[test]
fn test_crust_vault_ceiling_is_a_closed_magma_lit_beveled_block_dome() {
    let vault = CrustVault::beneath(128);
    let mut patches = ShellPatches { quads: Vec::new() };
    for zone in ZONES {
        let frame = zone.surface_frame(64.0);
        for u in -128..128 {
            for v in -128..128 {
                let before = patches.quads.len();
                vault.emit_ceiling(GridCell::new(0.max(u) - 0.max(u) + u, v), 0, &frame, &mut patches);
                let (uc, vc) = ((u as f32) * 0.5 + 0.25, (v as f32) * 0.5 + 0.25);
                if vault.encloses(uc, vc, -16.5, 0.5) {
                    let cell_area: f32 = patches.quads[before..].iter().map(quad_area).sum();
                    assert!(cell_area >= 0.24, "Every vault cell must emit a complete ceiling without holes at {zone:?} ({u}, {v}), got area {cell_area}");
                }
            }
        }
    }
    let area: f32 = patches.quads.iter().map(quad_area).sum();
    let sphere = 4.0 * std::f32::consts::PI * 48.0 * 48.0;
    assert!(area > sphere && area < sphere * 1.25, "Beveled block dome ceiling must wrap the whole core, got area {area} vs {sphere}");
    let (mut flat_blocks, mut gentle_bevels, mut steep_bevels) = (0usize, 0usize, 0usize);
    for quad in &patches.quads {
        assert!(quad.radiate(|glow| glow) > 0.5, "Crust ceiling must be lit by the magma glow instead of rendering black");
        quad.transfer(|v0, _, v2, _, normal, color| {
            assert!((v0.length() - 48.0).abs() < 0.75 && normal.dot((v0 + v2).normalize()) < -0.2, "Beveled block ceiling must lie on the radius-48 vault facing inward");
            assert!(color.x > color.z && color.x > 0.2, "Crust must read as earthy rock, got {color:?}");
            let max_axis = normal.x.abs().max(normal.y.abs()).max(normal.z.abs());
            if max_axis > 0.98 { flat_blocks += 1; } else if max_axis > 0.80 { gentle_bevels += 1; } else { steep_bevels += 1; }
        });
    }
    assert!(flat_blocks > 100 && gentle_bevels > 100 && steep_bevels > 100, "Ceiling must combine flat cubes/slabs, gentle 2:1 bevels, and 45° bevels: ({flat_blocks}, {gentle_bevels}, {steep_bevels})");
}

#[test]
fn test_crust_vault_breaches_after_four_layers_never_lifts_ceiling_and_confines_rising_eyes() {
    let vault = CrustVault::beneath(64);
    assert_eq!(vault.floor_steps_at(0, 0), -16);
    assert_eq!(vault.excavated_steps(0, 0, -12, -32), -13, "1st crust layer digs to -13");
    assert_eq!(vault.excavated_steps(0, 0, -13, -32), -14, "2nd crust layer digs to -14");
    assert_eq!(vault.excavated_steps(0, 0, -14, -32), -15, "3rd crust layer digs to -15");
    assert_eq!(vault.excavated_steps(0, 0, -15, -32), -32, "4th crust layer breaches into the hollow world shaft");
    assert_eq!(vault.excavated_steps(60, 60, -35, -55), -36, "Crust outside the vault stops at the floor without breaching");
    assert_eq!(vault.confine(0.0, 0.0, -9.0), -9.0, "Eyes well beneath the ceiling move freely");
    assert!((vault.confine(0.0, 0.0, -8.1) + 8.3).abs() < 1e-4, "Eyes rising into the ceiling are pushed back beneath it");
    assert!(vault.encloses(0.0, 0.0, -8.5, 0.1) && !vault.encloses(0.0, 0.0, -7.95, 0.1));

    let frame = ClimateZone::Temperate.surface_frame(32.0);
    for stage in 0..=3 {
        let mut cell_quads = ShellPatches { quads: Vec::new() };
        vault.emit_ceiling(GridCell::new(0, 0), stage, &frame, &mut cell_quads);
        for quad in &cell_quads.quads {
            quad.transfer(|v0, v1, v2, v3, _, _| {
                for v in [v0, v1, v2, v3] { assert!((v.y - 24.0).abs() < 1e-4, "Ceiling must never lift when digging from above (stage={stage}), got y={}", v.y); }
            });
        }
    }
}
