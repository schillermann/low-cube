# Planetary Climate Zones & Biomes

In **Low-Cube**, freestanding cubic planets drift through the cosmos. Every planet is composed of **six distinct faces**, where each face hosts a dedicated **climate zone** ([`ClimateZone`](../crates/world/src/topology/mod.rs)) with its own procedural relief, geological strata, color palette, native vegetation, and survival challenges.

Gravity bends smoothly around 90-degree planetary edges, allowing players to traverse between wildly contrasting ecosystems within seconds simply by walking across the seam.

---

## 1. The Six Planetary Faces

| Face | Climate Zone | Day & Night Views | Terrain & Landforms | Native Flora | Primary Resources |
| :---: | :---: | :---: | :--- | :--- | :--- |
| **+Y**<br>Top | **Temperate** | <img src="images/climates/temperate-day.png" width="150" alt="Temperate Day" /><br><img src="images/climates/temperate-night.png" width="150" alt="Temperate Night" /> | Rolling loam hills, lush meadows, alpine horn peaks ([`VoxelMountain`](../crates/world/src/mountain.rs)). | Deciduous Oak ([`TemperateOak`](TREES.md#temperate-oak)) | Standard Lumber, Copper, Iron, Saplings, Apples |
| **-X**<br>West | **Arctic** | <img src="images/climates/arctic-day.png" width="150" alt="Arctic Day" /><br><img src="images/climates/arctic-night.png" width="150" alt="Arctic Night" /> | Glacial cirque basins, ice horns, crystalline snow ridges ([`GlacialMassif`](../crates/world/src/landscape/glacial.rs)), and nocturnal polar auroras (*Aurora Borealis*). | Alpine Pine ([`ArcticPine`](TREES.md#arctic-pine)) | Dense Hardwood, Ice, Pine Cones, Combustible Resin |
| **+X**<br>East | **Desert** | <img src="images/climates/desert-day.png" width="150" alt="Desert Day" /><br><img src="images/climates/desert-night.png" width="150" alt="Desert Night" /> | Barchan sand dunes, incised wadi canyon valleys, slip faces ([`DesertDunes`](../crates/world/src/landscape/dunes.rs)). | Joshua Tree ([`DesertSucculent`](TREES.md#desert-succulent)) | Sandstone, Soft Wood, Cactus Fibers, Plant Water |
| **+Z**<br>North | **Jungle** | <img src="images/climates/jungle-day.png" width="150" alt="Jungle Day" /><br><img src="images/climates/jungle-night.png" width="150" alt="Jungle Night" /> | Tower karst limestone pillars, winding tropical river gorges ([`JungleKarst`](../crates/world/src/landscape/karst.rs)). | Giant Emergent ([`JungleEmergent`](TREES.md#jungle-emergent)) | Tropical Hardwood, Lianas/Vines, Jungle Fruits |
| **-Z**<br>South | **Steppe** | <img src="images/climates/steppe-day.png" width="150" alt="Steppe Day" /><br><img src="images/climates/steppe-night.png" width="150" alt="Steppe Night" /> | Tiered mesa plateaus, stratified coulee ravines ([`SteppeTerrace`](../crates/world/src/landscape/steppe.rs)). | Umbrella Acacia ([`SteppeAcacia`](TREES.md#steppe-acacia)) | Tough Drywood, Savanna Clay, Acacia Pods, Bark Strips |
| **-Y**<br>Bottom | **Volcanic** | <img src="images/climates/volcanic-day.png" width="150" alt="Volcanic Day" /><br><img src="images/climates/volcanic-night.png" width="150" alt="Volcanic Night" /> | Sunken magma caldera basin, elevated basalt rim ([`VolcanicCaldera`](../crates/world/src/landscape/caldera.rs)). | Basalt Spire ([`VolcanicSpire`](TREES.md#volcanic-spire)) | Fireproof Basaltwood, Obsidian, Charcoal, Sulfur |

---

## 2. Deep Dive: Climate Zone Profiles

### Temperate (+Y Face)
| Day (Solar Firmament) | Night (Lunar & Stellar Vault) |
| :---: | :---: |
| ![Temperate Day](images/climates/temperate-day.png) | ![Temperate Night](images/climates/temperate-night.png) |

- **Atmosphere & Visuals**: Rich emerald green grasslands bathed in mild daylight, transitioning into rock shelves and snowy peaks. At night, crisp starlight and the cratered lunar sphere illuminate the woodland canopy.
- **Topography**: Balanced rolling terrain featuring standard blocks, gentle wedges, and the iconic alpine massif ([`VoxelMountain`](../crates/world/src/mountain.rs)).
- **Flora**: Deciduous [`TemperateOak`](TREES.md#temperate-oak) with faceted hexagonal trunks and lush icosahedral leaf clusters.
- **Survival Profile**: The ideal starting cradle. Abundant food drops, mild temperature, and accessible surface stone and ores make it forgiving for new settlers.

---

### Arctic (-X Face)
| Day (Solar Firmament) | Night (Lunar & Stellar Vault) |
| :---: | :---: |
| ![Arctic Day](images/climates/arctic-day.png) | ![Arctic Night](images/climates/arctic-night.png) |

- **Atmosphere & Visuals**: Chilling pale cyan and crystalline snow facets reflecting solar rays by day. Nocturnal twilight reveals high-contrast lunar maria, brilliant Tycho crater rays, and luminous polar auroras (*Aurora Borealis*) shimmering in undulating emerald green (`557.7 nm` atomic oxygen) and violet ribbons across the Arctic sky vault. The aurora canopy ([`AuroraCanopy`](../crates/client/src/aurora_canopy.rs)) is geometrically confined to the Arctic face boundaries, scaling with planetary dimensions while respecting calibrated minimum (`14.0 m`) and maximum (`96.0 m`) bounds.
- **Topography**: Rugged glacial amphitheaters and horn spires constructed by [`GlacialMassif`](../crates/world/src/landscape/glacial.rs).
- **Flora**: Frost-resistant [`ArcticPine`](TREES.md#arctic-pine) conifers featuring tiered skirts engineered to shed snow.
- **Survival Profile**: **Freezing hazard**. Prolonged exposure rapidly depletes body heat unless sheltered or carrying combustible resin lamps. Snow slopes offer fast downhill ski-surfing.

---

### Desert (+X Face)
| Day (Solar Firmament) | Night (Lunar & Stellar Vault) |
| :---: | :---: |
| ![Desert Day](images/climates/desert-day.png) | ![Desert Night](images/climates/desert-night.png) |

- **Atmosphere & Visuals**: Blistering golden ochre sands and sun-scorched bedrock under relentless sunlight. Desert nights bring sudden temperature drops with crystal-clear celestial visibility.
- **Topography**: Rolling barchan dune ridges and steep slip-face wedges shaped by [`DesertDunes`](../crates/world/src/landscape/dunes.rs).
- **Flora**: Segmented [`DesertSucculent`](TREES.md#desert-succulent) (Joshua tree / cactus) storing vital water reserves.
- **Survival Profile**: **Dehydration hazard**. Rapid thirst depletion in daytime. Water must be extracted by harvesting native succulents or tapping deep underground aquifers.

---

### Jungle (+Z Face)
| Day (Solar Firmament) | Night (Lunar & Stellar Vault) |
| :---: | :---: |
| ![Jungle Day](images/climates/jungle-day.png) | ![Jungle Night](images/climates/jungle-night.png) |

- **Atmosphere & Visuals**: Dense, shadowy tropical canopy filtering daytime solar radiance. Nightfall envelopes the towering limestone spires under a starry canopy.
- **Topography**: Tower karst limestone monoliths and steep river gorges generated by [`JungleKarst`](../crates/world/src/landscape/karst.rs).
- **Flora**: Giant buttressed [`JungleEmergent`](TREES.md#jungle-emergent) trees whose dense canopies provide physical aerodynamic drag ([`CanopyBraking`](../crates/client/src/phase/canopy.rs)) to soften falls.
- **Survival Profile**: **Navigation & verticality hazard**. Dense vegetation obscures sightlines and precipices. Navigating the high canopy offers fast arboreal travel, but slipping requires mastering canopy braking.

---

### Steppe (-Z Face)
| Day (Solar Firmament) | Night (Lunar & Stellar Vault) |
| :---: | :---: |
| ![Steppe Day](images/climates/steppe-day.png) | ![Steppe Night](images/climates/steppe-night.png) |

- **Atmosphere & Visuals**: Warm terracotta clay, dusty savannas, and wide-open horizons beneath pale skies, giving way to an expansive, unobstructed night sky with the full moon overhead.
- **Topography**: Multi-tiered horizontal mesas, terraced plateaus, and coulee ravines sculpted by [`SteppeTerrace`](../crates/world/src/landscape/steppe.rs).
- **Flora**: Characteristic [`SteppeAcacia`](TREES.md#steppe-acacia) with knobby zig-zag trunks and broad horizontal parasol discs.
- **Survival Profile**: **Predator exposure**. Wide vistas allow hostile nocturnal predators to spot players from great distances. Building defensible cliffside forts on top of mesa terraces is critical.

---

### Volcanic (-Y Face)
| Day (Solar Firmament) | Night (Lunar & Stellar Vault) |
| :---: | :---: |
| ![Volcanic Day](images/climates/volcanic-day.png) | ![Volcanic Night](images/climates/volcanic-night.png) |

- **Atmosphere & Visuals**: Pitch-black basalt crags, smoking fissures, and incandescent molten rock illumination. At night, the stark volcanic spires form sharp silhouettes against the starry cosmic vault and cratered moon.
- **Topography**: Colossal crater rim descending into a sunken magma basin floor created by [`VolcanicCaldera`](../crates/world/src/landscape/caldera.rs).
- **Flora**: Charred, leafless [`VolcanicSpire`](TREES.md#volcanic-spire) needles formed from crystallized obsidian and basalt.
- **Survival Profile**: **Extreme heat & lethal lava**. Ground-level lava pools incinerate equipment on contact. Only fireproof basaltwood and high-tier heat-resistant armor allow harvesting the rich sulfur and obsidian veins here.

---

## 3. Planetary Seams & Biome Transitions

Every cubic planet connects all 6 faces through **12 undirected edge seams**. In **Low-Cube**, edges are not invisible teleporters or artificial barriers:

```
    ┌───────────┐
    │  Jungle   │ (+Z)
┌───┼───────────┼───┬───────────┐
│Arc│ Temperate │Des│ Volcanic  │ (-Y, Antipode)
└───┼───────────┼───┴───────────┘
    │  Steppe   │ (-Z)
    └───────────┘
```

1. **Continuous 90° Normal Folding**: As the player strides across the seam, gravity smoothly folds 90 degrees ([`FoldedFace`](../crates/world/src/topology/folded_face.rs) and [`SeamBevel`](../crates/world/src/topology/bevel.rs)), transforming the wall into the floor underfoot.
2. **Watertight Color Transitions**: Beveled seam chamfers blend the distinct color palettes of neighboring biomes without cracks or light leaks.
3. **Pacing Across Incline Boundaries**: The kinematically calibrated [`InclinePace`](../crates/client/src/phase/incline.rs) system accurately penalizes uphill climbs and accelerates downhill descents across all faces.

---

## 4. Architectural Implementation

Climate rendering adheres strictly to the **Pure Object-Oriented Programming (Elegant Objects)** architecture:
- [`ClimateZone`](../crates/world/src/topology/mod.rs): Declarative enumeration of the six planetary faces.
- [`ReliefPalette`](../crates/world/src/relief/palette.rs): Encapsulated color derivation using Tell-Don't-Ask inspector closures.
- [`FacetedReliefFace`](../crates/world/src/relief/face.rs): Procedural fractal relief decorator generating elevation and subterranean cavern cavities.
- [`ForestedFace`](../crates/world/src/tree/forested_face.rs): Composite decorator scattering climate-appropriate tree species across receptive ground.
- [`BiomedCubeWorld`](../crates/world/src/biome.rs): Canonical factory assembling all 6 faces with beveled seam normals into a cohesive planetary sphere-cube.
- [`AuroraCanopy`](../crates/client/src/aurora_canopy.rs): Pure OOP trait abstracting the atmospheric polar aurora canopy.
- [`CalibratedCanopy`](../crates/client/src/aurora_canopy.rs): Atmospheric canopy entity encapsulating Arctic face boundaries, altitude ceiling, and planetary min/max radius calibration.
