# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
and [Conventional Commits](https://www.conventionalcommits.org/).

## [Unreleased]

## [0.7.0] - 2026-10-12

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
