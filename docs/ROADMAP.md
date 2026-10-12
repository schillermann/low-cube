# Roadmap: 90° Edge Traversal & Planetary Seam Navigation

> **Objective**: Enable seamless, continuous locomotion across all six faces of the cubic planetoid (*Continuous 90° Seam Traversal*), including consistent gravity reorientation, convex edge kinematics, and smooth camera horizon tilting.

---

## 1. Progress Summary

| Phase | Title | Scope | Status | Progress |
| :--- | :--- | :--- | :--- | :--- |
| **Foundation** | Planetary Baseline & Face Physics | `world`, `client` | Completed | 4/4 |
| **Phase 1** | Planetary Topology & Edge Neighborhoods | `world` | Completed | 4/4 |
| **Phase 2** | Seam Surface Geometry & Coordinate Folding | `world` | Completed | 4/4 |
| **Phase 3** | Locomotion Kinematics & Edge Traversal | `client` | Planned | 0/4 |
| **Phase 4** | Continuous Viewport Orientation & Horizon Tilt | `client` | Planned | 0/4 |
| **Phase 5** | Ballistic Edge Flight & Gravitational Wells | `client`, `world` | Planned | 0/3 |
| **Phase 6** | Integration, Planetary Loop Verification & Polish | `client`, `world` | Planned | 0/3 |

---

## 2. Phase Breakdown & Task Tracking

### Foundation: Planetary Baseline & Face Physics
- [x] Discrete 6-faced cubic planetoid with distinct biomes ([`BiomedCubeWorld`](../crates/world/src/biome.rs)).
- [x] Gravitational pull and surface-up vector calculation per face ([`CubePlanet`](../crates/world/src/planet.rs)).
- [x] Avatar surface reorientation via Slerp alignment ([`PlanetaryAvatar::reorient_surface`](../crates/client/src/avatar.rs)).
- [x] Intrafacial obstacle sliding, slab climbing, and sneak cliff arrest ([`SurfaceStride`](../crates/client/src/stride.rs)).

### Phase 1: Planetary Topology & Edge Neighborhoods (`world` crate)
- [x] Define [`FacePerimeter`](../crates/world/src/topology/perimeter.rs) encapsulating 4 outer edge thresholds $(+U, -U, +V, -V)$.
- [x] Implement [`SeamFold`](../crates/world/src/topology/seam.rs) encapsulating 90° rotation and coordinate mapping across seams.
- [x] Implement [`PlanetaryTopology`](../crates/world/src/topology/mod.rs) connecting 6 faces across 12 seam neighborhoods.
- [x] Comprehensive unit tests verifying coordinate continuity across all 12 edge transitions ([`topology::tests`](../crates/world/src/topology/tests.rs)).

### Phase 2: Seam Surface Geometry & Coordinate Folding (`world` crate)
- [x] Define [`FoldedPoint`](../crates/world/src/topology/folded_point.rs) transforming spatial positions across 90° edge folds onto destination frames.
- [x] Implement [`SeamBevel`](../crates/world/src/topology/bevel.rs) encapsulating edge seam geometry and continuous surface normal transitions.
- [x] Refactor [`PlanarFace::constrain`](../crates/world/src/face.rs) to delegate perimeter crossings to `SeamFold`.
- [x] Comprehensive unit tests verifying no clamping/pinning at corner coordinates (e.g. $(0, 17, 17)$) ([`corner_tests`](../crates/world/src/topology/corner_tests.rs), [`bevel_tests`](../crates/world/src/topology/bevel_tests.rs)).

### Phase 3: Locomotion Kinematics & Edge Traversal (`client` crate)
- [ ] Implement `FoldedStride` translating tangential velocity across 90° seam folds.
- [ ] Implement `SeamClearance` distinguishing open-air drops from continuous 90° corner transitions.
- [ ] Adapt [`WalkTrial`](../crates/client/src/stride.rs) and [`SurfaceStride`](../crates/client/src/stride.rs) for continuous ground traversal over edges.
- [ ] Define edge behavior for [`SneakGait`](../crates/client/src/gait.rs) (cliff arrest vs. edge peering).

### Phase 4: Continuous Viewport Orientation & Horizon Tilt (`client` crate)
- [ ] Implement `ParallelTransportBasis` replacing reference branch singularity (`up.y.abs() < 0.9`).
- [ ] Implement `HorizonTilt` encapsulating smooth camera horizon roll during seam traversal.
- [ ] Preserve avatar pitch and yaw angles relative to the local horizon during edge crossings.
- [ ] Posture unit tests verifying singularity-free view matrices across all 12 edges and 8 vertices.

### Phase 5: Ballistic Edge Flight & Gravitational Wells (`client` & `world` crates)
- [ ] Extend [`AirborneFlight`](../crates/client/src/phase.rs) for parabolic edge leaps landing on adjacent faces.
- [ ] Implement `CentripetalGravity` calculating angular gravity in edge transition wedges.
- [ ] Flight unit tests verifying smooth mid-air edge trajectories without velocity discontinuity.

### Phase 6: Integration, Planetary Loop Verification & Polish
- [ ] Automated end-to-end test: Avatar completes a 360° planetary circuit and returns to start position.
- [ ] Visual verification of directional lighting, shadow continuity, and biome transitions at seams.
- [ ] Performance audit: Zero runtime allocations in hot simulation loop and strict 200-line file limits.

---

## 3. Key Design Decisions

> [!IMPORTANT]
> The following architectural and gameplay decisions guide the implementation:

1. **Seam Geometry (Sharp 90° Fold vs. Beveled Seam)**:
   - *Sharp 90° Edge*: True voxel aesthetic with an immediate transition of surface normal at the perimeter line.
   - *Beveled Seam (e.g. 0.25–0.5 m zone)*: A narrow transition strip where surface normals rotate continuously.
2. **Sneak Restraint (`Shift`)**:
   - Whether sneaking should stop the player at the edge like a cliff ledge, or permit creeping over the edge onto the adjacent face.
3. **Camera Horizon Tilt Dynamics**:
   - Time-damped horizon transition (e.g., 200–300 ms Slerp) vs. distance-coupled rotation linked strictly to physical progression across the seam.
