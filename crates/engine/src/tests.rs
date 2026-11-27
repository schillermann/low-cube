use std::time::Instant;
use glam::Vec3;
use world::{
    BlockMedium, Brick8, BrickArena, BrickCell, BrickCoord, BrickVolume, CardinalTurn, CellCoord,
    LocalSlot, MachineKind, MeshTarget, ModuleSpan, ModuleSpec, Quad, RawAtom, ScalableKind,
    SpaceModule,
};
use crate::{
    ArenaNeighbors, AtmosphereSkyLut, BufferSlot, GpuPowerProfile, GreedyBrickQuads,
    LuminanceHarvestTarget, ModuleArchetypeBank, ModuleInstance, PackedQuad, PointLightCluster,
    QuadBatch, Rgb565Swatch, SkyLutMedium, StagingTarget, StrictVramGuard, UploadBudgetRing,
    VacantNeighbors, VramAccount, VramLedger, FIRMAMENT_LUT_WGSL, FREE_QUAD_WGSL, MAX_POINT_LIGHTS,
    MODULE_INSTANCE_WGSL, MONITOR_BLIT_WGSL, PACKED_QUAD_WGSL, PCIE_FRAME_UPLOAD_CAP_BYTES,
    SKY_LUT_HEIGHT, SKY_LUT_WIDTH, THINKPAD_VRAM_BUDGET_BYTES,
};

struct ByteCountStaging {
    written: usize,
    calls: usize,
}

impl StagingTarget for ByteCountStaging {
    fn write_chunk(&mut self, _slot: BufferSlot, _byte_offset: u64, payload: &[u8]) {
        self.written += payload.len();
        self.calls += 1;
    }
}

struct TestRayleighLut {
    sun: Vec3,
}

impl SkyLutMedium for TestRayleighLut {
    fn sample_sky_ray(&self, ray_dir: Vec3) -> [f32; 4] {
        let horizon = (1.0 - ray_dir.y.abs()).clamp(0.0, 1.0);
        let sun_glow = ray_dir.dot(self.sun).max(0.0);
        [0.18 + 0.55 * sun_glow, 0.42, 0.86 * (1.0 - 0.3 * horizon), 0.75]
    }
}

struct WgslInspection<'a> {
    label: &'a str,
    source: &'a str,
}

impl<'a> WgslInspection<'a> {
    const fn new(label: &'a str, source: &'a str) -> Self {
        Self { label, source }
    }

    fn verify(&self) {
        let module = wgpu::naga::front::wgsl::parse_str(self.source)
            .unwrap_or_else(|e| panic!("WGSL parse failed in {}: {}", self.label, e.emit_to_string(self.source)));
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::empty(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|e| panic!("WGSL validation failed in {}: {e:?}", self.label));
    }
}

#[test]
fn test_packed_quad_is_exact_8_bytes_and_roundtrips_all_bitfields() {
    assert_eq!(core::mem::size_of::<PackedQuad>(), 8, "PackedQuad must be exact 8 Bytes (23x smaller than 184B)");
    let swatch = Rgb565Swatch::from_linear(Vec3::new(0.72, 0.78, 0.84)).code();
    let quad = PackedQuad::encoded([5, 12, 3], 2, [7, 3], 0b11_10_01_00, 1, swatch);
    quad.transfer(|coords, normal, span_m1, ao, tag, col| {
        assert_eq!(coords, [5, 12, 3]);
        assert_eq!(normal, 2);
        assert_eq!(span_m1, [7, 3]);
        assert_eq!(ao, 0b11_10_01_00);
        assert_eq!(tag, 1);
        assert_eq!(col, swatch);
    });
    let (corners, n) = quad.reconstruct_corners([4.0, -2.0, 8.0, 0.5]);
    assert_eq!(n, Vec3::Y);
    let cross = (corners[1] - corners[0]).cross(corners[2] - corners[0]).normalize();
    assert!((cross - Vec3::Y).length() < 1e-5, "PackedQuad winding must match outward CCW normal");
}

#[test]
fn test_greedy_brick_quads_merges_512_solid_cubes_into_6_packed_quads_and_computes_corner_ao() {
    let mut brick = Brick8::empty();
    let titanium = BrickCell::RawAtom(RawAtom::full_cube(BlockMedium::TitaniumHull));
    for idx in 0..512u16 {
        let _ = brick.write_slot(LocalSlot::new(idx), titanium);
    }
    let mut quads = Vec::new();
    let mut modules = Vec::new();
    GreedyBrickQuads::new(&brick, BrickCoord::new(0, 0, 0), 0.0).extract(
        &VacantNeighbors::new(),
        &mut quads,
        &mut modules,
    );
    assert_eq!(quads.len(), 6, "512 uniform full-cubes in 8x8x8 Brick8 must merge into exact 6 8-Byte PackedQuads (48 B)");
    assert!(modules.is_empty());

    // Verify neighbor brick seam culling across two touching 8x8x8 bricks:
    let mut arena = BrickArena::untouched();
    for x in 0..16 {
        for z in 0..8 {
            for y in 0..8 {
                let _ = arena.place_atom(CellCoord::new(x, z, y), RawAtom::full_cube(BlockMedium::TitaniumHull));
            }
        }
    }
    let mut seam_quads = Vec::new();
    for bx in 0..2 {
        let bc = BrickCoord::new(bx, 0, 0);
        let b = arena.brick_at(bc).expect("brick present");
        GreedyBrickQuads::new(b, bc, 0.0).extract(&ArenaNeighbors::new(&arena, bc), &mut seam_quads, &mut modules);
    }
    assert_eq!(seam_quads.len(), 10, "Shared interior 8x8 brick face must be 100% culled (10 outer faces instead of 12)");

    // Verify 2-bit corner AO darkens concave L-step crease:
    let mut step_brick = Brick8::empty();
    for z in 0..4u8 {
        for x in 0..4u8 {
            let _ = step_brick.write_slot(LocalSlot::from_axes(x, z, 0), titanium);
        }
    }
    let _ = step_brick.write_slot(LocalSlot::from_axes(0, 0, 1), titanium);
    let mut ao_quads = Vec::new();
    GreedyBrickQuads::new(&step_brick, BrickCoord::new(0, 0, 0), 0.0).extract(
        &VacantNeighbors::new(),
        &mut ao_quads,
        &mut modules,
    );
    let has_occluded_corner = ao_quads.iter().any(|q| q.transfer(|_, n, _, ao, _, _| n == 2 && ao != 0xFF));
    assert!(has_occluded_corner, "Concave L-corner on +Y face must bake darkened 2-bit corner AO (< 0xFF)");
}

#[test]
fn test_greedy_brick_extraction_speed_and_module_anchor_routing() {
    let mut brick = Brick8::empty();
    let hull = BrickCell::RawAtom(RawAtom::full_cube(BlockMedium::TitaniumHull));
    let stone = BrickCell::RawAtom(RawAtom::full_cube(BlockMedium::CarbonSilicate));
    for lw in 0..4u8 {
        for lv in 0..8u8 {
            for lu in 0..8u8 {
                if !(2..6).contains(&lu) || !(2..6).contains(&lv) || lw == 0 {
                    let cell = if (lw & 1) == 0 { hull } else { stone };
                    let _ = brick.write_slot(LocalSlot::from_axes(lu, lv, lw), cell);
                }
            }
        }
    }
    let furnace = ModuleSpec::machine(MachineKind::ArcFurnace, CardinalTurn::from_code(0));
    let _ = brick.write_slot(LocalSlot::from_axes(3, 3, 1), BrickCell::ModuleAnchor(furnace));

    let mut quads = Vec::with_capacity(64);
    let mut modules = Vec::with_capacity(4);
    let start = Instant::now();
    for _ in 0..100 {
        quads.clear();
        modules.clear();
        GreedyBrickQuads::new(&brick, BrickCoord::new(0, 0, 0), 0.0).extract(
            &VacantNeighbors::new(),
            &mut quads,
            &mut modules,
        );
    }
    let avg_us = start.elapsed().as_secs_f64() * 1e6 / 100.0;
    assert!(!quads.is_empty() && quads.len() < 90, "Striped hollow courtyard must merge down < 90 quads, got {}", quads.len());
    assert_eq!(modules.len(), 1, "ModuleAnchor inside Brick8 must emit 1 hardware-instanced ModuleInstance");
    assert_eq!(modules[0].module(), SpaceModule::Machine(MachineKind::ArcFurnace));
    assert!(avg_us < 250.0, "GreedyBrickQuads must stay fast (<250us debug, ~15us release), got {avg_us:.2}us");
}

#[test]
fn test_all_11_space_module_archetypes_stay_under_150_triangles_and_scale_bht() {
    let bank = ModuleArchetypeBank::assembled();
    for code in 1..=11u8 {
        let module = SpaceModule::from_code(code).expect("valid module");
        let slice = bank.slice_for(module);
        assert!(
            slice.triangle_count() >= 12 && slice.triangle_count() < 150,
            "{module:?} triangle count {} must stay strictly < 150",
            slice.triangle_count()
        );
    }
    let bell = ModuleSpec::scalable(ScalableKind::PropulsionBell, ModuleSpan::clamped(4, 2, 6), CardinalTurn::from_code(0));
    let inst = ModuleInstance::from_grid_anchor(CellCoord::new(2, 4, 0), bell, 0.0);
    let min_world = inst.model_matrix().transform_point3(Vec3::ZERO);
    let max_world = inst.model_matrix().transform_point3(Vec3::ONE);
    let span = (max_world - min_world).abs();
    assert!((span.x - 2.0).abs() < 1e-4 && (span.y - 3.0).abs() < 1e-4 && (span.z - 1.0).abs() < 1e-4);
}

#[test]
fn test_pcie_upload_budget_ring_caps_at_1_5_mb_per_frame_and_vram_stays_under_250_mb() {
    let mut ring = UploadBudgetRing::thinkpad_budget();
    let mut sink = ByteCountStaging { written: 0, calls: 0 };
    let four_mb = vec![0xABu8; 4 * 1024 * 1024];

    let f0 = ring.submit(BufferSlot::new(1), 0, &four_mb, &mut sink);
    assert_eq!(f0, PCIE_FRAME_UPLOAD_CAP_BYTES, "Frame 0 must cap PCIe upload strictly at 1.5 MB (1,572,864 B)");
    assert_eq!(ring.remaining_frame_bytes(), 0);
    assert_eq!(ring.queued_backlog_bytes(), four_mb.len() - PCIE_FRAME_UPLOAD_CAP_BYTES);

    let f1 = ring.begin_frame(&mut sink);
    assert_eq!(f1, PCIE_FRAME_UPLOAD_CAP_BYTES, "Frame 1 must drain at most 1.5 MB");
    let f2 = ring.begin_frame(&mut sink);
    assert_eq!(f2, four_mb.len() - 2 * PCIE_FRAME_UPLOAD_CAP_BYTES, "Frame 2 must finish remaining 1.0 MB cleanly");
    assert_eq!(ring.queued_backlog_bytes(), 0);
    assert_eq!(sink.written, four_mb.len());

    // Verify full ThinkPad T480 1080p scene VRAM stays well below 250 MB:
    let mut vram = StrictVramGuard::new(VramLedger::thinkpad_250mb());
    vram.credit_allocation(1920 * 1080 * 4); // 1080p Depth32Float (8.3 MB)
    vram.credit_allocation(1024 * 1024 * 4); // 1024^2 Sun Shadow (4.2 MB)
    vram.credit_allocation(128 * 64 * 4);    // 128x64 Sky LUT (32 KB)
    vram.credit_allocation(240 * 160 * 10 * 4); // Monitor Atlas + Tile (1.5 MB)
    vram.credit_allocation(2_000_000 * 8);   // 2,000,000 PackedQuads (16.0 MB)
    assert!(vram.within_budget());
    assert!(vram.live_bytes() < THINKPAD_VRAM_BUDGET_BYTES / 6, "ThinkPad scene VRAM must stay < 42 MB (<< 250 MB)");
}

#[test]
fn test_atmosphere_lut_2hz_cadence_and_point_light_16_cap_and_gpu_profiles() {
    let mut lut = AtmosphereSkyLut::cold();
    assert_eq!(lut.dimensions(), (SKY_LUT_WIDTH, SKY_LUT_HEIGHT));
    assert_eq!(lut.texel_bytes().len(), 128 * 64 * 4);
    let medium = TestRayleighLut { sun: Vec3::new(0.6, 0.8, 0.0).normalize() };
    assert!(lut.refresh_at_2hz(0.0, [1, 2, 3, 4], Vec3::Y, &medium), "Cold LUT must bake immediately");
    assert!(!lut.refresh_at_2hz(0.25, [1, 2, 3, 4], Vec3::Y, &medium), "LUT must skip bake before 0.5s (2 Hz)");
    assert!(lut.refresh_at_2hz(0.51, [1, 2, 3, 4], Vec3::Y, &medium), "LUT must refresh once >= 0.5s elapsed");

    let mut batch = QuadBatch::empty();
    let mut cluster = PointLightCluster::centered_on(Vec3::ZERO);
    {
        let mut decorated = LuminanceHarvestTarget::new(&mut batch, &mut cluster);
        for i in 0..24 {
            let x = (i as f32) * 2.0;
            decorated.push_quad(Quad::glowing(
                Vec3::new(x, 0.0, 0.0),
                Vec3::new(x + 0.5, 0.0, 0.0),
                Vec3::new(x + 0.5, 0.5, 0.0),
                Vec3::new(x, 0.5, 0.0),
                Vec3::Z,
                Vec3::new(1.0, 0.8, 0.4),
                1.85,
            ));
        }
    }
    assert_eq!(cluster.active_count(), MAX_POINT_LIGHTS, "LuminanceHarvestTarget must cap active lights at 16");
    assert_eq!(GpuPowerProfile::from_token("mx150"), Some(GpuPowerProfile::DiscreteHighPerformance));
    assert_eq!(GpuPowerProfile::from_token("low"), Some(GpuPowerProfile::IntegratedLowPower));
    assert_eq!(
        GpuPowerProfile::DiscreteHighPerformance.paced_present_mode(&[wgpu::PresentMode::Immediate, wgpu::PresentMode::Fifo]),
        wgpu::PresentMode::Fifo
    );
}

#[test]
fn test_all_five_wgsl_pipelines_pass_naga_full_validation() {
    for shader in [
        WgslInspection::new("packed_quad.wgsl", PACKED_QUAD_WGSL),
        WgslInspection::new("free_quad.wgsl", FREE_QUAD_WGSL),
        WgslInspection::new("module_instance.wgsl", MODULE_INSTANCE_WGSL),
        WgslInspection::new("firmament_lut.wgsl", FIRMAMENT_LUT_WGSL),
        WgslInspection::new("monitor_blit.wgsl", MONITOR_BLIT_WGSL),
    ] {
        shader.verify();
    }
}
