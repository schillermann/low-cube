# Game Concept: Low-Cube (Working Title)

A minimalist, high-performance voxel survival sandbox set on discrete cubic planetary worlds.

---

## 1. Overview & Vision

**Low-Cube** is a multiplayer voxel survival game where each world is a freestanding cube suspended in space. Players explore the six distinct faces of a cubic planetoid, adapt to 90-degree gravitational shifts when crossing edges, survive harsh environmental hazards, mine subterranean resources, construct structural networks, and traverse worlds via ancient Ring Gates. The game features a non-textured, flat-shaded aesthetic optimized to run at high frame rates on accessible hardware, powered by a custom engine developed from scratch.

---

## 2. World Structure & Physics

### 2.1 Cubic Planetary Topology
- **Dimensions**: Exactly $256 \times 256 \times 256$ blocks per cube world.
- **Indestructible Planetary Core**: A dense, unbreakable central core at the heart of the cube prevents players from mining straight through the world and serves as the gravitational anchor.
- **Face-Based Biomes**: Each of the 6 faces represents a distinct climate zone (e.g., Arctic, Desert, Temperate Forest, Volcanic, Jungle, Arid Steppe).
- **World-Variable Gravity**: Different cube worlds possess unique gravity constants (e.g., low-gravity moon cubes with floaty jumps vs. super-dense high-gravity worlds).

### 2.2 Edge Gravity & 90° Seam Traversal
- **Player & Camera Orientation**: Approaching and stepping over a cube edge smoothly reorients the player coordinate system and camera by 90° toward the new face's gravitational plane.
- **Continuous Edge Physics**:
  - **Fluids**: Water and lava flow over edges, curving around the 90° corner into the new gravitational down-vector.
  - **Projectiles**: Arrows and thrown objects that fly past an edge bend around the curve under the influence of the adjacent face's gravitational pull.
  - **Falling Blocks**: Dislodged blocks falling past an edge will curve and descend onto the adjacent face.

### 2.3 Structural Integrity (Limited Stability System)
- **Support-Based Physics**: No floating blocks. Blocks require physical connection to a grounded foundation (bedrock or planetary core).
- **Material Stress Model**: Each block type possesses a horizontal and vertical load-bearing strength (similar to *Valheim* / *7 Days to Die*). Exceeding maximum support stress triggers a cascade collapse of unsupported blocks into falling debris.

### 2.4 Atmospheric Gradient & Space Boundary
- **Atmosphere Height Limit**: Each face has a natural atmospheric layer extending upward from the surface.
- **Space Boundary Hazards**: Building or climbing excessively high toward the vacuum of space causes the atmosphere to thin rapidly:
  - **Oxygen Depletion**: Asphyxiation damage begins when exceeding the atmospheric threshold.
  - **Freezing Temperature**: Extreme hypothermia kicks in at upper limits.
  - This establishes an organic, physical ceiling for construction without arbitrary invisible walls.

### 2.5 Elements & Fluids
- **Fluids**: Water and Lava with dynamic pressure-based cell propagation.
- **Fire**: Spreads through combustible vegetation and wooden structures.
- **Thermodynamics**: Lava meeting water solidifies into stone or obsidian.

---

## 3. Survival, Ecology & Combat

### 3.1 Vital Status Systems
- **Health**: Depleted by damage, falls, attacks, and extreme conditions.
- **Hunger & Thirst**: Depletes over time; requires hunting, foraging, farming, and clean water sources.
- **Oxygen**: Depleted when submerged underwater or entering high-altitude space boundary zones.
- **Body Temperature**: Affected by climate zones, weather, and day/night transitions (hypothermia in arctic zones, heat exhaustion in deserts).

### 3.2 Diurnal Shift & Edge-Based Sun Mechanics
- **Directional Sunlight**: A directional celestial light source illuminates the cube, causing 2–3 faces to experience daytime while opposite faces remain in darkness.
- **Fleeing into Daylight**: Players on a night-shrouded face can physically escape nocturnal predators by traveling across a 90° edge onto a sunlit face.
- **Creature Archetypes**:
  - **Swimming**: Aquatic wildlife and deep-water predators.
  - **Crawling**: Burrowing insects, spiders, and subterranean creepers.
  - **Walking**: Bipedal/quadrupedal wildlife, grazers, and nocturnal hunter beasts.
  - **Flying**: Aerial predators and scavenger birds.
- **Spawn Suppression**: Torches, campfires, and placeable lamps emit light zones that actively suppress hostile creature spawns within their illumination radius.

### 3.3 Death & Respawn
- **Life Capsule (Lebenskapsel)**: A craftable, deployable personal module that anchors the player's biometric data. Placing or activating a Life Capsule sets/updates the player's active respawn point upon death.

---

## 4. Progression, Crafting & Logistics

### 4.1 Mining & Underground Layers
- **Stratified Ores**: Copper, iron, gold, and diamonds distributed in deeper geological strata surrounding the indestructible core.
- **Transparent Blocks**: Smeltable glass and crystals for observation domes and greenhouse habitats.

### 4.2 Tools, Weapons & The Forge
- **Tools**: Pickaxes, axes, shovels, and specialized equipment with durability ratings.
- **The Forge**: Workstation dedicated to repairing worn tools and upgrading gear with refined alloys.
- **Combat Gear**: Melee weapons (swords, spears), ranged weapons (bows, crossbows), shields, and wearable armor for PvE and PvP combat.

### 4.3 Storage & Inventory
- **Player Inventory**: Personal grid-based carry capacity.
- **Containers**: Lockable storage chests and bulk crates with access control permissions for securing loot against other players in multiplayer.

### 4.4 Transportation & Infrastructure
- **90° Rail Networks**: Placeable rails, switches, and minecarts/trains capable of traversing terrain, slopes, and bending over 90° cube edges. Enables seamless high-speed cargo and player transport across adjacent faces.
- **Vehicles**: Craftable land vehicles (rovers) and aerial vehicles for cross-face exploration.

### 4.5 Inter-World Ring Gates
- **Transit & Animation**: Ancient circular portals bridging distinct cube worlds. Stepping into a Ring Gate initiates a hyper-space wormhole / tunnel animation.
- **Technical Role**: The tunnel sequence serves as an in-game loading transition, masking network handoff between server processes and chunk streaming.

---

## 5. Technical Architecture & Constraints

### 5.1 Target Hardware & Graphics Pipeline
- **Baseline Hardware**: Intel Core i5/i7 (ThinkPad T480 class) with NVIDIA GeForce MX150.
- **Low-End Target**: Smooth 60 FPS on integrated GPUs (Intel UHD 620).
- **Target OS**: **Linux and Windows** (macOS omitted to avoid deprecated OpenGL constraints and proprietary Metal API complexity).
- **Graphics API**: Modern core OpenGL (3.3 / 4.3) or Vulkan for Linux and Windows.
- **Styling & Shading**: Flat-shaded, vertex-colored or palette-shaded voxels with Greedy Meshing (reducing vertex/draw call overhead by up to 90%). No heavy texture sampling.

### 5.2 Server & Network Architecture
- **Dedicated Client-Server**: Authoritative server for both dedicated multiplayer and local single-player loopback.
- **Process Model**: One server process manages one $256^3$ cube world.
- **Ring Gate Handoff**: Stepping through a portal triggers an authoritative client transfer to the destination world's server instance during the tunnel animation.
- **Capacity**: Up to 50 concurrent players per world instance.
