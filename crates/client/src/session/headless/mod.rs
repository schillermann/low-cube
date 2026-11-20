mod surface;

use std::cell::RefCell;
use std::rc::Rc;
use glam::Vec3;
use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, PlanetaryMesh, VoxelShaderSource};
use world::{BiomedCubeWorld, PlanetaryWorld, Quad};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::black_box::{BlackBoxAvatar, LiveTape};
use crate::controls::{ActionBindings, InputTrigger, KeyMotion};
use crate::menu::{EscapeMenu, ScreenPoint, SessionModal};
use crate::server_satchel::ServerSatchel;
use crate::surface::PlatformSurface;
use super::display::{PolaroidScenePass, SessionDisplay, ShadedScenePass};
use super::pulse::PulseBeat;
use super::stage::SessionStage;
use self::surface::HeadlessPipeline;

pub use self::surface::HeadlessSurface;

/// Interactive windowless game session driving `SessionStage` and `SessionDisplay` without a GPU or X11/Wayland server.
pub struct HeadlessSession {
    stage: SessionStage,
    display: SessionDisplay,
    ui_quads: Rc<RefCell<Vec<Quad>>>,
}

impl HeadlessSession {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(stage: SessionStage, display: SessionDisplay, ui_quads: Rc<RefCell<Vec<Quad>>>) -> Self {
        Self { stage, display, ui_quads }
    }

    pub fn with_world_and_modal(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        modal: Box<dyn SessionModal>,
    ) -> Self {
        let ui_quads = Rc::new(RefCell::new(Vec::new()));
        let surface = Box::new(HeadlessSurface::hd(Rc::clone(&ui_quads)));
        let slice_count = world.face_count() * 2 + 1;
        let mut primary_slices: Vec<Box<dyn engine::Mesh>> = Vec::with_capacity(slice_count);
        let empty_cpu = CpuMeshBatch::empty();
        for idx in 0..slice_count {
            let patch_count = world.face_patch_count(idx);
            if patch_count > 0 {
                let patches = (0..patch_count).map(|_| surface.upload_mesh(&empty_cpu)).collect();
                primary_slices.push(Box::new(engine::SectorGridMesh::new(patches)));
            } else {
                primary_slices.push(surface.upload_mesh(&empty_cpu));
            }
        }
        let sh = surface.compile_shader(&VoxelShaderSource::embedded());
        let shaded = PolaroidScenePass::new(ShadedScenePass::new(
            (Box::new(HeadlessPipeline), sh),
            PlanetaryMesh::with_mask(primary_slices, u32::MAX),
            engine::StreamingMesh::empty(),
            world::MeadowRing::cold(),
        ));
        let display = SessionDisplay::new(surface, Box::new(shaded));
        let controls = Box::new(KeyMotion::with_bindings(ActionBindings::seeded()));
        let stage = SessionStage::new(world, avatar, controls, modal);
        Self::new(stage, display, ui_quads)
    }

    pub fn solitary(blocks: u32, modal: Box<dyn SessionModal>) -> Self {
        let world: Box<dyn PlanetaryWorld> = Box::new(BiomedCubeWorld::barren_beveled(blocks, 1.0));
        let half = (blocks as f32) * 0.5;
        let spawn = world.constrain_to_surface(Vec3::new(0.25, half + 2.0, 0.25), 1.0);
        let inner = PlanetaryAvatar::with_attire_and_bindings(
            spawn,
            Box::new(crate::zero_g_flight::ZeroGFlight::wrapping(
                crate::flight::SurfaceFlight::calibrated(0.522, 10.0, 1.0),
            )),
            YouthAttire::squire(),
            ActionBindings::seeded(),
        )
        .with_starting_satchel(&ServerSatchel::assembled());
        let avatar = Box::new(BlackBoxAvatar::wrapping(Box::new(inner), Box::new(LiveTape::fresh())));
        Self::with_world_and_modal(world, avatar, modal)
    }

    pub fn modal_open(&self) -> bool { self.stage.modal_open() }
    pub fn dialing_active(&self) -> bool { self.stage.dialing_active() }
    pub fn companion_count(&self) -> usize { self.stage.companion_count() }
    pub fn begin_bound_dial(&mut self, link: sanctuary::RemoteLink) -> String {
        self.stage.begin_bound_dial(link)
    }
    pub fn focus_window(&mut self) { self.stage.focus_regained(); }
    pub fn press_escape(&mut self) { self.stage.toggle_escape(self.display.surface()); }
    pub fn move_cursor(&mut self, point: ScreenPoint) { self.stage.move_cursor(point); }
    pub fn click_modal(&mut self) { let _ = self.stage.press_modal_mouse(&mut self.display); }
    pub fn rotate_mouse(&mut self, dx: f32, dy: f32) { self.stage.rotate_avatar(dx, dy); }

    pub fn type_text(&mut self, text: &str) {
        if self.stage.modal_open() {
            self.stage.handle_modal_text(text);
        } else {
            let _ = self.stage.handle_avatar_text(text);
        }
    }

    pub fn input_key(&mut self, code: KeyCode, pressed: bool) {
        if code == KeyCode::Escape && pressed {
            self.press_escape();
        } else if self.stage.modal_open() {
            if pressed {
                let _ = self.stage.handle_modal_key(code, &mut self.display);
            } else {
                self.stage.release_control_key(code);
            }
        } else if self.stage.dispatch_avatar_or_control(InputTrigger::Key(code), pressed) {
            self.stage.refresh_surface(&mut self.display);
        }
    }

    pub fn use_left_hand(&mut self) {
        if self.stage.use_left() {
            self.stage.refresh_surface(&mut self.display);
        }
    }

    pub fn use_right_hand(&mut self) {
        if self.stage.use_right() {
            self.stage.refresh_surface(&mut self.display);
        }
    }

    pub fn release_hand(&mut self) {
        if self.stage.release_hand() {
            self.stage.refresh_surface(&mut self.display);
        }
    }

    pub fn advance_ticks(&mut self, due_ticks: u32) {
        let beat = PulseBeat::new(due_ticks, 0.0, 1.0);
        self.stage.step_frame(beat, &mut self.display);
        self.stage.render_onto(&mut self.display, 1.0);
    }

    pub fn ui_quad_count(&self) -> usize {
        self.ui_quads.borrow().len()
    }

    pub fn closed_modal() -> Box<dyn SessionModal> {
        Box::new(EscapeMenu::with_clipboard(
            crate::menu::PortalInvite::empty(),
            crate::menu::SystemClipboard::new().detached(),
        ))
    }
}
