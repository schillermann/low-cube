use std::f32::consts::PI;
use glam::{Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{HeldRelic, SightMode, StrideRhythm, Wardrobe, YouthAttire};
use crate::avatar::Avatar;
use crate::axe::{FieldAxe, RelicSite};
use crate::controls::KeyboardControls;
use crate::posture::AvatarPosture;

/// Interface for an ensemble of animated character figures stationed in the world.
pub trait FigureTroupe {
    fn step(&mut self, dt: f32);
    fn trigger_strike(&mut self);
    fn sync_wardrobe(&mut self, code: KeyCode);
    fn sync_relic(&mut self, relics: (HeldRelic, HeldRelic));
    fn emit(&self, target: &mut dyn MeshTarget);
}

/// Immutable animated character mannequin stationed on the planetary surface with a continuous stride rhythm.
#[derive(Clone, Copy, Debug)]
pub struct GaitFigure {
    position: Vec3,
    surface_rot: Quat,
    view_rot: Quat,
    rhythm: StrideRhythm,
    angular_pace: f32,
}

impl GaitFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(position: Vec3, surface_rot: Quat, view_rot: Quat, rhythm: StrideRhythm, angular_pace: f32) -> Self {
        Self { position, surface_rot, view_rot, rhythm, angular_pace }
    }

    /// Secondary constructor for a standing figure with subtle idle breathing animation (Stehen).
    pub fn standing_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::idle(), 1.5)
    }

    /// Secondary constructor for a walking figure with steady stride swing (Gehen).
    pub fn walking_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::walking(), 8.4)
    }

    /// Secondary constructor for a running figure with athletic sprint cadence and clenched fists (Laufen).
    pub fn running_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::running(), 14.2)
    }

    /// Returns a new GaitFigure with its animation phase advanced by dt.
    pub fn step(self, dt: f32) -> Self {
        let next_rhythm = self.rhythm.step_cycle(self.angular_pace, dt);
        Self::new(self.position, self.surface_rot, self.view_rot, next_rhythm, self.angular_pace)
    }

    /// Returns a new GaitFigure initiating a synchronized axe strike.
    pub fn trigger_strike(self) -> Self {
        Self::new(self.position, self.surface_rot, self.view_rot, self.rhythm.trigger_strike(), self.angular_pace)
    }

    /// Emits complete third-person character appearance (head, hair/hood, torso, limbs, boots, relic).
    pub fn emit(&self, attire: &YouthAttire, target: &mut dyn MeshTarget) {
        let pose = AvatarPosture::standing().body_pose(self.position, self.surface_rot, self.view_rot, self.rhythm);
        attire.emit(&pose, SightMode::LookingGlassSight, target);
    }
}

/// Trio of animated character figures stationed near the spawn point showcasing standing, walking, and running gaits.
pub struct SpawnTroupe {
    figures: [GaitFigure; 3],
    attire: YouthAttire,
}

impl SpawnTroupe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(figures: [GaitFigure; 3], attire: YouthAttire) -> Self {
        Self { figures, attire }
    }

    /// Secondary constructor placing three figures (Stehen, Gehen, Laufen) in front of the spawn point facing the player.
    pub fn near_spawn(spawn_position: Vec3, world: &dyn PlanetaryWorld, attire: YouthAttire) -> Self {
        let spawn_up = world.surface_up_at(spawn_position);
        let spawn_rot = Quat::from_rotation_arc(Vec3::Y, spawn_up);
        let forward = spawn_rot * Vec3::NEG_Z;
        let right = spawn_rot * Vec3::X;

        let place = |lateral_offset: f32| -> (Vec3, Quat, Quat) {
            let candidate = spawn_position + right * lateral_offset + forward * 2.8;
            let grounded = world.constrain_to_surface(candidate, 1.0);
            let up = world.surface_up_at(grounded);
            let surface_rot = Quat::from_rotation_arc(Vec3::Y, up);
            let view_rot = surface_rot * Quat::from_rotation_y(PI);
            (grounded, surface_rot, view_rot)
        };

        let (pos_stand, rot_stand, view_stand) = place(-1.15);
        let (pos_walk, rot_walk, view_walk) = place(0.0);
        let (pos_run, rot_run, view_run) = place(1.15);

        Self::new(
            [
                GaitFigure::standing_at(pos_stand, rot_stand, view_stand),
                GaitFigure::walking_at(pos_walk, rot_walk, view_walk),
                GaitFigure::running_at(pos_run, rot_run, view_run),
            ],
            attire,
        )
    }
}

impl FigureTroupe for SpawnTroupe {
    fn step(&mut self, dt: f32) {
        self.figures = [self.figures[0].step(dt), self.figures[1].step(dt), self.figures[2].step(dt)];
    }

    fn trigger_strike(&mut self) {
        if self.attire.armed() {
            self.figures = [self.figures[0].trigger_strike(), self.figures[1].trigger_strike(), self.figures[2].trigger_strike()];
        }
    }

    fn sync_wardrobe(&mut self, code: KeyCode) {
        if let Some(updated) = self.attire.with_wardrobe_key(code) {
            self.attire = updated;
        }
    }

    fn sync_relic(&mut self, relics: (HeldRelic, HeldRelic)) { self.attire = self.attire.with_relics(relics.0, relics.1); }

    fn emit(&self, target: &mut dyn MeshTarget) {
        for figure in &self.figures { figure.emit(&self.attire, target); }
    }
}

/// Decorator accompanying an Avatar with a FigureTroupe and interactive FieldAxe stationed near the planetary spawn.
pub struct TroupeAvatar {
    origin: Box<dyn Avatar>,
    troupe: Box<dyn FigureTroupe>,
    axe: Box<dyn RelicSite>,
}

impl TroupeAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn Avatar>, troupe: Box<dyn FigureTroupe>, axe: Box<dyn RelicSite>) -> Self {
        Self { origin, troupe, axe }
    }

    /// Secondary constructor decorating an Avatar with the three showcase figures and interactive axe near spawn.
    pub fn near_spawn(origin: Box<dyn Avatar>, spawn_position: Vec3, world: &dyn PlanetaryWorld, attire: YouthAttire) -> Self {
        Self::new(
            origin,
            Box::new(SpawnTroupe::near_spawn(spawn_position, world, attire)),
            Box::new(FieldAxe::near_spawn(spawn_position, world)),
        )
    }
}

impl Avatar for TroupeAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) { self.origin.rotate_view(dx, dy); }
    fn toggle_invert_y(&mut self) { self.origin.toggle_invert_y(); }
    fn toggle_invert_x(&mut self) { self.origin.toggle_invert_x(); }

    fn trigger_strike(&mut self) {
        self.origin.trigger_strike();
        self.troupe.trigger_strike();
    }

    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool {
        if self.axe.register_key(code) {
            return true;
        }
        self.troupe.sync_wardrobe(code);
        self.origin.dispatch_wardrobe_key(code)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        self.origin.step_motion(dt, controls, world);
        if let Some(relic) = self.origin.sync_relic_site(&mut *self.axe, dt, world) {
            self.troupe.sync_relic(relic);
        }
        self.troupe.step(dt);
    }

    fn sync_relic_site(&mut self, site: &mut dyn RelicSite, dt: f32, world: &dyn PlanetaryWorld) -> Option<(HeldRelic, HeldRelic)> {
        self.origin.sync_relic_site(site, dt, world)
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 { self.origin.view_projection_matrix(aspect) }
    fn surface_rotation(&self) -> Quat { self.origin.surface_rotation() }

    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_appearance(target);
        self.troupe.emit(target);
        self.axe.emit(target);
    }
}
