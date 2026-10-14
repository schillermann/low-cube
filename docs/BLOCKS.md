# Block Catalog & Geometry Reference

In **Low-Cube**, the block system is designed according to the principles of **Pure Object-Oriented Programming (OOP)** and **Elegant Objects**:
Geometric shapes ([`FormBlock`](file:///home/mario/code/low-cube/crates/world/src/block/form.rs)) and material properties ([`MaterialBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/material_block.rs)) are decoupled as **decorators**.

Every geometric shape can be represented in any material. Every block has a unique identifier following the convention:
```text
<material_id>_<shape_id>
```
Examples: `dirt_stair`, `stone_wedge`, `sand_gentle_lower_wedge`, `dark_stone_cube`.

---

## 1. Available Block Shapes (15 Shapes)

| Shape ID | Shape Name | Height / Incline | Climbing Physics (Earth, $g=10$) | Image |
|---|---|---|---|:---:|
| `cube` | **Full Cube** | $1.0\,\text{m}$ (vertical 90°) | Impassable wall; requires a low-G jump or ramps/stairs. | <img src="images/blocks/form_cube.png" width="96" alt="Full Cube" /> |
| `slab` | **Half Slab** | $0.5\,\text{m}$ (plateau) | Directly scalable (human step height under standard gravity). | <img src="images/blocks/form_slab.png" width="96" alt="Half Slab" /> |
| `wedge` | **Ramp Wedge (45°)** | $0.0 \to 1.0\,\text{m}$ (1:1 incline) | Smoothly walkable without jumping impulse. | <img src="images/blocks/form_wedge.png" width="96" alt="Ramp Wedge" /> |
| `ridge_corner` | **Ridge Corner (45°)** | $0.0 \to 1.0\,\text{m}$ (diagonal ridge) | Convex corner joining two 45° ramps. | <img src="images/blocks/form_ridge_corner.png" width="96" alt="Ridge Corner" /> |
| `valley_corner` | **Valley Corner (45°)** | $0.0 \to 1.0\,\text{m}$ (diagonal valley) | Concave inner corner joining two 45° ramps. | <img src="images/blocks/form_valley_corner.png" width="96" alt="Valley Corner" /> |
| `peak` | **Pyramid Peak** | $0.5\,\text{m}$ (45° peak) | 4-sided pyramid peak as a natural summit point. | <img src="images/blocks/form_peak.png" width="96" alt="Pyramid Peak" /> |
| `stair` | **Stair Flight** | $2 \times 0.5\,\text{m}$ steps | Comfortable 2-step climb to full block height. | <img src="images/blocks/form_stair.png" width="96" alt="Stair Flight" /> |
| `stair_corner_convex` | **Convex Stair Corner** | $2 \times 0.5\,\text{m}$ steps | 90° exterior corner for stairways. | <img src="images/blocks/form_stair_corner_convex.png" width="96" alt="Convex Stair Corner" /> |
| `stair_corner_concave` | **Concave Stair Corner** | $2 \times 0.5\,\text{m}$ steps | 90° interior corner for winding stairways. | <img src="images/blocks/form_stair_corner_concave.png" width="96" alt="Concave Stair Corner" /> |
| `gentle_lower_wedge` | **Gentle Lower Wedge** | $0.0 \to 0.5\,\text{m}$ (2:1 incline) | Gentle shallow slope leading to first plateau or half slab. | <img src="images/blocks/form_gentle_lower_wedge.png" width="96" alt="Gentle Lower Wedge" /> |
| `gentle_upper_wedge` | **Gentle Upper Wedge** | $0.5 \to 1.0\,\text{m}$ (2:1 incline) | Continuation of the gentle slope to full block height. | <img src="images/blocks/form_gentle_upper_wedge.png" width="96" alt="Gentle Upper Wedge" /> |
| `gentle_lower_ridge` | **Gentle Lower Ridge** | $0.0 \to 0.5\,\text{m}$ (2:1 incline) | Shallow outer corner for gentle terrain ridges. | <img src="images/blocks/form_gentle_lower_ridge.png" width="96" alt="Gentle Lower Ridge" /> |
| `gentle_upper_ridge` | **Gentle Upper Ridge** | $0.5 \to 1.0\,\text{m}$ (2:1 incline) | Upper outer corner for gentle terrain ridges. | <img src="images/blocks/form_gentle_upper_ridge.png" width="96" alt="Gentle Upper Ridge" /> |
| `gentle_lower_valley` | **Gentle Lower Valley** | $0.0 \to 0.5\,\text{m}$ (2:1 incline) | Shallow inner corner for gentle terrain hollows. | <img src="images/blocks/form_gentle_lower_valley.png" width="96" alt="Gentle Lower Valley" /> |
| `gentle_upper_valley` | **Gentle Upper Valley** | $0.5 \to 1.0\,\text{m}$ (2:1 incline) | Upper inner corner for gentle terrain hollows. | <img src="images/blocks/form_gentle_upper_valley.png" width="96" alt="Gentle Upper Valley" /> |

---

## 2. Available Materials

| Material ID | Name | RGB Color | Rust Entity | Occurrence / Usage |
|---|---|---|---|---|
| `form` | **Form Block (Template)** | `Vec3(1.0, 1.0, 1.0)` | [`FormBlock`](file:///home/mario/code/low-cube/crates/world/src/block/form.rs) | Neutral building template / geometric prototype. |
| `dirt` | **Dirt (Loam)** | `Vec3(0.48, 0.35, 0.22)` | [`DirtBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/dirt.rs) | Fertile topsoil, temperate valleys. |
| `sand` | **Sand** | `Vec3(0.85, 0.75, 0.35)` | [`SandBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/sand.rs) | Warm desert sand, dune landscapes. |
| `snow` | **Snow** | `Vec3(0.92, 0.95, 0.98)` | [`SnowBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/snow.rs) | Crystalline alpine snow, polar ice caps. |
| `stone` | **Stone (Granite)** | `Vec3(0.48, 0.50, 0.46)` | [`StoneBlock::natural`](file:///home/mario/code/low-cube/crates/world/src/block/material/stone.rs) | Alpine rock, mountain ranges. |
| `dark_stone` | **Dark Stone (Shale)** | `Vec3(0.35, 0.42, 0.35)` | [`StoneBlock::dark`](file:///home/mario/code/low-cube/crates/world/src/block/material/stone.rs) | Dark volcanic rock, crater rims. |
| `lava` | **Lava / Magma** | `Vec3(0.95, 0.28, 0.05)` | [`MaterialBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/material_block.rs) | Glowing magma pools in calderas. |

---

## 3. Full Block Matrix (Materials $\times$ Shapes)

Each cell shows the rendered image of the corresponding block along with its unique block ID for referencing:

### Basic and Step Blocks

| Shape | Template (`form_*`) | Dirt (`dirt_*`) | Sand (`sand_*`) | Snow (`snow_*`) | Stone (`stone_*`) | Dark Stone (`dark_stone_*`) |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Full Cube** | <img src="images/blocks/form_cube.png" width="80" /><br>`form_cube` | <img src="images/blocks/dirt_cube.png" width="80" /><br>`dirt_cube` | <img src="images/blocks/sand_cube.png" width="80" /><br>`sand_cube` | <img src="images/blocks/snow_cube.png" width="80" /><br>`snow_cube` | <img src="images/blocks/stone_cube.png" width="80" /><br>`stone_cube` | <img src="images/blocks/dark_stone_cube.png" width="80" /><br>`dark_stone_cube` |
| **Half Slab** | <img src="images/blocks/form_slab.png" width="80" /><br>`form_slab` | <img src="images/blocks/dirt_slab.png" width="80" /><br>`dirt_slab` | <img src="images/blocks/sand_slab.png" width="80" /><br>`sand_slab` | <img src="images/blocks/snow_slab.png" width="80" /><br>`snow_slab` | <img src="images/blocks/stone_slab.png" width="80" /><br>`stone_slab` | <img src="images/blocks/dark_stone_slab.png" width="80" /><br>`dark_stone_slab` |
| **Stair Flight** | <img src="images/blocks/form_stair.png" width="80" /><br>`form_stair` | <img src="images/blocks/dirt_stair.png" width="80" /><br>`dirt_stair` | <img src="images/blocks/sand_stair.png" width="80" /><br>`sand_stair` | <img src="images/blocks/snow_stair.png" width="80" /><br>`snow_stair` | <img src="images/blocks/stone_stair.png" width="80" /><br>`stone_stair` | <img src="images/blocks/dark_stone_stair.png" width="80" /><br>`dark_stone_stair` |
| **Convex Stair Corner** | <img src="images/blocks/form_stair_corner_convex.png" width="80" /><br>`form_stair_corner_convex` | <img src="images/blocks/dirt_stair_corner_convex.png" width="80" /><br>`dirt_stair_corner_convex` | <img src="images/blocks/sand_stair_corner_convex.png" width="80" /><br>`sand_stair_corner_convex` | <img src="images/blocks/snow_stair_corner_convex.png" width="80" /><br>`snow_stair_corner_convex` | <img src="images/blocks/stone_stair_corner_convex.png" width="80" /><br>`stone_stair_corner_convex` | <img src="images/blocks/dark_stone_stair_corner_convex.png" width="80" /><br>`dark_stone_stair_corner_convex` |
| **Concave Stair Corner** | <img src="images/blocks/form_stair_corner_concave.png" width="80" /><br>`form_stair_corner_concave` | <img src="images/blocks/dirt_stair_corner_concave.png" width="80" /><br>`dirt_stair_corner_concave` | <img src="images/blocks/sand_stair_corner_concave.png" width="80" /><br>`sand_stair_corner_concave` | <img src="images/blocks/snow_stair_corner_concave.png" width="80" /><br>`snow_stair_corner_concave` | <img src="images/blocks/stone_stair_corner_concave.png" width="80" /><br>`stone_stair_corner_concave` | <img src="images/blocks/dark_stone_stair_corner_concave.png" width="80" /><br>`dark_stone_stair_corner_concave` |

---

### Standard Ramps & Corners (45°, 1:1)

| Shape | Template (`form_*`) | Dirt (`dirt_*`) | Sand (`sand_*`) | Snow (`snow_*`) | Stone (`stone_*`) | Dark Stone (`dark_stone_*`) |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Ramp Wedge (45°)** | <img src="images/blocks/form_wedge.png" width="80" /><br>`form_wedge` | <img src="images/blocks/dirt_wedge.png" width="80" /><br>`dirt_wedge` | <img src="images/blocks/sand_wedge.png" width="80" /><br>`sand_wedge` | <img src="images/blocks/snow_wedge.png" width="80" /><br>`snow_wedge` | <img src="images/blocks/stone_wedge.png" width="80" /><br>`stone_wedge` | <img src="images/blocks/dark_stone_wedge.png" width="80" /><br>`dark_stone_wedge` |
| **Ridge Corner (45°)** | <img src="images/blocks/form_ridge_corner.png" width="80" /><br>`form_ridge_corner` | <img src="images/blocks/dirt_ridge_corner.png" width="80" /><br>`dirt_ridge_corner` | <img src="images/blocks/sand_ridge_corner.png" width="80" /><br>`sand_ridge_corner` | <img src="images/blocks/snow_ridge_corner.png" width="80" /><br>`snow_ridge_corner` | <img src="images/blocks/stone_ridge_corner.png" width="80" /><br>`stone_ridge_corner` | <img src="images/blocks/dark_stone_ridge_corner.png" width="80" /><br>`dark_stone_ridge_corner` |
| **Valley Corner (45°)** | <img src="images/blocks/form_valley_corner.png" width="80" /><br>`form_valley_corner` | <img src="images/blocks/dirt_valley_corner.png" width="80" /><br>`dirt_valley_corner` | <img src="images/blocks/sand_valley_corner.png" width="80" /><br>`sand_valley_corner` | <img src="images/blocks/snow_valley_corner.png" width="80" /><br>`snow_valley_corner` | <img src="images/blocks/stone_valley_corner.png" width="80" /><br>`stone_valley_corner` | <img src="images/blocks/dark_stone_valley_corner.png" width="80" /><br>`dark_stone_valley_corner` |
| **Pyramid Peak** | <img src="images/blocks/form_peak.png" width="80" /><br>`form_peak` | <img src="images/blocks/dirt_peak.png" width="80" /><br>`dirt_peak` | <img src="images/blocks/sand_peak.png" width="80" /><br>`sand_peak` | <img src="images/blocks/snow_peak.png" width="80" /><br>`snow_peak` | <img src="images/blocks/stone_peak.png" width="80" /><br>`stone_peak` | <img src="images/blocks/dark_stone_peak.png" width="80" /><br>`dark_stone_peak` |

---

### Gentle Ramps & Corners (2:1 Incline)

| Shape | Template (`form_*`) | Dirt (`dirt_*`) | Sand (`sand_*`) | Snow (`snow_*`) | Stone (`stone_*`) | Dark Stone (`dark_stone_*`) |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Gentle Lower Wedge** | <img src="images/blocks/form_gentle_lower_wedge.png" width="80" /><br>`form_gentle_lower_wedge` | <img src="images/blocks/dirt_gentle_lower_wedge.png" width="80" /><br>`dirt_gentle_lower_wedge` | <img src="images/blocks/sand_gentle_lower_wedge.png" width="80" /><br>`sand_gentle_lower_wedge` | <img src="images/blocks/snow_gentle_lower_wedge.png" width="80" /><br>`snow_gentle_lower_wedge` | <img src="images/blocks/stone_gentle_lower_wedge.png" width="80" /><br>`stone_gentle_lower_wedge` | <img src="images/blocks/dark_stone_gentle_lower_wedge.png" width="80" /><br>`dark_stone_gentle_lower_wedge` |
| **Gentle Upper Wedge** | <img src="images/blocks/form_gentle_upper_wedge.png" width="80" /><br>`form_gentle_upper_wedge` | <img src="images/blocks/dirt_gentle_upper_wedge.png" width="80" /><br>`dirt_gentle_upper_wedge` | <img src="images/blocks/sand_gentle_upper_wedge.png" width="80" /><br>`sand_gentle_upper_wedge` | <img src="images/blocks/snow_gentle_upper_wedge.png" width="80" /><br>`snow_gentle_upper_wedge` | <img src="images/blocks/stone_gentle_upper_wedge.png" width="80" /><br>`stone_gentle_upper_wedge` | <img src="images/blocks/dark_stone_gentle_upper_wedge.png" width="80" /><br>`dark_stone_gentle_upper_wedge` |
| **Gentle Lower Ridge** | <img src="images/blocks/form_gentle_lower_ridge.png" width="80" /><br>`form_gentle_lower_ridge` | <img src="images/blocks/dirt_gentle_lower_ridge.png" width="80" /><br>`dirt_gentle_lower_ridge` | <img src="images/blocks/sand_gentle_lower_ridge.png" width="80" /><br>`sand_gentle_lower_ridge` | <img src="images/blocks/snow_gentle_lower_ridge.png" width="80" /><br>`snow_gentle_lower_ridge` | <img src="images/blocks/stone_gentle_lower_ridge.png" width="80" /><br>`stone_gentle_lower_ridge` | <img src="images/blocks/dark_stone_gentle_lower_ridge.png" width="80" /><br>`dark_stone_gentle_lower_ridge` |
| **Gentle Upper Ridge** | <img src="images/blocks/form_gentle_upper_ridge.png" width="80" /><br>`form_gentle_upper_ridge` | <img src="images/blocks/dirt_gentle_upper_ridge.png" width="80" /><br>`dirt_gentle_upper_ridge` | <img src="images/blocks/sand_gentle_upper_ridge.png" width="80" /><br>`sand_gentle_upper_ridge` | <img src="images/blocks/snow_gentle_upper_ridge.png" width="80" /><br>`snow_gentle_upper_ridge` | <img src="images/blocks/stone_gentle_upper_ridge.png" width="80" /><br>`stone_gentle_upper_ridge` | <img src="images/blocks/dark_stone_gentle_upper_ridge.png" width="80" /><br>`dark_stone_gentle_upper_ridge` |
| **Gentle Lower Valley** | <img src="images/blocks/form_gentle_lower_valley.png" width="80" /><br>`form_gentle_lower_valley` | <img src="images/blocks/dirt_gentle_lower_valley.png" width="80" /><br>`dirt_gentle_lower_valley` | <img src="images/blocks/sand_gentle_lower_valley.png" width="80" /><br>`sand_gentle_lower_valley` | <img src="images/blocks/snow_gentle_lower_valley.png" width="80" /><br>`snow_gentle_lower_valley` | <img src="images/blocks/stone_gentle_lower_valley.png" width="80" /><br>`stone_gentle_lower_valley` | <img src="images/blocks/dark_stone_gentle_lower_valley.png" width="80" /><br>`dark_stone_gentle_lower_valley` |
| **Gentle Upper Valley** | <img src="images/blocks/form_gentle_upper_valley.png" width="80" /><br>`form_gentle_upper_valley` | <img src="images/blocks/dirt_gentle_upper_valley.png" width="80" /><br>`dirt_gentle_upper_valley` | <img src="images/blocks/sand_gentle_upper_valley.png" width="80" /><br>`sand_gentle_upper_valley` | <img src="images/blocks/snow_gentle_upper_valley.png" width="80" /><br>`snow_gentle_upper_valley` | <img src="images/blocks/stone_gentle_upper_valley.png" width="80" /><br>`stone_gentle_upper_valley` | <img src="images/blocks/dark_stone_gentle_upper_valley.png" width="80" /><br>`dark_stone_gentle_upper_valley` |

---

## 4. Block Catalog & Snapshot CLI (`scripts/block_catalog.py`)

To inspect and list blocks or to render new documentation images after code changes, the CLI tool [`scripts/block_catalog.py`](file:///home/mario/code/low-cube/scripts/block_catalog.py) is available:

### Commands

- **List all available blocks**:
  ```bash
  ./scripts/block_catalog.py list
  ```
- **List all block shapes**:
  ```bash
  ./scripts/block_catalog.py list --forms
  ```
- **List all materials**:
  ```bash
  ./scripts/block_catalog.py list --materials
  ```
- **Render a snapshot of a specific block**:
  ```bash
  ./scripts/block_catalog.py render dirt_stair my_stair.png
  ```
- **Regenerate all 105 block images**:
  ```bash
  ./scripts/block_catalog.py render-all docs/images/blocks/
  ```
