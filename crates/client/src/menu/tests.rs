use std::cell::Cell;
use glam::Vec3;
use engine::{CpuMeshBatch, CpuTronBatch, GraphicPipeline, Mesh, ShaderProgram, ShaderSource};
use world::{MeshTarget, Quad};
use crate::attire::{AttireProfile, BodyRegion, ComplexionSwatch, FileAttire, HeldRelic, SightMode, StrideRhythm, Wardrobe, YouthAttire};
use crate::aurora_cadence::AuroraCadence;
use crate::aurora_canopy::AuroraCanopy;
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::celestial::CelestialCadence;
use crate::controls::ControlAction;
use crate::posture::AvatarPosture;
use crate::surface::PlatformSurface;
use super::{AvatarArchetype, ControlStudio, EscapeMenu, ModalDirective, ScreenPoint, SessionModal, StudioAnimation, TurntableStage, WardrobeStudio};

struct MockSurface { captured: Cell<bool> }
impl MockSurface {
    fn active() -> Self { Self { captured: Cell::new(true) } }
    fn cursor_captured(&self) -> bool { self.captured.get() }
}

impl PlatformSurface for MockSurface {
    fn resize(&mut self, _w: u32, _h: u32) {}
    fn capture_cursor(&self) { self.captured.set(true); }
    fn release_cursor(&self) { self.captured.set(false); }
    fn normalize_cursor(&self, x: f64, y: f64) -> ScreenPoint { ScreenPoint::new(x as f32, y as f32) }
    fn upload_mesh(&self, _b: &CpuMeshBatch) -> Box<dyn Mesh> { panic!("unused") }
    fn upload_tron_mesh(&self, _b: &CpuTronBatch) -> Box<dyn Mesh> { panic!("unused") }
    fn compile_shader(&self, _s: &dyn ShaderSource) -> Box<dyn ShaderProgram> { panic!("unused") }
    fn assemble_diurnal_pipeline(&self, _c: Box<dyn CelestialCadence>, _a: Box<dyn AuroraCadence>, _p: Box<dyn AuroraCanopy>) -> Box<dyn GraphicPipeline> { panic!("unused") }
    fn render_frame(&self, _p: &dyn GraphicPipeline, _s: &dyn ShaderProgram, _m: &dyn Mesh, _a: &dyn Avatar, _os: &dyn ShaderProgram, _mo: &dyn SessionModal) {}
}

struct QuadCapture { quads: Vec<Quad> }
impl QuadCapture {
    fn empty() -> Self { Self { quads: Vec::new() } }
    fn count(&self) -> usize { self.quads.len() }
    fn positions(&self) -> Vec<Vec3> { self.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect() }
    fn colors(&self) -> Vec<Vec3> { self.quads.iter().map(|q| q.transfer(|_, _, _, _, _, col| col)).collect() }
    fn bust_projection(&self) -> f32 {
        self.quads.iter().flat_map(|q| q.transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]))
            .filter(|v| (0.26..=0.44).contains(&v.y) && v.x.abs() <= 0.08).map(|v| -v.z).fold(0.0_f32, f32::max)
    }
}

impl MeshTarget for QuadCapture {
    fn push_quad(&mut self, quad: Quad) { self.quads.push(quad); }
}

#[test]
fn test_escape_key_opens_menu_character_stage_and_keybinding_settings() {
    let surface = MockSurface::active();
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 9.0, 0.0));
    let mut menu = EscapeMenu::closed();

    assert!(!menu.open() && surface.cursor_captured());
    let (mut crosshair_ui, mut satchel_ui) = (QuadCapture::empty(), QuadCapture::empty());
    menu.emit_ui(&avatar, &mut crosshair_ui);
    assert_eq!(crosshair_ui.count(), 10, "Gameplay HUD must render a 5-part framed crosshair in the screen center");
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab) && { menu.emit_ui(&avatar, &mut satchel_ui); satchel_ui.count() == 0 } && avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab));
    menu.toggle_escape(&surface);
    assert!(menu.open() && !surface.cursor_captured());

    // Click CHARACTER button to open Character Studio
    menu.move_cursor(ScreenPoint::new(0.0, -0.01));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    let (attire, stage) = (YouthAttire::squire(), TurntableStage::front_showcase());
    let (mut single_figure, mut stage_mesh, mut empty_fem, mut held_fem) = (QuadCapture::empty(), QuadCapture::empty(), QuadCapture::empty(), QuadCapture::empty());
    StudioAnimation::StandingIdle.emit_figure(Vec3::new(0.0, 1.0, 0.0), 0.0, 0.0, &attire, &mut single_figure);
    stage.emit_stage(&attire, &mut stage_mesh);
    assert_eq!(stage_mesh.count(), single_figure.count() + 32, "Turntable stage must display only one single character plus pedestal");

    let vp = stage.view_projection(16.0 / 9.0);
    for v in stage_mesh.positions() {
        let clip = vp * v.extend(1.0);
        let ndc_y = clip.y / clip.w;
        assert!((-0.70..=0.68).contains(&ndc_y), "Entire character height must fit inside studio viewport, got {ndc_y}");
    }

    // Equip a block in hand, then click FEMALE (-0.59, -0.70) in Character Studio: block must remain in hand
    menu.move_cursor(ScreenPoint::new(-0.59, -0.70));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    avatar.emit_appearance(&mut empty_fem);
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::ExcavatedEarth, HeldRelic::EmptyPalm));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    avatar.emit_appearance(&mut held_fem);
    assert!(held_fem.count() > empty_fem.count(), "Switching gender in Character Studio must preserve block held in hand");
    menu.move_cursor(ScreenPoint::new(-0.83, -0.70));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);

    // Click BACK button in Character Studio to return to Main Menu, then click SETTINGS button
    menu.move_cursor(ScreenPoint::new(0.0, -0.89));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    menu.move_cursor(ScreenPoint::new(0.0, -0.16));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let mut settings_ui = QuadCapture::empty();
    menu.emit_ui(&avatar, &mut settings_ui);
    assert!(settings_ui.count() > 200, "Settings screen must render all 18 single-key action buttons");

    // Click Satchel action button and rebind from Tab to KeyN
    let satchel_btn = ControlStudio::action_bounds(ControlAction::Satchel);
    menu.move_cursor(ScreenPoint::new(satchel_btn.center_x(), satchel_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    assert!(menu.handle_key(winit::keyboard::KeyCode::KeyN, &mut avatar));
    assert!(!avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab));
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::KeyN));

    // Reset keys, return to Main Menu, and click QUIT GAME
    let (reset_btn, back_btn) = (ControlStudio::reset_button(), ControlStudio::back_button());
    menu.move_cursor(ScreenPoint::new(reset_btn.center_x(), reset_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    let _ = std::fs::remove_file("controls.toml");
    assert!(avatar.dispatch_wardrobe_key(winit::keyboard::KeyCode::Tab));
    menu.move_cursor(ScreenPoint::new(back_btn.center_x(), back_btn.center_y()));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ContinueSession);
    menu.move_cursor(ScreenPoint::new(0.0, -0.34));
    assert_eq!(menu.press_mouse(&surface, &mut avatar), ModalDirective::ExitGame);
}

#[test]
fn test_single_character_cycles_all_animations_and_archetypes_differ_fundamentally() {
    let (attire, stage_front) = (YouthAttire::squire(), TurntableStage::front_showcase());
    let (mut front_quads, mut turned_quads) = (QuadCapture::empty(), QuadCapture::empty());
    stage_front.emit_stage(&attire, &mut front_quads);
    stage_front.rotated(1.2).emit_stage(&attire, &mut turned_quads);
    assert_ne!(front_quads.positions(), turned_quads.positions());

    for idx in 0..(StudioAnimation::sequence().len() - 1) {
        let (t0, t1) = ((idx as f32) * 2.2 + 0.35, ((idx + 1) as f32) * 2.2 + 0.35);
        assert_ne!(StudioAnimation::from_elapsed(t0), StudioAnimation::from_elapsed(t1));
        let (mut q0, mut q1) = (QuadCapture::empty(), QuadCapture::empty());
        TurntableStage::new(0.0, t0).emit_stage(&attire, &mut q0);
        TurntableStage::new(0.0, t1).emit_stage(&attire, &mut q1);
        assert_ne!(q0.positions(), q1.positions());
    }

    let pose = AvatarPosture::standing().body_pose(Vec3::ZERO, glam::Quat::IDENTITY, glam::Quat::IDENTITY, StrideRhythm::idle());
    let archetypes = AvatarArchetype::all();
    for i in 0..archetypes.len() {
        assert_ne!(archetypes[i].gender_title(false), archetypes[i].gender_title(true));
        let (mut m_male, mut m_fem) = (QuadCapture::empty(), QuadCapture::empty());
        archetypes[i].attire().emit(&pose, SightMode::LookingGlassSight, &mut m_male);
        archetypes[i].female_attire().emit(&pose, SightMode::LookingGlassSight, &mut m_fem);
        assert!(m_fem.bust_projection() > m_male.bust_projection() + 0.035);
        for other in archetypes.iter().skip(i + 1) {
            let (mut m_j, mut f_j) = (QuadCapture::empty(), QuadCapture::empty());
            other.attire().emit(&pose, SightMode::LookingGlassSight, &mut m_j);
            other.female_attire().emit(&pose, SightMode::LookingGlassSight, &mut f_j);
            assert_ne!(m_male.positions(), m_j.positions());
            assert_ne!(m_fem.positions(), f_j.positions());
        }
    }
}

#[test]
fn test_individual_body_part_size_and_thickness_and_skin_color_selection() {
    let base_attire = YouthAttire::squire();
    let pose = AvatarPosture::standing().body_pose(Vec3::ZERO, glam::Quat::IDENTITY, glam::Quat::IDENTITY, StrideRhythm::idle());

    let mut base_mesh = QuadCapture::empty();
    base_attire.emit(&pose, SightMode::LookingGlassSight, &mut base_mesh);

    for region in BodyRegion::all() {
        let (taller, thicker) = (base_attire.with_adjusted_region(region, 2, 0), base_attire.with_adjusted_region(region, 0, 2));
        let (mut taller_mesh, mut thicker_mesh) = (QuadCapture::empty(), QuadCapture::empty());
        taller.emit(&pose, SightMode::LookingGlassSight, &mut taller_mesh);
        thicker.emit(&pose, SightMode::LookingGlassSight, &mut thicker_mesh);
        assert_ne!(base_mesh.positions(), taller_mesh.positions());
        assert_ne!(base_mesh.positions(), thicker_mesh.positions());
        assert_ne!(taller_mesh.positions(), thicker_mesh.positions());
    }

    let ebony_attire = base_attire.with_complexion(ComplexionSwatch::Ebony);
    let mut ebony_mesh = QuadCapture::empty();
    ebony_attire.emit(&pose, SightMode::LookingGlassSight, &mut ebony_mesh);
    assert_ne!(base_mesh.colors(), ebony_mesh.colors());

    let temp_path = std::env::temp_dir().join("test_studio_body_proportions.toml");
    let store = FileAttire::new(temp_path.to_str().unwrap().to_string());
    let custom = AvatarArchetype::Blacksmith.female_attire().with_complexion(ComplexionSwatch::Bronze).with_adjusted_region(BodyRegion::Crown, 1, -1).with_adjusted_region(BodyRegion::Arms, 2, 2);
    store.save(&custom);
    let loaded = store.load_or_squire();
    let _ = std::fs::remove_file(&temp_path);
    assert_eq!(custom, loaded);
    assert!(loaded.feminine());

    let (mut studio, mut male_stage, mut female_stage) = (WardrobeStudio::from_profile(), QuadCapture::empty(), QuadCapture::empty());
    studio.emit_stage(&mut male_stage);
    studio.move_pointer(ScreenPoint::new(-0.59, -0.70));
    studio.press_pointer();
    assert!(studio.current_attire().feminine());
    studio.emit_stage(&mut female_stage);
    assert_ne!(male_stage.positions(), female_stage.positions());
    studio.move_pointer(ScreenPoint::new(-0.83, -0.70));
    studio.press_pointer();
    assert!(!studio.current_attire().feminine());
}
