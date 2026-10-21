# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
and [Conventional Commits](https://www.conventionalcommits.org/).

## [Unreleased]

## [0.29.0] - 2026-10-21

### Added
- Hold-to-dig excavation lifecycle (`PalmEarthworks`, `Avatar::release_hand`, `Avatar::step_digging`, `PlanetaryWorld::loosen_block`, `TerracedRelief`) requiring continuous mouse button hold across strikes and aborting excavation progress upon early release.
- True 3D sunken excavation crater (`ReliefFacet::emit_dig_crater`) with 4 outer rim trapezoids, 4 shaded inner walls, and 1 sunken floor scaling across digging stages (1/4 to 3/4) with automatic material depth shading.
- Ground-anchored outward ballistic soil clod splash (`PalmEarthworks::emit_debris`) emitted directly above the targeted block during active excavation strikes.
- Dedicated spade and bare-hand digging martial kinematics (`MartialStyle::SpadeDig`, `MartialStyle::Unarmed`, `BodyPose::external_arm`, `BodyPose::first_person_arm`) with outward-then-inward elbow motion, forward-reaching forearm arcs, inward wrist rotation plunging the spade tip diagonally forward-downward toward the target block, and dual-button bare-hand scooping that brings both palms together in a diagonal outward cup.
- Two additional spawn showcase figures (`GaitFigure::digging_at`, `GaitFigure::palm_digging_at`, `SpawnTroupe`) and Character Studio preview animations (`StudioAnimation::SpadeDig`, `StudioAnimation::PalmDig`) demonstrating spade and bare-hand digging.

### Changed
- Renamed `HeldRelic::IronShovel` to `HeldRelic::IronSpade` (`HeldRelic::spade_blade`) and doubled excavation thresholds (`EXCAVATION_THRESHOLD = 8`: 2 strikes with iron spade, 4 strikes with both empty hands when holding both mouse buttons, 8 strikes with a single empty hand).
- Replaced fixed distance ray sampling in `SurfaceSightRay::aimed_cell` with discrete 3D grid traversal (18 steps up to `1.5m`) constrained to a maximum reach of 3 blocks (`|du| <= 3`, `|dv| <= 3`) around the avatar.

## [0.28.0] - 2026-10-21

### Added
- Block excavation and placement system (`SurfaceSightRay`, `TerracedRelief`, `PalmEarthworks`, `HeldRelic::IronShovel`, `HeldRelic::ExcavatedEarth`) allowing players to dig terrain and placed blocks with bare hands (4 strikes with 1 free hand, 2 strikes with 2 free hands) or an iron shovel (1 strike) and place collected earth blocks back into the world using the hand holding the block.
- Ground iron shovel relic (`FieldAxe`) stationed near the planetary spawn point alongside the woodman's axe and knightly sword.
- Multi-face planetary GPU mesh (`PlanetaryMesh`, `PlanetaryWorld::face_count`, `PlanetaryWorld::emit_face`, `PlanetaryWorld::emit_modified_face`, `GlPlatformSurface::replace_mesh`, `Mesh::release`) uploading each of the 6 planetary faces as an independent GPU buffer and replacing only the single modified face slice upon block excavation or placement.
- Row-level relief quad cache (`FacetedReliefFace`), `O(1)` natural elevation grid cache (`TerracedRelief`), and static flora quad cache (`ForestedFace`) so block updates recompute only the 3 affected cell rows (`cv - 1 ..= cv + 1`) on the modified face without re-evaluating untouched terrain or trees.

### Fixed
- Preserved natural slope, ridge, and valley geometry and collision heights on neighboring cells when excavating sloped blocks (`ReliefFacet::emit_facet`, `ReliefFacet::emit_walls`, `TerracedRelief::surface_height_at`).

## [0.27.0] - 2026-10-21

### Added
- Rendered 12 high-resolution 3D character portraits (`docs/images/character/{knight,valkyrie,ranger,huntress,woodsman,herbalist,blacksmith,weaver,sentinel,shieldmaid,noble,lady}.png`) and added a complete male and female **Character Archetypes Gallery** with descriptions in `docs/CHARACTER.md`.

### Changed
- Scaled all full-cube and non-full-cube block shapes (`CuboidBlock`, `WedgeBlock`, `GentleWedgeBlock`, `GentleRidgeBlock`, `GentleValleyBlock`, `RidgeCornerBlock`, `ValleyCornerBlock`, `PyramidBlock`, `StairBlock`, `StairCornerBlock`, `GridCell`, `FormBlock`) to half size (`0.5m × 0.5m × 0.5m` with `0.25m` slabs/steps and `0.125m` quarter-height increments) and adapted planetary relief (`ElevatedFacet`, `ElevationStrata`, `SeamCorner`, `SubterraneanCavity`), biome factory (`BiomedWorld`), forestry ground checks (`FullCubeGround`, `MeridianGround`, `SpawnGround`, `SurfaceGround`, `WoodlandGround`, `ForestedFace`), and mountain/biome landscapes accordingly.
- Refined the female bust in `TunicStyle::emit` and `PolygonAccoutrement::emit_tunic_details` into a clearly recognizable, non-pointy, continuous clothed 3-tier bust with a subtle center indentation (`x = 0.0`) and underbust bodice framing across all archetypes.
- Removed the `Architectural Design` section from `docs/CHARACTER.md` and updated the `create git message` workflow in `AGENTS.md` to stage all files and execute the commit via CLI.

## [0.26.0] - 2026-10-21

### Added
- Fifth spawn showcase character with continuous sneaking animation (`GaitFigure::sneaking_at`, `StrideRhythm::sneaking`, `SpawnTroupe`) placed alongside standing, walking, running, and jumping figures.
- Gender switch (`MALE` / `FEMALE` buttons and hotkey `F`) in the 3D Character Studio (`WardrobeStudio`, `YouthPhysique::feminine`, `YouthAttire::feminine`) with 6 dedicated female character archetypes (`AvatarArchetype::gender_title`, `AvatarArchetype::female_attire`: `VALKYRIE`, `HUNTRESS`, `HERBALIST`, `WEAVER`, `SHIELDMAID`, `LADY`).
- Distinctly feminine 3D character anatomy and wardrobe (`TunicStyle::emit`, `Accoutrement::emit`, `CrownStyle::emit`, `PolygonHeadgear::emit`, `FootwearStyle::emit`) featuring narrower shoulders, hourglass corset waist, flared A-line skirt/dress hem, sculpted 3D bust and Valkyrie breastplate, softer facial jawline with rosy lips and eyelashes, long flowing rear hair with twin front shoulder braids, and open-faced feminine headgear (winged diadem, huntress hood, floral wreath, high ribbon bun, shieldmaiden circlet, lady tiara and veil).

### Changed
- Deepened sneaking posture kinematics (`SneakGait::crouching`, `BodyPose::hip_elevation`, `BodyPose::leg_joints`, `BodyPose::torso_frame`, `BodyPose::external_arm`, `SightMode::arm_joints`) with deeper knee flexion (`elevation = 0.36`), forward-leaning upper torso hinged from the hips, forward-bent elbows, slightly inward-angled forearms, and diagonally inward-rotated hands.

## [0.25.0] - 2026-10-20

### Added
- Independent left-hand and right-hand weapon strike routing (`MouseButton::Left` for left offhand, `MouseButton::Right` for right relic, `StrideRhythm::trigger_left_strike`, `StrideRhythm::trigger_right_strike`, `BodyPose::with_strikes`) with centered HUD aiming crosshair.
- Calibrated 3-phase axe and sword strike kinematics (`BodyPose::axe_joints`, `BodyPose::sword_joints`, `BodyPose::wrist_snap`, `HeldRelic::hand_roll`) featuring downward wrist hinge flexion, overhead semicircular axe cleave (`wind` -> `bow` -> `strike`) with outward hand/axe-head roll touching the crosshair at apex, and tucked-elbow diagonal sword cross-slash aligning the sword tip directly with the crosshair.
- Four-phase two-legged jumping animation (`BodyPose::with_leap`, `AvatarPosture::advance_leap`, `GaitFigure::jumping_at`, `StudioAnimation::JumpingLeap`) modeling deep takeoff knee crouch, airborne full straight-leg backward extension, landing knee absorption, upright recovery, and $C^1$-continuous arm swing across the player avatar, the 4th spawn showcase figure (`SpawnTroupe`), and the 7-animation 3D Character Studio turntable (`TurntableStage`).
- Character Studio showcase screenshot (`docs/images/character/character-studio.png`) and expanded English documentation in `docs/CHARACTER.md` and `README.md`.

### Changed
- Decoupled First-Person camera `eye_origin` from `leap` posture flex (`BodyPose`) and anchored anatomical torso/head vertical proportions above `hip_elevation()` (`TunicStyle::emit`, `GirdleStyle::emit`, `PolygonAccoutrement::emit`) so crouching and jumping bend only the knees without squashing the torso or detaching the head.
- Shortened Character Studio right-hand panel title (`CharacterStudio`) to `"BODY PARTS"` so the header fits cleanly inside the border box.

### Fixed
- Resolved sprint-jump slope collision hang (`BodyEnvelope::unobstructed_aerial`, `BodyEnvelope::unobstructed_ground`, `AerialTrial::destination`) by evaluating radial clearance in two `0.16 m` half-steps and letting `AirborneFlight::advance` detect and land cleanly on inclined terrain up to `2:1` (`63°`).

## [0.24.0] - 2026-10-20

### Added
- Interactive in-game `Esc` modal menu (`EscapeMenu`, `SessionModal`, `ModalDirective`) with `RESUME GAME`, `CHARACTER`, and `QUIT GAME` buttons, keyboard shortcuts, and cursor release/re-capture.
- Full-height 3D Character Studio (`WardrobeStudio`, `TurntableStage`, `StudioAnimation`, `PixelStamp`, `ScreenPoint`, `ScreenBounds`) rendering a single centered character figure on a stone pedestal at `z = 4.65` with 360-degree mouse-drag and button/key turntable rotation (`< ROTATE`, `FRONT`, `ROTATE >`) and automatic sequential demonstration of all 6 animations (`StandingIdle`, `WalkingStride`, `RunningSprint`, `CrouchingSneak`, `AxeCleave`, `SwordSlash`).
- Six fundamentally distinct Polygon-City-style character body archetypes (`AvatarArchetype`: `Knight`, `Ranger`, `Woodsman`, `Blacksmith`, `Sentinel`, `Noble`) with bespoke 3D silhouette architecture (`PolygonAccoutrement`, `PolygonHeadgear`) including sallet visors, hooded cloaks, blacksmith aprons, tower shields, and velvet capes.
- Independent per-body-part `SIZE` (length/height) and `THICKNESS` (width/girth) customization (`BodyRegion`: `Head`, `Torso`, `Arms`, `Legs`, `PartProportion`, `YouthPhysique`) with real-time 3D joint chain and bone-length scaling (`BodyPose`).
- Expanded skin complexion palette (`ComplexionSwatch`) with 8 selectable skin tones (`Birch`, `Meadow`, `Amber`, `Umber`, `Porcelain`, `Olive`, `Bronze`, `Ebony`) and dedicated clothing selection controls (`TUNIC`, `SLEEVES`, `BELT`, `BOOTS`, `HEADGEAR`, `DYE COLOR`) alongside a single `BACK` navigation button.

### Changed
- Extended `FileAttire` (`character.toml`) and deterministic 16-byte wire serialization to persist per-region `SIZE` and `THICKNESS` proportions and expanded wardrobe variants.

## [0.23.0] - 2026-10-20

### Added
- Spawn showcase figure troupe (`FigureTroupe`, `SpawnTroupe`, `GaitFigure`, `TroupeAvatar`) placing three animated character instances in front of the spawn point facing the player to demonstrate standing (*Stehen*), walking (*Gehen*), and running (*Laufen*) gaits with zero runtime heap allocations.
- Interactive medieval ground weapons near spawn (`RelicSite`, `FieldAxe`, `FieldRelic`) featuring an authentic medieval bearded felling axe (`HeldRelic::WoodmanAxe`) and a medieval knightly sword (`HeldRelic::KnightlySword` with wheel pommel, leather grip, cruciform crossguard, and double-edged blade with fuller).
- Dual-hand item holding (`YouthAttire` with `relic` for right hand and `offhand` for left hand, hotkeys `8`/`H` and `9`/`G`, plus `E` pickup and `Q` ballistic throw) allowing the avatar to hold and wield items in both hands simultaneously in First-Person and Third-Person views.
- Weapon-specific mouse-click martial animations (`MartialStyle::LumberjackCleave` and `MartialStyle::SwordplaySlash`, `StrideRhythm::trigger_strike`, `cleave_arc`, `cleave_surge`) featuring a full-arm lateral woodcutter felling swing (*seitlicher Schwung*) with forward upper-arm surge and inward-and-forward axe shaft inclination stopping at the body center line (`x >= 0.0`), alongside a distinct diagonal sword cross-slash.
- Dedicated `SightMode` module (`crates/client/src/attire/sight.rs`) and troupe/weapon test suite (`crates/client/src/troupe_tests.rs`).

### Changed
- Refined 3D anatomical character proportions: broad-shouldered V-taper torso (`TunicStyle::emit`), 3-tier sculpted head with full rear-cranium hair/headgear coverage (`CrownStyle::emit`), hip sockets aligned cleanly at the tunic skirt hem (`y = 0.78 m`) to prevent thigh-torso clipping, and low running elbows (`>= 0.20 m` below shoulders) with slight lateral abduction.
- Seamless wrist-to-palm transition and tapered hand geometry (`VambraceStyle::emit`) with wristbands (`RawhideWraps`, `StuddedVambraces`) terminating cleanly behind the wrist joint (`z = -0.018 m`), tapered palm and 4-cm thumb, and demand-driven First-Person hand raising (`SightMode::arm_joints`) so empty hands hang naturally at the sides.
- Pulled First-Person camera eye position slightly forward (`CameraHorizon`, `BodyPose::eye_origin`) and expanded `BodyEnvelope` radius (`0.30 m`) for clear downward foot visibility without chest occlusion.
- Updated `docs/CHARACTER.md` with spawn showcase troupe, dual-wielding controls, interactive ground weapons, and martial kinematics documentation.

## [0.22.0] - 2026-10-19

### Added
- Faceted low-poly medieval youth / squire avatar system (`Wardrobe`, `YouthAttire`, `BodyPose`, `StrideRhythm`, `LimbFrame`, `FacetedSolid`, `TaperedPrism`, `AxialLimb`) with full First-Person Body Awareness and Multiplayer / Looking-Glass Mirror (`M`) rendering (`SightMode`).
- Articulated 5-finger hands with 3 phalanges per finger, natural anatomical wrist inflection, collinear forearm-to-wrist frame alignment, and gait-responsive finger curl (`VambraceStyle`, `BodyPose::finger_curl`).
- Constant-length 3D trigonometric leg joint chain (`BodyPose::leg_joints`) with human swing-through flexion, forward step extension, 3D knee perspective flare, wide hip sockets, sculpted thighs, knee garters, muscular calf bellies, and medieval boots (`FootwearStyle`).
- Live wardrobe customization hotkeys (`1`–`8`, `H`, `M`) across upper tunics (`TunicStyle`), forearm bracers (`VambraceStyle`), waist girdles (`GirdleStyle`), footwear (`FootwearStyle`), hair and headgear (`CrownStyle`), natural cloth dyes (`PigmentSwatch`), skin complexions (`ComplexionSwatch`), stature builds (`YouthPhysique`), and held relics (`HeldRelic`).
- Persistent TOML character profile storage (`character.toml`) and deterministic 9-byte multiplayer wire codec (`AttireProfile`, `FileAttire`).
- Octagonal physical body collision envelope (`BodyEnvelope`) preventing First-Person camera and upper torso clipping into solid block walls while preserving smooth wall sliding.
- Dynamic per-frame avatar mesh compositing over static world geometry (`CompositeMesh`, `GpuBuffer::render_combined`, `LineBuffer::render_combined`).
- Comprehensive character concept and wardrobe customization documentation in `docs/CHARACTER.md` with showcase illustrations in `docs/images/character/`.

### Changed
- Forward-sloped torso geometry (`TunicStyle::emit`, `GirdleStyle::emit`) with recessed collarbone yoke and wide flared hip skirt so the upper chest never occludes the waist or hips when looking down from First-Person.
- Partitioned forearm and bracer geometry (`BareForearms`, `RawhideWraps`, `SplintedVambraces`) into non-overlapping axial segments to eliminate skin Z-fighting through leather wraps.
- Completed Pure OOP / Elegant Objects refactoring across `client`, `engine`, and `world` crates and resolved all compiler and Clippy warnings.

## [0.21.0] - 2026-10-16

### Added
- Nocturnal Arctic polar lights (auroras) with dynamic waving green and violet curtains constrained to the Arctic face (`AuroraCanopy`, `CalibratedCanopy`, `VoidCanopy`).
- Server-configurable aurora scheduling via `[aurora]` section in `server.toml`, `--aurora` CLI flag, and `LOW_CUBE_AURORA` env var, supporting `nightly`, `off`, `rare`, `normal`, and `frequent` modes (`AuroraCadence`, `NightlyAuroraCadence`, `DormantAuroraCadence`, `StochasticAuroraCadence`, `ServerAurora`, `FileAurora`, `ArgumentAurora`, `EnvironmentAurora`).
- Server-configurable day/night cycle duration in minutes via `cycle = <minutes>` in `server.toml`, `--cycle <minutes>` CLI flag, and `LOW_CUBE_CYCLE=<minutes>` env var (`CycleDuration`, `ServerCelestial`, `FileCelestial`, `ArgumentCelestial`, `EnvironmentCelestial`).
- In-game screenshot capturing nocturnal polar lights over the Arctic face in `docs/images/climates/arctic-night.png`.

### Changed
- Refactored `TunedCadence` into a pure `CelestialCadence` decorator delegating sun trajectory and sky coloring to inner origin.
- Refactored `StochasticAuroraCadence` into a pure `AuroraCadence` decorator modulating base nocturnal activity.
- Refactored `FileCelestial`, `ArgumentCelestial`, and `EnvironmentCelestial` to evaluate configuration once at startup, eliminating runtime disk I/O, regex/string scanning, and per-frame heap allocations in the 60 FPS animation loop.
- Eliminated static utility functions in favor of encapsulated domain objects (`StochasticHash`) and message passing (`Vec4::lerp`).
- Updated `docs/SETTINGS.md` and `server.toml` to document minute-based cycle durations and polar light settings.

## [0.20.0] - 2026-10-16

### Added
- Dynamic diurnal day/night cycle and celestial vault rendering pipeline (`DiurnalPipeline`, `CelestialCadence`, `TerrestrialCadence`, `TunedCadence`).
- Celestial sky vault shader (`GlFirmament`, `firmament_glsl.rs`) rendering a fiery sun with atmospheric extinction and corona, twinkling starfield, and an opaque, cratered lunar sphere.
- High-contrast multi-scale procedural crater field generator (`crater_field`, `lunar_basin`) modeling large dark basalt maria (Oceanus Procellarum, Imbrium, Serenitatis, Tranquillitatis, Crisium), medium impact craters with raised rims, dense micro-craterlets, and Tycho impact ejecta rays.
- Complete star occlusion behind solid celestial bodies (moon and sun) and solid lunar body compositing, preventing stars from shining through the moon.
- Stable celestial pole alignment (`u_celestial_pole`) preventing gimbal flips, rotation snaps, or singularity distortions as the moon traverses the zenith.
- Server-configurable day/night cadence supporting CLI flags (`--day-cycle <seconds>`, `--day-cycle-seconds=<seconds>`), environment variable (`LOW_CUBE_DAY_CYCLE`), and configuration file (`server.toml`), defaulting to 1200 seconds (20 minutes).
- Principle of Static Configuration on Startup documented in `docs/SETTINGS.md`.
- In-game day and night comparison gallery across all 6 biomes in `docs/CLIMATES.md` and `README.md`.

### Changed
- Extended `GraphicPipeline::render` signature and `Avatar::surface_rotation` to pass current planetary surface rotation for horizon and celestial alignment.
- Split firmament GLSL shader sources into dedicated `firmament_glsl.rs` to maintain strict adherence to the 200-line ceiling rule.

### Added
- Climate-adapted procedural low-poly tree and forestry system (`Arboreal`, `TreeSpecies`, `StandingTree`, `ClearedPlot`) across all six planetary biomes (`TemperateOak`, `ArcticPine`, `JungleEmergent`, `DesertSucculent`, `SteppeAcacia`, `VolcanicSpire`).
- Solid physical trunk collision and direct timber harvesting lifecycle with haptic strikes, wood particle bursts, and instant zero-stump vanishing to direct inventory deposit.
- Forestry decorator pipeline (`PlantingGround`, `FullCubeGround`, `WoodlandGround`, `MeridianGround`, `SpawnSanctuaryGround`, `ForestedFace`) providing flat-cube ground support checks, climate density modulation, and spawn sanctuary protection.
- Aerodynamic canopy drag physics (`CanopyBraking`) softening vertical falls and enabling high-altitude tree canopy navigation.
- Asymmetric slope pacing (`InclinePace`) modulating stride vectors across uphill inclines (-25% penalty) and downhill declines (+35% boost) across all gaits (walking, sneaking, and sprinting).
- Comprehensive planetary climate zones documentation in `docs/CLIMATES.md` with in-game photography, hazard profiles, and geological breakdown.
- Tree catalog and forestry reference documentation in `docs/TREES.md`.
- Interactive Python tree visualization script in `scripts/tree_catalog.py`.
- In-game climate zone screenshots in `docs/images/climates/` (Temperate, Arctic, Desert, Jungle, Steppe, Volcanic).

### Changed
- Refactored material blocks (`DirtBlock`, `SandBlock`, `SnowBlock`, `StoneBlock`) from fake inheritance wrapper structs into clean factories producing the canonical `MaterialBlock` decorator directly.
- Refactored landscape entities (`VolcanicCaldera`, `DesertDunes`, `GlacialMassif`, `JungleKarst`, `SteppeTerrace`, `VoxelMountain`, `BlockGallery`) from simulated subclasses into direct `VoxelCluster` decorator factories, eliminating double-boxing and forwarding boilerplate.
- Refactored `BiomedCubeWorld` into a factory assembling `CubePlanet` and its surrounding decorators directly.
- Embedded 6-climate showcase slideshow grid and documentation links into `README.md`.
- Renamed internal decorator implementation files (`decorator.rs` -> `modulation.rs`, `trial.rs`) to eliminate design pattern jargon.

## [0.18.0] - 2026-10-15

### Added
- Flush 2-row wedge seam alignment across all 12 planetary cube seams, terminating slope coordinates at the symmetric midpoint boundary line ($u \pm 0.5, v \pm 0.5$) with zero gap and zero polygon overlap.
- 3-way tri-chamfer corner cap convergence across all 8 cube vertices, joining orthogonal edge wedge rows at the shared triple point ($H - 0.5, H - 0.5, H - 0.5$).
- Seam wedge verification test suite in `crates/world/src/relief/wedge_seam_tests.rs` testing flush edge contact, corner cap convergence, and biome-differentiated palette preservation.
- Active Milestone 3 roadmap in `docs/ROADMAP.md` tracking flush 2-row wedge seams, corner caps, and edge-inward terrain heightfield anchoring.

### Fixed
- Eliminated geometry interpenetration and nested polygon clipping between adjacent planetary cube faces along 90-degree seams.

## [0.17.0] - 2026-10-14

### Added
- Diagonal slope block form (`DiagonalSlope`) and typed constructors (`diagonal_slope_ne`, `diagonal_slope_nw`, `diagonal_slope_se`, `diagonal_slope_sw`) across all material blocks (`DirtBlock`, `SandBlock`, `SnowBlock`, `StoneBlock`) providing 45-degree diagonal slopes with watertight boundary triangles.
- Watertight wall sealing regression test suite in `crates/world/src/relief/watertight_tests.rs` and corner seam test suite in `crates/world/src/relief/corner_seam_tests.rs`.

### Fixed
- Watertight relief mesh sealing between Full-Cube blocks and all non-flush adjacent shapes (ramp wedges, valley corners, ridge corners, and seam chamfers), preventing see-through gaps into cube interiors.
- Preserved sub-second client launch latency with zero-cost plateau early-exit and short-circuited boundary seam elevation lookups.

### Changed
- Updated gameplay demonstration screenshot in `docs/images/gameplay.png` and `README.md`.
- Documented diagonal slope block forms in `docs/BLOCKS.md`.

## [0.16.0] - 2026-10-14

### Added
- Diagnostic Tron wireframe rendering mode (`--tron` CLI argument and in-game toggle key `T`) rendering terrain and block geometries purely as glowing unlit neon vector lines against a pitch-black cosmic void.
- Emissive diagnostic signature colors for all 15 block shapes (`VoxelShape::shape_color`) enabling immediate visual classification of cuboids, ramps, diagonal ridge/valley corners, stairs, pyramids, and gentle slopes.
- Elevation discontinuity and seam mismatch diagnostics: non-matching adjoining blocks and vertical cliff steps emit high-contrast warning red lines (`#FF1744`).
- Tron GPU rendering pipeline (`TronPipeline`, `TronShaderSource`, `TronProgram`) with black occlusion face pass via polygon offset preventing see-through backfaces, paired with emissive line mesh drawing (`glow::LINES`).
- CPU wireframe line batching (`CpuLineBatch`, `CpuTronBatch`, `GlLineMesh`, `TronMesh`).
- Faceted relief Tron palettes in `ReliefPalette::tron()` and `FacetedReliefFace::tron` for planetary cube terrain.
- Decoupled OpenGL host environment initialization into `GlHostEnvironment` (`crates/client/src/host_env.rs`) maintaining strict single-responsibility and file line limits.
- Comprehensive Tron wireframe color and mismatch reference documentation in `docs/BLOCKS.md`.

## [0.15.0] - 2026-10-14

### Added
- Complete shape constructors across all typed material blocks (`DirtBlock`, `SandBlock`, `SnowBlock`, `StoneBlock`) supporting gentle lower/upper wedges, gentle ridge/valley corners, and stair corners (`stair_corner_convex`, `stair_corner_concave`).
- Concave stepped inner corner mesh generation in `StairCorner::emit_shape`.
- Block geometry exporter in `crates/world/examples/export_blocks.rs`.
- Interactive block catalog and snapshot CLI in `scripts/block_catalog.py` providing listing (`list`, `list --forms`, `list --materials`), single-block rendering (`render`), and batch rendering (`render-all`).
- Comprehensive block catalog and geometry reference documentation in `docs/BLOCKS.md` with visual matrix across all materials and shapes.
- High-resolution (512x512 with 2x SSAA and alpha transparency) snapshot images for all 105 available blocks in `docs/images/blocks/`.
- Markdown-only commit workflow exception and line-count exemption in `AGENTS.md`, alongside CI `paths-ignore` filter in `.github/workflows/release.yml`.

### Changed
- Linked block catalog documentation in `README.md`.

### Added
- Procedural planetary relief generation in `crates/world/src/relief/` (`ReliefElevation`, `ZonedRelief`, `ReliefFace`, `ReliefFacet`, `ReliefCorner`, `ReliefPalette`, `ReliefStrata`, `ReliefCavity`) generating continuous, watertight terrain facets and slopes.
- Coherent gradient noise engine in `crates/world/src/noise/` (`SpatialFrequency`, `CoherentNoise`, `DomainWarp`, `FractalHarmonics`) providing continuous multi-octave 3D frequency sampling.
- Distinct biome landscape topologies in `crates/world/src/landscape/`:
  - `GlacialMassif` (Alpine horns, arêtes, and cirque basins in Arctic)
  - `VolcanicCaldera` (Summit crater, caldera rim, and basalt lava flows in Volcanic)
  - `KarstTowers` (Limestone tower karst pillars and river valleys in Jungle)
  - `SteppeTerrace` (Stratified mesas and terraced plateaus in Steppe)
  - `DesertDunes` (Barchan sand dunes, slip faces, and interdune corridors in Desert)
- Slope and corner block forms in `crates/world/src/block/`:
  - `GentleWedge` (half-pitch gentle slope along primary axes)
  - `GentleRidge` (diagonal ridge corner connecting gentle slopes)
  - `GentleValley` (concave diagonal valley corner connecting gentle slopes)
  - `StairCorner` (stepped corner block)
- Ridge and valley diagonal corner generation in `ReliefCorner` with closed manifold facets and slope surface collision height in `ZonedRelief`.
- Test suites in `relief/tests.rs`, `noise/tests.rs`, and `landscape/tests.rs`.

### Changed
- Refactored `crates/client/src/launch.rs` and `main.rs` to launch directly with zoned procedural relief planet mesh assembly.
- Streamlined `Biome` in `crates/world/src/biome.rs` to configure distinct biome-specific relief surfaces and stratum color palettes.
- Purged `crates/client/src/world_choice.rs`.

## [0.13.0] - 2026-10-13

### Added
- 3D volume-partitioned cubic planet architecture (`VolumePlanet`, `SparsePlanetaryGrid`, `VoxelVolume`, `StratifiedVolume`, `MonolithVolume`, `VoidVolume`) with multi-layer surface voxel crusts across all 6 faces.
- Impenetrable bedrock containment shell (`BedrockShell`, `VoxelBarrier`, `PlanetaryCrust`) sealing the 0-byte hollow core void.
- Greedy meshing (`GreedyMesh`, `emit_volume_mesh`) with cross-volume boundary occlusion culling across internal and external seams.
- Integrated `PlanetaryTopology` seam coordinate folding into `VolumePlanet::constrain_to_surface`, enabling seamless 90° edge traversal and continuous 360° circuits on voxel worlds.
- Sneak cliff-arrest policy (`permits_seam_fold`) at 90° planetary cube seams preventing accidental edge crossings and camera flips, while enabling lateral sliding along edges.
- Polymorphic world assembly (`WorldAssembly`, `VolumeAssembly`, `BiomedAssembly`, `ArgumentWorld`), setting `VolumePlanet` as the default world when launching the client.
- Integration test suite in `crates/client/src/chunk_traversal_tests.rs` verifying voxel planet locomotion, jumping, edge crossing, and sneak cliff arrest.

### Changed
- Purged all legacy aliases and shims (`VoidChunk`, `UniformChunk`, `StratifiedChunk`, `FacetedChunk`, `ChunkCoord`, `Chunk`, etc.), strictly standardizing the domain model around `VolumePlanet`, `VoxelVolume`, `VolumeCoord`, and `VoxelCoord`.
- Replaced vague constructor `BedrockShell::standard_shell` with expressive domain constructor `BedrockShell::boundary`.

### Added
- Completed Roadmap Phases 4, 5, and 6: Decoupled camera horizon convergence, ballistic flight, 360° circuit verification, visual continuity, and performance audit.
- Decoupled physical locomotion surface reorientation (`surface_rot`) and visual camera horizon convergence (`camera_rot`) in `PlanetaryAvatar`.
- Time-damped smooth camera horizon convergence (`converge_camera`) with gradual exponential slerp upon reaching and traversing seams.
- Automated end-to-end 360° planetary circuit test in `crates/client/src/circuit_tests.rs` verifying continuous 4-face traversal and steady-state horizontal view pitch.
- Ballistic edge leap and mid-air trajectory continuity unit tests in `crates/client/src/ballistic_tests.rs`.
- Visual regression tests in `crates/client/src/visual_tests.rs` for directional lighting, shadow umbra continuity, and quad alignment across all 12 seams.
- Hot simulation loop zero-runtime-allocation audit in `crates/client/src/allocation_audit.rs`.
- Architectural rule "Fail Fast (Never Fail Safe)" added to `AGENTS.md`.

### Fixed
- Fixed centripetal altitude calculation in `CentripetalWorld` using cubic $L_\infty$ distance (`max(|x|, |y|, |z|) - half_size`) instead of Euclidean length, eliminating false radial gravity and forced downward tilt on cube surfaces.
- Restored sharp 90° voxel planetary world in `launch.rs` (`BiomedCubeWorld::of_size`), ensuring the avatar's gaze remains level and fully under player control before reaching edges.
- Fixed avatar edge sticking on subsequent seams by ensuring immediate physical gravity frame reorientation and accurate seam clearance classification in `threshold.rs`.

### Changed
- Modularized flight phases by splitting monolithic `phase.rs` into `crates/client/src/phase/` submodules (`grounded.rs`, `airborne.rs`, `context.rs`, `resolution.rs`), enforcing strict 200-line limits across all source files.
- Consolidated gait decorators into `crates/client/src/gait/mod.rs` and streamlined locomotion order dispatch in `crates/client/src/order.rs`.

## [0.11.0] - 2026-10-12

### Added
- Phases 3 & 4 of 90° Edge Traversal Roadmap: Continuous Locomotion Kinematics & Horizon Viewport Orientation.
- `SurfaceClearance` and `StepThreshold` in `crates/client/src/threshold.rs` distinguishing open-air drops from continuous 90° corner transitions without procedural boolean flags.
- `ViewFrame` trait in `crates/client/src/horizon.rs` providing pure Tell-Don't-Ask viewport view-matrix generation.
- `CameraHorizon` encapsulating surface-aligned look direction and `HorizonTilt` decorator applying camera roll without vector leaks.
- `SurfaceHeading` in `crates/client/src/heading.rs` encapsulating tangential locomotion steering.
- `ProjectionLens` trait and `MagnifiedLens` decorator in `crates/client/src/lens.rs`.
- `CentripetalWorld` decorator in `crates/world/src/centripetal_world.rs` (Phase 5 of Roadmap) implementing altitude-dependent radial gravity well falloff toward the planetary core.
- `ClimateZone` domain enum in `crates/world/src/topology/climate_zone.rs` replacing `CubeFace` with expressive domain naming.
- Ken Perlin's $C^2$ quintic smootherstep and elevation-invariant surface delta in `SeamBevel` for seamless, jerk-free edge transitions.
- Comprehensive unit test suites in `tilt_tests.rs`, `seam_clearance_tests.rs`, and `centripetal_tests.rs`.

### Changed
- Purged `basis.rs`, splitting responsibilities into cohesive domain objects `horizon.rs` and `heading.rs` according to Yegor Bugayenko's declarative naming principles.
- Replaced framerate-dependent camera slerp and snap threshold with exponential decay time-damped slerp ($1 - e^{-1.5 \Delta t}$) and widened bevel transition corridor.
- Purged all dead code, eliminated all `#[allow(dead_code)]`, and removed all `is_` / `has_` method prefixes repository-wide.
- Updated `AGENTS.md` to enforce the 200-line limit immediately upon each code edit and streamlined the `create git message` workflow.
- Updated `docs/ROADMAP.md` tracking progress across Phases 3, 4, and 5.

## [0.10.0] - 2026-10-12

### Added
- Phase 2 of 90° Edge Traversal Roadmap: Seam Surface Geometry & Coordinate Folding (`world` crate).
- `SeamBevel` and `BevelTransition` interface in `crates/world/src/topology/bevel.rs` providing continuous spherical slerp normal transitions across 90° edge seams.
- `BeveledWorld` decorator in `crates/world/src/beveled_world.rs` decorating any `PlanetaryWorld` with smooth seam normal interpolation.
- `FoldedFace` decorator in `crates/world/src/topology/folded_face.rs` wrapping any underlying `Face`, cleanly separating seam folding from planar math.
- Comprehensive unit tests in `bevel_tests.rs` and `corner_tests.rs` verifying seamless 4-edge coordinate folding and C1 normal continuity without corner pinning.
- `MeshTarget` decorators in `crates/world/src/mesh_target.rs`: `ColoredMeshTarget`, `TintedMeshTarget`, `OffsetMeshTarget`, and `CountedMeshTarget`.
- `ElevatedFace` and `TintedFace` decorators in `crates/world` for normal offsets and atmospheric surface tints.
- `ElevatedShape` decorator in `crates/world/src/block/elevated_shape.rs` for vertical voxel shape stacking.
- `SensitivityLook` in `crates/client/src/orientation.rs` scaling mouse rotation deltas.
- `InvertibleLook` with polymorphic `AxisPolarity` (`DirectPolarity`, `ReversePolarity`), eliminating primitive boolean flags for view inversion.
- `DampedLocomotionTarget`, `GroundedLocomotionTarget`, `TallyLocomotionTarget`, and `MutedControls` in `crates/client/src/controls/decorator.rs`.
- `OffsetClock` and `FrozenClock` decorators in `crates/client/src/clock.rs` for monotonic timeline manipulation.

### Changed
- Refactored material blocks (`DirtBlock`, `StoneBlock`, `SandBlock`, `SnowBlock`) into living objects implementing `Block` and decorating `MaterialBlock`.
- Refactored `MaterialBlock` to polymorphically wrap any `Box<dyn Block>`, using `ColoredMeshTarget` for surface emission.
- Eliminated procedural `emit_form` from `FormBlock`; mesh emission now runs uniformly through `Block::emit_surface`.
- Eliminated all boolean flags (`invert_x`, `invert_y`) from `MouseLook`.
- Removed passive getters `source_face`, `source_edge`, `destination_face`, and `destination_edge` from `SeamFold`, replacing them with Tell-Don't-Ask methods (`bevel`, `destination`, `reciprocal_on`).
- Removed passive getters `edge` and `overshoot` from `PerimeterCrossing` in favor of polymorphic `dispatch`.
- Removed unused `color` getter from `MaterialBlock`.
- Split `gait.rs` into `gait/mod.rs` and `gait/decorator.rs`, and `controls.rs` into `controls/mod.rs` and `controls/decorator.rs`, enforcing the 200-line limit across all files.
- Updated `docs/ROADMAP.md` marking Phase 2 as completed (4/4).

## [0.9.0] - 2026-10-12

### Added
- Phase 1 of 90° Edge Traversal Roadmap: Planetary Topology & Edge Neighborhoods (`world` crate).
- `PlanetaryTopology` connecting the 6 faces of a cubic world across 12 seam neighborhoods (24 directed seam folds) with $O(1)$ constant-time canonical array indexing.
- `SeamFold` encapsulating 90° rotation, seam axis, center translation, and continuous coordinate/vector folding across seams.
- `FoldedPoint` domain entity encapsulating 3D spatial position on destination frames with Tell-Don't-Ask transfer.
- `FacePerimeter` and `PerimeterCrossing` domain objects classifying tangential boundary states (`Interior`, `Crossing`) and clamping without procedural data extraction.
- `CubeFace` domain enum modeling the 6 canonical cubic planetary faces with normal, tangent, bitangent, and surface frame construction.
- Comprehensive unit test suite in `crates/world/src/topology/tests.rs` verifying coordinate continuity, seam reciprocity, 90° normal switches, and 360° closed planetary circuit.
- Updated gameplay demonstration screenshot in `docs/images/gameplay.png`.

### Changed
- Refactored `BiomedCubeWorld::of_size` to construct all 6 biome faces declaratively via `PlanarFace::on_cube` and `CubeFace` domain entities, eliminating vector duplication.
- Updated `docs/ROADMAP.md` marking Phase 1 (Planetary Topology & Edge Neighborhoods) as completed (4/4).

## [0.8.0] - 2026-10-12

### Changed
- Simplified server physics configuration to a single intuitive `gravity` parameter using clean whole integers without decimals.
- Derived avatar jump height dynamically and naturally from planetary gravity ($h = \frac{5.12}{g}$), preserving constant human leap muscle impulse.
- Replaced verbose Server & World Configuration section in `README.md` with direct link to the dedicated settings guide.
- Streamlined Windows pre-built binary launch instructions in `README.md` to direct double-clicking.
- Rewrote `docs/SETTINGS.md` into a dedicated settings guide featuring integer planetary presets (`10`, `5`, `2`, `1`, `20`), configuration template, and precedence order.
- Updated `FilePhysics`, `ArgumentPhysics`, and `EnvironmentPhysics` to parse only `gravity` and eliminate redundant `jump_height` parameters.
- Updated `CONCEPT.md` and `server.toml` to align with the single-parameter integer gravity model.

### Added
- Decoupled planetary jump height and gravity from world-cube dimensions across all world scales.
- Server and world physics configurability with Earth gravity ($g = 9.81\,\text{m/s}^2$) and realistic human vertical leap ($h \approx 0.522\,\text{m}$) as standard baseline.
- `server.toml` configuration template supporting custom `jump_height` and `gravity` parameters.
- Command-line argument support (`--jump-height <METERS>`, `--gravity <M/S2>`) and environment variable overrides (`LOW_CUBE_JUMP_HEIGHT`, `LOW_CUBE_GRAVITY`).
- Pure OOP `PlanetaryPhysics` interface with `EarthPhysics` and `TunedPhysics` implementations.
- Pure OOP decorator pipeline for physics resolution: `ArgumentPhysics` $\to$ `EnvironmentPhysics` $\to$ `FilePhysics` $\to$ `EarthPhysics`.
- Dedicated player and server-host documentation in `docs/SETTINGS.md` detailing presets, configuration options, and evaluation precedence.
- Comprehensive unit test suite in `crates/client/src/physics_tests.rs` covering all physics decorators, file configuration parsing, and CLI flag evaluation.

### Changed
- Refactored `SurfaceFlight` with `calibrated` secondary constructor computing initial vertical velocity from target peak jump height and gravitational acceleration.
- Refactored `PlanetaryAvatar` with `with_flight` secondary constructor for polymorphic flight injection.
- Updated `CONCEPT.md` Section 2.1 and Section 5.2 to document world-size independence and server physics configuration.
- Updated `README.md` with Server & World Configuration section and links to `docs/SETTINGS.md`.

## [0.6.0] - 2026-10-12

### Added
- Double-tap forward sprint locomotion: tapping `W` twice within 300 ms triggers high-speed sprinting at 160% walking pace.
- High-speed dynamic FOV expansion: `PerspectiveLens` smoothly interpolates field of view from 65° to 78° during sprint locomotion with zero GPU overhead.
- Pure OOP `SprintCadence` state pattern (`RestingCadence`, `EngagedCadence`, `PrimedCadence`, `SprintingCadence`, and `TransitioningCadence`) encapsulating timeout duration without procedural branching, primitive flags, or half-born objects.
- Pure OOP `MonotonicTime` trait decoupling clock reads from static platform calls, with `SystemClock` for hardware time and `ControllableClock` for deterministic microsecond-precision testing.
- Pure OOP `SprintGait` implementing `LocomotionGait` alongside refactored `WalkGait` and `SneakGait`, encapsulating pacing factor, elevation, and target FOV as private fields with expressive secondary constructors (`WalkGait::upright`, `SneakGait::crouching`, `SprintGait::swift`).
- Sprint cancellation upon backward key press (`KeyCode::KeyS`).
- Comprehensive unit tests verifying cadence transitions, timeout rejection, lens expansion/restoration, and deterministic mock clock control.

### Added
- Sneak locomotion activated by `Shift` (`ShiftLeft` / `ShiftRight`) with 35% movement pacing.
- Edge cliff protection restraining the avatar from stepping off elevated blocks while sneaking, while allowing smooth lateral sliding along ledge perimeters.
- Crouch kinematics with smooth avatar viewpoint lowering from standing eye level (0.8) to crouched eye level (0.45).
- Pure OOP `LocomotionGait` trait (`WalkGait` and `SneakGait`) encapsulating gait-specific pacing, posture shaping, and precipice resolution without getters, setters, or procedural branching.
- Pure OOP `AvatarPosture` encapsulating eye-level transitions and orientation basis view matrix generation.
- Polymorphic `LeapIntent` (`RestingLeap` and `ActiveLeap`) eliminating primitive boolean jump request flags from `LocomotionOrder`.
- Comprehensive test coverage for sneak edge restraint, posture transitions, crouch view matrices, and shift key controls.
- Updated gameplay demonstration screenshot in `docs/images/gameplay.png`.

## [0.4.0] - 2026-10-12

### Added
- Multi-form voxel geometry objects in `crates/world/src/block/`: `CuboidShape` (full cube and half slab), `StairFlight` (two-tiered stepped stair), `RampWedge` (45° climbable incline), `RidgeCorner` (convex edge corner), `ValleyCorner` (concave valley corner), and `PyramidPeak` (45° pyramidal apex).
- Pure OOP material blocks derived from `FormBlock` via decorator composition: `StoneBlock`, `DirtBlock`, `SandBlock`, and `SnowBlock` in dedicated `crates/world/src/block/material/` submodules.
- `BlockGallery` exhibition in `crates/world/src/gallery.rs` showcasing all materials in parallel lines with 1-block spacing across all geometric block shapes.
- Alpine `VoxelMountain` massif in `crates/world/src/mountain.rs` with smooth rock slab entrance shelf, continuous 45° slope, and crystalline snow pyramid summit.
- Wall-sliding mechanics preserving lateral strafe motion when running diagonally against obstacle walls in `crates/client`.
- Pure OOP `SurfaceStride`, `WalkTrial`, and `AerialTrial` in `crates/client/src/stride.rs` encapsulating obstacle deflection and step resolution without getters or control-flow `Option`.
- `SurfaceClearance` in `crates/client/src/threshold.rs` encapsulating altitude clearance relative to surface up vectors with precipice, floor contact, and cliff impact semantics.
- `FlightContext` and `FlightProgress` in `crates/client/src/phase.rs` encapsulating simulation environment and phase transitions, reducing `FlightPhase::advance` to a single context parameter.
- `GridCell::corner_points` in `crates/world/src/block/cell.rs` constructing quad vertices directly without coordinate data extraction.
- Comprehensive unit tests verifying block form constraints, material derivation, gallery exhibition, mountain elevation profile, and obstacle wall-sliding traversal.

### Removed
- Legacy `BlockGallery::single_row` compatibility constructor and `VoxelBlock` type alias.
- Redundant `Face` parameter from `Block::emit_surface`, `FormBlock::emit_form`, and `VoxelShape::emit_shape`.
- Dummy `let _ = face.pull_direction()` statements in `StairFlight` and `PyramidPeak`.

## [0.3.0] - 2026-10-12

### Added
- Authentic Earth gravity baseline ($g = 9.81\,\text{m/s}^2$) and realistic human vertical leap ($h \approx 0.5\,\text{m}$, $v_0 \approx 3.2\,\text{m/s}$) in `CONCEPT.md` and `crates/client`.
- Micro-Topology & Multi-Form Voxel Shapes in `CONCEPT.md` Section 2.6 specifying climbable $45^\circ$ Wedges, $0.5\,\text{m}$ rock shelves (Slabs), and vertical precipices.
- Variable World Cube dimensions ($128^3$ to $2048^3$) with $1024^3$ as Earth baseline, scale-derived jump dynamics, and hollow core specification with an indestructible 1-block bedrock shell in `CONCEPT.md` Section 2.1.
- Pure OOP `ReliefFace` and `RidgeElevation` in `crates/world` introducing 3D faceted mountain terrain with continuous $45^\circ$ slope climbing and rock shelves.
- Pure OOP `GravitationalFlight` and polymorphic `FlightPhase` (`GroundedFlight`, `AirborneFlight`) eliminating boolean state flags in `crates/client`.
- Unified `LocomotionTarget` messaging in `crates/client` eliminating primitive boolean bit-bags from `KeyMotion`.
- Steerable Mounts & Domestication in `CONCEPT.md` Sections 3.2 and 4.5 specifying craftable saddles, pack bags, mounted combat, and 90° edge traversal for large fauna.
- Architecture & Interactive Fixtures section to `CONCEPT.md` specifying openable doors and windows with environmental sealing and access controls.
- Updated gameplay screenshot in `README.md` and `docs/images/gameplay.png`.

## [0.2.0] - 2026-10-11

### Added
- GitHub Actions workflow (`.github/workflows/release.yml`) building standalone Linux (`low-cube-linux`) and Windows (`low-cube-windows.exe`) executables published as GitHub Releases and workflow artifacts.
- Dedicated developer guide in `docs/DEVELOPMENT.md` covering setup, dependencies, execution, and controls.
- Pre-built binary download and direct execution instructions in `README.md`.
- Workflow rules for version increments and changelog updates upon `create git message` prompt in `AGENTS.md`.

### Changed
- **Pure OOP Refactoring (Elegant Objects / Alan Kay)**:
  - Eliminated `normal()` getter on `Face` trait in favor of behavioral methods (`surface_up()`, `pull_direction()`, `constrain()`, `emit_surface()`).
  - Encapsulated `half_size` and `divisions` directly within `PlanarFace`, eliminating procedural parameter passing.
  - Replaced procedural data fields in `CubePlanet`, making it a pure composite of `Face` objects.
  - Removed `aspect_ratio()` getter from `PlatformSurface` trait; `render_frame` now takes `avatar: &dyn Avatar` directly ("Tell, Don't Ask").
  - Removed procedural matrix orchestration and data extraction in `GameSession::handle_window_event`.
  - Added `OrientationBasis::aligned` secondary constructor and `view_matrix_at` method to encapsulate coordinate alignment logic.
  - Renamed `KeyState` to `KeyMotion` to avoid passive data container semantics.
  - Added `ForwardPipeline::daylight()` secondary constructor to encapsulate sunlight direction and clear color without magic numbers.

## [0.1.0] - 2026-10-11

### Added
- Minimal runnable prototype for planetary voxel exploration in Rust and OpenGL 3.3.
- `crates/world`: Spherical-cubic planetary world body with multi-biome faces (`BiomedCubeWorld`).
- `crates/engine`: Forward graphics pipeline, shader compilation, and mesh batching.
- `crates/client`: First-person avatar navigation with surface-relative orientation, mouse look, and keyboard controls.
- Architecture and codebase rules enforced via Alan Kay and Yegor Bugayenko (*Elegant Objects*) principles in `AGENTS.md`.
