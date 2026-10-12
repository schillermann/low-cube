# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
and [Conventional Commits](https://www.conventionalcommits.org/).

## [Unreleased]

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
