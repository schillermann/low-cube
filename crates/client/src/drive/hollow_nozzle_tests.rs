use glam::Vec3;
use world::{
    AttitudeGuidance, AttitudeNozzleMesh, BlockMedium, BlockMold, CardinalTurn, CraftBlueprint,
    CraftPose, GridCell, MeshTarget, MoldArchetype, NozzleAxis, PilotOrder, Quad, RocketNozzleMesh,
    SurfaceFrame, VehicleAxes, VehiclePart, WedgeHeading,
};

struct BellInspector {
    translucent_plumes: usize,
    incandescent: usize,
    inward_cavity: usize,
    outward_shell: usize,
}

impl BellInspector {
    const fn empty() -> Self {
        Self { translucent_plumes: 0, incandescent: 0, inward_cavity: 0, outward_shell: 0 }
    }
}

impl MeshTarget for BellInspector {
    fn push_quad(&mut self, quad: Quad) {
        let g = quad.radiate(|v| v);
        if g < -1.5 {
            self.translucent_plumes += 1;
            return;
        }
        if g > 0.2 {
            self.incandescent += 1;
        }
        quad.transfer(|v0, v1, v2, v3, norm, _| {
            let center = (v0 + v1 + v2 + v3) * 0.25;
            let radial = Vec3::new(center.x - 0.25, 0.0, center.z - 0.25);
            if radial.length_squared() > 1e-4 && norm.y.abs() < 0.85 {
                if radial.dot(Vec3::new(norm.x, 0.0, norm.z)) < -1e-4 {
                    self.inward_cavity += 1;
                } else {
                    self.outward_shell += 1;
                }
            }
        });
    }
}

#[test]
fn test_thrusters_are_hollow_when_idle_and_emit_translucent_jet_only_on_active_horns() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let cell = GridCell::new(0, 0);

    let mut idle_bell = BellInspector::empty();
    RocketNozzleMesh::new(NozzleAxis::Downward, 0.0).emit(cell, 0.0, &frame, &mut idle_bell);
    assert_eq!(idle_bell.translucent_plumes, 0, "Idle main nozzle must emit zero fire quads");
    assert_eq!(idle_bell.incandescent, 0, "Idle main nozzle throat must stay dark");
    assert!(
        idle_bell.inward_cavity >= 16 && idle_bell.outward_shell >= 16,
        "Main nozzle must have hollow inward cavity walls as well as outer shell"
    );

    let mut lit_bell = BellInspector::empty();
    RocketNozzleMesh::new(NozzleAxis::Downward, 1.0).emit(cell, 0.0, &frame, &mut lit_bell);
    assert_eq!(lit_bell.translucent_plumes, 24, "Active main nozzle must emit 3-layer translucent jet");
    assert!(lit_bell.incandescent > 0, "Active main nozzle deep throat injector must glow");

    let mut idle_rcs = BellInspector::empty();
    AttitudeNozzleMesh::with_throttles(WedgeHeading::North, [0.0; 5]).emit(cell, 0.0, &frame, &mut idle_rcs);
    assert_eq!(idle_rcs.translucent_plumes, 0, "Idle 5-horn RCS pod must emit zero fire quads");
    assert_eq!(idle_rcs.incandescent, 0, "Idle 5-horn RCS pod must remain unlit");

    let mut one_horn = BellInspector::empty();
    AttitudeNozzleMesh::with_throttles(WedgeHeading::North, [0.0, 0.0, 1.0, 0.0, 0.0])
        .emit(cell, 0.0, &frame, &mut one_horn);
    let mut two_horns = BellInspector::empty();
    AttitudeNozzleMesh::with_throttles(WedgeHeading::North, [1.0, 0.0, 1.0, 0.0, 0.0])
        .emit(cell, 0.0, &frame, &mut two_horns);
    assert_eq!(one_horn.translucent_plumes, 18, "Single active RCS horn must emit 18 translucent plume quads");
    assert_eq!(two_horns.translucent_plumes, 36, "Two active RCS horns must emit 36 translucent plume quads");
}

#[test]
fn test_missing_side_steuerduese_produces_asymmetric_lever_moment() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let pose = CraftPose::parked(Vec3::ZERO, axes, Vec3::Z);
    let seat = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::CabinSeat);
    let console = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::FlightConsole);
    let left_pod = BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle);
    let right_pod = BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle);

    let asym_bp = CraftBlueprint::commission(
        Vec3::ZERO,
        vec![
            VehiclePart::new((0, 0), 0.0, seat.atom_for(Vec3::Z, &frame)),
            VehiclePart::new((0, 1), 0.0, console.atom_for(Vec3::Z, &frame)),
            VehiclePart::new((-2, 0), 0.0, left_pod.atom_for(Vec3::X, &frame)),
        ],
    );
    let sym_bp = CraftBlueprint::commission(
        Vec3::ZERO,
        vec![
            VehiclePart::new((0, 0), 0.0, seat.atom_for(Vec3::Z, &frame)),
            VehiclePart::new((0, 1), 0.0, console.atom_for(Vec3::Z, &frame)),
            VehiclePart::new((-2, 0), 0.0, left_pod.atom_for(Vec3::X, &frame)),
            VehiclePart::new((2, 0), 0.0, right_pod.atom_for(Vec3::NEG_X, &frame)),
        ],
    );

    let retro = PilotOrder::from_channels(Vec3::NEG_Y, Vec3::ZERO, Vec3::ZERO, AttitudeGuidance::Engaged);
    let sym_up = sym_bp.sample_impulse(&retro, pose).attitude_step(0.25) * Vec3::Y;
    let asym_up = asym_bp.sample_impulse(&retro, pose).attitude_step(0.25) * Vec3::Y;
    assert!(sym_up.x.abs() < 1e-4, "Symmetric left+right RCS pods must cancel roll moments, got {sym_up:?}");
    assert!(
        asym_up.x.abs() > 0.06,
        "Missing right-side Steuerdüse must create noticeable r × F roll tilt, got {asym_up:?}"
    );
}
