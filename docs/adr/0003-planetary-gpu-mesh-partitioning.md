# Architecture Decision Record: Planetary GPU Mesh Partitioning (6 Per-Face Meshes with Row & Flora Caching)

- **Status**: Accepted
- **Date**: 2026-10-21
- **Deciders**: Core Development Team
- **Technical Context**: Planetary Voxel Rendering, Terrain Modification, and OpenGL Buffer Management (`crates/engine`, `crates/world`, `crates/client`)

---

## 1. Context and Problem Statement

*Low-Cube* models the world as a $256 \times 256$ cubic planet consisting of 6 planetary faces (`CubePlanet`), where each face composes a terraced terrain relief (`FacetedReliefFace`) and procedural surface vegetation (`ForestedFace`).

Previously, the entire planet—all 6 faces and their trees—was emitted and uploaded into a single monolithic GPU mesh (`UploadedMesh`) containing approximately $6 \times 256 \times 256 = 393{,}216$ surface tiles plus interior cliff walls and flora quads (~500,000+ quads, ~20–30 MB of vertex/index buffers). Whenever a player excavated (`dig_block`) or placed (`place_block`) a single block on one face:
1. All 6 planetary faces had to re-emit their terrain and tree geometry on the CPU.
2. The entire monolithic vertex and index buffer had to be re-assembled in RAM and re-uploaded to the GPU via PCIe.

Even with CPU-side caching of elevation grids and row quads, re-aggregating and uploading all 6 faces on every block interaction caused redundant memory traffic (~83% of the uploaded geometry belonged to the 5 unmodified faces) and risked noticeable frame-time spikes on the baseline hardware (ThinkPad T480 with integrated Intel UHD 620 GPU).

We needed to decide how to partition the planetary GPU mesh to achieve instantaneous, stutter-free block excavation and placement while keeping per-frame OpenGL draw call overhead minimal.

---

## 2. Decision Drivers

1. **Sub-Millisecond Block Modification Latency**: Excavating or placing a block must feel instantaneous without causing frame drops below 60 FPS (16.6 ms frame budget).
2. **Minimal PCIe Bandwidth & RAM Allocation on Updates**: Only the geometry affected by the modified planetary face should be re-assembled and transferred to VRAM.
3. **Low Per-Frame Draw Call Overhead**: Integrated GPUs (Intel UHD 620) are sensitive to excessive OpenGL state switches and `glDrawElements` invocations per frame.
4. **Zero VRAM Leaks**: Replacing a partition's GPU mesh during runtime block modifications must deterministically free the superseded OpenGL Vertex Array (`VAO`), Vertex Buffer (`VBO`), and Element Buffer (`EBO`) resources.
5. **Pure OOP Alignment (Elegant Objects)**: The partitioning must be encapsulated behind clean domain interfaces (`Mesh`, `Planet`, `Surface`, `Relief`) without exposing internal buffer handles, getters, or procedural utility classes.

---

## 3. Considered Options

* **Option 1: Single Monolithic Planetary GPU Mesh (1 VAO/VBO/EBO for all 6 faces)**
   All 6 faces are merged into a single `UploadedMesh` and drawn with 1 `glDrawElements` call per frame. Every block edit re-collects quads from all 6 faces and re-uploads the entire planet to the GPU.
* **Option 2: 6 Per-Face GPU Meshes + CPU Row & Flora Caching (1 VAO/VBO/EBO per planetary face)**
   Each of the 6 planetary faces is uploaded as an independent `UploadedMesh` managed by a composite `PlanetaryMesh` (`[Box<dyn Mesh>; 6]`). Combined with CPU-side 3-row dirty caching in `FacetedReliefFace` and `OnceLock` flora caching in `ForestedFace`, a block modification only rebuilds the 3 affected terrain rows on the modified face and re-uploads that single face's GPU buffer (1/6 of the planet).
* **Option 3: Fine-Grained Sub-Face Sector/Chunk GPU Meshes (e.g., $16 \times 16$ sectors per face = 1,536 GPU meshes)**
   Each $256 \times 256$ face is subdivided into $16 \times 16$ sector meshes (256 meshes per face, 1,536 total across the planet). A block edit only uploads a tiny 16×16 patch, but rendering requires up to 1,536 draw calls and buffer bindings per frame (or complex frustum/occlusion culling infrastructure).

---

## 4. Evaluation & Comparison

| Criterion | Option 1: Single Monolithic Mesh (1 Mesh) | Option 2: Per-Face Meshes + Row Cache (6 Meshes) | Option 3: Sub-Face Sector Meshes (1,536 Meshes) |
| :--- | :--- | :--- | :--- |
| **Draw Calls per Frame** | **1** (`glDrawElements`) | **6** (`glDrawElements` — negligible overhead) | **Up to 1,536** (high driver overhead without culling) |
| **GPU Upload Volume per Edit** | 100% of planet (~20–30 MB) | **1/6 of planet (~3–5 MB, modified face only)** | < 0.1% of planet (~15 KB) |
| **CPU Terrain Rebuild per Edit** | 6 faces (or 1 face + 5 full face copies) | **3 rows ($3 \times 256 = 768$ cells) on 1 face** | $16 \times 16 = 256$ cells in 1–4 sectors |
| **CPU Flora Rebuild per Edit** | Recomputed or copied across 6 faces | **0 calculations** (cached in `OnceLock<Vec<Quad>>`) | 0 calculations (if cached per sector) |
| **VRAM Buffer Management** | 1 VAO/VBO/EBO | **6 VAO/VBO/EBOs** (simple `[Box<dyn Mesh>; 6]`) | 1,536 VAO/VBO/EBOs (high descriptor pressure) |
| **Architectural Complexity** | Low | **Low & Clean** (matches `CubePlanet` 6-face domain) | High (requires sector addressing & cross-sector borders) |

---

## 5. Decision Outcome

**Chosen Option: Option 2 — 6 Per-Face GPU Meshes (`PlanetaryMesh`) combined with CPU-side Row, Elevation, and Flora Caching.**

### Rationale:
1. **83% Reduction in GPU Upload & RAM Traffic**: Because block excavation and placement always occur on a single planetary face (`ModifiedSurface`), partitioning the GPU mesh into 6 face meshes ensures the other 5 faces remain completely untouched in both CPU RAM and GPU VRAM.
2. **Negligible Draw Call Cost**: Issuing 6 `glDrawElements` calls per frame instead of 1 has virtually zero measurable impact on the GPU or OpenGL driver, whereas 1,536 draw calls (Option 3) would require additional frustum culling and batching complexity.
3. **Synergy with 3-Row CPU Caching**: Within the single modified face, `FacetedReliefFace` caches emitted terrain quads per row (`Vec<Vec<Quad>>`). Since a block modification at $(col, row)$ only affects the slope and cliff walls of its direct cardinal neighbors, only 3 rows (`row - 1`, `row`, `row + 1` = 768 cells out of 65,536) are recomputed on the CPU.
4. **Zero Redundant Noise Evaluations**: `TerracedRelief` caches the $256 \times 256$ procedural elevation grid in a `OnceLock<Vec<i16>>` (turning subsequent height lookups into $O(1)$ array indexing), and `ForestedFace` caches all procedural tree quads in a `OnceLock<Vec<Quad>>`.
5. **Natural Domain Alignment**: `CubePlanet` is already modeled as 6 `Surface` instances indexed `0..6`. Mapping each planetary face to its own `Mesh` inside `PlanetaryMesh` preserves strict 1-to-1 domain correspondence.

---

## 6. Consequences & Implementation Guidelines

1. **Composite Planetary Mesh (`crates/engine/src/composite_mesh.rs`)**:
   - `PlanetaryMesh` encapsulates `[Box<dyn Mesh>; 6]` and implements the `Mesh` trait.
   - `PlanetaryMesh::render` iterates over all 6 face meshes to draw the planet.
   - `PlanetaryMesh::replace_face(face, next)` invokes `release(gl)` on the outgoing face mesh before swapping in the newly uploaded `UploadedMesh`.
2. **Explicit OpenGL Resource Deallocation (`crates/engine/src/lib.rs`, `crates/engine/src/uploaded_mesh.rs`)**:
   - The `Mesh` trait defines `fn release(&self, gl: &glow::Context)` so `UploadedMesh` explicitly deletes its OpenGL vertex array (`delete_vertex_array`), vertex buffer (`delete_buffer`), and element buffer when a face mesh is replaced, preventing VRAM leaks during repeated digging/building.
3. **Selective Face Emission (`crates/world/src/planet.rs`)**:
   - `Planet::emit_face(face, target)` emits the geometry of a single face (`0..6`) during initial world assembly (`SessionLaunch`) and targeted updates.
   - `Planet::emit_modified_face(target)` delegates directly to the face tracked by `last_modified` (`RefCell<usize>`), allowing `GameSession::refresh_modified_face` to rebuild and re-upload only the modified face after `dig_block` or `place_block`.
4. **Three-Tier CPU Caching (`crates/world/src/relief/`, `crates/world/src/tree/`)**:
   - **Elevation Cache (`TerracedRelief`)**: Lazily populates a $256 \times 256$ `OnceLock<Vec<i16>>` grid on first access so procedural noise is never evaluated twice for the same coordinate.
   - **Row Quad Cache (`FacetedReliefFace`)**: Stores terrain quads per row (`rows: Vec<Vec<Quad>>`, `dirty: Vec<bool>`). `WithBlock::excavated` and `WithBlock::elevated` invalidate only `row - 1`, `row`, and `row + 1`.
   - **Flora Cache (`ForestedFace`)**: Caches procedural tree geometry in `OnceLock<Vec<Quad>>` on first emission and replays it directly on subsequent face rebuilds.
