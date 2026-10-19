# Game Concept: Low-Cube (Working Title)

A minimalist, high-performance voxel survival sandbox set on discrete cubic planetary worlds.

---

## 1. Overview & Vision

**Low-Cube** is a multiplayer voxel survival game where each world is a freestanding cube suspended in space. Players explore the six distinct faces of a cubic planetoid, adapt to 90-degree gravitational shifts when crossing edges, survive harsh environmental hazards, mine subterranean resources, construct structural networks, and traverse worlds via ancient Ring Gates. The game features a non-textured, flat-shaded aesthetic optimized to run at high frame rates on accessible hardware, powered by a custom engine developed from scratch.

---

## 2. World Structure & Physics

### 2.1 Cubic Planetary Topology & Variable Dimensions
- **Variable World Dimensions**: Cube worlds vary in scale across the multiverse, ranging from Pocket Moonlets ($128 \times 128 \times 128$ blocks), Moons ($256^3$), and Dwarf Worlds ($512^3$), up to the Standard Terrestrial World ($1024 \times 1024 \times 1024$ blocks) designed for 50-player community civilizations, and rare super-dense Mega-Cubes ($2048^3$).
- **The Hollow Core & 1-Block Indestructible Core Shell**:
  - The deep interior of each planetary cube is **hollow** (*The Hollow Core*), drastically minimizing memory footprint by representing the massive inner abyss as empty void (0-byte null chunks).
  - The boundary separating the minable geological crust from the hollow core is an **indestructible 1-block core shell** (*Bedrock Shell*).
  - This impenetrable, single-block foundation layer prevents players from falling into the internal void, serves as the rigid gravitational anchor, and provides the unbreakable bedrock foundation for structural stability (Section 2.3).
- **Face-Based Biomes**: Each of the 6 faces represents a distinct climate zone (e.g., Arctic, Desert, Temperate Forest, Volcanic, Jungle, Arid Steppe).
- **Planetary Gravity & Configurable Jump Dynamics (Decoupled from Cube Scale)**:
  - Planetary surface gravity and vertical jump height are **completely decoupled from cube world size**. Jump dynamics do not automatically change or scale based on the dimensions of the planetary cube ($128^3$ to $2048^3$).
  - **Earth Gravity as Standard Baseline**: The standard default configuration for any cube world is authentic Earth gravity ($g = 9.81\,\text{m/s}^2$) paired with a realistic human vertical leap ($h \approx 0.52\,\text{m}$, initial velocity $v_0 \approx 3.2\,\text{m/s}$). This default enables players to climb $50\,\text{cm}$ rock shelves (slabs) while preserving 1-meter vertical walls and cliffs as real physical barriers that require crafted ramps or carved paths.
  - **Authoritative Server Configurability**: Jump height and gravitational acceleration are fully configurable per server and world instance. Operators can configure custom jump heights (e.g. $h = 1.0\,\text{m}$ for high mobility, or lower values for restrictive survival) and gravity via the server configuration (`server.toml`), command-line parameters, or environment variables.

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

### 2.6 Micro-Topology & Multi-Form Voxel Shapes (Ramps, Wedges & Slabs)
- **The Anti-Pogo Principle**: Natural terrain is not a jagged staircase of 1-meter cliffs. Low-Cube eliminates the need for constant, repetitive jump-spamming to navigate hills and mountains.
- **Faceted Slopes & Wedges ($45^\circ$ Ramps)**: Climbable mountain flanks, ridges, sand dunes, and rail beds feature sloped blocks (Wedges) that avatars walk up smoothly and continuously without jumping.
- **Rock Shelves & Half-Steps ($0.5\,\text{m}$ Slabs)**: Terraced ledges, foothills, and rock steps calibrate exactly to the realistic human jump height ($0.5\,\text{m}$), rewarding intentional pathfinding.
- **Vertical Precipices ($1\,\text{m}$ Full Cubes)**: True cliffs and sheer rock faces represent genuine physical obstacles that require switchback paths, carving stairs, or structural ladders.
- **Faceted Flat-Shaded Aesthetic**: Angled geometric facets catch directional sunlight, producing sharp, sculptural low-poly terrain contours.

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
- **Creature Archetypes & Fauna**:
  - **Swimming**: Aquatic wildlife and deep-water predators.
  - **Crawling**: Burrowing insects, spiders, and subterranean creepers.
  - **Walking & Mounts**: Bipedal/quadrupedal wildlife, grazers, nocturnal hunter beasts, and large domesticable beasts (pack quadrupeds, heavy horned striders, and desert saurians) that players can tame, harness, and ride.
  - **Flying**: Aerial predators and scavenger birds.
- **Domestication & Taming**: Craftable saddles, reins, and feed allow players to domesticate large wild beasts, anchoring them at bases or utilizing them as active mounts.
- **Spawn Suppression**: Torches, campfires, and placeable lamps emit light zones that actively suppress hostile creature spawns within their illumination radius.

### 3.3 Death & Respawn
- **Life Capsule**: A craftable, deployable personal module that anchors the player's biometric data. Placing or activating a Life Capsule sets/updates the player's active respawn point upon death.

### 3.4 Avatar Embodiment & Medieval Wardrobe Customization
- **Medieval Youth Archetype**: Players inhabit a faceted low-poly medieval youth / squire (*Junger Knappe / Lehrling*) harmonized with the non-textured polyhedral world aesthetic.
- **First-Person Body Awareness & Looking-Glass Mirror**: Looking down in First-Person reveals the flat-front tunic, belt, chausses, knees, muscular calves, and boots stepping across the terrain alongside outstretched 5-finger articulated hands. Pressing **`M`** toggles the frontal Looking-Glass Mirror view (matching full multiplayer peer rendering with head, hair, or medieval hood).
- **Live Wardrobe & Persistent Profile (`character.toml`)**: Players customize tunics (`1`), forearm bracers (`2`), waist girdles (`3`), footwear (`4`), hair/headgear (`5`), natural cloth dyes (`6`), skin complexion & stature build (`7`), and held relics (`8` / `H`), persisted locally to `character.toml` and synchronized across multiplayer in a compact 9-byte packet. See the [Character Concept & Wardrobe Customization Reference](docs/CHARACTER.md) for full details.

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

### 4.4 Architecture & Interactive Fixtures
- **Openable Doors & Windows**: Craftable doors and hinged windows (available in wood, reinforced metal, and glazed varieties) that players can interact with to toggle between open and closed states.
- **Environmental Sealing & Shelter**: Closed doors and windows seal interior habitats against hostile creature pathfinding, extreme temperatures, and fluid breach, establishing a secure indoor refuge. When open, they permit passage, projectile fire, and ventilation.
- **Locking & Permissions**: Doors integrate with player access controls to secure private bases and outposts against unauthorized access in multiplayer.

### 4.5 Transportation & Infrastructure
- **Steerable Mounts & Beasts of Burden**: Players can craft saddles and bridles to mount and directly steer large domesticated beasts. Mounts provide biome-adapted cross-face mobility (effortlessly ascending $45^\circ$ mountain wedges, wading deep rivers), expanded mobile storage via saddlebags, and mounted combat capabilities. When ridden over 90° cube seams, mounts smoothly reorient into the adjacent face's gravitational plane.
- **90° Rail Networks**: Placeable rails, switches, and minecarts/trains capable of traversing terrain, slopes, and bending over 90° cube edges. Enables seamless high-speed cargo and player transport across adjacent faces.
- **Vehicles**: Craftable land vehicles (rovers) and aerial vehicles for cross-face exploration.

### 4.6 Inter-World Ring Gates
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
- **Process Model**: One server process manages one cube world instance (ranging from $128^3$ moonlets to $1024^3$ terrestrial worlds, up to $2048^3$ mega-worlds).
- **Capacity**: Up to 50 concurrent players per world instance.
- **Server Physics & Gravity Configuration**:
  - The dedicated server authoritatively governs player flight kinematics and trajectory physics.
  - Server operators configure planetary gravity via `server.toml` (in the server working directory):
    ```toml
    [physics]
    # Planetary gravity (default: 10 for standard Earth gravity)
    # Jump height derives automatically from gravity (lower gravity -> higher jumps)
    gravity = 10
    ```
  - Options can also be overridden via command-line flags (`--gravity <INTEGER>`) or environment variables (`LOW_CUBE_GRAVITY`). See the [Server & Game Settings Guide](docs/SETTINGS.md) for full configuration details and planetary presets.

