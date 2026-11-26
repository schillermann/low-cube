use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, BlockMold, PlanetaryWorld};
use crate::attire::compartment::{CompartmentBackplate, ShowcasedCompartment, SlotPlaque, WeightedCompartment};
use crate::attire::burden::SlotBurden;
use crate::attire::tome::{
    ArtisanTome, CanonicalShelf, IngredientFormula, PinnedPolaroid, PolaroidCard,
    ReadinessPipCompartment, ReadyStockShelf, RecipeShelf, TabbedShelf, TestFolioSelection,
    TestFolioSpread as _, TestFolioTab, TestIndexTabbedFolio, TestRingSpineFolio,
    TestSlateFolioCovers, TestStockScope, VacantScopePlaque,
};
use crate::attire::{BeltSatchel, HeldRelic, LimbFrame, YouthAttire};
use crate::avatar::{Avatar as _, AvatarOptics as _, PlanetaryAvatar};
use crate::controls::{ActionBindings, InputTrigger};
use super::QuadLedger;

#[test]
fn test_recipe_shelf_decorators_partition_all_thirty_recipes_and_filter_ready_stock() {
    let mut all = Vec::new();
    CanonicalShelf::catalog().collect_recipes(&mut all);
    assert_eq!(all.len(), 33);

    let mut total_partitioned = 0usize;
    for tab in TestFolioTab::all() {
        let mut group = Vec::new();
        TabbedShelf::new(CanonicalShelf::catalog(), tab).collect_recipes(&mut group);
        assert!(
            (5..=12).contains(&group.len()),
            "Every FolioTab must fit on a zero-scroll 3x4 page, got {} in {tab:?}",
            group.len()
        );
        total_partitioned += group.len();
    }
    assert_eq!(total_partitioned, 33, "Four FolioTabs must partition all 33 canonical recipes without overlap");

    let squire = YouthAttire::squire().with_relics(HeldRelic::TimberBlock, HeldRelic::EmptyPalm);
    let satchel = BeltSatchel::empty().reconcile_attire(squire);
    let mut ready_blocks = Vec::new();
    ReadyStockShelf::new(
        TabbedShelf::new(CanonicalShelf::catalog(), TestFolioTab::Blocks),
        TestStockScope::ReadyStock,
        &satchel,
        squire,
    )
    .collect_recipes(&mut ready_blocks);
    assert_eq!(ready_blocks.len(), 1);
    assert_eq!(ready_blocks[0].0, 1, "PlankBlock must keep canonical recipe index #1 through decorator chain");
    assert_eq!(ready_blocks[0].1.product(), HeldRelic::PlankBlock);
}

#[test]
fn test_folio_spread_and_polaroid_slot_plaque_decorators_compose_layered_ring_binder() {
    let frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::NEG_Z);
    let squire = YouthAttire::squire();
    let satchel = BeltSatchel::empty().with_stowed_relic(HeldRelic::DeadwoodBough, 1).with_stowed_relic(HeldRelic::FlintStone, 1);

    let (mut covers, mut ringed, mut tabbed) = (QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty());
    TestSlateFolioCovers::new(TestFolioTab::Tools.pigment()).emit_spread(&frame, &mut covers);
    TestRingSpineFolio::new(TestSlateFolioCovers::new(TestFolioTab::Tools.pigment())).emit_spread(&frame, &mut ringed);
    TestIndexTabbedFolio::new(
        TestRingSpineFolio::new(TestSlateFolioCovers::new(TestFolioTab::Tools.pigment())),
        TestFolioTab::Tools,
        &satchel,
        squire,
    )
    .emit_spread(&frame, &mut tabbed);

    assert!(covers.populated());
    assert!(ringed.exceeds(&covers), "RingSpineFolio decorator must sculpt 3D steel binder coils over covers");
    assert!(tabbed.exceeds(&ringed), "IndexTabbedFolio decorator must add four protruding side index tabs");

    let (mut unpipped, mut pipped, mut polaroid_card, mut pinned_polaroid, mut vacant) = (
        QuadLedger::empty(),
        QuadLedger::empty(),
        QuadLedger::empty(),
        QuadLedger::empty(),
        QuadLedger::empty(),
    );
    let base_slot = ShowcasedCompartment::new(
        WeightedCompartment::new(
            CompartmentBackplate::new(true, true, Vec3::new(0.96, 0.80, 0.26)),
            SlotBurden::new(HeldRelic::PlankBlock, 4),
        ),
        HeldRelic::PlankBlock.figure(BlockMold::cube()),
    );
    base_slot.emit_slot(&frame, &mut unpipped);
    ReadinessPipCompartment::new(base_slot, true, 4).emit_slot(&frame, &mut pipped);
    assert!(pipped.exceeds(&unpipped), "ReadinessPipCompartment must add readiness corner pip and X4 yield tag");

    let axe_burden = SlotBurden::new(HeldRelic::WoodmanAxe, 1);
    PolaroidCard::new(axe_burden).emit_slot(&frame, &mut polaroid_card);
    PinnedPolaroid::new(
        PolaroidCard::new(axe_burden),
        HeldRelic::WoodmanAxe.figure(BlockMold::cube()),
    )
    .emit_slot(&frame, &mut pinned_polaroid);
    assert!(pinned_polaroid.exceeds(&polaroid_card), "PinnedPolaroid decorator must add 3D showcase and thumbtack pin");

    VacantScopePlaque::new(TestFolioTab::Orbital).emit_slot(&frame, &mut vacant);
    assert!(vacant.populated());
    assert!(!TestFolioSelection::VacantTab(TestFolioTab::Orbital).highlights_recipe(0));
}

#[test]
fn test_ingredient_formula_omits_locked_badge_and_trailing_product_duplicate() {
    let frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::NEG_Z);
    let squire = YouthAttire::squire();
    let entries = ArtisanTome::entries();
    let plank_entry = entries[1];
    let axe_entry = entries[0];

    let empty_satchel = BeltSatchel::empty();
    let stocked_satchel = BeltSatchel::empty().with_stowed_relic(HeldRelic::TimberBlock, 1);
    let (mut missing_planks, mut ready_planks, mut missing_axe) = (
        QuadLedger::empty(),
        QuadLedger::empty(),
        QuadLedger::empty(),
    );
    IngredientFormula::new(plank_entry, &empty_satchel, squire).emit_slot(&frame, &mut missing_planks);
    IngredientFormula::new(plank_entry, &stocked_satchel, squire).emit_slot(&frame, &mut ready_planks);
    IngredientFormula::new(axe_entry, &empty_satchel, squire).emit_slot(&frame, &mut missing_axe);

    assert!(
        ready_planks.exceeds(&missing_planks),
        "Craftable recipe must add CRAFT button over bare burden badge without rendering a LOCKED plaque when unready"
    );
    assert!(
        missing_axe.exceeds(&missing_planks),
        "Dual-ingredient formula must render both ingredient compartments around '+' while single-ingredient renders one"
    );
}

#[test]
fn test_tab_switching_two_dimensional_grid_navigation_and_vacant_ready_scope_guard() {
    let bindings = ActionBindings::seeded();
    let squire = YouthAttire::squire();
    let satchel = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::GoldOre, 2)
        .with_stowed_relic(HeldRelic::CopperOre, 1);

    let tome = ArtisanTome::new(0, true);
    assert_eq!(tome.selected_entry().product(), HeldRelic::WoodmanAxe);

    // W/S step vertically by 3 columns on the 3x4 Tools grid (`#0 -> #6 KnightlySword` at slot 3)
    let (down_row, _, _, ok_s) = tome.dispatch_trigger(InputTrigger::Key(KeyCode::KeyS), &bindings, satchel, squire);
    assert!(ok_s && down_row.selected_entry().product() == HeldRelic::KnightlySword);

    // Digit3 jumps directly to Vehicles (`#9 CabinSeat`), Tab cycles to Orbital (`#13 FieldCamera`)
    let (vehicles, _, _, ok_3) = down_row.dispatch_trigger(InputTrigger::Key(KeyCode::Digit3), &bindings, satchel, squire);
    assert!(ok_3 && vehicles.selected_entry().product() == HeldRelic::CabinSeat);
    let (orbital, _, _, ok_tab) = vehicles.dispatch_trigger(InputTrigger::Key(KeyCode::Tab), &bindings, satchel, squire);
    assert!(ok_tab && orbital.selected_entry().product() == HeldRelic::FieldCamera);

    // Pressing F from Tools (0 ready items) snaps to Orbital where LaserBeacon (#24) is ready (`2 Gold + 1 Copper`)
    let (ready_tome, _, _, ok_f) = tome.dispatch_trigger(InputTrigger::Key(KeyCode::KeyF), &bindings, satchel, squire);
    assert!(ok_f && ready_tome.selected_entry().product() == HeldRelic::LaserBeacon);

    // Switching back to Digit1 (Tools) while ReadyStock is active yields a vacant tab that refuses crafting
    let (vacant_tools, _, _, ok_1) = ready_tome.dispatch_trigger(InputTrigger::Key(KeyCode::Digit1), &bindings, satchel, squire);
    assert!(ok_1);
    let (still_open, _, after_attire, crafted) = vacant_tools.craft_selected(1.0, satchel, squire);
    assert!(!crafted && still_open.open() && after_attire == squire);
}

#[test]
fn test_tactile_field_folio_projects_inside_sixteen_by_nine_and_four_by_three_viewports() {
    let world = BiomedCubeWorld::barren_beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::DeadwoodBough, HeldRelic::FlintStone));
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyC));

    for aspect in [16.0 / 9.0, 4.0 / 3.0] {
        let mut stage = QuadLedger::empty();
        let (vp, _) = avatar
            .emit_inspection_stage(aspect, &mut stage)
            .expect("open ArtisanTome must emit foreground inspection stage");
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (1.0f32, -1.0f32, 1.0f32, -1.0f32);
        for quad in &stage.quads {
            quad.transfer(|v0, v1, v2, v3, _, _| {
                for v in [v0, v1, v2, v3] {
                    let ndc = vp.project_point3(v);
                    min_x = min_x.min(ndc.x);
                    max_x = max_x.max(ndc.x);
                    min_y = min_y.min(ndc.y);
                    max_y = max_y.max(ndc.y);
                }
            });
        }
        println!("aspect={aspect:.2} -> NDC x=[{min_x:.3}, {max_x:.3}], y=[{min_y:.3}, {max_y:.3}], quads={}", stage.quads.len());
        assert!(
            min_x > -0.96 && max_x < 0.96 && min_y > -0.94 && max_y < 0.94,
            "Tactile Field Folio must stay comfortably inside viewport at aspect {aspect:.2}: x=[{min_x:.3}, {max_x:.3}], y=[{min_y:.3}, {max_y:.3}]"
        );
    }
}
