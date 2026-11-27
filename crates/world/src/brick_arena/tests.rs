use glam::{DMat3, DVec3, Vec3};
use crate::block::{BlockMedium, CardinalTurn, GridCell};
use crate::frame::SurfaceFrame;
use crate::relief::MountDomain;
use crate::{MeshTarget, Quad};
use super::{
    Brick8, BrickArena, BrickCell, BrickCoord, BrickVolume, CellCoord, DismantledSpan,
    GranularFlag, InertialBricks, LocalSlot, MachineKind, ModuleSculpture, ModuleSpan, ModuleSpec,
    RawAtom, ScalableKind, SpanHologram,
};

struct QuadCounter(usize);
impl MeshTarget for QuadCounter {
    fn push_quad(&mut self, _quad: Quad) {
        self.0 += 1;
    }
}

#[test]
fn test_brick8_is_unboxed_copy_cells_with_exact_u64_occupancy_and_granular_masks() {
    assert!(!std::mem::needs_drop::<BrickCell>(), "BrickCell must contain zero heap pointers");
    assert!(!std::mem::needs_drop::<Brick8>(), "Brick8 must contain zero heap pointers");

    let mut brick = Brick8::empty();
    assert!(brick.vacant() && brick.occupied_count() == 0);

    let sand_slot = LocalSlot::from_axes(3, 5, 2);
    let hull_slot = LocalSlot::from_axes(3, 5, 7);
    let _ = brick.write_slot(sand_slot, BrickCell::RawAtom(RawAtom::full_cube(BlockMedium::Sand)));
    let _ = brick.write_slot(hull_slot, BrickCell::RawAtom(RawAtom::full_cube(BlockMedium::TitaniumHull)));

    assert_eq!(brick.occupied_count(), 2);
    assert_eq!(brick.column_mask(3, 5), (1u8 << 2) | (1u8 << 7));
    assert_ne!(brick.granular_plane_mask(2), 0, "Sand atom must set granular bitmask");
    assert_eq!(brick.granular_plane_mask(7), 0, "Rigid TitaniumHull must leave granular bitmask clear");
    assert_eq!(RawAtom::full_cube(BlockMedium::Sand).granular(), GranularFlag::Granular);

    let _ = brick.clear_slot(sand_slot);
    let _ = brick.clear_slot(hull_slot);
    assert!(brick.vacant() && brick.granular_masks() == [0; 8]);
}

#[test]
fn test_all_eight_fixed_multiblock_machines_exceed_single_cell_and_prevent_overlap() {
    let mut arena = BrickArena::untouched();
    let mut cursor_u = 0i32;

    for kind in MachineKind::all() {
        let (b, t, h) = kind.canonical_dims();
        assert!((b as usize) * (t as usize) * (h as usize) > 1, "{kind:?} must occupy > 1 block");

        let spec = ModuleSpec::machine(kind, CardinalTurn::Forward);
        let anchor = CellCoord::new(cursor_u, 0, 0);
        assert!(arena.place_module(anchor, spec), "{kind:?} must place into vacant span");
        assert!(!arena.place_module(anchor.shifted((b - 1) as i32, (t - 1) as i32, (h - 1) as i32), spec), "Overlapping {kind:?} must be rejected");

        // Turned 90° swaps width_u and depth_v
        let turned = ModuleSpec::machine(kind, CardinalTurn::Right);
        assert_eq!(turned.span().dimensions(), (t, b, h));

        cursor_u += 6;
    }

    assert_eq!(MachineKind::ArcFurnace.canonical_dims(), (2, 2, 3));
    assert_eq!(MachineKind::AssemblyBench.canonical_dims(), (2, 1, 2));
    assert_eq!(MachineKind::ElectrolysisPlant.canonical_dims(), (2, 2, 3));
    assert_eq!(MachineKind::AirlockChamber.canonical_dims(), (2, 2, 4));
    assert_eq!(MachineKind::PilotSeat.canonical_dims(), (2, 2, 3));
    assert_eq!(MachineKind::PhosphorConsole.canonical_dims(), (2, 2, 2));
    assert_eq!(MachineKind::DockingRing.canonical_dims(), (2, 2, 1));
    assert_eq!(MachineKind::CrewBerth.canonical_dims(), (2, 4, 2));
}

#[test]
fn test_scalable_modules_cross_brick_boundaries_emit_drag_hologram_and_dismantle_atomically_via_any_proxy() {
    let mut arena = BrickArena::untouched();
    let (anchor, span) = ModuleSpan::dragged_between(CellCoord::new(6, 7, 1), CellCoord::new(9, 9, 1), 4);
    assert_eq!(anchor, CellCoord::new(6, 7, 1));
    assert_eq!(span.dimensions(), (4, 3, 5)); // 4×3×5 = 60 cells crossing BrickCoord (0,0,0) -> (1,1,0)!

    for kind in ScalableKind::all() {
        let spec = ModuleSpec::scalable(kind, span, CardinalTurn::Forward);
        assert!(arena.place_module(anchor, spec));
        assert!(arena.active_brick_count() >= 4, "Cross-boundary 4x3x5 module must populate 4 bricks");

        let far_proxy = anchor.shifted(3, 2, 4);
        assert!(matches!(arena.cell_at(far_proxy), BrickCell::ModuleProxy(_)));
        assert_eq!(arena.resolve_module(far_proxy), Some((anchor, spec)));

        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let (mut mesh_q, mut holo_q) = (QuadCounter(0), QuadCounter(0));
        ModuleSculpture::new(anchor, spec, 0.0).emit(&frame, &mut mesh_q);
        SpanHologram::new(anchor, spec, MountDomain::CraftRig).emit(0.0, &frame, false, &mut holo_q);
        assert!(mesh_q.0 >= 6 && holo_q.0 == 8 * 3 * 6, "Drag hologram must emit 12 corner docking brackets");

        // Striking the farthest ModuleProxy cell dismantles all 60 cells and recycles every vacated Brick8!
        let removed = arena.dismantle_at(far_proxy);
        assert_eq!(removed, DismantledSpan::Module { anchor, spec });
        assert!(arena.vacant() && arena.active_brick_count() == 0, "All bricks must recycle on atomic proxy removal");
    }

    // Re-inserting after dismantling reuses the recycled brick slots without growing Vec<Brick8>:
    let _ = arena.place_atom(CellCoord::new(0, 0, 0), RawAtom::full_cube(BlockMedium::TitaniumHull));
    assert_eq!(arena.brick_at(BrickCoord::new(0, 0, 0)).map(Brick8::occupied_count), Some(1));
}

#[test]
fn test_inertial_bricks_decorator_maintains_exact_o1_steiner_mass_com_and_inertia_tensor() {
    let mut vessel = InertialBricks::wrapping(BrickArena::untouched());

    let gold = RawAtom::full_cube(BlockMedium::GoldOre);
    let hull = RawAtom::full_cube(BlockMedium::TitaniumHull);
    let _ = vessel.place_atom(CellCoord::new(2, -1, 4), gold);
    let _ = vessel.place_atom(CellCoord::new(-5, 3, 0), hull);

    let tank_spec = ModuleSpec::scalable(
        ScalableKind::PropellantTank,
        ModuleSpan::clamped(3, 2, 4),
        CardinalTurn::Forward,
    );
    assert!(vessel.place_module(CellCoord::new(0, 0, 1), tank_spec));

    // Place and subsequently dismantle a heavy ArcFurnace to verify exact subtraction without drift!
    let furnace = ModuleSpec::machine(MachineKind::ArcFurnace, CardinalTurn::Right);
    assert!(vessel.place_module(CellCoord::new(7, 7, 2), furnace));
    let _ = vessel.dismantle_at(CellCoord::new(8, 8, 4)); // hit proxy cell -> removes entire 2×2×3 furnace!

    // Place a CrewBerth and overwrite one of its proxy cells via place_atom to verify atomic Steiner subtraction!
    let berth = ModuleSpec::machine(MachineKind::CrewBerth, CardinalTurn::Forward);
    assert!(vessel.place_module(CellCoord::new(-3, -6, 1), berth));
    let _ = vessel.place_atom(CellCoord::new(-2, -4, 2), hull);

    // Brute-force analytical integration over remaining occupied cells:
    let mut expected_mass = 0.0_f64;
    let mut weighted_sum = DVec3::ZERO;
    let mut cells: Vec<(DVec3, f64)> = Vec::new();
    vessel.inner().visit_occupied_cells(|coord, cell| {
        let m = match cell {
            BrickCell::Empty => 0.0,
            BrickCell::RawAtom(a) => a.mass_kg(),
            BrickCell::ModuleAnchor(s) => s.cell_mass_kg(),
            BrickCell::ModuleProxy(_) => vessel.resolve_module(coord).map_or(0.0, |(_, s)| s.cell_mass_kg()),
        };
        let p = coord.metric_center(0.0);
        expected_mass += m;
        weighted_sum += p * m;
        cells.push((p, m));
    });
    let expected_com = weighted_sum / expected_mass;
    let s = f64::from(GridCell::SPAN);
    let mut expected_tensor = DMat3::ZERO;
    for (p, m) in cells {
        let r = p - expected_com;
        let cube_i = (1.0 / 6.0) * m * s * s;
        let col0 = DVec3::new(cube_i + m * (r.y * r.y + r.z * r.z), -m * r.x * r.y, -m * r.x * r.z);
        let col1 = DVec3::new(-m * r.x * r.y, cube_i + m * (r.x * r.x + r.z * r.z), -m * r.y * r.z);
        let col2 = DVec3::new(-m * r.x * r.z, -m * r.y * r.z, cube_i + m * (r.x * r.x + r.y * r.y));
        expected_tensor += DMat3::from_cols(col0, col1, col2);
    }

    assert!((vessel.total_mass_kg() - expected_mass).abs() < 1e-9);
    assert!((vessel.center_of_mass() - expected_com).length() < 1e-9);
    let diff = vessel.inertia_matrix() - expected_tensor;
    for col in 0..3 {
        assert!(diff.col(col).length() < 1e-8, "Steiner 3x3 inertia mismatch col {col}: {:?}", diff);
    }
}
