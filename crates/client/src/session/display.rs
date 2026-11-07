use engine::{CompositeMesh, CpuMeshBatch, CpuTronBatch, GraphicPipeline, PlanetaryMesh, ShaderProgram};
use world::PlanetaryWorld;
use crate::attire::{PhotoShutter, SightMode};
use crate::avatar::Avatar;
use crate::clock::{MonotonicTime, SystemClock};
use crate::controls::{ControlAction, InputTrigger, KeyboardControls};
use crate::menu::{ModalDirective, SessionModal};
use crate::surface::PlatformSurface;

/// Encapsulated monotonic frame delta tracker decoupling redraw steps from static clock calls.
pub(super) struct FramePace {
    clock: Box<dyn MonotonicTime>,
    last_tick: std::time::Instant,
}

impl FramePace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(clock: Box<dyn MonotonicTime>, last_tick: std::time::Instant) -> Self {
        Self { clock, last_tick }
    }

    /// Secondary constructor anchored at the current monotonic instant.
    pub(super) fn realtime() -> Self {
        let clock = Box::new(SystemClock::new());
        let now = clock.current_instant();
        Self::new(clock, now)
    }

    pub(super) fn advance(&mut self) -> f32 {
        let now = self.clock.current_instant();
        let dt = now.duration_since(self.last_tick).as_secs_f32().clamp(1.0 / 240.0, 0.10);
        self.last_tick = now;
        dt
    }
}

/// Encapsulated graphics pipeline, shader program, and GPU mesh slices for a render mode.
pub(super) struct RenderPass {
    pipeline: Box<dyn GraphicPipeline>,
    shader: Box<dyn ShaderProgram>,
    mesh: PlanetaryMesh,
}

impl RenderPass {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(pipeline: Box<dyn GraphicPipeline>, shader: Box<dyn ShaderProgram>, mesh: PlanetaryMesh) -> Self {
        Self { pipeline, shader, mesh }
    }
}

/// Encapsulated platform window surface and dual primary/Tron rendering passes.
pub(super) struct SessionDisplay {
    surface: Box<dyn PlatformSurface>,
    primary: RenderPass,
    tron: RenderPass,
    tron_active: bool,
}

impl SessionDisplay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(surface: Box<dyn PlatformSurface>, primary: RenderPass, tron: RenderPass, tron_active: bool) -> Self {
        Self { surface, primary, tron, tron_active }
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

    pub(super) fn refresh_face_slice(&mut self, world: &dyn PlanetaryWorld, idx: usize) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            world.emit_face(idx, &mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron.mesh.slice_at(idx), &tron_buf);
            self.tron.mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::empty();
            world.emit_face(idx, &mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(idx), &cpu_buf);
            self.primary.mesh.store_slice(idx, next);
        }
    }

    pub(super) fn refresh_meadow_slice(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        let (ax, ay, az) = avatar.meadow_anchor();
        let pos = glam::Vec3::new((ax as f32) * 0.5, (ay as f32) * 0.5, (az as f32) * 0.5);
        let mask = world.visible_slice_mask(pos);
        self.primary.mesh.select_visible(mask);
        self.tron.mesh.select_visible(mask);
        if !self.tron_active {
            let meadow_idx = world.face_count() * 2;
            let mut meadow_buf = CpuMeshBatch::empty();
            avatar.emit_surroundings(world, &mut meadow_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(meadow_idx), &meadow_buf);
            self.primary.mesh.store_slice(meadow_idx, next);
        }
    }

    pub(super) fn refresh_modified_face(&mut self, world: &mut dyn PlanetaryWorld) {
        if self.tron_active {
            let mut tron_buf = CpuTronBatch::empty();
            let idx = world.emit_modified_face(&mut tron_buf);
            let next = self.surface.replace_tron_mesh(self.tron.mesh.slice_at(idx), &tron_buf);
            self.tron.mesh.store_slice(idx, next);
        } else {
            let mut cpu_buf = CpuMeshBatch::empty();
            let idx = world.emit_modified_face(&mut cpu_buf);
            let next = self.surface.replace_mesh(self.primary.mesh.slice_at(idx), &cpu_buf);
            self.primary.mesh.store_slice(idx, next);
        }
    }

    pub(super) fn refresh_modified_and_meadow(&mut self, stage: &mut SessionStage) {
        self.refresh_modified_face(stage.world_mut());
        self.refresh_meadow_slice(stage.world(), stage.avatar());
    }

    pub(super) fn refresh_world_mesh(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        for idx in 0..(world.face_count() * 2) {
            self.refresh_face_slice(world, idx);
        }
        self.refresh_meadow_slice(world, avatar);
    }

    pub(super) fn toggle_tron(&mut self, world: &dyn PlanetaryWorld, avatar: &dyn Avatar) {
        self.tron_active = !self.tron_active;
        self.refresh_world_mesh(world, avatar);
    }

    pub(super) fn render_stage(&self, stage: &mut SessionStage) {
        let shutter = stage.take_photo_shutter();
        if self.tron_active {
            if let PhotoShutter::Triggered(sight) = shutter {
                let mut shot_tron = CpuTronBatch::empty();
                stage.emit_avatar_for_sight(sight, &mut shot_tron);
                let composed = CompositeMesh::new(&self.tron.mesh, &shot_tron);
                let mosaic = self.surface.capture_sight_frame(
                    self.tron.pipeline.as_ref(),
                    self.tron.shader.as_ref(),
                    &composed,
                    stage.avatar(),
                    sight,
                    sight.photo_label(),
                );
                stage.apply_polaroid_mosaic(mosaic);
            }
            let mut avatar_tron = CpuTronBatch::empty();
            stage.emit_avatar(&mut avatar_tron);
            let composed = CompositeMesh::new(&self.tron.mesh, &avatar_tron);
            self.surface.render_frame(
                self.tron.pipeline.as_ref(),
                self.tron.shader.as_ref(),
                &composed,
                stage.avatar(),
                self.primary.shader.as_ref(),
                stage.modal(),
            );
        } else {
            if let PhotoShutter::Triggered(sight) = shutter {
                let mut shot_batch = CpuMeshBatch::empty();
                stage.emit_avatar_for_sight(sight, &mut shot_batch);
                let composed = CompositeMesh::new(&self.primary.mesh, &shot_batch);
                let mosaic = self.surface.capture_sight_frame(
                    self.primary.pipeline.as_ref(),
                    self.primary.shader.as_ref(),
                    &composed,
                    stage.avatar(),
                    sight,
                    sight.photo_label(),
                );
                stage.apply_polaroid_mosaic(mosaic);
            }
            let mut avatar_batch = CpuMeshBatch::empty();
            stage.emit_avatar(&mut avatar_batch);
            let composed = CompositeMesh::new(&self.primary.mesh, &avatar_batch);
            self.surface.render_frame(
                self.primary.pipeline.as_ref(),
                self.primary.shader.as_ref(),
                &composed,
                stage.avatar(),
                self.primary.shader.as_ref(),
                stage.modal(),
            );
        }
    }
}

/// Encapsulated planetary world, player avatar, keyboard controls, and pause/studio modal.
pub(super) struct SessionStage {
    world: Box<dyn PlanetaryWorld>,
    avatar: Box<dyn Avatar>,
    controls: Box<dyn KeyboardControls>,
    modal: Box<dyn SessionModal>,
}

impl SessionStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        world: Box<dyn PlanetaryWorld>,
        avatar: Box<dyn Avatar>,
        controls: Box<dyn KeyboardControls>,
        modal: Box<dyn SessionModal>,
    ) -> Self {
        Self { world, avatar, controls, modal }
    }

    pub(super) fn world(&self) -> &dyn PlanetaryWorld { self.world.as_ref() }
    pub(super) fn world_mut(&mut self) -> &mut dyn PlanetaryWorld { self.world.as_mut() }
    pub(super) fn avatar(&self) -> &dyn Avatar { self.avatar.as_ref() }
    pub(super) fn modal(&self) -> &dyn SessionModal { self.modal.as_ref() }
    pub(super) fn modal_open(&self) -> bool { self.modal.open() }
    pub(super) fn emit_avatar(&self, target: &mut dyn world::MeshTarget) { self.avatar.emit_appearance(target); }
    pub(super) fn emit_avatar_for_sight(&self, sight: SightMode, target: &mut dyn world::MeshTarget) {
        self.avatar.emit_appearance_for_sight(sight, target);
    }
    pub(super) fn take_photo_shutter(&mut self) -> PhotoShutter { self.avatar.take_photo_shutter() }
    pub(super) fn apply_polaroid_mosaic(&mut self, mosaic: Vec<glam::Vec3>) { self.avatar.apply_polaroid_mosaic(mosaic); }
    pub(super) fn move_cursor(&mut self, point: crate::menu::ScreenPoint) { self.modal.move_cursor(point); }
    pub(super) fn release_modal_mouse(&mut self) { self.modal.release_mouse(); }
    pub(super) fn rotate_avatar(&mut self, dx: f32, dy: f32) { self.avatar.rotate_view(dx, dy); }
    pub(super) fn use_left(&mut self) -> bool { self.avatar.use_left_hand(self.world.as_mut()) }
    pub(super) fn use_right(&mut self) -> bool { self.avatar.use_right_hand(self.world.as_mut()) }
    pub(super) fn release_hand(&mut self) -> bool { self.avatar.release_hand(self.world.as_mut()) }
    pub(super) fn matches_tron(&self, trigger: InputTrigger) -> bool { self.controls.matches_action(ControlAction::TronMode, trigger) }

    pub(super) fn dispatch_avatar_or_control(&mut self, trigger: InputTrigger, pressed: bool) -> bool {
        if pressed && self.avatar.adjust_aimed_rig(trigger, self.world.as_mut()) {
            return true;
        }
        if !pressed || !self.avatar.dispatch_trigger(trigger) {
            self.controls.register_trigger(trigger, pressed);
        }
        false
    }

    pub(super) fn press_modal_mouse(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
        let directive = self.modal.press_mouse(surface, self.avatar.as_mut());
        self.modal.sync_controls(self.controls.as_mut());
        directive
    }

    pub(super) fn handle_modal_trigger(&mut self, trigger: InputTrigger) {
        if self.modal.handle_trigger(trigger, self.avatar.as_mut()) {
            self.modal.sync_controls(self.controls.as_mut());
        }
    }

    pub(super) fn handle_modal_key(&mut self, code: winit::keyboard::KeyCode) {
        if self.modal.handle_key(code, self.avatar.as_mut()) {
            self.modal.sync_controls(self.controls.as_mut());
        }
    }

    pub(super) fn toggle_escape(&mut self, surface: &dyn PlatformSurface) {
        if self.modal.open() || !self.avatar.dispatch_trigger(InputTrigger::Key(winit::keyboard::KeyCode::Escape)) {
            self.modal.toggle_escape(surface);
        }
    }

    pub(super) fn step_frame(&mut self, dt: f32, display: &mut SessionDisplay) {
        if self.modal.open() {
            self.modal.step(dt);
        } else {
            let prev_anchor = self.avatar.meadow_anchor();
            let total_dt = dt.clamp(1.0 / 240.0, 0.10);
            let steps = (total_dt * 45.0).ceil().max(1.0) as u32;
            let step = total_dt / (steps as f32);
            let mut changed = false;
            for _ in 0..steps {
                self.avatar.step_motion(step, self.controls.as_ref(), self.world.as_ref());
                if self.avatar.step_digging(self.world.as_mut()) || self.world.step_surface(step) {
                    changed = true;
                }
            }
            if changed {
                display.refresh_modified_face(self.world.as_mut());
            }
            if changed || self.avatar.meadow_anchor() != prev_anchor {
                display.refresh_meadow_slice(self.world.as_ref(), self.avatar.as_ref());
            }
        }
    }
}
