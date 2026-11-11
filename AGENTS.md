# Agent Guidelines

## Git Commit Conventions

All git commit messages in this repository must strictly follow the **Conventional Commits v1.0.0** specification.

### Format
```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc.)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files

### Guidelines
- Use the imperative present tense in the description (e.g., `feat: add ...` instead of `feat: added ...`).
- Keep the first line concise (50–72 characters).
- Include relevant context or motivations in the body when necessary.

### Workflow: `create git message` / `create git commit`
Upon the prompt `create git message` or `create git commit`, check the modified files:
- **Markdown-Only Exception**: If only Markdown files (`*.md`) were modified, do NOT execute version increment, changelog update, or version tagging. Directly stage the files (`git add -A`) and execute the commit via CLI (`git commit -m ...`) using the generated commit message (typically of type `docs`).
- **Standard Sequence** (when any code, assets, or non-Markdown files are modified):
  1. **Version Increment**: Increment the version number following Semantic Versioning across crate manifests (`crates/*/Cargo.toml`).
  2. **Changelog Update**: Update `CHANGELOG.md` according to Keep a Changelog conventions, promoting unreleased items under the new version header with the current date.
  3. **Message Generation & CLI Commit**: Generate the commit message following the format and types specified above, stage all modified and untracked files (`git add -A`), and commit them directly via CLI (`git commit`).
  4. **Version Tagging**: Create an annotated Git tag with the incremented version number (`git tag -a v<version> -m "Release v<version>"`, e.g., `git tag -a v0.31.0 -m "Release v0.31.0"`) pointing to the newly created commit so `git push --follow-tags` pushes it automatically.

### CI Pipeline Conventions
- **Markdown-Only Filter**: All CI/CD build and release workflows in `.github/workflows/` must configure `paths-ignore: ['**/*.md']` to prevent redundant builds and artifact releases when only documentation files are modified.

---

## Architectural & Code Philosophy: Pure OOP (Alan Kay & Yegor Bugayenko)

All code in this project must strictly adhere to **Pure Object-Oriented Programming (OOP)** as conceptualized by **Alan Kay** and formulated by **Yegor Bugayenko (Elegant Objects)**.

### 1. Alan Kay's Foundation
- **Messaging over Data**: Objects communicate exclusively by sending and receiving messages. Never treat objects as passive data containers.
- **Encapsulation & Local Retention**: Internal state and processes are strictly encapsulated, private, and fully concealed from other objects.
- **Late Binding**: Behavior is bound dynamically at runtime via interfaces and polymorphism, not statically or via procedural branching.

### 2. Elegant Objects Rules (Yegor Bugayenko)
- **No Getters and Setters**: Never extract data from an object to process it outside ("Tell, Don't Ask"). Objects represent living entities with behavior, not bags of data.
- **No NULL / No Half-Born Objects**: Never use, accept, or return `null` / `nullptr`. In Rust, never use `Option<T>` to represent uninitialized or half-born object states; use the Polymorphic State Pattern or Null Object pattern instead. (Technical `Option` as a return type from standard library lookup/iteration is permitted).
- **No Static Methods or Utility Classes**: No `Utils`, `Helpers`, or static procedural functions. Everything must be an instance of an object that implements an interface.
- **Immutability by Default**: Objects should be immutable whenever feasible. State transitions produce new object representations rather than mutating existing fields.
- **No Public Fields**: All attributes and state are strictly private.
- **No Type Casting or Type Inspection**: Never use `instanceof`, `dynamic_cast`, type checking, or reflection to determine behavior. Use polymorphism and method dispatch.
- **Interfaces Everywhere**: Code against clean, focused interfaces / traits. Objects must be decoupled and swappable.
- **Clean Constructors**: Primary constructors do nothing except assign arguments to internal fields. No I/O, heavy calculations, or side effects during construction.
- **Composition & Decorators**: Favor object composition and decorators over class inheritance.
- **Max 4 Encapsulated Fields per Object**: No struct or class may encapsulate more than 4 fields. If an entity requires more state, decompose it into smaller collaborating objects or decorators.
- **Small Files (Max 300 Lines) & No Single-Line Compression**: No source code file may exceed 300 lines of code. Never cram multiple statements, nested conditionals, or loops onto a single line to bypass the line limit; format methods across readable lines and split larger objects into dedicated, decoupled files instead. Verification of modified files' line counts is performed immediately after every code change. Documentation files (`*.md`) are explicitly exempt from this constraint.
- **Focused Interfaces & Atomic Tests**: Keep traits/interfaces narrow and cohesive, and write small, fast, atomic unit tests focused on a single behavior rather than monolithic multi-feature scenario scripts.
- **No "-er" in Names (Declarative Naming)**: Never name classes, structs, or traits with "-er" or "-or" agent suffixes (e.g., Manager, Controller, Helper, Handler, Renderer, Compiler, Uploader, Receiver, Bootstrapper, Player). Objects must be named as declarative nouns representing what they ARE, never procedural workers or action executors (what they DO).
- **No Generic or Vague Names (`Standard`, `Default`, `Common`, `Base`, etc.)**: Never use generic, lazy, or vague words such as `Standard`, `Default`, `Common`, `Base`, `Simple`, `Generic`, `Abstract`, or `Data` in names of project-owned classes, structs, traits, methods, or modules. Names must be specific, expressive domain nouns representing what the entity is in its model. (Required external crate traits like `std::default::Default` or `std::fmt::Display` are exempt when implementing upstream interfaces).
- **No Dead Code**: Never leave unused code or suppress dead code compiler warnings with `#[allow(dead_code)]`. If code is unneeded, delete it immediately without compromise.
- **No `is_` Prefix for Methods/Functions**: In Elegant Objects, boolean-returning methods must never be prefixed with `is_` (or `has_`). Use declarative adjectives, predicates, or nouns directly (e.g., `passable()`, `unobstructed()`, `precipice()`, `floor_contact()`, `cliff_impact()`, `crouching()`, `expanded()`, `in_zone()`, `reciprocal_to()`, `walkable_with()`).
- **Fail Fast (Never Fail Safe)**: Never swallow errors, suppress failures, or silently substitute dummy fallback values when an invalid state, contract violation, or corrupt condition occurs. Fail immediately at the earliest possible moment (e.g., via panic or error propagation) rather than masking defects with fail-safe compromises.

---

## Domain & Decorator Map (Fast Navigation)

- **Coordinates, Block Molds & Grid Cells (`crates/world`)**:
  - `SurfaceFrame` (`crates/world/src/frame.rs`): Local 3D face coordinate frame (`point_at`, `planar_coords`, `cell_at`, `step_elevation`).
  - `GridCell` (`crates/world/src/block/cell.rs`): 0.5-unit terrain cell (`GridCell::SPAN`, `at_planar`, `center`, `corners`).
  - `BlockMold` (`crates/world/src/block/mold.rs`), `BlockMedium` / `MediumShape` (`crates/world/src/block/timber_grain.rs`), `EarthClods` (`crates/world/src/block/earth_clods.rs`), `MeadowCrown` / `NaturalMeadowTarget` (`crates/world/src/mesh_target/mod.rs` & `crates/world/src/mesh_target/crown.rs`), `FieldCameraTripod` (`crates/world/src/block/field_camera.rs`) & `CabinSeatMesh` (`crates/world/src/block/cabin_seat.rs`): Placed voxel forms, mediums (`CabinSeat`, `FieldCamera`, vehicle/crane parts), unified soil/turf clod patterns (`CROWN_PATTERN`, `TURF_FRINGE_PATTERN`, `FLANK_PATTERN`, `SOLE_PATTERN`) and 5-tier 3D contour terraces (`emit_contour_terraces`) + 30°-splayed grass tufts (`emit_natural_patch`, `emit_hybrid`) shared between dirt blocks and the World-Cube surface (`emit_meadow_around`), centralized camera rotation angles (`palm_camera_yaw`, `placed_camera_yaw`), 3-legged tripod camera (`tripod_hub`, `emit`), and sculpted seat meshes.
- **Terrain Relief, Excavation & Soil Clods (`crates/world/src/relief/`)**:
  - `TerracedRelief` (`terraced.rs`), `FacetedReliefFace` / `ReliefCellCache` (`face/mod.rs` & `face/cache.rs`), `ReliefFacet` (`facet.rs`) & `ReliefWall` (`wall.rs`): Decorator over cached relief faces composing `NaturalStrata`, `ShaftExcavation`, and `OverhangMolds`, plus `O(1)` local camera turf emission (`emit_meadow_around`) and excavated pit wall clods.
  - `ColumnClearance` (`column_clearance/mod.rs` & `column_clearance/decorators.rs`): Unified 3D vertical column support, crust/shaft confinement (`confine`), lethal core-floor contact (`lethal_at`), and ray-hit resolution via the `MoldedColumn(ShaftColumn(NaturalColumn))` decorator chain.
  - `ShaftExcavation` (`shaft_excavation.rs`), `OverhangMolds` (`overhang_molds.rs`), `MoldedStructure` (`molded_structure/mod.rs`, `cells.rs`, `decorators.rs`) & `TurfScars` (`turf_scars.rs`): Downward shafts, overhanging voxel molds via the `RiggedStructure(ParkedStructure(&WorldBlocks))` decorator chain, bare-dirt regrowth, and dig-crack stages.
  - `NaturalStrata` (`natural_strata.rs`) & `CrustVault` / `VaultGeometry` (`crust_vault/mod.rs` & `crust_vault/mesh.rs`): Subterranean crust layers and indestructible lethal core floor.
  - `SurfaceSightRay` (`sight_ray/mod.rs` & `sight_ray/contact.rs`): 3D voxel raycast for excavating and placing blocks (`brow_origin`, `target_layer`, `placement_cell`, `seat_backrest_cell`).
  - `CantileverRig` / `RigArm` / `RigRegistry` (`cantilever_rig/mod.rs`, `arm.rs`, `registry.rs`, `mesh.rs`), `CraftCluster` / `ParkedFleet` (`craft_cluster/mod.rs` & `fleet.rs`), `VehicleAssembly` (`vehicle_assembly/mod.rs` & `part.rs`) & `VehicleBlockMesh` / `WheelProfile` (`crates/world/src/block/vehicle_parts.rs`): Independent crane-mounted craft construction (any block medium), seat-gated perching, steering-gated crane release, parked vehicle fleet, and rotating wheel profiles.
- **Trees & Biomes (`crates/world/src/tree/`)**:
  - `WoodlandGround` $\rightarrow$ `StandingTree` $\rightarrow$ `SolidArboreal` / `ClearedPlot` / `HewnTarget` composing `PrismTrunk`, `PolyhedralCrown`, `ConicalSkirt`, and `FloorLitter`.
- **Avatar, Kinematics, Photography & Inventory (`crates/client/src/`)**:
  - `PlanetaryAvatar` (`avatar/mod.rs`): Composes `AvatarBody` & `AvatarSight` (`avatar/body_sight.rs`) and `AvatarBearing` & `AvatarGear` (`avatar/bearing_gear.rs`).
  - `CameraExposure` & `PolaroidPrint` (`photo_plate/exposure.rs`), `PhotoPlate` (`photo_plate/mod.rs`) & `CameraMount` / `TripodMount` / `SightMode` (`attire/sight.rs`): Polymorphic camera mount (`Unmounted`, `Handheld`, `Placed`), 3D tripod lens ray projection (`TripodMount::lens_ray`), placed-only self-timer countdown, immediate handheld shutter trigger, PNG development, Polaroid HUD overlay, and 8-way camera sight angles.
  - `DrivenCar` & `DriveKinematics` & `VehiclePilot` (`drive.rs`): Seated vehicle throttle/brake/reverse physics, steering yaw, and wheel spin.
  - `BodyEnvelope` & `CrustEnvelope` (`envelope/mod.rs` & `envelope/crust.rs`): Horizontal stride/aerial collision and vertical crust/shaft confinement decorator.
  - `AirborneFlight` (`phase/mod.rs` & `phase/aerial.rs`): Airborne trajectory decorator chain (`CliffReboundTrajectory(ParabolicTrajectory)`).
  - `PalmEarthworks` (`earthworks/mod.rs` & `earthworks/debris.rs`) & `DigGrip` (`grip.rs`): Dual-hand excavation progress, debris splash, foraging, block placement, and `BeltSatchel` / `PalmStack` (`attire/pouch/mod.rs` & `attire/pouch/palms.rs`).
  - `YouthAttire` & `SatchelWardrobe` (`attire/mod.rs`) & `HeldRelic` / `RelicFigure` (`attire/relic/mod.rs`, `attire/relic/mesh.rs`): Wardrobe composition (`GarmentEnsemble`, `IdentityTraits`, `HeldHands`).
  - `BodyPose` & `ArmKinematics` (`attire/pose/mod.rs` & `attire/pose/arms.rs`), `AvatarPosture` (`posture/mod.rs` & `posture/vital.rs`) & `StrideRhythm` / `MartialStyle` (`attire/rhythm/mod.rs` & `attire/rhythm/martial.rs`): 3D joint kinematics, cardiac blackout strain, and polymorphic weapon/digging swings.
  - `SparrowFlock`, `Sparrow` / `SparrowActivity` & `SparrowFigure` (`sparrow/mod.rs`, `sparrow/activity.rs`, `sparrow/mesh.rs`) & `SparrowChirp` (`cardiac/birdsong.rs`): Woodland house sparrows cycling through flight (`Flying`), ground foraging (`GroundPecking`), and tree crown perching (`TreePerched` via `arboreal_perch_near`), plus synthesized 3-syllable birdsong chirps.
  - `ActionBindings` & `InputTrigger` (`controls/mod.rs` & `controls/bindings.rs`): Keyboard, mouse button, and wheel input bindings.
  - `Session` (`session/mod.rs`, `session/display.rs`, `session/stage.rs`) & `WardrobeStudio` / `StudioLayout` (`menu/studio/mod.rs` & `menu/studio/layout.rs`): Interactive window loop, GPU scene orchestration, and 3D wardrobe preview studio.

---

## Fast Iteration & Verification Rules

- **Direct Navigation (No Git Archeology)**: Jump directly to the target files via the Domain & Decorator Map above. Read only the 1–3 directly involved files and apply edits immediately without running `git log`, `git show`, or broad exploratory searches.
- **Zero Pass-Through Delegation Chains**: When an action couples inventory (`BeltSatchel` / `PalmEarthworks`) with posture or camera state (`AvatarBearing`), encapsulate the collaboration directly in `AvatarGear::dispatch_trigger` (`avatar/bearing_gear.rs`) instead of forwarding helper methods through `PlanetaryAvatar` (`avatar/mod.rs`).
- **Decoupled Voxel Bounds vs. Sculpted Mesh**: Keep `MediumShape::elevation_at` at full `GridCell::SPAN` (`0.5m`) for all vehicle and crane mediums so raycast targeting (`intercepts`), excavation (`remove_layer`), and lateral attachment (`ContactFace::classify`) remain invariant across visual mesh variations (`WheelProfile`, `CabinSeat` backrest).
- **Unified Block & World-Cube Turf Patterns**: Keep all soil clod, grass sod, and overhanging turf-rim geometry constants (`CROWN_PATTERN`, `TURF_FRINGE_PATTERN`, `FLANK_PATTERN`, `SOLE_PATTERN`) centralized in `EarthClods` (`crates/world/src/block/earth_clods.rs`) and consumed directly by `MeadowCrown` / `NaturalMeadowTarget` (`crates/world/src/mesh_target.rs`), `ReliefFacet` (`crates/world/src/relief/facet.rs`), and `ReliefWall` (`crates/world/src/relief/wall.rs`) so block and World-Cube surface changes never diverge.
- **Targeted Test Execution**: Never run an unfiltered workspace-wide `cargo test` for localized changes. Run only the affected test module (e.g., `cargo test -p client --bin client drive::tests` or `cargo test -p world --lib relief::soil_clod_tests`) together with `wc -l` on modified files.
- **Line-Budget Headroom (< 250 Lines per File)**: Proactively split any module approaching 260–300 lines into a submodule directory (`mod.rs` + focused collaborator files) so new behavior can be added directly without iterative line-trimming passes.
