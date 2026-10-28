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

- **Coordinates & Grid Cells (`crates/world`)**:
  - `SurfaceFrame` (`crates/world/src/frame.rs`): Local 3D face coordinate frame (`point_at`, `planar_coords`, `cell_at`, `step_elevation`).
  - `GridCell` (`crates/world/src/block/cell.rs`): 0.5-unit terrain cell (`GridCell::SPAN`, `at_planar`, `center`, `corners`).
- **Terrain Relief, Excavation & Soil Clods (`crates/world/src/relief/`)**:
  - `TerracedRelief` (`terraced.rs`): Decorator over `FacetedReliefFace` coordinating block mutations, cavities, and strata.
  - `StrataShell` (`strata.rs`): Subterranean crust layers, hollow-world vault ceiling, and glowing magma core.
  - `BlockFacets` (`facets.rs`): Beveled surface block emission, wall sealing, and damage craters (`BlockFissure` in `block/fissure.rs`).
  - `EarthClods` (`block/earth_clods.rs`): Turf clod and soil pebble structure on top, walls, and overhanging undersides (`emit_terraced`, `emit_crown`, `emit_sole`).
  - `SurfaceSightRay` (`sight_ray.rs`): 3D voxel raycast for excavating and placing blocks (`brow_origin`, `target_excavation`, `target_placement`).
- **Trees & Biomes (`crates/world/src/tree/`)**:
  - `WoodlandGround` $\rightarrow$ `StandingTree` $\rightarrow$ `SolidArboreal` / `ClearedPlot` / `HewnTarget` composing `PrismTrunk`, `PolyhedralCrown`, `ConicalSkirt`, and `FloorLitter`.
- **Avatar, Kinematics & Inventory (`crates/client/src/`)**:
  - `PlanetaryAvatar` (`avatar.rs`): Composes `AvatarBody`, `AvatarSight`, `AvatarBearing`, and `AvatarGear`.
  - `PalmEarthworks` (`earthworks.rs`) & `DigGrip`: Dual-hand excavation progress, foraging, block placement, and `BeltSatchel` (`attire/pouch.rs`).
  - `YouthAttire` & `SatchelWardrobe` (`attire/mod.rs`): Wardrobe composition (`GarmentEnsemble`, `IdentityTraits`, `HeldHands`).
  - `BodyPose` (`attire/pose.rs`) & `StrideRhythm` / `MartialStyle` (`attire/rhythm.rs`): 3D joint kinematics and polymorphic weapon/digging swings.
