use world::PlanetaryWorld;
use crate::attire::{PhotoShutter, SightMode};
use crate::avatar::Avatar;
use crate::menu::SessionModal;
use crate::surface::PlatformSurface;

/// Polymorphic rendering pass refreshing world slices and drawing the 3D scene.
pub trait ScenePass {
    fn select_visible(&mut self, mask: u32);
    fn reset_horizon(&mut self) {}
    fn refresh_face_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, idx: usize);
    fn refresh_meadow_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, avatar: &dyn Avatar);
    fn stream_meadow_slice(
        &mut self,
        surface: &dyn PlatformSurface,
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        reopen: bool,
    ) {
        if reopen {
            self.refresh_meadow_slice(surface, world, avatar);
        }
    }
    fn refresh_modified_face(&mut self, surface: &dyn PlatformSurface, world: &mut dyn PlanetaryWorld);
    fn capture_sight(
        &mut self,
        surface: &dyn PlatformSurface,
        scene: (&dyn PlanetaryWorld, &dyn Avatar),
        sight: SightMode,
        sight_scale: f32,
    ) -> Vec<glam::Vec3>;
    fn draw_frame(
        &mut self,
        surface: &dyn PlatformSurface,
        scene: (&dyn PlanetaryWorld, &dyn Avatar),
        modal: &dyn SessionModal,
        sight_scale: f32,
    );
    fn render_with_shutter(
        &mut self,
        surface: &dyn PlatformSurface,
        world: &dyn PlanetaryWorld,
        avatar: &mut dyn Avatar,
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        self.draw_frame(surface, (world, avatar), modal, sight_scale);
    }
}

/// Decorator over any `ScenePass` developing triggered Polaroid camera exposures prior to drawing the live frame.
pub struct PolaroidScenePass<P> {
    origin: P,
}

impl<P> PolaroidScenePass<P> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: P) -> Self {
        Self { origin }
    }
}

impl<P: ScenePass> ScenePass for PolaroidScenePass<P> {
    fn select_visible(&mut self, mask: u32) {
        self.origin.select_visible(mask);
    }

    fn reset_horizon(&mut self) {
        self.origin.reset_horizon();
    }

    fn refresh_face_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, idx: usize) {
        self.origin.refresh_face_slice(surface, world, idx);
    }

    fn refresh_meadow_slice(&mut self, surface: &dyn PlatformSurface, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        self.origin.refresh_meadow_slice(surface, world, avatar);
    }

    fn stream_meadow_slice(
        &mut self,
        surface: &dyn PlatformSurface,
        world: &dyn PlanetaryWorld,
        avatar: &dyn Avatar,
        reopen: bool,
    ) {
        self.origin.stream_meadow_slice(surface, world, avatar, reopen);
    }

    fn refresh_modified_face(&mut self, surface: &dyn PlatformSurface, world: &mut dyn PlanetaryWorld) {
        self.origin.refresh_modified_face(surface, world);
    }

    fn capture_sight(
        &mut self,
        surface: &dyn PlatformSurface,
        scene: (&dyn PlanetaryWorld, &dyn Avatar),
        sight: SightMode,
        sight_scale: f32,
    ) -> Vec<glam::Vec3> {
        self.origin.capture_sight(surface, scene, sight, sight_scale)
    }

    fn draw_frame(
        &mut self,
        surface: &dyn PlatformSurface,
        scene: (&dyn PlanetaryWorld, &dyn Avatar),
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        self.origin.draw_frame(surface, scene, modal, sight_scale);
    }

    fn render_with_shutter(
        &mut self,
        surface: &dyn PlatformSurface,
        world: &dyn PlanetaryWorld,
        avatar: &mut dyn Avatar,
        modal: &dyn SessionModal,
        sight_scale: f32,
    ) {
        if let PhotoShutter::Triggered(sight) = avatar.take_photo_shutter() {
            let mosaic = self.origin.capture_sight(surface, (world, avatar), sight, sight_scale);
            avatar.apply_polaroid_mosaic(mosaic);
        }
        self.origin.draw_frame(surface, (world, avatar), modal, sight_scale);
    }
}
