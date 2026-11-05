use engine::{CompositeMesh, CpuMeshBatch, CpuTronBatch, GraphicPipeline, PlanetaryMesh, ShaderProgram};
use world::PlanetaryWorld;
use crate::avatar::Avatar;
use crate::controls::{ControlAction, InputTrigger, KeyboardControls};
use crate::menu::{ModalDirective, SessionModal};
use crate::surface::PlatformSurface;

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

    pub(super) fn refresh_world_mesh(&mut self, world: &dyn PlanetaryWorld) {
        for idx in 0..(world.face_count() * 2) {
            self.refresh_face_slice(world, idx);
        }
    }

    pub(super) fn toggle_tron(&mut self, world: &dyn PlanetaryWorld) {
        self.tron_active = !self.tron_active;
        self.refresh_world_mesh(world);
    }

    pub(super) fn render_stage(&self, stage: &SessionStage) {
        if self.tron_active {
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
    pub(super) fn move_cursor(&mut self, point: crate::menu::ScreenPoint) { self.modal.move_cursor(point); }
    pub(super) fn release_modal_mouse(&mut self) { self.modal.release_mouse(); }
    pub(super) fn rotate_avatar(&mut self, dx: f32, dy: f32) { self.avatar.rotate_view(dx, dy); }
    pub(super) fn use_left(&mut self) -> bool { self.avatar.use_left_hand(self.world.as_mut()) }
    pub(super) fn use_right(&mut self) -> bool { self.avatar.use_right_hand(self.world.as_mut()) }
    pub(super) fn release_hand(&mut self) -> bool { self.avatar.release_hand(self.world.as_mut()) }
    pub(super) fn matches_tron(&self, trigger: InputTrigger) -> bool { self.controls.matches_action(ControlAction::TronMode, trigger) }

    pub(super) fn dispatch_avatar_or_control(&mut self, trigger: InputTrigger, pressed: bool) {
        if !pressed || !self.avatar.dispatch_trigger(trigger) {
            self.controls.register_trigger(trigger, pressed);
        }
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
            self.avatar.step_motion(dt, self.controls.as_ref(), self.world.as_ref());
            if self.avatar.step_digging(self.world.as_mut()) {
                display.refresh_modified_face(self.world.as_mut());
            }
            if self.world.step_surface(dt) {
                display.refresh_modified_face(self.world.as_mut());
            }
        }
    }
}
