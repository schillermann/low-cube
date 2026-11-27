use std::cell::RefCell;
use std::sync::Arc;
use glam::Vec3;
use winit::window::Window;

use engine::{
    GraphicPipeline, Mesh, PassDraft, PointLightCluster, QuadBatch, SlotReclamation, WgpuStage,
};
use crate::attire::SightMode;
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::AuroraCanopy;
use crate::avatar::Avatar;
use crate::celestial::CelestialCadence;
use crate::diurnal_pipeline::{DiurnalStageAssembly, WgpuDiurnalStage};
use crate::menu::{PreviewStripLayout, ScreenPoint, SessionModal, SystemClipboard, TextClipboard};
use crate::photo_plate::PhotoPlate;
use crate::preview_strip::PreviewAtlas;
use crate::rayleigh_shell::{PlateauZenith, ZenithField};

/// Interface for platform window and `wgpu` display surface operations.
pub trait PlatformSurface {
    fn resize(&mut self, width: u32, height: u32);
    fn capture_cursor(&self);
    fn release_cursor(&self);
    fn normalize_cursor(&self, x: f64, y: f64) -> ScreenPoint;
    fn attach_clipboard(&self) -> Box<dyn TextClipboard> {
        SystemClipboard::new().detached()
    }
    fn upload_mesh(&self, batch: &QuadBatch) -> Box<dyn Mesh>;
    fn replace_mesh(&self, _previous: &dyn Mesh, batch: &QuadBatch) -> Box<dyn Mesh> {
        self.upload_mesh(batch)
    }
    fn assemble_diurnal_pipeline(
        &self,
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Box<dyn GraphicPipeline>;
    fn render_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        scenes: (&dyn Mesh, &dyn Mesh),
        lights: &PointLightCluster,
        avatar: &dyn Avatar,
        modal: &dyn SessionModal,
    );
    fn present_frame(&self) {}
    fn request_redraw(&self) {}
    fn capture_sight_frame(
        &self,
        _pipeline: &dyn GraphicPipeline,
        _mesh: &dyn Mesh,
        _lights: &PointLightCluster,
        _avatar: &dyn Avatar,
        _sight: SightMode,
        _angle_label: &str,
    ) -> Vec<Vec3> {
        Vec::new()
    }
}

enum FrameLease {
    Idle,
    Acquired(wgpu::SurfaceTexture),
}

struct SwapChainState {
    stage: WgpuStage,
    config: wgpu::SurfaceConfiguration,
    lease: FrameLease,
}

/// Encapsulated `winit` window, `wgpu` Vulkan/DX12 surface, `<= 1.5 MB/Frame` stage, and `240×160` monitor atlas (`3` fields).
pub struct WgpuPlatformSurface {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    state: RefCell<(SwapChainState, PreviewAtlas, PassDraft)>,
}

impl WgpuPlatformSurface {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn with_state(
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        state: RefCell<(SwapChainState, PreviewAtlas, PassDraft)>,
    ) -> Self {
        Self { window, surface, state }
    }

    pub fn new(
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        stage: WgpuStage,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let swap = SwapChainState { stage, config, lease: FrameLease::Idle };
        Self::with_state(window, surface, RefCell::new((swap, PreviewAtlas::dormant(), PassDraft::empty())))
    }

    fn aspect_ratio(&self) -> f32 {
        let size = self.window.inner_size();
        (size.width as f32) / (size.height.max(1) as f32)
    }
}

impl PlatformSurface for WgpuPlatformSurface {
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            let mut guard = self.state.borrow_mut();
            let swap = &mut guard.0;
            swap.config.width = width;
            swap.config.height = height;
            self.surface.configure(swap.stage.device(), &swap.config);
            swap.stage.resize_targets(width, height);
        }
    }

    fn capture_cursor(&self) {
        let _ = self
            .window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
        self.window.set_cursor_visible(false);
    }

    fn release_cursor(&self) {
        let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::None);
        self.window.set_cursor_visible(true);
    }

    fn normalize_cursor(&self, x: f64, y: f64) -> ScreenPoint {
        let size = self.window.inner_size();
        let w = size.width.max(1) as f32;
        let h = size.height.max(1) as f32;
        ScreenPoint::new((x as f32 / w) * 2.0 - 1.0, 1.0 - (y as f32 / h) * 2.0)
    }

    fn attach_clipboard(&self) -> Box<dyn TextClipboard> {
        SystemClipboard::new().attach_window(&self.window)
    }

    fn upload_mesh(&self, batch: &QuadBatch) -> Box<dyn Mesh> {
        self.state.borrow_mut().0.stage.upload_patch(batch)
    }

    fn replace_mesh(&self, previous: &dyn Mesh, batch: &QuadBatch) -> Box<dyn Mesh> {
        let mut guard = self.state.borrow_mut();
        let stage: &mut dyn SlotReclamation = &mut guard.0.stage;
        previous.release(stage);
        guard.0.stage.upload_patch(batch)
    }

    fn assemble_diurnal_pipeline(
        &self,
        cadence: Box<dyn CelestialCadence>,
        aurora: Box<dyn AuroraCadence>,
        canopy: Box<dyn AuroraCanopy>,
    ) -> Box<dyn GraphicPipeline> {
        WgpuDiurnalStage::new(cadence, aurora, canopy).assemble_stage()
    }

    fn render_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        (mesh, coarse_mesh): (&dyn Mesh, &dyn Mesh),
        lights: &PointLightCluster,
        avatar: &dyn Avatar,
        modal: &dyn SessionModal,
    ) {
        let mut guard = self.state.borrow_mut();
        let (ref mut swap, ref mut monitors, ref mut draft) = *guard;
        swap.stage.begin_frame_uploads();

        let frame = match self.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(_) => {
                self.surface.configure(swap.stage.device(), &swap.config);
                let Ok(tex) = self.surface.get_current_texture() else { return };
                tex
            }
        };
        let target_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let size = self.window.inner_size();
        let aspect = self.aspect_ratio();
        let viewer = avatar.sight_cone().observe_from(|eye| eye);
        let up = PlateauZenith::new().zenith_at(viewer);

        draft.reset();
        let view_proj = avatar.view_projection_matrix(aspect);
        pipeline.render(draft, mesh, &view_proj);
        swap.stage.submit_main_draft(&target_view, draft, (viewer, up, 1.0), lights);

        if !modal.open() {
            let mut views = Vec::new();
            let active_slot = avatar.collect_camera_previews(PreviewStripLayout::TILE_ASPECT, &mut views);
            let mut intercom_batch = QuadBatch::empty();
            let intercom = avatar
                .emit_intercom_bust(PreviewStripLayout::TILE_ASPECT, &mut intercom_batch)
                .map(|vp| (vp, &intercom_batch));
            monitors.render_and_blit(
                &swap.stage,
                &target_view,
                pipeline,
                coarse_mesh,
                (&views, active_slot, (viewer, up)),
                (intercom, lights, (size.width, size.height)),
            );
        }

        draft.reset();
        modal.draw_layer(draft, avatar, aspect);
        swap.stage.submit_overlay_draft(&target_view, None, draft, lights);
        swap.lease = FrameLease::Acquired(frame);
    }

    fn present_frame(&self) {
        let prior = std::mem::replace(&mut self.state.borrow_mut().0.lease, FrameLease::Idle);
        if let FrameLease::Acquired(frame) = prior {
            frame.present();
        }
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn capture_sight_frame(
        &self,
        pipeline: &dyn GraphicPipeline,
        mesh: &dyn Mesh,
        lights: &PointLightCluster,
        avatar: &dyn Avatar,
        sight: SightMode,
        angle_label: &str,
    ) -> Vec<Vec3> {
        let size = self.window.inner_size();
        let (width, height) = (size.width.max(1), size.height.max(1));
        let aspect = (width as f32) / (height as f32);
        let view_proj = avatar.view_projection_for_sight(aspect, sight);
        let viewer = avatar.sight_cone_for_mode(sight).observe_from(|eye| eye);
        let up = PlateauZenith::new().zenith_at(viewer);

        let mut guard = self.state.borrow_mut();
        let (ref swap, _, ref mut draft) = *guard;
        draft.reset();
        pipeline.render(draft, mesh, &view_proj);
        let raw_rgba = swap.stage.capture_polaroid_rgba((width, height), draft, (viewer, up, 1.0), lights);
        let plate = PhotoPlate::new(width, height, raw_rgba);
        let _ = plate.develop_to_disk(angle_label);
        plate.sample_mosaic()
    }
}
