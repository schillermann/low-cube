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
    fn trigger_left_strike(&mut self);
    fn trigger_right_strike(&mut self);
    fn apply_attire(&mut self, attire: YouthAttire);
    fn sync_wardrobe(&mut self, code: KeyCode);
    fn sync_relic(&mut self, relics: (HeldRelic, HeldRelic));
    fn emit(&self, target: &mut dyn MeshTarget);
}

/// Immutable animated character mannequin stationed on the planetary surface with a continuous stride or jump rhythm.
#[derive(Clone, Copy, Debug)]
pub struct GaitFigure {
    position: Vec3,
    surface_rot: Quat,
    view_rot: Quat,
    rhythm: StrideRhythm,
    angular_pace: f32,
    leap_phase: f32,
    leap_apex: f32,
}

impl GaitFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(position: Vec3, surface_rot: Quat, view_rot: Quat, rhythm: StrideRhythm, angular_pace: f32, leap_phase: f32, leap_apex: f32) -> Self {
        Self { position, surface_rot, view_rot, rhythm, angular_pace, leap_phase, leap_apex }
    }

    pub fn standing_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::idle(), 1.5, 0.0, 0.0)
    }

    pub fn sneaking_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::sneaking(), 5.4, 0.0, -0.44)
    }

    pub fn walking_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::walking(), 8.4, 0.0, 0.0)
    }

    pub fn running_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::running(), 14.2, 0.0, 0.0)
    }

    pub fn jumping_at(position: Vec3, surface_rot: Quat, view_rot: Quat) -> Self {
        Self::new(position, surface_rot, view_rot, StrideRhythm::idle(), 0.0, 0.44, 0.58)
    }

    pub fn step(self, dt: f32) -> Self {
        let next_leap = if self.leap_apex > 0.0 { (self.leap_phase + 0.52 * dt).fract() } else { 0.0 };
        Self::new(self.position, self.surface_rot, self.view_rot, self.rhythm.step_cycle(self.angular_pace, dt), self.angular_pace, next_leap, self.leap_apex)
    }

    pub fn trigger_left_strike(self) -> Self {
        Self::new(self.position, self.surface_rot, self.view_rot, self.rhythm.trigger_left_strike(), self.angular_pace, self.leap_phase, self.leap_apex)
    }

    pub fn trigger_right_strike(self) -> Self {
        Self::new(self.position, self.surface_rot, self.view_rot, self.rhythm.trigger_right_strike(), self.angular_pace, self.leap_phase, self.leap_apex)
    }

    pub fn emit(&self, attire: &YouthAttire, target: &mut dyn MeshTarget) {
        let (u, up, fwd) = (self.leap_phase, (self.surface_rot * Vec3::Y).normalize(), (self.view_rot * Vec3::NEG_Z).normalize());
        let (lift, surge, eye) = if self.leap_apex < 0.0 { (0.0, 0.0, 0.80 + self.leap_apex) }
            else if self.leap_apex <= 0.0 || u < 0.18 { (0.0, 0.0, 0.80) }
            else if u < 0.40 { let s = ((u - 0.18) / 0.22 * PI).sin(); (0.0, 0.0, 0.80 - 0.44 * s * s) }
            else if u < 0.74 { let air = ((u - 0.40) / 0.34 * PI).sin(); (self.leap_apex * air, 0.16 * air, 0.80 + 0.36 * air * air) }
            else { let s = ((u - 0.74) / 0.26 * PI).sin(); (0.0, 0.0, 0.80 - 0.40 * s * s) };
        let pose = AvatarPosture::new(eye).body_pose(self.position + up * lift + fwd * surge, self.surface_rot, self.view_rot, self.rhythm);
        attire.emit(&pose, SightMode::LookingGlassSight, target);
    }
}

/// Quintet of animated character figures stationed near the spawn point showcasing standing, sneaking, walking, running, and jumping.
pub struct SpawnTroupe {
    figures: [GaitFigure; 5],
    attire: YouthAttire,
}

impl SpawnTroupe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(figures: [GaitFigure; 5], attire: YouthAttire) -> Self {
        Self { figures, attire }
    }

    pub fn near_spawn(spawn_position: Vec3, world: &dyn PlanetaryWorld, attire: YouthAttire) -> Self {
        let spawn_rot = Quat::from_rotation_arc(Vec3::Y, world.surface_up_at(spawn_position));
        let (forward, right) = (spawn_rot * Vec3::NEG_Z, spawn_rot * Vec3::X);
        let place = |lateral_offset: f32| -> (Vec3, Quat, Quat) {
            let grounded = world.constrain_to_surface(spawn_position + right * lateral_offset + forward * 3.0, 1.0);
            let surface_rot = Quat::from_rotation_arc(Vec3::Y, world.surface_up_at(grounded));
            (grounded, surface_rot, surface_rot * Quat::from_rotation_y(PI))
        };
        let ((p0, r0, v0), (p1, r1, v1), (p2, r2, v2), (p3, r3, v3), (p4, r4, v4)) = (place(-1.80), place(-0.90), place(0.0), place(0.90), place(1.80));
        Self::new([GaitFigure::standing_at(p0, r0, v0), GaitFigure::sneaking_at(p1, r1, v1), GaitFigure::walking_at(p2, r2, v2), GaitFigure::running_at(p3, r3, v3), GaitFigure::jumping_at(p4, r4, v4)], attire)
    }
}

impl FigureTroupe for SpawnTroupe {
    fn step(&mut self, dt: f32) { self.figures = self.figures.map(|f| f.step(dt)); }
    fn trigger_left_strike(&mut self) { if self.attire.left_armed() { self.figures = self.figures.map(|f| f.trigger_left_strike()); } }
    fn trigger_right_strike(&mut self) { if self.attire.right_armed() { self.figures = self.figures.map(|f| f.trigger_right_strike()); } }
    fn apply_attire(&mut self, attire: YouthAttire) { self.attire = attire; }
    fn sync_wardrobe(&mut self, code: KeyCode) { if let Some(updated) = self.attire.with_wardrobe_key(code) { self.attire = updated; } }
    fn sync_relic(&mut self, relics: (HeldRelic, HeldRelic)) { self.attire = self.attire.with_relics(relics.0, relics.1); }
    fn emit(&self, target: &mut dyn MeshTarget) { for figure in &self.figures { figure.emit(&self.attire, target); } }
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

    pub fn near_spawn(origin: Box<dyn Avatar>, spawn_position: Vec3, world: &dyn PlanetaryWorld, attire: YouthAttire) -> Self {
        Self::new(origin, Box::new(SpawnTroupe::near_spawn(spawn_position, world, attire)), Box::new(FieldAxe::near_spawn(spawn_position, world)))
    }
}

impl Avatar for TroupeAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) { self.origin.rotate_view(dx, dy); }
    fn toggle_invert_y(&mut self) { self.origin.toggle_invert_y(); }
    fn toggle_invert_x(&mut self) { self.origin.toggle_invert_x(); }

    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let changed = self.origin.use_left_hand(world);
        self.troupe.trigger_left_strike();
        changed
    }

    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let changed = self.origin.use_right_hand(world);
        self.troupe.trigger_right_strike();
        changed
    }

    fn apply_attire(&mut self, attire: YouthAttire) {
        self.origin.apply_attire(attire);
        self.troupe.apply_attire(attire);
    }

    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool {
        if self.axe.register_key(code) { return true; }
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
