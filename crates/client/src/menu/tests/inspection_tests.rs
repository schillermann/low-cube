use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, BlockMedium, BlockMold, PlanetaryWorld, WorldEarthworks as _};
use crate::attire::{EvaAttire, HeldRelic};
use crate::avatar::{Avatar as _, AvatarOptics as _, AvatarPresence as _, PlanetaryAvatar};
use crate::menu::{EscapeMenu, InspectionModal, SessionModal};
use crate::session::HeadlessSession;
use super::{MockSurface, QuadCapture};

#[test]
fn test_inspection_modal_isolates_satchel_from_close_rocket_blocks() {
    let mut world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 12.0, 0.25), 0.0).y;
    let rig = BlockMold::cube().in_medium(BlockMedium::CantileverRig);
    let nozzle = BlockMold::cube().in_medium(BlockMedium::RocketNozzle);
    let builder_pos = Vec3::new(0.25, g0, 1.75);
    let aim = Vec3::new(0.0, -0.8, -0.6).normalize();
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), aim, builder_pos, rig));
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), aim, builder_pos, nozzle));

    // Stand right against the rocket nozzle block (< 0.30 units away, inside the 0.60m satchel reach)
    let close_pos = Vec3::new(1.25, g0 + 1.75, 0.52);
    let mut avatar = PlanetaryAvatar::at(close_pos);
    avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::RocketNozzle, HeldRelic::FlightConsole));
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyE) && avatar.dispatch_wardrobe_key(KeyCode::KeyQ));

    let mut closed_figure = QuadCapture::empty();
    avatar.emit_figure(&mut closed_figure);

    let surface = MockSurface::active();
    let mut modal = InspectionModal::new(EscapeMenu::closed());
    let mut crosshair_only = QuadCapture::empty();
    modal.emit_ui(&avatar, &mut crosshair_only);
    assert_eq!(crosshair_only.count(), 10);

    // Open satchel right in front of the rocket block: world figure stays unchanged while InspectionModal emits 3D tray
    assert!(avatar.dispatch_wardrobe_key(KeyCode::Tab));
    let (mut open_figure, mut stage_quads, mut modal_ui) =
        (QuadCapture::empty(), QuadCapture::empty(), QuadCapture::empty());
    avatar.emit_figure(&mut open_figure);
    assert_eq!(open_figure.count(), closed_figure.count(), "3D world figure must not emit satchel quads into rocket block");

    let stage_proj = avatar.emit_inspection_stage(16.0 / 9.0, &mut stage_quads);
    assert!(stage_proj.is_some() && stage_quads.count() > 100);
    modal.emit_ui(&avatar, &mut modal_ui);
    assert_eq!(modal_ui.count(), stage_quads.count(), "InspectionModal must emit foreground 3D satchel stage");

    // Opening Escape pause menu hides the satchel stage behind the pause card
    modal.toggle_escape(&surface);
    let (mut raw_pause, mut decorated_pause) = (QuadCapture::empty(), QuadCapture::empty());
    let mut raw_menu = EscapeMenu::closed();
    raw_menu.toggle_escape(&surface);
    raw_menu.emit_ui(&avatar, &mut raw_pause);
    modal.emit_ui(&avatar, &mut decorated_pause);
    assert_eq!(decorated_pause.count(), raw_pause.count());
    modal.toggle_escape(&surface);

    // Closing satchel restores the 10-quad gameplay crosshair cleanly
    assert!(avatar.dispatch_wardrobe_key(KeyCode::Tab));
    let mut closed_ui = QuadCapture::empty();
    modal.emit_ui(&avatar, &mut closed_ui);
    assert_eq!(closed_ui.count(), 10);

    // End-to-end HeadlessSession check with default InspectionModal wrapper
    let mut session = HeadlessSession::with_world_and_modal(
        Box::new(world),
        Box::new(PlanetaryAvatar::at(close_pos)),
        HeadlessSession::closed_modal(),
    );
    session.advance_ticks(1);
    assert_eq!(session.ui_quad_count(), 10);
    session.input_key(KeyCode::Tab, true);
    session.advance_ticks(1);
    assert!(session.ui_quad_count() > 100);
}
