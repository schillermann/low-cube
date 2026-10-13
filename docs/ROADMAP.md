# Low-Cube Development Roadmap

> **Strategic Vision**: Deliver a high-performance, minimalist voxel survival sandbox across discrete cubic planetary worlds, engineered strictly under **Pure Object-Oriented Programming (Alan Kay & Yegor Bugayenko)** with zero compromises.

---

## Active Roadmap: Milestone 2 — Voxel Chunk Architecture, Hollow Core & Greedy Meshing

> **Objective**: Transition from procedural planar face surfaces to a scalable, high-performance 3D voxel chunk architecture supporting variable planetary dimensions ($128^3$ to $2048^3$), zero-memory hollow core voids, indestructible bedrock shells, and greedy meshing with coplanar face culling.

### Progress Summary

| Phase | Title | Scope | Status | Progress |
| :--- | :--- | :--- | :--- | :--- |
| **Phase 1** | Chunk Domain & Sparse Planetary Grid | `world` | Completed | 4/4 |
| **Phase 2** | Indestructible Bedrock Shell & Crust Stratification | `world` | Completed | 4/4 |
| **Phase 3** | Greedy Meshing & Occlusion Culling | `engine`, `world` | Completed | 4/4 |
| **Phase 4** | Multi-Form Voxel Chunk Integration | `world` | Completed | 3/3 |
| **Phase 5** | Planetary Chunk Topology & 90° Seam Alignment | `world` | Completed | 3/3 |
| **Phase 6** | Session Integration, Scalability & Performance Benchmarks | `client`, `world` | Completed | 4/4 |

---

### Phase Breakdown & Task Tracking

#### Phase 1: Chunk Domain & Sparse Planetary Grid (`world` crate)
- [x] Define [`Chunk`](../crates/world/src/chunk/mod.rs) behavioral trait encapsulating voxel queries and mesh emission without exposing internal arrays.
- [x] Implement [`VoidChunk`](../crates/world/src/chunk/void.rs) (0-byte Null Object) representing empty interior void in the hollow core without heap allocations.
- [x] Implement [`UniformChunk`](../crates/world/src/chunk/uniform.rs) representing solid or uniform bedrock/rock volumes with a single material descriptor.
- [x] Implement [`StratifiedChunk`](../crates/world/src/chunk/stratified.rs) ($16 \times 16 \times 16$ voxel grid) storing minable surface and sub-surface geological strata.
- [x] Unit tests verifying zero-byte memory allocation and $>90.9\%$ hollow core void savings ([`chunk::tests`](../crates/world/src/chunk/tests.rs)).

#### Phase 2: Indestructible Bedrock Shell & Crust Stratification (`world` crate)
- [x] Define [`VoxelBarrier`](../crates/world/src/chunk/barrier.rs) and [`BedrockShell`](../crates/world/src/chunk/bedrock_shell.rs) sealing the boundary between minable geological crust and the hollow core void.
- [x] Implement core containment constraint preventing avatars, projectiles, and dislodged blocks from entering the inner void.
- [x] Define [`PlanetaryCrust`](../crates/world/src/chunk/crust.rs) encapsulating variable crust thickness (16–48 blocks) tailored to world dimensions ($128^3$ to $2048^3$).
- [x] Unit tests verifying bedrock invulnerability, structural anchor integrity, and abyss containment ([`bedrock_tests.rs`](../crates/world/src/chunk/bedrock_tests.rs)).

#### Phase 3: Greedy Meshing & Occlusion Culling (`engine` & `world` crates)
- [x] Implement coplanar neighbor occlusion culling eliminating hidden internal faces between solid adjacent voxels ([`VoxelNeighborhood`](../crates/world/src/chunk/neighborhood.rs)).
- [x] Implement 2D greedy quad merging across coplanar voxel facets sharing identical material and normal ([`FacetSlice`](../crates/world/src/chunk/greedy.rs)).
- [x] Implement Tell-Don't-Ask chunk mesh emission directly appending merged quads into [`MeshTarget`](../crates/world/src/lib.rs) ([`GreedyMesh`](../crates/world/src/chunk/greedy_mesh.rs)).
- [x] Verification tests confirming $\ge 70\%$ to $99.6\%$ reduction in vertex/quad counts compared to naive meshing ([`greedy_tests.rs`](../crates/world/src/chunk/greedy_tests.rs)).

#### Phase 4: Multi-Form Voxel Chunk Integration (`world` crate)
- [x] Extend chunk meshing to multi-form voxels (wedges, slopes, slabs, ridges) without procedural type checks ([`FacetedChunk`](../crates/world/src/chunk/faceted_chunk.rs)).
- [x] Dispatch surface elevation queries through chunk-adorned forms ([`FacetedChunk::constrain_surface`](../crates/world/src/chunk/faceted_chunk.rs)).
- [x] Unit tests verifying normal orientation and continuous geometry for multi-form blocks within chunks ([`faceted_tests.rs`](../crates/world/src/chunk/faceted_tests.rs)).

#### Phase 5: Planetary Chunk Topology & 90° Seam Alignment (`world` crate)
- [x] Map 3D chunk coordinates to planetary face topologies across all 6 faces ([`ChunkCoord`](../crates/world/src/chunk/coord.rs)).
- [x] Implement seam neighbor resolution across 90° folds for cross-face greedy meshing and seam face culling ([`PlanetaryNeighborhood`](../crates/world/src/chunk/planetary_neighborhood.rs)).
- [x] Unit tests verifying gap-free, seamless quad continuity and normal alignment across all 12 planetary seams ([`seam_tests.rs`](../crates/world/src/chunk/seam_tests.rs)).

#### Phase 6: Session Integration, Scalability & Performance Benchmarks (`client` & `world` crates)
- [x] Integrate chunked planetary world into [`ChunkedCubeWorld`](../crates/world/src/chunk/world.rs).
- [x] Scale verification: instantiate $128^3$ (Moonlet), $512^3$ (Dwarf), and $1024^3$ (Terrestrial) worlds, confirming stable memory footprint ($>82\%$ to $>90.9\%$ void savings) ([`world_tests.rs`](../crates/world/src/chunk/world_tests.rs)).
- [x] Automated end-to-end traversal test: Avatar walks and jumps across chunk-backed planetary faces and 90° seams ([`chunk_traversal_tests.rs`](../crates/client/src/chunk_traversal_tests.rs)).
- [x] Performance audit: confirmed 60+ FPS and zero hot-loop allocations adhering strictly to file line limits ($\le 200$ lines).

---

## Completed Roadmaps

<details>
<summary><strong>Milestone 1: 90° Edge Traversal & Planetary Seam Navigation (Completed in v0.12.0)</strong></summary>

### 1. Progress Summary

| Phase | Title | Scope | Status | Progress |
| :--- | :--- | :--- | :--- | :--- |
| **Foundation** | Planetary Baseline & Face Physics | `world`, `client` | Completed | 4/4 |
| **Phase 1** | Planetary Topology & Edge Neighborhoods | `world` | Completed | 4/4 |
| **Phase 2** | Seam Surface Geometry & Coordinate Folding | `world` | Completed | 4/4 |
| **Phase 3** | Locomotion Kinematics & Edge Traversal | `client` | Completed | 4/4 |
| **Phase 4** | Continuous Viewport Orientation & Horizon Tilt | `client` | Completed | 4/4 |
| **Phase 5** | Ballistic Edge Flight & Gravitational Wells | `client`, `world` | Completed | 3/3 |
| **Phase 6** | Integration, Planetary Loop Verification & Polish | `client`, `world` | Completed | 3/3 |

### 2. Verified Deliverables
- Continuous 90° coordinate folding and surface stride across 12 seam neighborhoods.
- Decoupled physical surface reorientation and time-damped camera horizon convergence.
- Centripetal gravity well falloff using cubic $L_\infty$ distance.
- Singularity-free parallel transport viewport basis across all 12 edges and 8 vertices.
- Automated end-to-end 360° planetary circuit test, visual seam continuity, and zero hot-loop allocations.

</details>

---

## Key Architectural Principles

> [!IMPORTANT]
> All implementations must adhere strictly to **Pure Object-Oriented Programming (Alan Kay & Yegor Bugayenko)**:
> 1. **Tell, Don't Ask**: Objects encapsulate their behavior; no procedural getters/setters or raw array extraction.
> 2. **Polymorphism over Branches**: Use polymorphic states (e.g. `VoidChunk`, `UniformChunk`, `StratifiedChunk`) instead of `null` or procedural flags.
> 3. **Declarative Naming**: No `-er` or `-or` suffixes (no `ChunkManager`, `Mesher`).
> 4. **Strict File Limit**: Every source file must remain $\le 200$ lines of code.
> 5. **Fail Fast**: Immediate error propagation or panic on contract violations; never fail safe.
