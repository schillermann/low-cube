# Game Concept: Low-Cube (Working Title)

A minimalist, high-performance voxel sandbox and survival game set on cubic planetary worlds.

---

## 1. Overview & Vision

**Low-Cube** is a cross-platform, multiplayer voxel survival game where each world is a giant freestanding cube in space. Players explore six distinct faces of the cube planetoid, adapt to shifting gravitational fields when traversing edges, survive hostile nights, mine resources, forge equipment, and build structures without floating blocks. The game emphasizes high performance and a clean, non-textured aesthetic, built upon a custom engine from scratch.

---

## 2. Core Gameplay Mechanics

### 2.1 Players
- **Modes**: Single-player and cooperative/competitive multiplayer.
- **PvE & PvP**: Defend against hostile nighttime creatures, wildlife, and potentially hostile players.
- **Interaction**: Mine, craft, place items freely, build shelters, and explore.

### 2.2 The Cube World & Gravity
- **Planetary Cube Structure**: Each world is a finite, 6-faced giant cube.
- **Dynamic Gravity**:
  - Each face has its own directional down-vector pointing perpendicular to the face (or toward the planetary center).
  - Crossing an edge seamlessly pivots the player and camera 90° into the gravitational plane of the adjacent face.
- **Ring Gates (Portals)**:
  - Giant ring-shaped structures allowing players to travel between different cube worlds.

### 2.3 Environment & Biomes
- **Terrain**: Varied topography including mountains, valleys, plains, and cave networks.
- **Climate Zones**: Diverse biomes (temperate, arid, arctic, volcanic, etc.) distributed across world faces or regions.
- **Day/Night Cycle & Creature Ecology**:
  - **Daytime**: Peaceful fauna and safe resource gathering.
  - **Nighttime**: Aggressive predators and creatures emerge to hunt. Players must build defensive shelters or gear up to survive.

### 2.4 Elements & Fluids
- **Elements**: Water, Fire, and Lava.
- **Interactions**: Dynamic fluid flow, fire propagation, cooling of lava into stone, and hazardous environmental damage.

### 2.5 Voxel Blocks & Physics
- **Mineral Distribution**: Underground ore veins (copper, iron, diamonds, etc.) scaling with depth from the surface.
- **Material Properties**: Transparent blocks (e.g., glass) alongside opaque terrain blocks.
- **Structural Integrity & Gravity**:
  - Blocks cannot float freely in the air. Unsupported blocks detached from a stable foundation will succumb to gravity and collapse/fall.

### 2.6 Tools, Weapons & Progression
- **Resource Gathering**: Specialized tools for excavation, woodcutting, and mining.
- **The Forge**: Workstation used to repair degraded tools and upgrade equipment using refined materials.
- **Combat Gear**: Melee and ranged weapons, armor, and defensive fortifications.

### 2.7 Items & Building
- **Free Placement**: Functional and decorative items can be collected, crafted, and placed with arbitrary orientation or block-aligned grids.
- **Inventory & Crafting**: Crafting systems for survival essentials, construction, and survival infrastructure.

---

## 3. Technical Specifications & Architecture

### 3.1 Target Hardware & Target Platforms
- **Baseline Hardware**: Intel Core i5/i7 (ThinkPad T480 class) with NVIDIA GeForce MX150.
- **Low-End Target**: Playable at 60 FPS on integrated GPUs (e.g., Intel UHD 620).
- **Target Operating Systems**: Linux, macOS, and Windows.

### 3.2 Engine & Renderer
- **Custom Engine**: Built from scratch without third-party general-purpose game engines (e.g., Unity/Unreal).
- **Aesthetic**: Flat-shaded, vertex-colored or palette-shaded low-poly voxels. No heavy texture mapping to optimize cache locality and memory bandwidth.
- **Chunk Management**: 3D voxel chunking adapted to a 6-face cubic planetary topology.

### 3.3 Networking & Server Architecture
- **Client-Server Model**: Dedicated authoritative server architecture used for both multiplayer and single-player (local loopback server).
- **World Partitioning**: One cube world instance per server process.
- **Capacity**: Scalable up to 50 concurrent players per world server.

---

## 4. Key Design Challenges & Open Questions

1. **Gravity Transition at Cube Edges**:
   - Camera smoothing and player orientation handling across 90-degree seams.
   - Projectile trajectories, fluid flow, and falling blocks near or across cube edges.
2. **Structural Integrity Performance**:
   - Voxel connectivity checks (BFS/flood-fill) can be computationally expensive when large structures are modified. A tiered stability or localized stress model will be required.
3. **Fluid Simulation Across Faces**:
   - Rules for water and lava spilling over the edge onto an adjacent face with perpendicular gravity.
4. **Inter-World Ring Gate Networking**:
   - Handling seamless player handoff between different server instances when stepping through a ring gate.
