mod pass;
mod shaded;
mod tron;

use fauna::WoodlandFauna;
use world::PlanetaryWorld;
use crate::avatar::Avatar;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;
pub use self::pass::{PolaroidScenePass, ScenePass};
pub use self::shaded::ShadedScenePass;
pub use self::tron::TronScenePass;

/// Polymorphic visual pass selector avoiding raw boolean mode branching across scene operations.
#[derive(Clone, Copy, Debug)]
pub enum VisualPass {
    Shaded,
    Tron,
}

impl VisualPass {
    pub const fn from_flag(tron_wireframe: bool) -> Self {
        if tron_wireframe { Self::Tron } else { Self::Shaded }
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Shaded => Self::Tron,
            Self::Tron => Self::Shaded,
        }
    }
}

/// Encapsulated platform window surface and Polaroid-decorated shaded/Tron rendering passes.
pub struct SessionDisplay {
    surface: Box<dyn PlatformSurface>,
    shaded: Box<dyn ScenePass>,
    tron: Box<dyn ScenePass>,
    mode: VisualPass,
}

impl SessionDisplay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        surface: Box<dyn PlatformSurface>,
        shaded: Box<dyn ScenePass>,
        tron: Box<dyn ScenePass>,
        mode: VisualPass,
    ) -> Self {
        Self { surface, shaded, tron, mode }
    }

    fn active_pass(&self) -> &dyn ScenePass {
        match self.mode {
            VisualPass::Shaded => self.shaded.as_ref(),
            VisualPass::Tron => self.tron.as_ref(),
        }
    }

    fn active_pass_and_surface(&mut self) -> (&mut dyn ScenePass, &dyn PlatformSurface) {
        let surface = self.surface.as_ref();
        let pass = match self.mode {
            VisualPass::Shaded => self.shaded.as_mut(),
            VisualPass::Tron => self.tron.as_mut(),
        };
        (pass, surface)
    }

    pub(super) fn surface(&self) -> &dyn PlatformSurface {
        self.surface.as_ref()
    }

    pub(super) fn tron_enabled(&self) -> bool {
        self.active_pass().tron_wireframe()
    }

    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(width, height);
    }

    pub(super) fn capture_cursor(&self) {
        self.surface.capture_cursor();
    }

    pub(super) fn present(&self) {
        self.surface.present_frame();
    }

    pub(super) fn request_redraw(&self) {
        self.surface.request_redraw();
    }

    pub(super) fn refresh_face_slice(&mut self, world: &dyn PlanetaryWorld, idx: usize) {
        let (pass, surface) = self.active_pass_and_surface();
        pass.refresh_face_slice(surface, world, idx);
    }

    pub(super) fn refresh_meadow_slice(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        let mask = avatar.visible_slices(world);
        self.shaded.select_visible(mask);
        self.tron.select_visible(mask);
        let (pass, surface) = self.active_pass_and_surface();
        pass.refresh_meadow_slice(surface, world, avatar);
    }

    pub(super) fn refresh_modified_face(&mut self, world: &mut dyn PlanetaryWorld) {
        let (pass, surface) = self.active_pass_and_surface();
        pass.refresh_modified_face(surface, world);
    }

    pub(super) fn refresh_world_mesh(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        for idx in 0..(world.face_count() * 2) {
            self.refresh_face_slice(world, idx);
        }
        self.refresh_meadow_slice(world, avatar);
    }

    pub(super) fn toggle_tron(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        self.mode = self.mode.toggled();
        self.refresh_world_mesh(world, avatar);
    }

    pub(super) fn render_scene(
        &mut self,
        world: &dyn PlanetaryWorld,
        avatar: &mut dyn Avatar,
        fauna: &WoodlandFauna,
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        let (pass, surface) = self.active_pass_and_surface();
        pass.render_with_shutter(surface, world, avatar, fauna, modal, sight_scale);
    }
}
