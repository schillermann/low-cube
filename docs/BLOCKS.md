# Block-Katalog & Geometrie-Referenz

In **Low-Cube** ist das Block-System nach den Prinzipien des **Pure Object-Oriented Programming (OOP)** und **Elegant Objects** aufgebaut:
Geometrische Formgebung ([`FormBlock`](file:///home/mario/code/low-cube/crates/world/src/block/form.rs)) und Materialbeschaffenheit ([`MaterialBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/material_block.rs)) sind als **Dekoratoren** voneinander entkoppelt.

Jede geometrische Form kann in jedem beliebigen Material dargestellt werden. Jeder Block besitzt eine eindeutige ID nach der Konvention:
```text
<material_id>_<shape_id>
```
Beispiele: `dirt_stair`, `stone_wedge`, `sand_gentle_lower_wedge`, `dark_stone_cube`.

---

## 1. Verfügbare Block-Formen (15 Formen)

| Form-ID | Bezeichnung (DE / EN) | Höhe / Steigung | Kletterphysik (Erde, $g=10$) | Bild |
|---|---|---|---|:---:|
| `cube` | **Vollblock**<br>*Full Cube* | $1.0\,\text{m}$ (vertikal 90°) | Unüberwindbare Wand; erfordert Sprung bei Niedrig-G oder Rampen/Treppen. | <img src="images/blocks/form_cube.png" width="96" alt="Vollblock" /> |
| `slab` | **Halbstufe**<br>*Half Slab* | $0.5\,\text{m}$ (Plateau) | Direkt überwindbar (menschenmögliche Schritthöhe bei Standard-Gravitation). | <img src="images/blocks/form_slab.png" width="96" alt="Halbstufe" /> |
| `wedge` | **Rampe (45°)**<br>*Ramp Wedge* | $0.0 \to 1.0\,\text{m}$ (1:1 Incline) | Stufenlos begehbar ohne Sprungimpuls. | <img src="images/blocks/form_wedge.png" width="96" alt="Rampe" /> |
| `ridge_corner` | **Grat-Außenecke (45°)**<br>*Ridge Corner* | $0.0 \to 1.0\,\text{m}$ (Diagonalgrat) | Konvexe Außenecke zur Verbindung zweier 45°-Rampen. | <img src="images/blocks/form_ridge_corner.png" width="96" alt="Grat-Außenecke" /> |
| `valley_corner` | **Kehl-Innenecke (45°)**<br>*Valley Corner* | $0.0 \to 1.0\,\text{m}$ (Diagonalkehle) | Konkave Innenecke zur Verbindung zweier 45°-Rampen. | <img src="images/blocks/form_valley_corner.png" width="96" alt="Kehl-Innenecke" /> |
| `peak` | **Pyramidenspitze**<br>*Pyramid Peak* | $0.5\,\text{m}$ (45°-Spitze) | 4-seitige Pyramidenspitze als natürlicher Gipfelpunkt. | <img src="images/blocks/form_peak.png" width="96" alt="Pyramidenspitze" /> |
| `stair` | **Treppe**<br>*Stair Flight* | $2 \times 0.5\,\text{m}$ Stufen | Bequemer 2-Stufen-Aufstieg auf volle Blockhöhe. | <img src="images/blocks/form_stair.png" width="96" alt="Treppe" /> |
| `stair_corner_convex` | **Treppen-Außenecke**<br>*Convex Stair Corner* | $2 \times 0.5\,\text{m}$ Stufen | 90°-Außenkurve für Treppenaufgänge. | <img src="images/blocks/form_stair_corner_convex.png" width="96" alt="Treppen-Außenecke" /> |
| `stair_corner_concave` | **Treppen-Innenecke**<br>*Concave Stair Corner* | $2 \times 0.5\,\text{m}$ Stufen | 90°-Innenkurve für verwinkelte Treppenanlagen. | <img src="images/blocks/form_stair_corner_concave.png" width="96" alt="Treppen-Innenecke" /> |
| `gentle_lower_wedge` | **Sanfte Rampe Unten**<br>*Gentle Lower Wedge* | $0.0 \to 0.5\,\text{m}$ (2:1 Incline) | Sanfter flacher Aufstieg zur ersten Hochebene oder Halbstufe. | <img src="images/blocks/form_gentle_lower_wedge.png" width="96" alt="Sanfte Rampe Unten" /> |
| `gentle_upper_wedge` | **Sanfte Rampe Oben**<br>*Gentle Upper Wedge* | $0.5 \to 1.0\,\text{m}$ (2:1 Incline) | Fortführung der sanften Rampe auf volle Blockhöhe. | <img src="images/blocks/form_gentle_upper_wedge.png" width="96" alt="Sanfte Rampe Oben" /> |
| `gentle_lower_ridge` | **Sanfte Grat-Außenecke Unten**<br>*Gentle Lower Ridge* | $0.0 \to 0.5\,\text{m}$ (2:1 Incline) | Flache Außenecke für sanfte Geländekämme. | <img src="images/blocks/form_gentle_lower_ridge.png" width="96" alt="Sanfte Grat-Außenecke Unten" /> |
| `gentle_upper_ridge` | **Sanfte Grat-Außenecke Oben**<br>*Gentle Upper Ridge* | $0.5 \to 1.0\,\text{m}$ (2:1 Incline) | Obere Außenecke für sanfte Geländekämme. | <img src="images/blocks/form_gentle_upper_ridge.png" width="96" alt="Sanfte Grat-Außenecke Oben" /> |
| `gentle_lower_valley` | **Sanfte Kehl-Innenecke Unten**<br>*Gentle Lower Valley* | $0.0 \to 0.5\,\text{m}$ (2:1 Incline) | Flache Innenecke für sanfte Talsenken. | <img src="images/blocks/form_gentle_lower_valley.png" width="96" alt="Sanfte Kehl-Innenecke Unten" /> |
| `gentle_upper_valley` | **Sanfte Kehl-Innenecke Oben**<br>*Gentle Upper Valley* | $0.5 \to 1.0\,\text{m}$ (2:1 Incline) | Obere Innenecke für sanfte Talsenken. | <img src="images/blocks/form_gentle_upper_valley.png" width="96" alt="Sanfte Kehl-Innenecke Oben" /> |

---

## 2. Verfügbare Materialien

| Material-ID | Bezeichnung | RGB-Farbwert | Rust-Entität | Vorkommen / Verwendung |
|---|---|---|---|---|
| `form` | **Formblock (Schablone)** | `Vec3(1.0, 1.0, 1.0)` | [`FormBlock`](file:///home/mario/code/low-cube/crates/world/src/block/form.rs) | Neutrale Bauschablone / Geometrischer Prototyp. |
| `dirt` | **Erde (Loam)** | `Vec3(0.48, 0.35, 0.22)` | [`DirtBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/dirt.rs) | Fruchtbarer Ackerboden, gemäßigte Täler. |
| `sand` | **Sand** | `Vec3(0.85, 0.75, 0.35)` | [`SandBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/sand.rs) | Warmer Wüstensand, Dünenlandschaften. |
| `snow` | **Schnee** | `Vec3(0.92, 0.95, 0.98)` | [`SnowBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/snow.rs) | Kristalliner Alpinschnee, Polarkappen. |
| `stone` | **Stein (Granit)** | `Vec3(0.48, 0.50, 0.46)` | [`StoneBlock::natural`](file:///home/mario/code/low-cube/crates/world/src/block/material/stone.rs) | Alpiner Fels, Bergmassive. |
| `dark_stone` | **Dunkler Stein (Schiefer)** | `Vec3(0.35, 0.42, 0.35)` | [`StoneBlock::dark`](file:///home/mario/code/low-cube/crates/world/src/block/material/stone.rs) | Dunkles Vulkangestein, Kraterränder. |
| `lava` | **Lava / Magma** | `Vec3(0.95, 0.28, 0.05)` | [`MaterialBlock`](file:///home/mario/code/low-cube/crates/world/src/block/material/material_block.rs) | Glühende Magmabecken in Calderas. |

---

## 3. Vollständige Block-Matrix (Materialien $\times$ Formen)

Jedes Feld zeigt das gerenderte Einzelbild des entsprechenden Blocks inklusive seiner eindeutigen Block-ID zur namentlichen Referenzierung:

### Basis- und Stufenblöcke

| Form | Schablone (`form_*`) | Erde (`dirt_*`) | Sand (`sand_*`) | Schnee (`snow_*`) | Stein (`stone_*`) | Dunkler Stein (`dark_stone_*`) |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Vollblock** | <img src="images/blocks/form_cube.png" width="80" /><br>`form_cube` | <img src="images/blocks/dirt_cube.png" width="80" /><br>`dirt_cube` | <img src="images/blocks/sand_cube.png" width="80" /><br>`sand_cube` | <img src="images/blocks/snow_cube.png" width="80" /><br>`snow_cube` | <img src="images/blocks/stone_cube.png" width="80" /><br>`stone_cube` | <img src="images/blocks/dark_stone_cube.png" width="80" /><br>`dark_stone_cube` |
| **Halbstufe** | <img src="images/blocks/form_slab.png" width="80" /><br>`form_slab` | <img src="images/blocks/dirt_slab.png" width="80" /><br>`dirt_slab` | <img src="images/blocks/sand_slab.png" width="80" /><br>`sand_slab` | <img src="images/blocks/snow_slab.png" width="80" /><br>`snow_slab` | <img src="images/blocks/stone_slab.png" width="80" /><br>`stone_slab` | <img src="images/blocks/dark_stone_slab.png" width="80" /><br>`dark_stone_slab` |
| **Treppe** | <img src="images/blocks/form_stair.png" width="80" /><br>`form_stair` | <img src="images/blocks/dirt_stair.png" width="80" /><br>`dirt_stair` | <img src="images/blocks/sand_stair.png" width="80" /><br>`sand_stair` | <img src="images/blocks/snow_stair.png" width="80" /><br>`snow_stair` | <img src="images/blocks/stone_stair.png" width="80" /><br>`stone_stair` | <img src="images/blocks/dark_stone_stair.png" width="80" /><br>`dark_stone_stair` |
| **Treppen-Außenecke** | <img src="images/blocks/form_stair_corner_convex.png" width="80" /><br>`form_stair_corner_convex` | <img src="images/blocks/dirt_stair_corner_convex.png" width="80" /><br>`dirt_stair_corner_convex` | <img src="images/blocks/sand_stair_corner_convex.png" width="80" /><br>`sand_stair_corner_convex` | <img src="images/blocks/snow_stair_corner_convex.png" width="80" /><br>`snow_stair_corner_convex` | <img src="images/blocks/stone_stair_corner_convex.png" width="80" /><br>`stone_stair_corner_convex` | <img src="images/blocks/dark_stone_stair_corner_convex.png" width="80" /><br>`dark_stone_stair_corner_convex` |
| **Treppen-Innenecke** | <img src="images/blocks/form_stair_corner_concave.png" width="80" /><br>`form_stair_corner_concave` | <img src="images/blocks/dirt_stair_corner_concave.png" width="80" /><br>`dirt_stair_corner_concave` | <img src="images/blocks/sand_stair_corner_concave.png" width="80" /><br>`sand_stair_corner_concave` | <img src="images/blocks/snow_stair_corner_concave.png" width="80" /><br>`snow_stair_corner_concave` | <img src="images/blocks/stone_stair_corner_concave.png" width="80" /><br>`stone_stair_corner_concave` | <img src="images/blocks/dark_stone_stair_corner_concave.png" width="80" /><br>`dark_stone_stair_corner_concave` |

---

### Standard-Rampen & Ecken (45°, 1:1)

| Form | Schablone (`form_*`) | Erde (`dirt_*`) | Sand (`sand_*`) | Schnee (`snow_*`) | Stein (`stone_*`) | Dunkler Stein (`dark_stone_*`) |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Rampe (45°)** | <img src="images/blocks/form_wedge.png" width="80" /><br>`form_wedge` | <img src="images/blocks/dirt_wedge.png" width="80" /><br>`dirt_wedge` | <img src="images/blocks/sand_wedge.png" width="80" /><br>`sand_wedge` | <img src="images/blocks/snow_wedge.png" width="80" /><br>`snow_wedge` | <img src="images/blocks/stone_wedge.png" width="80" /><br>`stone_wedge` | <img src="images/blocks/dark_stone_wedge.png" width="80" /><br>`dark_stone_wedge` |
| **Grat-Außenecke** | <img src="images/blocks/form_ridge_corner.png" width="80" /><br>`form_ridge_corner` | <img src="images/blocks/dirt_ridge_corner.png" width="80" /><br>`dirt_ridge_corner` | <img src="images/blocks/sand_ridge_corner.png" width="80" /><br>`sand_ridge_corner` | <img src="images/blocks/snow_ridge_corner.png" width="80" /><br>`snow_ridge_corner` | <img src="images/blocks/stone_ridge_corner.png" width="80" /><br>`stone_ridge_corner` | <img src="images/blocks/dark_stone_ridge_corner.png" width="80" /><br>`dark_stone_ridge_corner` |
| **Kehl-Innenecke** | <img src="images/blocks/form_valley_corner.png" width="80" /><br>`form_valley_corner` | <img src="images/blocks/dirt_valley_corner.png" width="80" /><br>`dirt_valley_corner` | <img src="images/blocks/sand_valley_corner.png" width="80" /><br>`sand_valley_corner` | <img src="images/blocks/snow_valley_corner.png" width="80" /><br>`snow_valley_corner` | <img src="images/blocks/stone_valley_corner.png" width="80" /><br>`stone_valley_corner` | <img src="images/blocks/dark_stone_valley_corner.png" width="80" /><br>`dark_stone_valley_corner` |
| **Pyramidenspitze** | <img src="images/blocks/form_peak.png" width="80" /><br>`form_peak` | <img src="images/blocks/dirt_peak.png" width="80" /><br>`dirt_peak` | <img src="images/blocks/sand_peak.png" width="80" /><br>`sand_peak` | <img src="images/blocks/snow_peak.png" width="80" /><br>`snow_peak` | <img src="images/blocks/stone_peak.png" width="80" /><br>`stone_peak` | <img src="images/blocks/dark_stone_peak.png" width="80" /><br>`dark_stone_peak` |

---

### Sanfte Rampen & Ecken (2:1 Steigung)

| Form | Schablone (`form_*`) | Erde (`dirt_*`) | Sand (`sand_*`) | Schnee (`snow_*`) | Stein (`stone_*`) | Dunkler Stein (`dark_stone_*`) |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Sanfte Rampe Unten** | <img src="images/blocks/form_gentle_lower_wedge.png" width="80" /><br>`form_gentle_lower_wedge` | <img src="images/blocks/dirt_gentle_lower_wedge.png" width="80" /><br>`dirt_gentle_lower_wedge` | <img src="images/blocks/sand_gentle_lower_wedge.png" width="80" /><br>`sand_gentle_lower_wedge` | <img src="images/blocks/snow_gentle_lower_wedge.png" width="80" /><br>`snow_gentle_lower_wedge` | <img src="images/blocks/stone_gentle_lower_wedge.png" width="80" /><br>`stone_gentle_lower_wedge` | <img src="images/blocks/dark_stone_gentle_lower_wedge.png" width="80" /><br>`dark_stone_gentle_lower_wedge` |
| **Sanfte Rampe Oben** | <img src="images/blocks/form_gentle_upper_wedge.png" width="80" /><br>`form_gentle_upper_wedge` | <img src="images/blocks/dirt_gentle_upper_wedge.png" width="80" /><br>`dirt_gentle_upper_wedge` | <img src="images/blocks/sand_gentle_upper_wedge.png" width="80" /><br>`sand_gentle_upper_wedge` | <img src="images/blocks/snow_gentle_upper_wedge.png" width="80" /><br>`snow_gentle_upper_wedge` | <img src="images/blocks/stone_gentle_upper_wedge.png" width="80" /><br>`stone_gentle_upper_wedge` | <img src="images/blocks/dark_stone_gentle_upper_wedge.png" width="80" /><br>`dark_stone_gentle_upper_wedge` |
| **Sanfter Grat Unten** | <img src="images/blocks/form_gentle_lower_ridge.png" width="80" /><br>`form_gentle_lower_ridge` | <img src="images/blocks/dirt_gentle_lower_ridge.png" width="80" /><br>`dirt_gentle_lower_ridge` | <img src="images/blocks/sand_gentle_lower_ridge.png" width="80" /><br>`sand_gentle_lower_ridge` | <img src="images/blocks/snow_gentle_lower_ridge.png" width="80" /><br>`snow_gentle_lower_ridge` | <img src="images/blocks/stone_gentle_lower_ridge.png" width="80" /><br>`stone_gentle_lower_ridge` | <img src="images/blocks/dark_stone_gentle_lower_ridge.png" width="80" /><br>`dark_stone_gentle_lower_ridge` |
| **Sanfter Grat Oben** | <img src="images/blocks/form_gentle_upper_ridge.png" width="80" /><br>`form_gentle_upper_ridge` | <img src="images/blocks/dirt_gentle_upper_ridge.png" width="80" /><br>`dirt_gentle_upper_ridge` | <img src="images/blocks/sand_gentle_upper_ridge.png" width="80" /><br>`sand_gentle_upper_ridge` | <img src="images/blocks/snow_gentle_upper_ridge.png" width="80" /><br>`snow_gentle_upper_ridge` | <img src="images/blocks/stone_gentle_upper_ridge.png" width="80" /><br>`stone_gentle_upper_ridge` | <img src="images/blocks/dark_stone_gentle_upper_ridge.png" width="80" /><br>`dark_stone_gentle_upper_ridge` |
| **Sanfte Kehle Unten** | <img src="images/blocks/form_gentle_lower_valley.png" width="80" /><br>`form_gentle_lower_valley` | <img src="images/blocks/dirt_gentle_lower_valley.png" width="80" /><br>`dirt_gentle_lower_valley` | <img src="images/blocks/sand_gentle_lower_valley.png" width="80" /><br>`sand_gentle_lower_valley` | <img src="images/blocks/snow_gentle_lower_valley.png" width="80" /><br>`snow_gentle_lower_valley` | <img src="images/blocks/stone_gentle_lower_valley.png" width="80" /><br>`stone_gentle_lower_valley` | <img src="images/blocks/dark_stone_gentle_lower_valley.png" width="80" /><br>`dark_stone_gentle_lower_valley` |
| **Sanfte Kehle Oben** | <img src="images/blocks/form_gentle_upper_valley.png" width="80" /><br>`form_gentle_upper_valley` | <img src="images/blocks/dirt_gentle_upper_valley.png" width="80" /><br>`dirt_gentle_upper_valley` | <img src="images/blocks/sand_gentle_upper_valley.png" width="80" /><br>`sand_gentle_upper_valley` | <img src="images/blocks/snow_gentle_upper_valley.png" width="80" /><br>`snow_gentle_upper_valley` | <img src="images/blocks/stone_gentle_upper_valley.png" width="80" /><br>`stone_gentle_upper_valley` | <img src="images/blocks/dark_stone_gentle_upper_valley.png" width="80" /><br>`dark_stone_gentle_upper_valley` |

---

## 4. Block-Katalog & Snapshot CLI (`scripts/block_catalog.py`)

Um neue Blöcke zu inspizieren, aufzulisten oder bei Code-Erweiterungen neue Dokumentationsbilder zu rendern, steht das CLI-Werkzeug [`scripts/block_catalog.py`](file:///home/mario/code/low-cube/scripts/block_catalog.py) bereit:

### Befehle

- **Alle verfügbaren Blöcke auflisten**:
  ```bash
  ./scripts/block_catalog.py list
  ```
- **Alle Block-Formen auflisten**:
  ```bash
  ./scripts/block_catalog.py list --forms
  ```
- **Alle Materialien auflisten**:
  ```bash
  ./scripts/block_catalog.py list --materials
  ```
- **Einzelbild eines beliebigen Blocks erstellen**:
  ```bash
  ./scripts/block_catalog.py render dirt_stair my_stair.png
  ```
- **Alle 105 Blockbilder neu generieren**:
  ```bash
  ./scripts/block_catalog.py render-all docs/images/blocks/
  ```

---

## 5. Programmier-Referenz (Rust Code-Beispiele)

### Reinen Formblock instanziieren
```rust
use world::{FormBlock, WedgeHeading, CornerHeading};

let cube = FormBlock::cube(grid_u, grid_v, base_elevation);
let ramp = FormBlock::wedge(grid_u, grid_v, base_elevation, WedgeHeading::North);
let gentle = FormBlock::gentle_lower_wedge(grid_u, grid_v, base_elevation, WedgeHeading::East);
```

### Typisierten Materialblock instanziieren
```rust
use world::{DirtBlock, SandBlock, SnowBlock, StoneBlock, WedgeHeading};

let dirt_stair = DirtBlock::stair(grid_u, grid_v, 0.0, WedgeHeading::North);
let sand_dune = SandBlock::gentle_lower_wedge(grid_u, grid_v, 0.0, WedgeHeading::West);
let granite_peak = StoneBlock::peak(grid_u, grid_v, 0.0);
let shale_ridge = StoneBlock::dark_ridge_corner(grid_u, grid_v, 0.0, CornerHeading::NorthEast);
```

### Beliebiges Material mit beliebiger Form dekorieren
```rust
use glam::Vec3;
use world::{FormBlock, MaterialBlock};

let custom_color = Vec3::new(0.95, 0.28, 0.05); // Lava Orange-Rot
let custom_form = FormBlock::slab(grid_u, grid_v, 0.0);
let lava_slab = MaterialBlock::new(Box::new(custom_form), custom_color);
```
