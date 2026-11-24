mod pass;
mod shaded;
mod vessel_stage;

use world::PlanetaryWorld;
use crate::avatar::Avatar;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;
pub use self::pass::{PolaroidScenePass, ScenePass};
pub use self::shaded::ShadedScenePass;

/// Encapsulated platform window surface and Polaroid-decorated rendering pass.
pub struct SessionDisplay {
    surface: Box<dyn PlatformSurface>,
    pass: Box<dyn ScenePass>,
}

impl SessionDisplay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(surface: Box<dyn PlatformSurface>, pass: Box<dyn ScenePass>) -> Self {
        Self { surface, pass }
    }

    pub(super) fn surface(&self) -> &dyn PlatformSurface {
        self.surface.as_ref()
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
        self.pass.refresh_face_slice(self.surface.as_ref(), world, idx);
    }

    pub(super) fn refresh_meadow_slice(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        let mask = avatar.visible_slices(world);
        self.pass.select_visible(mask);
        self.pass.refresh_meadow_slice(self.surface.as_ref(), world, avatar);
    }

    pub(super) fn stream_meadow_slice(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar, reopen: bool) {
        if reopen {
            let mask = avatar.visible_slices(world);
            self.pass.select_visible(mask);
        }
        self.pass.stream_meadow_slice(self.surface.as_ref(), world, avatar, reopen);
    }

    pub(super) fn refresh_modified_face(&mut self, world: &mut dyn PlanetaryWorld) {
        self.pass.refresh_modified_face(self.surface.as_ref(), world);
    }

    pub(super) fn refresh_world_mesh(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        self.pass.reset_horizon();
        let mask = avatar.visible_slices(world);
        for idx in 0..(world.face_count() * 2) {
            if (mask >> idx.min(31)) & 1 != 0 {
                self.refresh_face_slice(world, idx);
            }
        }
        self.refresh_meadow_slice(world, avatar);
    }

    pub(super) fn render_scene(
        &mut self,
        world: &dyn PlanetaryWorld,
        avatar: &mut dyn Avatar,
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        self.pass.render_with_shutter(self.surface.as_ref(), world, avatar, modal, sight_scale);
    }
}
