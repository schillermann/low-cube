use std::cell::{Cell, RefCell};
use std::rc::Rc;
use glam::Vec3;
use engine::{FramePass, GraphicPipeline, Mesh, PointLightCluster, QuadBatch};
use world::{ClimateZone, MeshTarget, Quad};
use crate::attire::SightMode;
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::AuroraCanopy;
use crate::avatar::Avatar;
use crate::black_box::PresenceProbe;
use crate::celestial::CelestialCadence;
use crate::menu::{ScreenPoint, SessionModal};
use crate::surface::PlatformSurface;

struct HeadlessMesh {
    quad_count: usize,
}

impl Mesh for HeadlessMesh {
    fn draw(&self, _pass: &mut dyn FramePass) {
        let _ = self.quad_count;
    }
}

pub(super) struct HeadlessPipeline;

impl GraphicPipeline for HeadlessPipeline {
    fn render(&self, _pass: &mut dyn FramePass, _mesh: &dyn Mesh, _view_proj: &glam::Mat4) {}
}

struct QuadSink {
    quads: Vec<Quad>,
}

impl MeshTarget for QuadSink {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

/// Windowless Fake-Object implementation of `PlatformSurface` enabling autonomous headless game sessions (`3` fields).
pub struct HeadlessSurface {
    viewport: (u32, u32),
    captured: Cell<bool>,
    ui_quads: Rc<RefCell<Vec<Quad>>>,
}

impl HeadlessSurface {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(viewport: (u32, u32), captured: Cell<bool>, ui_quads: Rc<RefCell<Vec<Quad>>>) -> Self {
        Self { viewport, captured, ui_quads }
    }

    pub fn hd(ui_quads: Rc<RefCell<Vec<Quad>>>) -> Self {
        Self::new((1280, 720), Cell::new(true), ui_quads)
    }

    pub fn cursor_captured(&self) -> bool {
        self.captured.get()
    }
}

impl PlatformSurface for HeadlessSurface {
    fn resize(&mut self, width: u32, height: u32) {
        self.viewport = (width.max(1), height.max(1));
    }

    fn capture_cursor(&self) {
        self.captured.set(true);
    }

    fn release_cursor(&self) {
        self.captured.set(false);
    }

    fn normalize_cursor(&self, x: f64, y: f64) -> ScreenPoint {
        let (w, h) = (self.viewport.0.max(1) as f32, self.viewport.1.max(1) as f32);
        ScreenPoint::new((x as f32 / w) * 2.0 - 1.0, 1.0 - (y as f32 / h) * 2.0)
    }

    fn upload_mesh(&self, batch: &QuadBatch) -> Box<dyn Mesh> {
        Box::new(HeadlessMesh { quad_count: batch.total_quad_count() })
    }

    fn assemble_diurnal_pipeline(
        &self,
        _cadence: Box<dyn CelestialCadence>,
        _aurora: Box<dyn AuroraCadence>,
        _canopy: Box<dyn AuroraCanopy>,
    ) -> Box<dyn GraphicPipeline> {
        Box::new(HeadlessPipeline)
    }

    fn render_frame(
        &self,
        _pipeline: &dyn GraphicPipeline,
        _scenes: (&dyn Mesh, &dyn Mesh),
        _lights: &PointLightCluster,
        avatar: &dyn Avatar,
        modal: &dyn SessionModal,
    ) {
        let mut previews = Vec::new();
        let _ = avatar.collect_camera_previews(1.62, &mut previews);
        let mut intercom = QuadSink { quads: Vec::new() };
        let _ = avatar.emit_intercom_bust(1.62, &mut intercom);
        let mut sink = QuadSink { quads: Vec::with_capacity(256) };
        modal.emit_ui(avatar, &mut sink);
        *self.ui_quads.borrow_mut() = sink.quads;
    }

    fn capture_sight_frame(
        &self,
        _pipeline: &dyn GraphicPipeline,
        _mesh: &dyn Mesh,
        _lights: &PointLightCluster,
        avatar: &dyn Avatar,
        sight: SightMode,
        angle_label: &str,
    ) -> Vec<Vec3> {
        let mut probe = PresenceProbe::at_origin();
        avatar.report_presence(&mut probe);
        let zone = ClimateZone::dominant(probe.position());
        let turf = match zone {
            ClimateZone::Temperate => Vec3::new(0.28, 0.58, 0.32),
            ClimateZone::Desert => Vec3::new(0.84, 0.72, 0.42),
            ClimateZone::Arctic => Vec3::new(0.82, 0.90, 0.96),
            ClimateZone::Jungle => Vec3::new(0.16, 0.48, 0.24),
            ClimateZone::Volcanic => Vec3::new(0.42, 0.22, 0.18),
            ClimateZone::Steppe => Vec3::new(0.66, 0.62, 0.32),
        };
        let sky = if angle_label == "incident" {
            Vec3::new(0.46, 0.64, 0.88)
        } else {
            Vec3::new(0.38, 0.66, 0.92)
        };
        let mut mosaic = Vec::with_capacity(300);
        for r in 0..15 {
            for c in 0..20 {
                let figure = !matches!(sight, SightMode::FirstPersonSight)
                    && (8..12).contains(&c)
                    && (3..11).contains(&r);
                mosaic.push(if figure {
                    Vec3::new(0.80, 0.66, 0.50)
                } else if r < 7 {
                    turf
                } else {
                    sky
                });
            }
        }
        mosaic
    }
}
