use glam::Vec3;
use winit::keyboard::KeyCode;
use engine::QuadBatch;
use world::{
    BiomedCubeWorld, Callsign, CompanionPortal, CountedMeshTarget, CubePlanet, PlanetaryWorld,
    ProximityUtterance,
};
use crate::avatar::PlanetaryAvatar;
use crate::controls::InputTrigger;
use crate::menu::{
    EscapeMenu, InspectionModal, ModalDirective, PortalInvite, ProximityChatModal, SessionModal,
    SystemClipboard,
};
use super::MockSurface;

struct LoopbackChatWorld {
    origin: CubePlanet,
    spoken: Vec<String>,
    inbox: Vec<ProximityUtterance>,
}

impl LoopbackChatWorld {
    fn new(inbox: Vec<ProximityUtterance>) -> Self {
        Self { origin: BiomedCubeWorld::of_size(16), spoken: Vec::new(), inbox }
    }
}

impl world::GravityField for LoopbackChatWorld {
    fn pull_direction_at(&self, p: Vec3) -> Vec3 { self.origin.pull_direction_at(p) }
    fn surface_up_at(&self, p: Vec3) -> Vec3 { self.origin.surface_up_at(p) }
}
impl world::WorldEarthworks for LoopbackChatWorld {}
impl world::TerrainMesh for LoopbackChatWorld {
    fn emit_surface(&self, t: &mut dyn world::MeshTarget) { self.origin.emit_surface(t); }
}
impl world::VesselDeck for LoopbackChatWorld {}
impl CompanionPortal for LoopbackChatWorld {
    fn utter_nearby(&mut self, text: &str) {
        self.spoken.push(text.to_string());
    }
    fn drain_utterances(&mut self, out: &mut Vec<ProximityUtterance>) {
        out.append(&mut self.inbox);
    }
}
impl PlanetaryWorld for LoopbackChatWorld {
    fn constrain_to_surface(&self, p: Vec3, h: f32) -> Vec3 { self.origin.constrain_to_surface(p, h) }
}

#[test]
fn test_proximity_chat_modal_decorator_composes_submits_and_fades_feed() {
    let base = InspectionModal::new(EscapeMenu::with_clipboard(
        PortalInvite::empty(),
        SystemClipboard::new().detached(),
    ));
    let mut modal = ProximityChatModal::wrapping(base);
    let surface = MockSurface::active();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let incoming = ProximityUtterance::named(4, Callsign::sanitized("Tom"), Vec3::ZERO, "Hi there")
        .expect("utterance");
    assert_eq!(incoming.formatted_line(), "Tom: Hi there");
    let mut world = LoopbackChatWorld::new(vec![incoming]);

    let mut base_batch = QuadBatch::empty();
    let mut base_counter = CountedMeshTarget::new(&mut base_batch);
    modal.emit_ui(&avatar, &mut base_counter);
    let reticle_quads = base_counter.count();

    assert!(!modal.open());
    assert!(modal.toggle_gameplay_trigger(InputTrigger::Key(KeyCode::KeyT), &surface, &mut avatar, &world));
    assert!(modal.open(), "Pressing KeyT must open ProximityChatModal");

    modal.handle_text("t");
    modal.handle_text("Glück auf!");
    assert_eq!(modal.handle_key(KeyCode::Enter, false, &mut avatar), ModalDirective::ContinueSession);
    assert!(!modal.open(), "Submitting with Enter must return to gameplay");

    modal.sync_portal_chat(&mut world);
    assert_eq!(world.spoken, vec!["Glück auf!"]);

    let mut ui_batch = QuadBatch::empty();
    let mut ui_counter = CountedMeshTarget::new(&mut ui_batch);
    modal.emit_ui(&avatar, &mut ui_counter);
    assert!(ui_counter.count() > reticle_quads + 20, "Active proximity chat feed must emit HUD glyph quads");

    modal.step(13.0);
    let mut faded_batch = QuadBatch::empty();
    let mut faded_counter = CountedMeshTarget::new(&mut faded_batch);
    modal.emit_ui(&avatar, &mut faded_counter);
    assert_eq!(faded_counter.count(), reticle_quads, "Proximity chat lines must expire cleanly after TTL");
}
