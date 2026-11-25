# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
and [Conventional Commits](https://www.conventionalcommits.org/).

## [Unreleased]

## [0.121.0] - 2026-11-25

### Added
- Added emergent Hard-Sci-Fi sandbox decorators (`Paket 5` — 100% horizon-local with zero global broadcasts):
  - `CargoBallast<S>` (`ImpulseStage` decorator in `crates/world/src/relief/vehicle_assembly/impulse/stages.rs` and `RigidImpulse::with_cargo_ballast` in `impulse/mod.rs`), physical mineral hull densities (`GoldOre > IronOre > CopperOre > QuarriedStone` in `part/propulsion.rs`), `SatchelBurden::ballast_tonnage` (`crates/client/src/attire/burden/mod.rs`), and `TransferArc::sample_flip_burn_with_accel` (`crates/client/src/drive/transfer_arc/mod.rs` and `RocketDrive`): scales spacecraft `a = F / m` acceleration, principal rotational inertia, `SolarChartModal` Brachistochrone plotting, and `FlipBurnReadout` turnaround warnings dynamically with hull ore blocks and pilot satchel cargo (`crates/client/src/drive/hard_scifi_sandbox_tests.rs`).
  - `ChatMedium` decorator chain (`SuitTransceiverMedium(AtmosphericVoiceMedium(CabinIntercomMedium(HelmetContactMedium)))` in `crates/sanctuary/src/chat_medium.rs`, `CosmicFrame::atmospheric_sound` in `crates/world/src/celestial/bubble.rs`, and `ProximityUtterance::radio_silent` in `crates/world/src/presence/utterance.rs`, wired into `CohortPulse::relay_utterance` and `HostAura::receive_utterance`): carries open voice up to `96 m` on atmosphere-bearing worlds (`Earth`, `Mars`, `Venus`, `Titan`), `0 m` open sound in airless vacuum (`Luna`, `BeltAsteroid`, `DeepSpace`), `<= 32 m` EVA suit VHF radio unless silenced with `~`, `<= 1.85 m` visor-to-visor helmet conduction under total radio silence, and `240 km/s` pressurized cabin intercom across shared `VesselBerth::Aboard` hulls (`crates/sanctuary/src/tests/vacuum_chat.rs`).
  - `BlockMedium::VitrifiedBasalt` / `VitrifiedBasaltMesh` (`code 24`, `HeldRelic::VitrifiedBasalt = 34`), `BlockMedium::BlackBoxPod` / `BlackBoxPodMesh` (`code 25`, `HeldRelic::BlackBoxPod = 35` with single-click `instant_pickup`, `ArtisanTome` recipe `#23` in `crates/world/src/survivor/catalog.rs` & `crates/client/src/attire/tome.rs`), and `FlightResidue` (`crates/client/src/drive/rocket/residue.rs`): vitrifies planetary launchpads beneath active `FusionTorch` plumes into obsidian-glassy basalt with glowing cooling fissures, ejects salvageable safety-orange `BlackBoxPod` capsules upon severe spacecraft impacts (`v_norm < -26 m/s`), and unfolds the tilted ruled `IncidentSheet` telemetry log on `WheelClick` in hand.

### Changed
- Consolidated solar-system scaling directly onto `[world] size = 1..=32` plus optional `asteroid_density` (`COSMOS_PLAN.md` Phase 2), removing independent distortion overrides (`body_scale`, `sun_scale`, `distance_scale`, `[cosmos] scale`, `--*-scale`, `BodyRatios`, and `CosmosOverrides`) across `SolarProportions` (`crates/world/src/celestial/proportions.rs`), `BeltDensity` (`crates/client/src/cosmos_scale.rs`), `ServerCosmosScale` (`crates/sanctuary/src/config/cosmos.rs`), `server.toml`, and `docs/SETTINGS.md`.

## [0.120.0] - 2026-11-25

### Added
- Added raycast-free `SectorSnapshot` cell-state WAL compaction and multi-core `ClimateZone` / `PulseInbox` sharding (`Paket 4`):
  - `SectorSnapshot`, `SectorColumnState`, `SnapshotBlock`, and `RigSnapshot` (`crates/world/src/chronicle/snapshot.rs`, wire tag `16` on `WorldFact::SectorSnapshot`), `NaturalOverhang` (`crates/world/src/relief/molded_structure/overhang.rs`), and `SectorMoldsView` / `SectorMoldsTarget` (`crates/world/src/relief/overhang_molds/snapshot.rs`): captures and applies net `4 m × 4 m` (`8 × 8` `GridCell`) `ShaftExcavation` deltas, `64`-bit `TurfScars` bare-soil masks, air-exposed `SubterraneanVeins` mineral seams (preserving Zero-Knowledge X-Ray anti-cheat), `WorldBlocks` (all sculpted `MoldArchetype`s, 8-way `FieldCamera` tripod angles, living `TurfCoat` sod, and undercut natural slope crowns), and `CantileverRig` cranes with suspended `CraftCluster`s in `O(cells)` with **zero `SurfaceSightRay` raycasts**.
  - `SnapshottedChronicle<S>` (`crates/sanctuary/src/snapshot_chronicle.rs`), `ChronicleEntry` (`Superseded` vs. `Active` in `crates/sanctuary/src/chronicle_log.rs`), atomic startup WAL compaction in `DiskVaultChronicle::replay_and_open` (`crates/sanctuary/src/chronicle_decorators.rs` and `vault_stream.rs`), and `SpatialFactIndex::prune_superseded` + `HorizonStream` unacked-tail pruning (`crates/sanctuary/src/spatial_chronicle/`): folds hot `3 × 3` sector neighborhoods into compact `SectorSnapshot`s live and rewrites SSD WAL journals atomically on server restart while rejecting untrusted guest `SectorSnapshot` petitions and withholding premature `Temperate` stream gathering before a guest's first live `GuestPulse` (`crates/sanctuary/src/tests/sector_sharding.rs`).
  - `ZoneShards` (`crates/world/src/planet/shards.rs`, wired into `CubePlanet`, `HeliosphereWorld`, `OffworldVault::step_warm_bodies`, `SanctuaryHost::step_active_surface`, and `HostedPlanet`) plus `PulseInbox` with `ParallelInbox(SequentialInbox)` (`crates/sanctuary/src/inbox_batch.rs`): keeps uninhabited undisturbed `ClimateZone`s at `0 %` CPU (`Dormant`), steps multiple occupied/regrowing `ClimateZone` faces and warm off-world bodies concurrently across CPU cores via `std::thread::scope`, and decrypts inbound ChaCha20-Poly1305 companion packet bursts in parallel across CPU cores in deterministic arrival order.

## [0.119.0] - 2026-11-24

### Added
- Added Expanse-inspired `TitaniumHull` and `CarbonSilicate` spacecraft hull materials, ultralight keel mass scaling, high-G Epstein burn authority, and ceramic atmospheric entry aerobraking:
  - `BlockMedium::TitaniumHull` & `BlockMedium::CarbonSilicate` (`crates/world/src/block/timber_grain/`), `AerospacePlating` (`crates/world/src/block/aerospace_hull.rs`), `HeldRelic::TitaniumHull` & `HeldRelic::CarbonSilicate` (`crates/client/src/attire/relic/`), `ArtisanTome` (`2× IronOre + 1× CrustBlock -> 4× TitaniumHull`, `1× GoldOre + 2× FlintStone -> 4× CarbonSilicate`), `RelicCatalog` (`RELIC_SLOT_COUNT = 34`), and 6-bit dual-palm wire packing in `YouthAttire::multiplayer_bytes` & `SatchelManifest::reconcile_from_presence`: sculpts riveted matte titanium keel plates and staggered stealth ceramic tiles over glowing cyan-blue carbon-silicate lace across all 16 `MoldArchetype` forms with zero extra network wire bytes.
  - `PartPropulsion::part_mass` (`crates/world/src/relief/vehicle_assembly/part/propulsion.rs`), `CraftBlueprint::aerospace_framed` / `thermal_shielded`, `RigidImpulse::keel_fusion_velocity_delta` (`crates/world/src/relief/vehicle_assembly/impulse/mod.rs`), `SurfaceContact::resolve_gas` (`crates/world/src/surface_contact.rs`), and `AtmosphericEntryStep` (`crates/client/src/drive/reentry_step.rs`, wired into `RocketDrive`): reduces aerospace hull dry mass (`0.12` titanium, `0.08` carbon-silicate vs. `0.35` timber/earth), unlocks ~11G (`108 m/s²`) sustained Epstein burns and faster `HelmChart` Brachistochrone transfers on aerospace-framed hulls, and aerobrakes smoothly without rotational buffeting during steep planetary reentry and gas-envelope flight (`crates/client/src/drive/expanse_hull_tests.rs`).

## [0.118.0] - 2026-11-24

### Added
- Added two-tier `FlintPickaxe` & `IronPickaxe` mining progression and solid rock/ore hardness guards:
  - `MiningEdge` (`BarePalm`, `FlintPick`, `IronPick` in `crates/world/src/block/timber_grain/mining_edge.rs`), `BlockMedium::yields_to`, `ReliefElevation::layer_yields_to`, `Face::surface_yields_to`, and `WorldEarthworks::surface_yields_to` across planetary, celestial, woodland, and Sanctuary world decorators: prevents bare hands and `IronSpade` from cracking or harvesting solid rock and ores (`QuarriedStone`, `CrustRock`, `CopperOre`, `IronOre`, `GoldOre`) while keeping `TurfEarth`, `Sand`, and placed timber/craft blocks diggable without a pickaxe.
  - `HeldRelic::FlintPickaxe` (`1× DeadwoodBough + 3× FlintStone` in `ArtisanTome` and `RelicCatalog`, cleaving `QuarriedStone`, `CopperOre`, and `IronOre` in 3 strikes) and `HeldRelic::IronPickaxe` (`1× PlankBlock + 2× IronOre`, cleaving deep `CrustRock` and `GoldOre` in 2 strikes), complete with 3D `ToolSculpture` palm/showcase meshes (`crates/client/src/attire/relic/tools.rs`), `GroundToolFigure` floor litter (`SurfaceFind::FlintPickaxe` & `SurfaceFind::IronPickaxe`), 5-bit multiplayer attire sync, and `SatchelManifest` survivor persistence (`crates/client/src/pickaxe_tests.rs`).

## [0.117.0] - 2026-11-24

### Added
- Added server-issued `PassportStamp` survivor identities, disambiguated `Callsign`s, human-readable server inventory rosters, and callsign-labeled proximity chat:
  - `Callsign` (`crates/world/src/presence/callsign.rs`), `ProximityUtterance::named` / `formatted_line` (`crates/world/src/presence/utterance.rs`), `SurvivorRecord::with_callsign` / `inscribe_roster_entry` (`crates/world/src/survivor/record.rs`), and `RelicCatalog::slug` / `SatchelManifest::inventory_summary` (`crates/world/src/survivor/`): models stack-allocated `<= 24 B` UTF-8 player callsigns (`Copy`), strips `#` from client submissions so only the server assigns `#2`/`#3` collision suffixes, and formats proximity chat lines (`Tom: ...`, `Tom#2: ...`, `HOST: ...`) via Tell-Don't-Ask (`crates/client/src/menu/chat_modal.rs`).
  - `PassportVault`, `DiskPassports(MemoryPassports)`, `FixedPassport`, `PassportStamp`, `PassportDocument`, `HexKey`, and `KnockPassport` (`crates/sanctuary/src/passport/`), wired into `ClientPassportBook` (`crates/client/src/portal_config.rs`, `--passports` / `--name` / `LOW_CUBE_NAME` / `passports.toml` / `character.toml`), `WelcomeFrame` (`crates/sanctuary/src/link_state/welcome.rs`), and `RemoteLink` (`crates/sanctuary/src/remote_link/`):
    - Requests a fresh identity on first join to any server (`KnockIntent::RequestIdentity`), receives the server-minted `32-byte` secret sealed inside `LWEL` via ChaCha20-Poly1305, persists isolated per-server passports in `passports.toml` (indexed by `host_key` and direct endpoint, including across 12-char `Wormhole` resolution), and proves possession via X25519 Diffie-Hellman on reconnect without transmitting cleartext secrets.
  - `SlotLedger`, `CohortAdmission`, and `IdentityGrant` (`crates/sanctuary/src/cohort/`), `WardCredential` (`crates/sanctuary/src/ward/`), `RosterSheet` (`crates/sanctuary/src/survivor_ledger/`), and `<journal>.host.key` persistence in `SanctuaryEndpoint::serve` (`crates/sanctuary/src/config/mod.rs`):
    - Mints idempotent server-issued identities in `GuestCohort::admit_knock`, disambiguates colliding or unnamed callsigns (`Tom` vs. `Tom#2`, `Survivor#1`), logs admissions/restorations on the server console, writes human-readable `<journal>.survivors.toml` inventory ledgers alongside `<journal>.survivors`, and keeps dedicated-server `LC1-...` tickets stable across restarts (`crates/sanctuary/src/tests/passport_identity.rs`).

## [0.116.0] - 2026-11-24

### Added
- Added server-authoritative `SurvivorStore` inventory & Hill-bubble position persistence bound to persistent X25519 player keys:
  - `SatchelManifest`, `SurvivorRecord`, `SurvivorWelcome`, and `RelicCatalog` (`crates/world/src/survivor/`), plus `WorldEarthworks::harvest_offworld`, `CompanionPortal::note_crafted_recipe` / `note_consumed_provision`, and `PlanetaryWorld::assigned_survivor` / `anchor_to_frame`: models all `30` `HeldRelic` item counts, equipped `(right, left)` palms, and `19` `ArtisanTome` crafting recipes as an authoritative domain value object (`crates/client/src/attire/tests/camera.rs`).
  - `SurvivorStore` with `DiskVaultSurvivors(MemorySurvivors)` (`crates/sanctuary/src/survivor_ledger.rs`), `HandshakeIdentity::load_or_create` (`crates/sanctuary/src/seal.rs`), `SurvivorKeyFile` (`crates/client/src/portal_config.rs`, `--identity` / `LOW_CUBE_IDENTITY` / `survivor.key`), `InventoryNotice` (`crates/sanctuary/src/envelope.rs`), `WelcomeArrival` (`crates/sanctuary/src/link_state/arrival.rs`), and `<journal>.survivors` WAL replay in `SanctuaryPlan` (`crates/sanctuary/src/config/mod.rs`):
    - Overwrites client-local `server.toml` `[inventory]` kits upon joining (`EmbodiedAvatar::land_from_world`, `BeltSatchel::from_survivor_manifest`) with the server's `[inventory] items` starter kit (stripping admin `SpawnStone`s) or the returning survivor's saved checkpoint, while rejecting forged recipe crafts when ingredients are missing.
    - Tracks Earth and off-world (`Mars`, `Europa`, `BeltRock`) mining yields, timber felling, foraging, block placement, blueprint construction, `BoarMeat` consumption, and `ArtisanTome` crafting in `GuestWard`, renews `SessionSeal` cleanly on rapid `< 12 s` reconnects, and restores returning miners directly onto moving planetary or `2.70 AU` `BeltAsteroid` voxel crusts even after hours of orbital drift and full server WAL restarts (`crates/sanctuary/src/tests/survivor_persistence.rs`).

## [0.115.0] - 2026-11-24

### Added
- Added hull-relative multi-crew presence & trajectory decorators eliminating high-speed cabin desync (`0.000 m` slip up to `240,000 m/s` and across `180°` Flip-&-Burn maneuvers):
  - `VesselBerth` (`Planetary` vs. `Aboard(CraftIdentity)` in `crates/world/src/presence/berth.rs`), `CompanionPresence::localized_to_hull` / `folded_onto_hull` (`crates/world/src/presence/mod.rs`), `CraftPose::hull_rotation` / `world_point_at` (`crates/world/src/relief/craft_entity/pose.rs`), and `CompanionPortal::sync_berth_presence`: captures pilot, seated `PassengerBerth`, and unseated `DeckFooting::Aboard` crew kinematics in orthonormal hull-local coordinates with `0` extra wire bytes for planetary players.
  - `VesselDeckCurve<C>` (`PresenceCurve` decorator in `crates/sanctuary/src/glide/curve.rs`), `SightRole` (`FullActor(VesselBerth)` vs. `DistantVessel` in `crates/sanctuary/src/glide/mod.rs` and `roster.rs`), `HostDeckView` (`crates/sanctuary/src/cohort/deck_view.rs`), and `CohortPulse` hull-anchor indexing (`crates/sanctuary/src/cohort/pulse.rs`): interpolates crew stride/posture smoothly in cabin space and folds onto the local pilot's sub-tick `HelmedCraft::visual_pose` or `SmoothRoster`'s interpolated `CraftPose`, preserves cabin `ProximityUtterance` chat at `200+ km/s`, and keeps distant `160..2,400 m` `CosmicCraftSight` vessels smooth in `SmoothRoster` (`crates/sanctuary/src/tests/multi_crew.rs`).

### Fixed
- Fixed `DeckFooting::step_aboard` (`crates/client/src/avatar/deck.rs`) dropping unseated ship-deck crewmates in `HeliosphereWorld` orbit where solar/orbital `apply_pull` exceeds `1e-4`, applying Einstein free-fall equivalence in space and smooth planetary-to-thrust gravity handoff from launchpad liftoff into orbit (`crates/client/src/drive/multiplayer_tests.rs`).

## [0.114.0] - 2026-11-24

### Added
- Added moving Hill-sphere Gravity Bubbles (`CosmicFrame`, `FrameHandoff`, and `GravityBubble` in `crates/world/src/celestial/bubble.rs`, wired into `SolarEphemeris`, `HeliosphereWorld`, `VicinityCache::recenter_active`, `OrbitalFleetWorld`, `PlanetaryWorld`, `SessionStage::step_frame`, `AvatarBody::reconcile_cosmic_frame`, `RocketDrive`, and `HelmChart`):
  - Anchors local `f32` (`Vec3`) physics dynamically to `CosmicFrame::Body(BodyId)`, `CosmicFrame::BeltAsteroid { id, cell }`, or hysteresis-guarded `4,096 m` `CosmicFrame::DeepSpace([i64; 3])` sectors (`|local_pos| <= 2,560 m`, `< 0.13 mm` ULP), preserving sub-millimeter `0.5 m` `GridCell` walking, Flip-&-Burn `0 m/s` turnarounds, and rocket touchdowns from Earth out to `3.20 AU` belt rocks and `30.07 AU` Triton (`crates/world/src/continental_cosmos_tests.rs`).
  - Removed the `if !locked` orbital freeze in `HeliosphereWorld::step_surface` (`crates/world/src/celestial/heliosphere/mod.rs`), added free-fall tidal gravity compensation (`-frame_accel`) in `SolarEphemeris::gravity_accel_at`, and enabled 3D `CosmicBodyCube::emit_distant_earth` whenever `!earth_centered()` so Earth, planets, and the Sun orbit continuously across alien skies while parked crafts remain stationary at `0.000 m` local drift.
- Added multiplayer `LWEL` `epoch_secs` synchronization and `CosmicFrame`-aware Sanctuary audience decorators (`CosmicBubbleAudience` and `CosmicCraftSight` in `crates/sanctuary/src/bubble_audience.rs`, `EpochSync` in `crates/sanctuary/src/link_state/sync.rs`, `CohortPulse` in `crates/sanctuary/src/cohort/`, `AbandonedDock` in `crates/sanctuary/src/host_state/`, `GuestWard::cosmic_frame` in `crates/sanctuary/src/ward.rs`, `WardedPlanet::enact_offworld`, `HorizonStream::gather_in_frame`, and `SectorCracks` per-frame buckets):
  - Synchronizes the server's live `SolarEphemeris` orbital clock during `LWEL` welcome handshakes (`HostWire::answer_knock` and `RemoteLink::try_accept_welcome`), replicates off-world `CosmicFrame` tags with `0` extra wire bytes on Earth, validates off-world reach and crust clearance locally in each body's Hill bubble, projects `<= 2,400 m` cross-seam crafts in `f64` via `4,096 m` heliocentric macro-cells, and keeps distant planets at `0 Hz` (`crates/sanctuary/src/tests/horizon_isolation.rs`).

## [0.113.0] - 2026-11-24

### Added
- Added strict horizon & `ClimateZone` isolation decorators eliminating cross-continent and far-field CPU, RAM, GPU, and network leakage across `4096³` worlds and large multiplayer servers:
  - `SectorForest` and `SectorGrove` (`crates/world/src/tree/sector_forest.rs`, `sector_grove.rs`, `zoned_forest.rs`, and `forested_face/mod.rs`): starts `4096³` forested planets with `0` eager heap-allocated trees, evaluates untouched trunk collision and canopy drag on the CPU stack with zero heap allocations (`constrain_untouched_sector` / `canopy_drag_untouched_sector`), ticks `step_flora` exclusively on active notched/felled `4m × 4m` sectors, and streams continental woodland flora per local `MeadowRing` sector (`crates/world/src/continental_cosmos_tests.rs`).
  - `PlanetaryMesh::with_horizon` (`crates/engine/src/composite_mesh.rs`, wired into `crates/client/src/launch.rs`, `SessionDisplay::refresh_world_mesh`, `ShadedScenePass`, `ModifiedSlices::horizon_mask`, and `HeliosphereWorld::visible_slice_mask`) plus `VesselDeck::present_vessels_near`: builds and uploads only the player's visible `ClimateZone` face at ground level (streaming adjacent `ClimateZone` faces within `180 m` of a cube seam or upon ascending into orbit) and skips off-horizon `ClimateZone` faces during per-frame vessel rendering.
  - `SectorGridAudience` and `VelocityScaledSight` (`crates/sanctuary/src/sector_audience.rs`), `SpatialFactIndex` and `HorizonStream` (`crates/sanctuary/src/spatial_chronicle.rs`), and `SectorCracks` (`crates/sanctuary/src/aura.rs`): replaces `O(N²)` audience scanning with `O(N)` `160 m` spatial buckets + hemisphere/seam isolation, extends sight distance up to `2,400 m` for fast descending spacecraft in the local sky dome, isolates `PeerCrack` queues per `160 m` sector, omits all `93` wildlife bytes outside `160 m` boar earshot, and streams `HostChronicle` `WorldFact`s exclusively within each guest's `256 m` `ClimateZone` / approached-body horizon (`crates/sanctuary/src/tests/horizon_isolation.rs`).
  - `VaultStream` buffered WAL (`DiskVaultChronicle` in `crates/sanctuary/src/chronicle_decorators.rs`), ring-evicted `ReliefCellCache` (`crates/world/src/relief/face/cache.rs`), and `OffworldVault` (`crates/world/src/celestial/heliosphere/vault.rs`): eliminates per-fact `open()`/`close()` syscalls via tick-flushed `BufWriter<File>`, retains local cell rings instead of `cells.clear()`, and keeps up to `32` warm authoritative off-world `CubePlanet`s in server RAM to prevent `2`-slot LRU thrashing.

## [0.112.0] - 2026-11-24

### Added
- Added Pure-OOP massive multiplayer & continental solar-system decorator chains scaling dedicated P2P/UDP servers to `500–2,500` concurrent players and `4096³` (`100.7 km²`) World-Cubes:
  - `PulseAudience` with `MtuBudgetAudience(ProximityAudience(WorldAudience))` (`crates/sanctuary/src/audience.rs`) plus `SmoothRoster` `0.85 s` Sight-Lease (`crates/sanctuary/src/glide/`): reduces far-field and off-planet companions (`> 160 m`) to `0 Hz / 0 Bytes`, guarantees immediate closest neighbors every pulse, and rotates `80–120+` crowded hotspot peers smoothly within a `16`-companion (`<= 1,240 B`) MTU-safe UDP frame without flicker (`crates/sanctuary/src/tests/massive.rs`).
  - `ChronicleStore` with `DiskVaultChronicle(CompactedChronicle(ChronicleLog))` (`crates/sanctuary/src/chronicle_log.rs` and `chronicle_decorators.rs`, configured via `[storage] journal`, `LOW_CUBE_JOURNAL`, or `--journal`): deduplicates idempotent `StratumDisclosed` / `CraftCommissioned` facts and persists/replays world history from an append-only binary SSD WAL, folding historical `HelmClaimed` + `HelmReleased` pairs to each vehicle's final parked pose on server restart.
  - `AdminBlockWard<W>` and `SpawnBeacons` (`crates/sanctuary/src/admin_ward.rs`), `BlockMedium::SpawnStone` / `SpawnPlinthMesh` (`crates/world/src/block/spawn_plinth.rs`), and `HeldRelic::SpawnStone` (`crates/client/src/attire/relic/`): restricts `SpawnStone` placement, cracking, and harvesting to the server host while distributing joining guests round-robin across placed `SpawnStone` plinths via `WelcomeSpawn::Beacon` in `LWEL`.
  - `PulseDelivery` with `ParallelSeals(SequentialDelivery)` (`crates/sanctuary/src/seal_batch.rs`): parallelizes ChaCha20-Poly1305 AEAD outbound packet sealing across CPU cores via `std::thread::scope` once `>= 12` peers are active.
  - `ProximityUtterance` (`crates/world/src/presence/utterance.rs`) and `ProximityChatModal<M>` (`crates/client/src/menu/chat_modal.rs`): adds `<= 96 m` earshot-isolated local text chat (`Enter` / `T` with fading 6-line HUD feed and zero global spam, verified in `crates/client/src/menu/tests/chat_tests.rs`).
  - Landable procedural `BeltRock` voxel asteroids (`2.20..3.20 AU` in `AsteroidBelt::nearest_rock` and `ActiveVicinity` in `crates/world/src/celestial/heliosphere/`) with minable `LivingVeins` mineral strata (`crates/world/src/continental_cosmos_tests.rs`).
  - Polymorphic `SurfaceContact` (`Vacuum`, `GasEnvelope`, `SolidCrust` with Tell-Don't-Ask `resolve_crust` in `crates/world/src/surface_contact.rs`, wired into `PlanetaryWorld`, `PerimeterFold`, `SeamBoundary`, `HullClearance`, and `TouchdownStep`), eliminating phantom ground bounces in interplanetary vacuum and gas giants.

### Changed
- Converted `ReliefCellCache` (`crates/world/src/relief/face/cache.rs`, `patch_grid.rs`, `orbital_shell.rs`, and `face/mod.rs`) from eager full-face `Vec<CachedCell>` allocation to a lazy on-demand `HashMap` (`<= 65,536` cached cells per face) paired with `16×16` continental `OrbitalShell` macro-patches and local `MeadowRing` `0.5 m` surface cells, enabling instant `size = 1..=32` (`128³..4096³` blocks) world startup without multi-gigabyte RAM spikes.
- Capped astronomical `1 AU` scaling via `SolarProportions::EPSTEIN_HOUR_AU_CAP` (`48,087,040 m` in `crates/world/src/celestial/proportions.rs` and `EpsteinReachCosmos<C>` in `crates/client/src/cosmos_scale.rs`), raised `MAX_FUSION_SPEED` to `240,000 m/s` (`crates/client/src/drive/rocket/mod.rs`), scaled chemical rocket ceilings with `sqrt(earth_blocks / 128)`, and extended Outerra GPU shader log-depth to `10,000,000,000 m` (`10 Gm` in `crates/engine/src/shader.rs` while keeping `PerspectiveLens` invertible in `f32`), guaranteeing Epstein-Drive (`FusionTorch`) Flip-&-Burn transits to the outer Mars–Jupiter Asteroid Belt (`3.20 AU`, including worst-case `4.40 AU` solar conjunction) complete in under 1 hour (`~41–55 min`) even at maximum `size = 32`.

## [0.111.0] - 2026-11-23

### Added
- Bundled 4 distinct real woodland turf, humus, and hand-scooping earth recordings plus real settling soil-clod crumbles (`palm_dig_0..3.wav` and `soil_crumble.wav` in `crates/acoustics/assets/`, wired into `TapeSample`, `PalmScrape` in `crates/acoustics/src/work_waves.rs`, and `CrumbledWave` / `CadencedWave::palm` in `crates/acoustics/src/dig_decorators.rs`) for warm, varied bare-hand digging acoustics without synthetic hash noise.

## [0.110.1] - 2026-11-23

### Fixed
- Eliminated digital clipping and PipeWire/PulseAudio/ALSA underrun scratching on 5-horn RCS Steuerdüsen (`SoundDeck` in `crates/acoustics/src/deck.rs`, `SoundOutbox` in `crates/acoustics/src/outbox.rs`, `LoopTrack` and `BurstVoice` in `crates/acoustics/src/voice.rs`, `PlatformSink` in `crates/acoustics/src/sink/`, `HullBurnPulse` in `crates/client/src/drive/rocket/acoustics.rs`, and laminar `90–820 Hz` `rcs_*.wav` assets): replaced the hard-clipping `1/(1 + 0.28|x|)` waveshaper (`|x| > 1.38` square-wave overload when `rcs_loop` + `rcs_burst` summed together) with a true `tanh` soft-knee limiter and single-track `LoopTrack` RCS envelope (`8 ms` valve attack, `140 ms` minimum pulse hold, sample-accurate `stereo_pan` glide), removed heap allocations in `BurstVoice::from_tape`, and added a `60 ms` 3-chunk pre-buffer cushion plus `3 s` stream keep-alive to prevent OS scheduler XRUNs.

## [0.110.0] - 2026-11-23

### Added
- Bundled real Public Domain spacecraft & foley thruster recordings directly inside the executable via `include_bytes!` (`TapeSample` in `crates/acoustics/src/tape.rs` and `crates/acoustics/assets/*.wav` — NASA crew-cabin launch hull resonance, NASA rocket-bell combustion roar, and mechanical solenoid switch + pressurized gas bursts) paired with a zero-dependency 24 kHz 16-bit polyphonic stereo streaming engine (`SoundDeck` and `HullDriveDemand` in `crates/acoustics/src/deck.rs`, `LoopTrack` and `BurstVoice` in `crates/acoustics/src/voice.rs`, and `WaveSink` in `crates/acoustics/src/sink/` using Win32 `waveOutOpen` ring buffers on Windows and persistent `pw-cat`/`pacat`/`aplay` raw PCM streams on Linux).
- Added *The Expanse*-style hull-conducted thruster acoustics (`AttitudeHiss` in `crates/acoustics/src/attitude_hiss.rs`, `HullChorus`, `HullWaveTrace`, and `PropulsionCue` in `crates/acoustics/src/hull_chorus.rs`, `FusionHum` in `crates/acoustics/src/fusion_hum.rs`, `NozzleRoar` in `crates/acoustics/src/nozzle_roar.rs`, and `HullBurnPulse` in `crates/client/src/drive/rocket/acoustics.rs`): streams continuous seamless-looped Epstein cabin rumble and rocket-bell roar with live throttle modulation and smooth release tails, stereo-pans 5-horn RCS "teakettle" valve + compressed-gas bursts to active `HullWireframe` hull pods, and layers RCS steering bursts polyphonically over sustained main burns (`crates/client/src/drive/expanse_acoustics_tests.rs`).

## [0.109.0] - 2026-11-23

### Changed
- Replaced the coarse bounding-box and synthetic nose-pyramid rocket HUD envelope with a unified 3D merged outer-hull crease wireframe (`HullSolid`, `HullFacet`, `FacetSink`, `HullContour`, `CollinearTrack`, and `WireframeEdge` in `crates/world/src/relief/craft_entity/blueprint/hull_solid.rs`, `hull_facet.rs`, `contour.rs`, `wire_primitives.rs`, and `wireframe.rs`, rendered via `CameraStage::emit_hull_edge` and `WireframeShipCard` in `crates/client/src/menu/orrery_hud/wire_stage.rs` and `wire_ship.rs`): fuses all assembled structural, cabin, and `RocketCamera` fairing blocks across all 16 `MoldArchetype`s into one seamless pure outer-edge wireframe matching the rocket's true silhouette without surface fills, interior block seams, or internal RCS crossbar struts (`crates/world/src/relief/craft_entity/blueprint/contour_tests.rs` and `crates/client/src/drive/wireframe_hud_tests/mod.rs`).

## [0.108.0] - 2026-11-23

### Added
- Added superellipse (`L^5`) `PlateauZenith` (`ZenithField`), `AirSample` (`StratumPoint`), `CobaltShadowArch`, and directional low-sun `CrimsonDusk` `AirStratum` decorators (`crates/client/src/rayleigh_shell/strata.rs`, `optics.rs`, and `FIRMAMENT_FRAGMENT_SRC` in `crates/client/src/firmament_glsl.rs`), keeping sunlit cube continents uniformly azure out to the world edge, confining crimson-gold glow to low-sun sunward horizons and orbital limbs, and shading anti-solar shadow boundaries with a cool cobalt-indigo Earth-shadow arch (`crates/client/src/rayleigh_shell/tests.rs`).

### Removed
- Removed dead legacy rendering remnants: unused `ForwardPipeline` (`crates/engine/src/pipeline.rs`), ignored `surface_rot: Quat` pipeline/preview parameters and the 5-layer `surface_rotation()` pass-through getter chain (`AvatarOptics`, `PlanetaryAvatar`, `BlackBoxAvatar`, `EmbodiedAvatar`, `AvatarSight`, `GlPlatformSurface`, and `PreviewAtlas`), legacy flat-sky uniforms (`u_sky_color`, `u_night_factor`, `u_local_up`) in `GlFirmament` / `VaultView` / `VaultGlow`, and the obsolete non-heliosphere Arctic-only aurora branch in `polar_aurora`.

## [0.107.0] - 2026-11-23

### Added
- Added penetrable 3D cubic Rayleigh atmosphere decorator chains (`AirStratum` with `BarometricStratum`, `CrimsonDusk`, `TropopauseRim`, and `SmoothBand` in `crates/client/src/rayleigh_shell/strata.rs`; `OpticalMedium` with `VacuumMedium`, `CubicAirVolume`, `HorizonHazeMedium`, `SolarFluxMedium`, and `CubicSlab` in `medium.rs`; `CubicRayleighShell` in `optics.rs`; `SolarAirCatalog` in `catalog.rs`; and `AtlasSkyProjection` / `FirmamentPlanets` in `planets.rs`), rendering full-altitude `[-outer, +outer]^3` cubic sky scattering with sharp 90° outer corners and a luminous `h ≈ 0.88` stratospheric ceiling from the ground through rocket ascent into space (`crates/client/src/firmament_glsl.rs` and `crates/client/src/rayleigh_shell/tests.rs`).
- Added encapsulated Tell-Don't-Ask firmament & diurnal stage objects (`VaultView`, `VaultGlow`, and polymorphic `VaultEphemeris` in `crates/client/src/firmament.rs`; `MeasuredEpoch`, `FirmamentQuadSource`, and `GlDiurnalStage` in `crates/client/src/diurnal_pipeline/`; and `FirmamentShaderSource` / `FirmamentGlslPair` in `gl_firmament.rs` and `firmament_glsl.rs`), removing `#[allow(clippy::too_many_arguments)]`, releasing GPU quad buffers cleanly, and early-exiting occluded inner-core ground rays (`body_h * 0.50`) in the fragment shader.

### Fixed
- Fixed hard 90° day/night sky, starfield, sun-halo, and aurora cuts when walking or driving across `ClimateZone` seams (`AtmosphericShell` in `crates/client/src/rayleigh_shell/mod.rs`, `DiurnalPipeline` in `crates/client/src/diurnal_pipeline/`, and `FIRMAMENT_FRAGMENT_SRC` in `crates/client/src/firmament_glsl.rs`) by replacing discrete `ClimateZone::dominant` zenith snaps and flat ground `u_sky_color` fills with continuous radial zeniths, per-ray 3D cubic Rayleigh integration, and per-pass camera re-centering for placed tripod and hull cameras.

## [0.106.0] - 2026-11-22

### Added
- Added 3D focal Keplerian elliptical orbits (`OrbitalLocus`, `KeplerAnomaly`, `EllipticShape`, `OrbitalPlane`, and `OrbitElements` in `crates/world/src/celestial/kepler.rs` and `SolarCatalog` in `crates/world/src/celestial/catalog.rs`) solving `M = E - e sin E` with exact Vis-Viva energy and Kepler second-law velocities across all planets and moons, pairing `EARTH`'s `e = 0.20` perihelion (`0.80 AU`) with Southern Summer Solstice (`Desert`, `Volcanic`, `Jungle`) and aphelion (`1.20 AU`) with Northern Summer Solstice (`Arctic`, `Temperate`, `Steppe`), and rendering focal-displaced 3D ellipses on `ChartHologram` (`crates/client/src/menu/solar_chart/hologram.rs`).
- Added corner-to-corner World-Cube diurnal rotation (`PlanetarySpin` and `ApexSpin` in `crates/world/src/celestial/spin.rs`, wired into `SolarEphemeris::sun_direction_from`, `CosmicBodyCube` in `crates/world/src/celestial/heliosphere/cosmic_sky.rs`, `FirmamentPlanets`, `AtmosphericShell::sky_illumination_with_flux`, `DiurnalPipeline`, and `polar_aurora` in `crates/client/src/firmament_glsl.rs`): spins the World-Cube every 20 minutes around its `(-1, +1, -1)` ↔ `(+1, -1, +1)` diagonal apex so all six fixed `ClimateZone` continents receive full diurnal day/night transitions without grazing-horizon dead zones, scales sky radiance with elliptical solar flux, crowns the northern `Arctic`/`Temperate`/`Steppe` tri-corner with perihelion-boosted aurora, and visibly rotates distant Earth from deep space (`crates/world/src/celestial_tests.rs` and `crates/client/src/celestial_tests.rs`).

## [0.105.0] - 2026-11-22

### Added
- Added 3D octahedral thruster pod rosettes (`CameraStage::emit_pod_rosette` in `crates/client/src/menu/orrery_hud/wire_stage.rs`), nozzle-mouth ignition sparks, back-to-front jet depth sorting, and a live `RCS:X  BELL:Y` hardware tally header in `WireframeShipCard` (`crates/client/src/menu/orrery_hud/wire_ship.rs`).

### Changed
- Collapsed multi-block structural spacecraft hulls into a single translucent ghost `CellRole::Fuselage` prism (`HullWireframe` in `crates/world/src/relief/craft_entity/blueprint/wireframe.rs` and `PartPropulsion::collect_wireframe` in `crates/world/src/relief/vehicle_assembly/part/propulsion.rs`), isolated active main-bell highlighting (`ActiveMainBell` / `ActiveFusionBell`) from RCS-only burns, and normalized linear `STEER >` vectors across heavy multi-block hulls (`crates/client/src/drive/wireframe_hud_tests.rs`).

## [0.104.0] - 2026-11-22

### Added
- Added camera-synchronized 3D wireframe rocket thruster HUD (`HullWireframe`, `WireframeCell`, `ExhaustJet`, `CellRole`, and `JetTone` in `crates/world/src/relief/craft_entity/blueprint/wireframe.rs`, `PartPropulsion::collect_wireframe` in `crates/world/src/relief/vehicle_assembly/part/propulsion.rs`, and `CameraStage`, `WireframeShipCard`, and `WireframeShipOverlay` in `crates/client/src/menu/orrery_hud/wire_stage.rs` and `wire_ship.rs`), projecting the assembled voxel craft, live 5-horn RCS / main-bell / fusion `EXHAUST` cones, and the resulting 6-DOF phosphor-green `STEER >` / `< STEER` vector in lockstep with `NOSE`, `BELLY`, and `SIDE` hull camera views (`crates/client/src/drive/wireframe_hud_tests.rs`).
- Added `[`, `]`, and `%` 5×7 pixel glyph stamps to `PixelStamp` (`crates/client/src/menu/glyph.rs`).

### Changed
- Increased baseline starting `RocketNozzle` satchel provision from `4` to `6` (`server.toml`, `TestKitSatchel` in `crates/client/src/server_satchel.rs`, and `SatchelIntake` in `crates/client/src/attire/pouch/intake.rs`) so custom rockets can mount main bells alongside 4-way perimeter attitude pods out of the box.

## [0.103.0] - 2026-11-22

### Added
- Added Outerra-style logarithmic depth buffer (`0.03 m .. 100,000,000 m` in `VoxelShaderSource` in `crates/engine/src/shader.rs` and `PerspectiveLens` in `crates/client/src/lens.rs`), rendering all 21 solar-system bodies (`SOL`, 8 planets, 11 moons) and procedural belt asteroids (`CosmicBodyCube`, `GiantCloudShell`, and `AsteroidBelt` in `crates/world/src/celestial/`) at true 1:1 metric coordinates and radii without `4096 m` far-plane clipping or `2800 m` proxy shrinking (`crates/client/src/heliosphere_tests.rs`).
- Added unified `[cosmos] scale` solar-system scale factor (`server.toml`, `CosmosScale` in `crates/client/src/cosmos_scale.rs`, `CosmosServerConfig` in `crates/sanctuary/src/config/cosmos.rs`, and `SolarProportions::with_system_scale` in `crates/world/src/celestial/proportions.rs`) governing planetary/lunar orbital distances, planet sizes, moon sizes, and Sun size across both 3D world geometry and holographic charts (`OrreryFrame` and `SolarChartModal`, verified in `crates/client/src/cosmos_scale_tests.rs`).
- Added dual-slot LRU off-world terrain pool (`VicinityCache` and `ActiveVicinity` in `crates/world/src/celestial/heliosphere/vicinity.rs` and `awakened.rs`, backed by `24` GPU terrain slices in `CompositeVoxelMesh`), keeping two nearby rocky bodies (such as Mars and Phobos or two Co-Op destinations) simultaneously awakened without mutual eviction.
- Added multi-world `AtmosphericShell` (`crates/client/src/rayleigh_shell/mod.rs`, `DiurnalPipeline`, and `FIRMAMENT_FRAGMENT_SRC` in `crates/client/src/firmament_glsl.rs`) casting azure Rayleigh sky on `Earth`, salmon-butterscotch dust haze on `Mars`, amber haze on `Venus` and `Titan`, and pure vacuum black sky on airless moons such as `Luna` even in broad daylight.
- Added `1× / 6× / 24×` hull-camera optical telescope magnification (`OpticZoom` in `crates/client/src/avatar/helm_optics.rs`, cycled via `ControlAction::MoldForm` / `F`), moving-ephemeris `CourseProfile::MovingBody` target locks (`crates/client/src/drive/transfer_arc/`), and right-handed `SO(3)` two-stage `OrreryFrame` (`TRANSFER` 1:1 scale vs. automatic `LUNAR APPROACH ZOOM` in `crates/client/src/menu/orrery_hud/frame.rs`).

### Changed
- Removed legacy duplicate sky structures (`StarVault`, `StarryPipeline`, and 2D shader planet/moon billboards) in favor of the unified 3D metric `Heliosphere` + infinite firmament starfield and subpixel planet beacons.

### Fixed
- Fixed central `SOL` appearing to follow spacecraft steering by decoupling `DiurnalPipeline` solar/firmament directions from avatar `surface_rot`, tilting `EARTH`'s orbital plane by `23.4°` (`SolarCatalog` in `crates/world/src/celestial/catalog.rs`) for natural spawn daylight, and rendering `SOL` as a fixed 3D solar cube at the center of the Solar System.
- Fixed Nose-Cam lunar approach where `OrreryConsole` displayed the spacecraft inside the Moon-Cube while no 3D Moon appeared ahead due to `4096 m` far-plane clipping, improper reflection basis in `OrreryCanvas`, and unawakened off-world GPU slices (`crates/client/src/session/stage.rs` and `shaded.rs`).

## [0.102.0] - 2026-11-22

### Added
- Added 5-layer boresight-locked `RangeScale` decorator chain (`ReticleStandoffScale`, `BoresightTape`, and `ShiftedTarget` in `crates/client/src/menu/orrery_hud/standoff.rs`, `GlassPlateScale` in `scale_decorators.rs`, `PhosphorTone` and `ClosureKinematics` in `phosphor.rs`, and `Quad::shifted` in `crates/world/src/quad.rs`), plus `ChannelStripOverlay` (`overlays.rs`) and `HullBoresight` (`crates/client/src/avatar/helm_optics.rs`), consolidating cockpit HUD emission into Tell-Don't-Ask messages across `EmbodiedAvatar`, `AvatarBody`, `CockpitBerth`, and `HelmOptics`.

### Changed
- Anchored the `RocketNose` (`TGT`) and `RocketBelly` (`ALT`) vertical laser range scales rigidly to the projected 3D boresight reticle crosshair (`HullBoresight::reticle_point`) at a clear `0.25` horizontal standoff so the scale remains fixed beside the crosshair instead of following mouse gimbal gaze, and unified all scale layers onto a monochromatic 129° avionics Head-Up Display phosphor-green tonal palette (`PhosphorTone::GlassTint` .. `PeakPhosphor`, verified in `crates/client/src/drive/optic_scale_tests.rs`).

## [0.101.0] - 2026-11-22

### Added
- Added scalable 21-body cubic Solar System (`SolarCatalog`, `BodyId`, `SolarEphemeris`, `HeliosphereWorld`, `VicinityCache`, and `AlienWorld` in `crates/world/src/celestial/`, configured via the `CosmosScale` decorator chain in `crates/client/src/cosmos_scale.rs`, `[cosmos]` server configuration in `crates/sanctuary/src/config/`, and `LWEL` multiplayer handshake tails): simulates moving 64-bit `DVec3` Keplerian orbits and long-range `1/r²` multi-body gravity with hyperbolic Jupiter gravity-assist slingshots, rotating nested cloud shells and Saturn's square ring (`GiantCloudShell`), on-demand single-slot off-world `CubePlanet` generation, and body-local `WorldFact::OffWorld` multiplayer chronicle synchronization (`crates/world/src/chronicle/`, `crates/sanctuary/src/tests/offworld_sync.rs`, and `crates/client/src/heliosphere_tests.rs`).
- Added orbital Earth Rayleigh atmosphere, cubic 3D solar corona, live planet uniform projection, and square solar-eclipse occultation (`CubicRayleighShell`, `CubicSolarBody`, and `FirmamentPlanets` in `crates/client/src/rayleigh_shell.rs`, `GlFirmament` in `crates/client/src/gl_firmament.rs`, `DiurnalPipeline` in `crates/client/src/diurnal_pipeline.rs`, `FIRMAMENT_FRAGMENT_SRC` in `crates/client/src/firmament_glsl.rs`, and `VoxelShaderSource` in `crates/engine/src/shader.rs`).
- Added interactive 3D holographic Solar System Map (`SolarChartModal`, `ChartViewport`, and `ChartHologram` in `crates/client/src/menu/solar_chart/`, bound to rebindable `ControlAction::SystemMap` / `KeyM` in `crates/client/src/controls/bindings.rs` and `ControlsSlateHud`), coupled with `SymplecticOrbit` (`Velocity-Verlet` curved Keplerian and slingshot arcs with moving-planet arrival lead solving) and `BrachistochroneArc` (`50%` *Flip-and-Burn* continuous-thrust transfers) in `crates/client/src/drive/transfer_arc/` (`crates/client/src/solar_chart_tests.rs`).
- Added *The Expanse*-style `BlockMedium::FusionTorch` / `HeldRelic::FusionTorch` Epstein magnetic-nozzle propulsion (`FusionTorchMesh` in `crates/world/src/block/space_parts/fusion_torch.rs`, `FusionTorchStage` in `crates/world/src/relief/vehicle_assembly/impulse/stages.rs`, and `MAX_FUSION_SPEED` in `crates/client/src/drive/space_step.rs`), `FlipBurnReadout` (`TIME TO FLIP` / `RETRO FLIP NOW` / `BRAKE BURN`) in `NoseOrreryHud` and `ViewfinderHud`, sustained `> 4 G` `HighGStrain` tunnel vision in `AvatarPosture` (`crates/client/src/posture/high_g.rs`), and synthesized `FusionHum` reactor acoustics (`crates/acoustics/src/fusion_hum.rs`).
- Added deterministic zero-baseline-memory 3D toroidal `AsteroidBelt` (`2.20 .. 3.20 AU` in `crates/world/src/celestial/belt.rs`) with `O(1)` spatial neighborhood queries, on-demand landable/mineable asteroid `CubePlanet` materialization, and `0`-heap-allocation flythrough streaming verified in `crates/client/src/allocation_audit.rs`.

## [0.100.0] - 2026-11-21

### Added
- Added decorator-composed vertical hull-camera laser range scales (`RangeScale`, `ScaleDomain`, `GraduatedSpine`, `ApproachZoneScale`, and `ChevronReadoutScale` in `crates/client/src/menu/orrery_hud/spine.rs` and `scale_decorators.rs`, composed via `OpticOverlay`, `SilentOverlay`, `TargetLockOverlay`, `RangeTapeOverlay`, and `ShipOpticHud` in `crates/client/src/menu/orrery_hud/overlays.rs`): `OpticMount::RocketNose` projects a `0–1000m` `TGT` target-distance and closing-speed scale once a `TransferArc` destination is locked, while `OpticMount::RocketBelly` projects a high-resolution `0–100m` `ALT` skid-clearance radar scale with dynamic sink-rate caution/danger alerts alongside `[ + ]` surface target brackets (`crates/client/src/drive/optic_scale_tests.rs`).
- Added polymorphic `OpticTelemetry`, `BellySnapshot`, `PadClearance`, and `HullClearance` (`crates/client/src/drive/clearance.rs` and `helm_chart.rs`), sharing hull-sole and nadir terrain clearance across `TouchdownStep` physics and Belly-Camera radar telemetry up to `105m` above Homeworld and Moon surfaces without `Option` state fields.

## [0.99.0] - 2026-11-21

### Changed
- Repositioned the nose-camera 3D `NoseOrreryHud` (`crates/client/src/menu/orrery_hud/mod.rs`) to bottom-center (`cx = 0.0`) and relocated the multi-camera / Intercom preview strip (`PreviewStripLayout::slot_bounds` in `crates/client/src/menu/viewfinder.rs`) to a top-right cluster with 4-column wrapping so the forward reticle and `FLIGHT COMPUTER` banner remain unobstructed.
- Replaced the hologram rocket mesh inside `OrreryCanvas` (`emit_pyramid_frame` in `crates/client/src/menu/orrery_hud/canvas.rs`) with a 3D wireframe triangular-tip pyramid matching the wireframe `Earth-Cube` and `Moon-Cube` hologram aesthetic for clear 3D heading orientation.

## [0.98.0] - 2026-11-21

### Added
- Added cosmic nose-camera target locking and holographic 3D `ConsoleOrrery`-style trajectory guidance (`CelestialAtlas` in `crates/world/src/celestial/atlas.rs`, `TransferArc`, `NavigationCourse`, `RouteTelemetry`, `OrrerySnapshot`, and `HelmChart` in `crates/client/src/drive/transfer_arc.rs`, and `NoseOrreryHud` / `OrreryCanvas` in `crates/client/src/menu/orrery_hud/`): left-clicking in `OpticMount::RocketNose` locks onto lunar or homeworld terrain without releasing `FlightConsole`, integrates two-body `BinaryGravity` (`PlanetaryWorld::apply_pull`) via symplectic shooting into a 3D transfer curve, frames the marked destination with `[ + ]` brackets, and renders a live 3D golden rocket symbol with magenta flight vector and `ON COURSE` / `OFF COURSE` telemetry inside the bottom-right wireframe orrery (`crates/client/src/drive/orrery_route_tests.rs`).
- Added composable stack-allocated `HeadSculpture` decorator hierarchy (`BareVisage` in `crates/client/src/attire/visage.rs`, `CollaredHead` and `StyledCrown` in `crates/client/src/attire/crown.rs`, `TressedHead` / `HairCoiffure` in `crates/client/src/attire/coiffure.rs`, `BeardedHead` / `BeardCut` in `crates/client/src/attire/beard.rs`, `CrownedHead` / `BrowAdornment` in `crates/client/src/attire/headgear.rs`, and `HeadsetHead` in `crates/client/src/attire/intercom.rs`) plus `CrownStyle::VoyagerCrest`, sculpting cheekbone planes, 3D nasal bridges, almond eyes with corneal catchlights, anatomical facial hair, open-faced hoods/helms, and 3/4 portrait Intercom lighting across all male and female characters.

### Changed
- Localized the `H` controls legend (`ControlsSlateHud` in `crates/client/src/menu/controls_slate.rs`), `ControlAction::title` (`crates/client/src/controls/bindings.rs`), and nose-camera flight computer banner (`ViewfinderHud` in `crates/client/src/menu/viewfinder.rs`) to English (`ROCKET CONTROLS`, `VEHICLE CONTROLS`, `CONTROLS`, `FLIGHT COMPUTER: ON/OFF`, `MARK TARGET`).

## [0.97.0] - 2026-11-21

### Added
- Added toggleable on-screen controls HUD (`ControlsSlate`, `ControlsSlateHud`, and `SlateContext` in `crates/client/src/menu/controls_slate.rs`, bound to `ControlAction::ControlsLegend` / `H` by default and rebindable in `ControlStudio`) displaying live bound shortcuts tailored to pedestrian, wheeled vehicle, and spacecraft flight contexts (`crates/client/src/drive/rcs_guidance_tests.rs`).

### Changed
- Bound `ControlAction::RollLeft` and `ControlAction::RollRight` to `Q` and `E` by default with domain-scoped swap uniqueness (`ControlAction::spaceflight_domain` in `crates/client/src/controls/bindings.rs`) and cockpit trigger routing (`EmbodiedAvatar` in `crates/client/src/avatar/embodied/mod.rs`) so holding `Q` / `E` rolls spacecraft without stowing or casting held palm items.

## [0.96.0] - 2026-11-21

### Added
- Added 5-nozzle `RcsManifold` (`Outward`, `Dorsal`, `Ventral`, `Port`, `Starboard` in `crates/world/src/block/space_parts/rcs_manifold.rs`) shared 1:1 between 3D mesh emission (`AttitudeNozzleMesh`) and rigid-body `r × F` lever-moment propulsion (`PartPropulsion::attitude_burn` and `VernierAttitudeStage`), plus mass-weighted center-of-mass and principal-axis `InertiaTensor` integration (`HullBallast` and `RigidImpulse` in `crates/world/src/relief/vehicle_assembly/impulse/`) so missing or asymmetric side thrusters produce authentic rotational coupling and lateral drift (`crates/client/src/drive/hollow_nozzle_tests.rs`).
- Added toggleable spacecraft flight guidance (`AttitudeGuidance::Engaged` vs. `AttitudeGuidance::Manual` in `crates/world/src/relief/vehicle_assembly/impulse/` and `RocketDrive` in `crates/client/src/drive/rocket.rs`) with persistent 3D vacuum `angular_velocity`, console screen status feedback (`FlightConsoleMesh`), and a live `FLUGCOMPUTER: EIN` / `FLUGCOMPUTER: AUS` telemetry badge in the nose camera viewfinder (`ViewfinderHud` and `GuidanceReadout` in `crates/client/src/menu/viewfinder.rs` and `crates/client/src/drive/rcs_guidance_tests.rs`).
- Extended customizable keybindings (`ActionBindings` and `ControlAction` in `crates/client/src/controls/bindings.rs` and 3-column `SPACEFLIGHT` studio in `crates/client/src/menu/settings/mod.rs`) with `FlightGuidance` (`T`), `RollLeft` (`Z`), `RollRight` (`X`), `RcsTranslate` (`LALT` + `WASD` pure linear translation), and `RetroFlip` (`R`).

### Changed
- Sculpted `RocketNozzleMesh` and `AttitudeNozzleMesh` around hollow 3D De Laval bells (`HollowBell` in `crates/world/src/block/space_parts/hollow_bell.rs`) that remain dark and unlit when idle and emit layered, alpha-blended supersonic exhaust plumes (`VoxelShaderSource` in `crates/engine/src/shader.rs`, `DiurnalPipeline` in `crates/client/src/diurnal_pipeline.rs`, and `PlumeSplitTarget` / `SceneActors` in `crates/client/src/session/display/vessel_stage.rs`) strictly from active bells and individual firing RCS horns.

## [0.95.0] - 2026-11-21

### Added
- Segregated `PlanetaryWorld` into focused, narrow decorator contracts (`TerrainMesh`, `WoodlandLife`, `VesselDeck`, and `CompanionPortal` alongside `GravityField` and `WorldEarthworks` in `crates/world/src/contracts.rs` and `crates/world/src/planet/facets.rs`), plus the 3-method `FaunaRealm` wildlife contract (`AmbientRealm` in `crates/fauna/src/planet/ambient.rs`), the 1-method `PortalBeat` standby UDP socket contract (`HostWire` in `crates/sanctuary/src/host_wire.rs` and `StandbyLink` in `crates/sanctuary/src/standby_link.rs`), and the 1-method immutable `PresenceCurve` trajectory decorator chain (`CircularStrideCurve(WaypointCurve)` and `TimelineClock` in `crates/sanctuary/src/glide/`).

### Changed
- Decoupled `PlanetaryWorld` terrain, meshes, and flora completely from cross-thread `Mutex` locking (`SanctuaryHost` / `HostedPlanet::spawning` and `GuestJournal` / `RemotePlanet::spawning` in `crates/sanctuary/src/`), keeping `Box<dyn PlanetaryWorld>` owned lock-free on the main simulation thread while restricting `SocketLoop` standby (`>= 20 ms`) strictly to `Arc<Mutex<dyn PortalBeat>>` UDP wire sessions (`crates/sanctuary/src/tests/conduit.rs`).
- Converted `CompanionGlide` and `RemoteWildlife` (`crates/sanctuary/src/glide/mod.rs` and `remote_wildlife.rs`) into pure `Copy` immutable time-window trajectories evaluated at `render_secs` (`presence_at`, `craft_at`, `sounder_at`) without mutable `subtick_secs` fields or per-entity `advance_tick` / `blend_subtick` mutation.

### Removed
- Removed `ConduitPlanet` (`crates/sanctuary/src/conduit_planet.rs`) and its `Arc<Mutex<Box<dyn PlanetaryWorld>>>` global planet lock, as well as duplicated `PlanetaryWorld` pass-through boilerplate in `crates/fauna/src/planet/ambient.rs`.

## [0.94.1] - 2026-11-21

### Fixed
- Eliminated fundamental local and remote walking stutter in multiplayer (`ConduitPlanet`, `PortalMailbox`, and `SocketLoop` in `crates/sanctuary/src/conduit_planet.rs` and `crates/sanctuary/src/mailbox/`, `WakeGate` and `ElasticHorizon` in `crates/client/src/session/pulse/cadence.rs` and `mod.rs`, `GlideClock` and `CompanionGlide` in `crates/sanctuary/src/glide/`, `LinkSync` and `GuestCohort` in `crates/sanctuary/src/link_state.rs` and `cohort.rs`, `PresenceRhythm` in `crates/world/src/presence/rhythm.rs`, `MeadowRing` / `MeadowSweep` and `MeadowScan` in `crates/world/src/relief/face/meadow_ring.rs` and `crates/world/src/planet/patches.rs`, and `SessionStage` / `ShadedScenePass` in `crates/client/src/session/`): drove P2P socket pumping synchronously on the 120 Hz `EpochPulse` simulation tick while keeping `SocketLoop` as a lock-free standby heartbeat (`>= 20 ms` stall failover) so active gameplay incurs zero cross-thread `Mutex` contention or clock drift, anchored `WakeGate` frame pacing to pre-swap `frame_origin` rather than post-VSync `presented_at` so 60 Hz VSync never shifts sampling phase or drops to 30 FPS under load, replaced unsmoothed 40 Hz `GlideClock` clamping with 60 Hz phase-preserving pulses and an asymmetric EMA playout horizon, wrapped companion stride phase modulo `TAU` and swing arcs forward via `PresenceRhythm`, removed per-tick `Vec<DrivenCraft>` heap allocations in multiplayer surface/deck collision, guarded `SectorGridMesh` against monolithic full-face rebuilds, and streamed stale `MeadowRing` grass sectors center-outward across frames (`crates/sanctuary/src/tests/conduit.rs`, `crates/client/src/session/pulse_tests.rs`, and `crates/client/src/allocation_audit.rs`).

## [0.94.0] - 2026-11-21

### Added
- Added customizable `ControlAction::Sprint` (`LCTRL` + `W` by default, rebindable under `MOVEMENT` in `ControlStudio` and persisted as `sprint` in `controls.toml`) along with the `SprintControls` `KeyboardControls` decorator, stack-allocated `SprintTarget` `LocomotionTarget` decorator, and self-governing `SprintChord` / `ChordGrip` state machine (`crates/client/src/controls/sprint.rs`, `crates/client/src/controls/bindings.rs`, `crates/client/src/menu/settings/mod.rs`, and `crates/client/src/sprint_tests.rs`).

### Changed
- Simplified `KeyMotion` (`crates/client/src/controls/mod.rs`) into a pure two-field momentary trigger registry and encapsulated Tell-Don't-Ask `active_in`, `label_for`, and `rebind_token` methods on `ActionBindings` (`crates/client/src/controls/bindings.rs` and `crates/client/src/controls/file_bindings.rs`).

### Removed
- Removed double-tap `W` sprint cadence (`crates/client/src/cadence.rs`) in favor of `SprintControls` chord latching while `MoveForward` remains held.

## [0.93.1] - 2026-11-21

### Fixed
- Aborted active digging, partial `TurfScars` wall cracks, `bare_hand_attire` digging hand posture, and open inspection overlays immediately upon lethal core contact or fatal strain (`MortalMotor` in `crates/client/src/avatar/planetary.rs`, `EmbodiedAvatar::advance_blackout` in `crates/client/src/avatar/embodied/mod.rs`, `AvatarGear::relinquish_perished` and `dispatch_trigger` in `crates/client/src/avatar/gear.rs`, and `PalmEarthworks::relinquish_perished` in `crates/client/src/earthworks/mod.rs`), and flushed urgent `GuestPulse` updates before post-respawn petitions (`LinkSync::update_presence` and `RemoteLink::send_petition` in `crates/sanctuary/src/link_state.rs` and `remote_link.rs`) so companions never observe lingering digging hands during the 2.6-second blackout and immediate surface actions after respawning enact on `SanctuaryHost` (`crates/client/src/mortality_tests.rs`).

### Changed
- Replaced boolean state flags in `VitalStrain` (`perished: bool` in `crates/client/src/posture/vital.rs`) and `DebrisSplash` (`active: bool` in `crates/client/src/earthworks/debris.rs`) with polymorphic `VitalStrain::Breathing` / `VitalStrain::Perishing` and `DebrisSplash::Calm` / `DebrisSplash::Spraying` representations.

## [0.93.0] - 2026-11-21

### Added
- Added authoritative multiplayer wild boar synchronization (`BoarPresence`, `BoarStance`, `SounderPresence`, `GoreStamp`, and `WildlifePulse` in `crates/world/src/presence/boar.rs`, `WorldFact::BoarStruck` in `crates/world/src/chronicle/`, `HostSounder` in `crates/sanctuary/src/host_sounder.rs`, `RemoteWildlife` in `crates/sanctuary/src/remote_wildlife.rs`, and `RemoteSounder` in `crates/fauna/src/wild_boar/remote.rs`), simulating one shared 3-boar sounder on `SanctuaryHost` across all co-op companions, streaming interpolated boar kinematics and deduplicated tusk knockback impulses in `SanctuaryEnvelope::HostPulse`, vetting melee `BoarStruck` petitions through `WardedPlanet` / `HostState` reach guards, and chronicling dropped `SurfaceFind::BoarMeat` to all players (`crates/sanctuary/src/tests/wildlife.rs`).
- Added `WoodlandPlanet` and `AmbientWoodland` `PlanetaryWorld` wildlife decorators (`crates/fauna/src/planet/mod.rs` and `ambient.rs`, plus `PlanetaryWorld::step_wildlife`, `PlanetaryWorld::emit_wildlife`, `WorldEarthworks::strike_boar`, and `SightCone` in `crates/world/src/sight_cone.rs`), encapsulating fauna inside the world boundary instead of `SessionStage` so joining a multiplayer server atomically discards solitary boars (`crates/client/src/wild_boar_tests.rs`).

### Fixed
- Eliminated phantom and underground wild boar attacks (`BoarTerrain` in `crates/fauna/src/wild_boar/kinematics.rs`, plus polymorphic `SounderRoster`, `HerdRoster`, and `FlockRoster` replacing `anchored: bool` half-born sky states in `crates/fauna/src/wild_boar/mod.rs`, `roe_deer/mod.rs`, and `sparrow/mod.rs`), keeping boars reconciled to live ground beneath oak canopies, relocating boars buried under newly built structures, and blocking tusk charges across walls, pedestals, and excavated shafts (`crates/fauna/src/wild_boar_tests.rs`).

## [0.92.2] - 2026-11-21

### Fixed
- Isolated open 3D satchel (`SatchelTray`), crafting book (`ArtisanTome`), and vehicle blueprint (`BlueprintFolio`) stages from world and rocket hull depth buffers via the `InspectionModal` `SessionModal` decorator (`crates/client/src/menu/inspection_modal.rs`, `AvatarOptics::emit_inspection_stage` in `crates/client/src/avatar/embodied/optics.rs`, `BodyPose::inspection_light` in `crates/client/src/attire/pose/mod.rs`, `AvatarGear::emit_inspection` in `crates/client/src/avatar/gear.rs`, and `PalmEarthworks::emit_inspection` in `crates/client/src/earthworks/mod.rs`), clearing `DEPTH_BUFFER_BIT` after world, firmament, and hull-camera preview passes so close-up rocket or terrain blocks never clip through open inspection menus and external cameras never film floating UI trays (`crates/client/src/menu/tests/inspection_tests.rs`).

### Removed
- Removed the legacy `SatchelWardrobe` wrapper (`crates/client/src/attire/wardrobe.rs`), `DigOperation::bare_hand_digging` tuple extraction (`crates/client/src/earthworks/debris.rs`), and `PalmEarthworks::emit_folio` world-actor coupling (`crates/client/src/earthworks/mod.rs`) in favor of `DigOperation::bare_hand_attire` and `MoldedWardrobe`.

## [0.92.1] - 2026-11-21

### Changed
- Redesigned `BlueprintBlockMesh` (`crates/world/src/block/blueprint_block/mod.rs` and `schematic.rs`) from a flat wooden crate into a vertically wall-mounted medieval guildmaster's blueprint tablet ("Zunftmeister-Holz-Pad") without a kickstand, featuring a flush upright carved-oak bezel, rear forged-iron wall cleats with brass studs, hammered brass L-corner brackets with rivets, a glowing brass compass-gear medallion button, a leather-looped walnut-and-brass stylus ("Griffel"), and a recessed cobalt-sapphire slate display showing glowing cyan rover and rocket schematics (`crates/world/src/block/blueprint_block/tests.rs`).

## [0.92.0] - 2026-11-21

### Added
- Added a bottom-center multi-camera telemetry HUD strip (`PreviewAtlas` in `crates/client/src/preview_strip.rs`, `ViewfinderHud`, `PreviewStripLayout`, `CameraFeed`, and `ActiveChannel` in `crates/client/src/menu/viewfinder.rs`, `IntercomBust` in `crates/client/src/attire/intercom.rs`, `CoarseMesh` in `crates/engine/src/composite_mesh.rs`, and `HullOptic` / `OpticMount` in `crates/world/src/relief/craft_entity/blueprint/mount.rs`), combining a live studio-lit 3D `YouthAttire` flight-comms bust in Slot `0` (whose head turns with mouse input) with an active `● CAM X / NOSE|BELLY|SIDE` status badge and MSAA-compatible round-robin `24×16` `NEAREST` coarse previews of all inactive hull cameras (`crates/client/src/drive/camera_look_tests.rs`).

### Changed
- Doubled the hull-mounted camera pan-tilt cone radius to `0.52 rad` (`~30°`) and projected the HUD crosshair onto each active camera's fixed 3D boresight target (`HelmOptics::resolve_boresight`, `AvatarSight::reticle_point`, `VitalStrain::emit_reticle_at`, and `ReticleHud`) so the reticle remains locked on the craft's flight or landing target while panning the camera view (`crates/client/src/drive/camera_look_tests.rs`).

## [0.91.2] - 2026-11-21

### Changed
- Confined hull-mounted camera mouse pan and tilt within a small circular gimbal radius (`ConeLook` in `crates/client/src/orientation/cone.rs`, `HelmOptics` in `crates/client/src/avatar/helm_optics.rs`, and `CockpitBerth::rotate_view` in `crates/client/src/avatar/berth.rs`), clamping camera sight movement to a `0.26 rad` (`~15°`) circular cone around each `NoseCone`, `BellyPod`, `SideFairing`, or vehicle-mounted `FieldCamera` optical axis instead of rotating 360 degrees into the craft hull (`crates/client/src/drive/camera_look_tests.rs` and `crates/client/src/orientation/tests.rs`).

## [0.91.1] - 2026-11-21

### Fixed
- Fixed cameras attached underneath crane-suspended rockets rendering the hollow interior of the World-Cube (`SensorPodAxis::lens_ray` in `crates/world/src/block/space_parts/rocket_camera.rs`, `BlueprintOptics` in `crates/world/src/relief/craft_entity/blueprint/optics.rs`, `BlockMold::mounted_on_underside` and `SurfaceSightRay::oriented_placement_for_mold` in `crates/world/src/relief/sight_ray/mod.rs`, `PadCommission` in `crates/world/src/relief/craft_cluster/commission.rs`, and `TouchdownStep` in `crates/client/src/drive/space_step.rs`), anchoring the `BellyPod` lens directly outside its half-block sapphire dome (`+0.20m` above the cell floor instead of `-0.325m` below `min_floor`), auto-orienting underside `RocketCamera` placements to `BellyPod`, and clearing full multi-column rocket soles across stepped terrain (`crates/client/src/drive/camera_look_tests.rs`).

## [0.91.0] - 2026-11-20

### Added
- Added mouse pan-tilt free-look across all rocket hull cameras (`HelmOptics` and `CockpitBerth::rotate_view` in `crates/client/src/avatar/helm_optics.rs` and `crates/client/src/avatar/berth.rs`) and handheld external `SightMode` camera framing during rocket flight (`BodyPose::camera_orbit_frame` in `crates/client/src/attire/pose/mod.rs` and `SightMode::view_matrix` in `crates/client/src/attire/sight.rs`), enabling 360-degree camera observation across `NoseCone`, `BellyPod`, `SideFairing`, and handheld `FieldCamera` views without steering or tilting the spacecraft (`crates/client/src/drive/camera_look_tests.rs`).

## [0.90.0] - 2026-11-20

### Changed
- Replaced monolithic `blueprints.toml` vehicle blueprint persistence with directory-backed per-vehicle TOML files (`DirectoryBlueprints`, `BlueprintSheet`, `BlueprintSlug`, and `CleanTitle` in `crates/client/src/blueprint/archive.rs` and `crates/client/src/blueprint/sheet.rs`, plus `BlueprintArchive::store` and `BlueprintArchive::expunge`), saving each named rocket or vehicle in its own `blueprints/<slug>.toml` file with slug collision disambiguation and isolated single-file updates/deletions (`crates/client/src/blueprint_tests.rs`).

## [0.89.0] - 2026-11-20

### Changed
- Provisioned `BLUEPRINT BLOCK` (`vehicle_blueprint`) in the default `server.toml` starting satchel (`[inventory] items`) alongside `TestKitSatchel` and `SatchelIntake` so players start with a reusable blueprint block in their inventory (`crates/client/src/attire/tests/camera.rs`).

### Removed
- Removed the `CROSS AXLE` block (`BlockMedium::CrossAxle`, `VehicleBlockMesh::emit_cross_axle`, `HeldRelic::CrossAxle`, `ArtisanTome` recipe, starting satchel entries, and lexicon tokens) across `crates/world`, `crates/client`, `server.toml`, and block/inventory documentation.

## [0.88.0] - 2026-11-20

### Added
- Added reusable `BLUEPRINT BLOCK` (`BlockMedium::VehicleBlueprint` and `BlueprintBlockMesh` in `crates/world/src/block/blueprint_block.rs`, `HeldRelic::VehicleBlueprint`, `BlueprintFolio`, `SavedBlueprint`, `BlueprintArchive`, and `FileBlueprints` in `crates/client/src/blueprint/`, `BlueprintHoist` in `crates/world/src/relief/cantilever_rig/blueprint_mount.rs`, and `WorldFact::RigBlueprintBuilt` in `crates/world/src/chronicle.rs`) enabling players to capture crane-mounted or parked vehicles with custom typed names into `blueprints.toml` and select saved blueprints on any empty cantilever crane arm to assemble the complete vehicle while consuming required parts from `BeltSatchel` (`crates/client/src/blueprint_tests.rs`).

## [0.87.1] - 2026-11-20

### Changed
- Refactored spacecraft propulsion into an immutable 4-stage `ImpulseStage` decorator chain (`VernierAttitudeStage(MainBellStage(ConsoleGyroStage(HullBallast)))` and `CockpitAxes` in `crates/world/src/relief/vehicle_assembly/impulse/`, plus `PartPropulsion` and `PartWire` in `crates/world/src/relief/vehicle_assembly/part/`) replacing procedural `&mut RigidImpulse` accumulation while preserving strictly linear `F / m` mass scaling.

### Fixed
- Fixed compact skeleton and RCS-only rockets failing to lift off from the launchpad on `Space` (`PartPropulsion::part_mass`, `VernierAttitudeStage::resolve`, `AttitudeNozzleMesh`, and `NozzleAxis::plume_glow`) by calibrating half-meter voxel material masses (`0.20` light frame vs. `0.45` stone/ore hull) so single-bell skeleton craft lift off cleanly while heavy hulled ships require multiple main bells, and included `FlightConsole` and `RocketNozzle` in `VehiclePart::kinematic_part` so `VesselStage` streams live thruster plumes and cockpit levers every frame (`crates/client/src/drive/liftoff_tests.rs`).

## [0.87.0] - 2026-11-20

### Added
- Added shape-faithful 3D hologram placement previews (`PlannedMount`, `MountSite`, and `MountDomain` in `crates/world/src/relief/planned_mount.rs`, `PlanetaryWorld::plan_molded_block`, and `AvatarPresence::emit_placement_preview` in `crates/client/src/avatar/`) sharing a single source of truth with `place_molded_surface_block`, rendering oriented sculpted molds and 12 glowing corner docking brackets in electric cyan for `MountDomain::CraftRig` and warm emerald green for `MountDomain::WorldCube` (`crates/client/src/drive/preview_vessel_tests.rs`).
- Added GPU `u_model` (`Mat4`) local-hull vehicle rendering (`VoxelProgram::apply_model` and `Mesh::draw_posed` in `crates/engine/src/`, `CraftPose::model_matrix`, `VesselSink`, and `PosedHull` in `crates/world/src/relief/craft_entity/`, and `VesselStage` / `SceneActors` in `crates/client/src/session/display/vessel_stage.rs`) uploading rigid vehicle hulls once per `CraftIdentity` while streaming only animated wheels, steering columns, and `ConsoleOrrery` indicators.

### Changed
- Decoupled planetary surface-parked vehicles (`ParkedCraft` and `ParkedFleet` in `crates/world/src/relief/craft_cluster/fleet.rs`, `FacetedReliefFace`, `CubePlanet`, and `HelmedCraft::release_wheel`) from 90-degree `VoxelCells` terrain sectors so parking and re-boarding preserve continuous 360-degree 6-DOF `CraftPose` orientations with zero terrain sector rebuilds or meadow tuft invalidations.

### Removed
- Removed Tron wireframe mode across `crates/engine`, `crates/world`, `crates/sanctuary`, and `crates/client` (`TronShaderSource`, `TronPipeline`, `TronMesh`, `CpuTronBatch`, `TronScenePass`, `ControlAction::TronMode`, and the `LWEL` Tron aura byte).

### Fixed
- Fixed `CantileverRig` boom targeting (`CantileverRig::boom_slot_at`, `RigMountTarget`, and `StanceClearance` in `crates/world/src/relief/sight_ray/`) so standing one free block in front of the crane arm mounts `FlightConsole` and adjacent craft blocks directly without stepping back and forward (`crates/client/src/drive/nozzle_tests.rs` and `crates/world/src/relief/cantilever_rig_tests.rs`).

## [0.86.1] - 2026-11-20

### Changed
- Resculpted `RocketCameraMesh` (`PodMount` and `SensorPodAxis` in `crates/world/src/block/space_parts/rocket_camera.rs`) into a compact half-block (`0.25m`) V1 nose-cone dome with brass collars, dark graphite drum, glowing ruby status tally lamp, tapered ceramic cone, and two-tier glowing sapphire dome across zenith, belly, and side hull mounts, and added the six-variant showcase sheet (`rocket_camera_versions.jpg`).

## [0.86.0] - 2026-11-19

### Added
- Added dedicated aerodynamic spacecraft hull cameras (`BlockMedium::RocketCamera`, `HeldRelic::RocketCamera`, `RocketCameraMesh`, and `SensorPodAxis` in `crates/world/src/block/space_parts/rocket_camera.rs` and `crates/client/src/attire/relic/`) with three mouse-wheel selectable mounting profiles (`NoseCone` zenith dome, `BellyPod` nadir landing turret, and wall-bolted `SideFairing`), Crafting Book synthesis in `ArtisanTome`, and `4` starting `rocket_camera` units in `TestKitSatchel` and `server.toml`.
- Added automatic nose-camera engagement when clicking a rocket's `FlightConsole` while seated (`CraftDrive::helm_optic_channel` in `crates/client/src/drive/`, `CockpitBerth::mount_assembly` in `crates/client/src/avatar/berth.rs`, and zenith-first sorting in `CraftBlueprint::push_camera_views`), switching immediately to the highest `RocketCamera` on the spacecraft without affecting compact `FieldCamera` tripods or camera-less rockets.
- Added HOTAS rocket cockpit arm and hand kinematics (`RocketControls` and `CockpitPosture` in `crates/client/src/attire/pose/helm.rs`), polymorphic `CraftDrive::shape_pilot_pose` (`RocketDrive` vs. `WheeledDrive` in `crates/client/src/drive/`), sculpted fore/aft thrust throttle lever and 4-way flight control stick on `FlightConsoleMesh` (`crates/world/src/block/space_parts/console.rs`), and the `StudioAnimation::RocketPilot` (`ROCKET PILOT`) showcase animation in the Character Studio (`crates/client/src/menu/stage.rs`).

### Changed
- Extracted `CraftIdentity` into `crates/world/src/relief/craft_entity/identity.rs` and added `PlanetaryWorld::woodland_habitat` and `PlanetaryWorld::local_body_point` across `BinaryPlanet`, `TranslatedWorld`, `OrbitalFleetWorld`, `HostedPlanet`, `RemotePlanet`, and `ConduitPlanet`.

### Fixed
- Fixed spacecraft nose-first impacts penetrating into the Moon's interior by enforcing full-hull clearance (`CraftBlueprint::hull_envelope`), auto-righting touchdown kinematics (`TouchdownStep::resolve` in `crates/client/src/drive/space_step.rs`), and body-local seam offsets (`FaceContact` and `SeamRoad` in `crates/client/src/drive/`).
- Fixed woodland house sparrows, roe deer, and wild boars flying across vacuum onto the airless Moon (`WoodlandFauna::step_spatial`, `SparrowFlock::step_spatial`, and `TemperateGlade::temperate_surface` in `crates/fauna/`), and prevented underside nozzle grazes or non-seat crane clicks from consuming wall-mounted pods or perching off-seat.

## [0.85.0] - 2026-11-19

### Added
- Added lateral reaction-control Steuerdüsen (`AttitudeNozzleMesh` and `NozzleAxis::Attitude` in `crates/world/src/block/space_parts/`, `AttitudeImpulse` in `crates/world/src/relief/vehicle_assembly/impulse.rs`) toggled via mouse wheel scroll on `RocketNozzle` (`crates/world/src/block/mold/`), oriented with the nozzle cone facing the player in hand and bolting flush onto rocket side walls for active 6-DOF pitch/yaw steering and braking (`crates/client/src/drive/nozzle_tests.rs`).
- Added live cockpit outboard hull cameras (`CraftBlueprint::push_camera_views` in `crates/world/src/relief/craft_entity/blueprint.rs` and `CockpitBerth::handle_cockpit_trigger` in `crates/client/src/avatar/berth.rs`) providing Zenith nose-cone, Nadir landing-pad, and directional hull camera feeds with one-click Polaroid space photography for spacecraft carrying `FieldCamera` blocks.
- Added floating 3D holographic Earth-Moon-Spacecraft wireframe orrery `ConsoleOrrery` (`crates/world/src/block/space_parts/orrery.rs`) projected above helmed `FlightConsole` decks and automated 180-degree retrograde Flip-and-Burn maneuvers (`RocketDrive::step_retro_flip` in `crates/client/src/drive/rocket.rs`).
- Added zero-allocation ship-local Thrust Gravity and `0.0g` cabin float (`DeckFrame` in `crates/world/src/relief/craft_entity/deck.rs` and `DeckFooting` in `crates/client/src/avatar/deck.rs`) across `OrbitalFleetWorld`, `HostedPlanet`, `RemotePlanet`, and `ConduitPlanet` so unseated Co-op crew walk on accelerating spacecraft decks and float inside coasting cabins without falling behind (`crates/client/src/drive/expanse_tests.rs`).

### Changed
- Bounded maximum spacecraft acceleration (`MAX_THRUST_ACCEL` in `crates/world/src/relief/vehicle_assembly/impulse.rs`) and enforced quartic thrust tapering plus a strict `45.0 m/s` cosmic speed cap (`SpaceState::bounded` and `NozzleThrustStep` in `crates/client/src/drive/space_step.rs`) so sustained burns or stacked nozzles never accelerate infinitely.

### Fixed
- Fixed `RocketNozzle` palm mold initialization (`HeldRelic::independent_palm_mold` in `crates/client/src/attire/relic/mod.rs`, `BeltSatchel::transition_attire` in `crates/client/src/attire/pouch/mod.rs`, `PalmStack::reconcile_attire`, and `SatchelIntake`) and sculpted `RocketNozzleMesh` (`crates/world/src/block/space_parts/nozzle.rs`) with a closed top bulkhead and octagonal bottom bell so equipping a thruster always starts downward.

## [0.84.1] - 2026-11-19

### Changed
- Decoupled cockpit mouse free-look from spacecraft attitude steering across `ConsolePedals` and `RocketDrive` (`crates/client/src/drive/rocket.rs`), `GyroStep` (`crates/client/src/drive/space_step.rs`), `CraftDrive` / `HelmedCraft` (`crates/client/src/drive/mod.rs`), `VehiclePilot` (`crates/client/src/drive/pilot.rs`), `CockpitBerth` (`crates/client/src/avatar/berth.rs`), and `PilotOrder` / `RigidImpulse` (`crates/world/src/relief/vehicle_assembly/impulse.rs`), restricting mouse input to camera gaze while `FlightConsole` gyros hold spacecraft attitude during 6-DOF directional thruster burns (`crates/client/src/drive/rocket_tests.rs`).

## [0.84.0] - 2026-11-19

### Added
- Added spacecraft construction and 6-way physical thruster nozzles (`BlockMedium::FlightConsole` & `BlockMedium::RocketNozzle` in `crates/world/src/block/timber_grain/mod.rs`, `HeldRelic::FlightConsole` & `HeldRelic::RocketNozzle` in `crates/client/src/attire/relic/`, `NozzleAxis`, `FlightConsoleMesh`, and `RocketNozzleMesh` in `crates/world/src/block/space_parts/`, `ClusterHelm` in `crates/world/src/relief/craft_cluster/helm.rs`, `PilotOrder` & `RigidImpulse` in `crates/world/src/relief/vehicle_assembly/impulse.rs`, and `NozzleRoar` in `crates/acoustics/src/nozzle_roar.rs`) with off-center lever-arm torque and console reaction-wheel stabilization.
- Added polymorphic `CraftDrive` (`HelmedCraft`, `WheeledDrive`, and `RocketDrive` in `crates/client/src/drive/`) and the stack-allocated 5-layer `SpaceStep` decorator chain (`TouchdownStep(GyroStep(NozzleThrustStep(GravitationStep(InertialDrift))))` in `crates/client/src/drive/space_step.rs`).
- Added the Earth-Moon `CelestialSystem` decorator hierarchy (`RadialWell`, `BinaryGravity`, `LunarCraterRelief`, `LunarWorld`, `TranslatedWorld`, `BinaryPlanet`, `OrbitalFleetWorld`, and `StarVault` in `crates/world/src/celestial/`), `StarryPipeline` (`crates/engine/src/composite_mesh.rs`), stratosphere-to-space sky transition (`crates/client/src/firmament_glsl.rs`), and `ZeroGFlight` (`crates/client/src/zero_g_flight.rs`) enabling `0.0g` EVA spacewalks, `0.22g` lunar leaps, and multiplayer lunar excavation.

### Changed
- Replaced `BlockMedium::RocketEngine` and `HeldRelic::RocketEngine` with dedicated `FlightConsole` (rocket helm) and `RocketNozzle` (directional thruster) recipes in `ArtisanTome` (`crates/client/src/attire/tome.rs`), generalized `DrivenCar` into `HelmedCraft`, extended `GravityField::apply_pull` across `HostedPlanet`, `RemotePlanet`, and `ConduitPlanet`, and raised `PerspectiveLens::human_eye` far plane to `4096.0` (`crates/client/src/lens.rs`).

## [0.83.0] - 2026-11-19

### Added
- Added focused 2-method `SeamBoundary` (`OpenBoundary`, `CubicBoundary`, and `BeveledBoundary` in `crates/world/src/topology/boundary.rs`), `PerimeterFold` (`crates/world/src/topology/folded_face.rs`), `BeveledGravity` (`crates/world/src/beveled_world.rs`), and `WorldEarthworks` (`crates/world/src/lib.rs` and `crates/world/src/planet/earthworks.rs`) composed directly into `CubePlanet`.
- Segregated `Avatar` (`crates/client/src/avatar/mod.rs`) into narrow `AvatarMotor`, `AvatarOptics`, `AvatarPresence`, `PhotoPort`, and `BlackBoxDeck` contracts.

### Changed
- Narrowed `WardedPlanet` (`crates/sanctuary/src/warded_planet.rs`), `ChronicledPlanet` (`crates/sanctuary/src/chronicled_planet.rs`), `FleetLedger::park_by_id` (`crates/sanctuary/src/fleet_ledger.rs`), and `WorldFact::enact_on` (`crates/world/src/chronicle.rs`) to decorate `WorldEarthworks` without `PlanetaryWorld` or `GravityField` forwarding boilerplate.
- Narrowed `SatchelAvatar` (`crates/client/src/avatar/satchel_guard.rs`), `MortalMotor` (`crates/client/src/avatar/planetary.rs`), and `ReticleHud` (`crates/client/src/menu/portal/beacon.rs`) to `AvatarMotor`, and narrowed `BlackBoxPanel` / `BlackBoxBody` (`crates/client/src/menu/black_box_panel.rs`) to `BlackBoxDeck`.

### Removed
- Removed 418-line `BeveledWorld` and `FoldedFace` pass-through wrapper shells (`crates/world/src/beveled_world.rs` and `crates/world/src/topology/folded_face.rs`), eliminating seven redundant heap-allocated wrapper boxes per cubic planet and fixing missing `portal_phrase` / `punch_nat_peer` delegation on beveled worlds.

## [0.82.0] - 2026-11-19

### Added
- Added live multiplayer synchronization of dual-palm field-camera and block mold orientations via `PalmMolds` (`crates/world/src/block/mold/palms.rs`), `PresenceAttire` (`crates/world/src/presence/attire.rs`), `HeldPalms` (`crates/client/src/attire/hands.rs`), and `MoldedWardrobe` (`crates/client/src/attire/wardrobe.rs`), enabling peers to see 8-way handheld `FieldCamera` rotations and 16-form / 4-turn block shapes in a companion's right and left hands (`crates/client/src/companion.rs`).

### Changed
- Replaced raw `([u8; 9], [u8; 4])` byte-tuple attire payloads in `CompanionPresence` (`crates/world/src/presence/mod.rs`) with encapsulated `PresenceAttire`, decoupled `VambraceStyle::emit` (`crates/client/src/attire/hands.rs`) from `BeltSatchel`, refactored `YouthAttire::new` (`crates/client/src/attire/mod.rs`) and `AvatarArchetype` (`crates/client/src/menu/archetype.rs`) to compose `GarmentEnsemble`, `IdentityTraits`, and `HeldHands` without `#[allow(clippy::too_many_arguments)]`, and narrowed `AttireProfile` (`crates/client/src/attire/file_attire.rs`) to persistence methods.

### Removed
- Removed legacy ghost `placed_camera` tracking from `PalmStack` (`crates/client/src/attire/pouch/palms.rs`, `crates/client/src/attire/pouch/intake.rs`, `crates/world/src/block/mold/mod.rs`), half-born `FileAttire::new(String::new())` decoding in `YouthAttire` (`crates/client/src/attire/mod.rs`), and duplicate `YouthAttire::emit_companion` / `emit_peer` wrappers.

## [0.81.0] - 2026-11-18

### Added
- Added seamless 90-degree planetary cube-seam traversal for land vehicles via the zero-allocation `SlidingRoad(BumperRoad(GradedRoad(SeamRoad)))` `RoadProbe` decorator chain (`crates/client/src/drive/road.rs`), `WheelTrack` and `FaceContact` (`crates/client/src/drive/track.rs`), `PassengerBerth` (`crates/client/src/drive/passenger.rs`), and `AvatarSight::reorient_cockpit` / `HorizonPacing` (`crates/client/src/avatar/sight.rs`), enabling continuous 360-degree circumnavigation, `0.5m` step climbs, `1.0m` wall arrest, and multiplayer driver + passenger seam crossings.
- Added handedness-aware 2x2 grid basis projection (`VehicleAxes::grid_basis` in `crates/world/src/relief/vehicle_assembly/axes.rs`, `VehiclePart::park_into` in `crates/world/src/relief/vehicle_assembly/part.rs`, `CraftBlueprint::clamped_seat` in `crates/world/src/relief/craft_entity/blueprint.rs`, and canonical orientation normalization in `CraftCluster::into_vehicle_with_id` in `crates/world/src/relief/craft_cluster/mod.rs`) so vehicles park within face bounds and re-board without left/right mirroring across all six cube faces.

### Changed
- Converted `DriveKinematics` (`crates/client/src/drive/mod.rs`) into an immutable value object, encapsulated `PedalInput` fields privately with Tell-Don't-Ask dismount methods, replaced quantized `meadow_anchor` with face-aware `sector_stamp`, `visible_slices`, and `drain_meadow` (`crates/client/src/avatar/body.rs`), and unified 12-byte finite vector wire serialization in `WireVector` / `WireSlice` (`crates/world/src/wire_vector.rs`) eliminating duplicate procedural `push_vec3` / `read_vec3` functions.

## [0.80.0] - 2026-11-18

### Added
- Added stack-allocated `BurdenedGait<G>` decorator (`crates/client/src/gait/burdened.rs`) and `LocomotionGait::pace_leap` (`crates/client/src/gait/mod.rs`) scaling horizontal stride distance and vertical leap velocity polymorphically without heap allocations in the 120 Hz simulation loop.
- Added `CockpitBerth` (`crates/client/src/avatar/berth.rs`) encapsulating cockpit boarding, passenger perching, steering release, helm reconciliation, and vehicle mesh emission for `AvatarBody` (`crates/client/src/avatar/body.rs`).

### Changed
- Decoupled `SurfaceFlight` (`crates/client/src/flight.rs`), `LeapIntent`, `LocomotionOrder` (`crates/client/src/order.rs`), and `AvatarBody::prepare_stride` / `step_pedestrian` (`crates/client/src/avatar/body.rs`) from `SatchelBurden`, and replaced `carried_burden()` pass-through getters on `PalmEarthworks` (`crates/client/src/earthworks/mod.rs`) and `AvatarGear` (`crates/client/src/avatar/gear.rs`) with Tell-Don't-Ask `encumber_order`.
- Converted `SatchelTray` (`crates/client/src/attire/tray.rs`) into a pure immutable value object (`closed`, `focused_on`, `opened_if_populated`, `stowed_with`, `taken_stowed`, `toggled_open`, `navigated`) across `BeltSatchel` (`crates/client/src/attire/pouch/mod.rs`) and `SatchelIntake` (`crates/client/src/attire/pouch/intake.rs`).
- Split `crates/client/src/attire/burden/` into `mod.rs` and `slot.rs`, privately encapsulating `WeightGrade`, `SlotBurden::unit_grade`, `summed_grade`, and `corner_pigment` while testing `SlotBurden` exclusively through `Display`, `CarryLoad`, and `SlotPlaque` (`crates/client/src/attire/tests/burden.rs` and `encumbrance.rs`).

### Removed
- Removed `GravitationalFlight::jump_burdened`, `PendingLeap` (`crates/client/src/flight.rs`), and completed root refactoring checklists (`YEGOR_CHECKLIST.md` and `YEGOR_MULTIPLAYER_CHECKLIST.md`).

## [0.79.0] - 2026-11-18

### Added
- Added entity-identified multiplayer vehicles (`CraftIdentity`, `CraftBlueprint`, `CraftPose`, `SeatOccupancy`, `WorldCraft`, `BlueprintCatalog`, and `HelmLocks` in `crates/world/src/relief/craft_entity/`, `DrivenCraft` and `PilotCraft` in `crates/world/src/presence/craft.rs`, `WorldFact::CraftCommissioned` / `HelmClaimed` / `HelmReleased` in `crates/world/src/chronicle.rs`, and `FleetLedger` in `crates/sanctuary/src/fleet_ledger.rs`) with atomic host helm ownership locks, automatic guest race revocation (`VehiclePilot::reconcile_helm`), 80-byte zero-allocation pose pulses, moving-vehicle surface collision, passenger riding on secondary `CabinSeat`s (`VehiclePilot::Passenger`), and automatic parking on companion disconnect.
- Added phase-locked 40 Hz P2P snapshot interpolation (`CompanionGlide`, `GlideClock`, and `SmoothRoster` in `crates/sanctuary/src/glide.rs`) with sub-tick frame blending (`PlanetaryWorld::blend_companions`) so remote companions and driven vehicles move continuously without teleporting.
- Added `16x16`-cell sector GPU terrain updates (`SectorGridMesh` in `crates/engine/src/sector_mesh.rs`, `DirtySectors` in `crates/world/src/relief/face/patch_grid.rs`, and `PatchCoord` in `crates/world/src/planet/patches.rs`), toroidal `9x9` grass sector caching (`MeadowRing` and `MeadowStamp` in `crates/world/src/relief/face/meadow_ring.rs`), and persistent `STREAM_DRAW` actor buffer reuse (`StreamingMesh` in `crates/engine/src/streaming_mesh.rs`).
- Added autonomous 120 Hz network heartbeat decoupling (`PortalMailbox` and `SocketLoop` in `crates/sanctuary/src/mailbox/`, decorated onto live sessions via `ConduitPlanet` in `crates/sanctuary/src/conduit_planet.rs` and `crates/client/src/portal_config.rs`) so window drags or VSync stalls never pause P2P packet relay.
- Added `YEGOR_MULTIPLAYER_CHECKLIST.md` documenting the three-phase Elegant Objects multiplayer architecture refactoring.

### Fixed
- Fixed ~5 FPS frame-time collapse while driving by offloading OS audio syscalls onto a non-blocking background queue (`SoundOutbox` in `crates/acoustics/src/outbox.rs`), pacing `EngineRumble` triggers to match its combustion cycle (`crates/client/src/drive/mod.rs`), interpolating `DrivenCar` across sub-ticks, and replacing whole-face / whole-meadow VBO rebuilds with `O(1)` sector updates in `ShadedScenePass` (`crates/client/src/session/display/shaded.rs`).

### Removed
- Removed obsolete `Avatar::emit_surroundings` delegation chain across `crates/client/src/avatar/` and `crates/client/src/black_box/avatar.rs` in favor of toroidal `MeadowRing` sector updates.

## [0.78.2] - 2026-11-18

### Changed
- Renamed Step 1/2 Join screen buttons (`JOIN WORLD (ENTER)` to `JOIN HOST`, `PASTE (CTRL+V)` to `PASTE`, and `BACK TO MULTIPLAYER` to `BACK`) in `PortalLayout::emit` (`crates/client/src/menu/portal/layout.rs`).

### Removed
- Removed unused `PortalBounds::expanded` constructor from `crates/client/src/menu/portal/layout.rs` to eliminate the `dead_code` compiler warning on binary builds.

## [0.78.1] - 2026-11-18

### Changed
- Renamed Multiplayer Hub buttons (`OPEN CO-OP PORTAL` to `OPEN`, `JOIN CO-OP / SERVER` to `JOIN`, and `BACK TO MAIN MENU` to `BACK`) in `MultiplayerHubBody` (`crates/client/src/menu/portal/layout.rs`) and updated the dedicated server console join hint in `SanctuaryEndpoint` (`crates/sanctuary/src/config.rs`).
- Aligned the unhosted Multiplayer Hub card width (`-0.36..0.36`) and uniform vertical button spacing (`0.06` gap across `OPEN`, `JOIN`, and `BACK`) with `PausePanel` (`MAIN MENU`) via `PortalBounds::compact` / `PortalBounds::expanded` (`crates/client/src/menu/portal/layout.rs` and `mod.rs`), expanding only while hosting a live portal or awaiting a host NAT punch.

### Removed
- Removed the redundant idle caption line between the `JOIN` and `BACK` buttons on `MultiplayerHubBody` (`crates/client/src/menu/portal/layout.rs`) and simplified `PunchBeacon::emit_hub_summary` (`crates/client/src/menu/portal/beacon.rs`) to a pure Tell-Don't-Ask command.

## [0.78.0] - 2026-11-17

### Added
- Added persistent background Co-op NAT counter-punching across menu closes (`PunchBeacon`, `PortalStatus`, `ReticleHud`, and `BeaconHud` `MenuView` decorator in `crates/client/src/menu/portal/beacon.rs`, plus `StageOverlay::advance_dial` in `crates/client/src/session/overlay.rs` and `stage.rs`) so guests closing the menu with `Escape` continue playing with a live top-screen signal banner until the host punches back.
- Added dedicated Step 2/2 Waiting card `PunchWaitBody` (`crates/client/src/menu/portal/waiting.rs`) with self-explaining clipboard export (`CO-OP REPLY CODE: XXXX-XXXX-XXXX`), animated signal countdown, `PLAY WHILE WAITING (ESC)`, and explicit `CANCEL CONNECTION`.
- Added `AutoPunchModal` `SessionModal` decorator (`crates/client/src/menu/portal/auto_punch.rs`), window-focus sensing (`WindowEvent::Focused(true)` in `crates/client/src/session/mod.rs`), one-click `PASTE GUEST REPLY (CTRL+V)` on `MultiplayerHubBody` (`crates/client/src/menu/portal/layout.rs`), and `ChatExcerptCodec` `CodeCodec` decorator (`crates/sanctuary/src/wormhole/crockford.rs`) extracting checksum-verified 12-char portal and counter-punch codes out of pasted chat messages.

### Changed
- Replaced static procedural helpers `PortalConfig::open_live_portal`, `CrockfordStamp::strip_code_prefix` / `resembles_twelve_char_code`, and `NatAperture::local_ipv4` with encapsulated `LivePortal` (`crates/client/src/portal_config.rs`), `CrockfordStamp` (`crates/sanctuary/src/wormhole/crockford.rs`), and `NatAperture` (`crates/sanctuary/src/nat/mod.rs`) instance methods, extracted `StageOverlay` (`crates/client/src/session/overlay.rs`) and `ModalPhase` (`crates/client/src/menu/escape_menu/phase.rs`), and replaced `PortalDraft::as_str` and `PortalInvite::punch_reply` getters with Tell-Don't-Ask methods.

## [0.77.0] - 2026-11-17

### Added
- Added `SettingsPanel` (`crates/client/src/menu/settings_panel.rs`) and `ModalPhase::SettingsHub` (`crates/client/src/menu/escape_menu/panels.rs`) grouping `KEYBINDINGS` and `LOG` under a dedicated `SETTINGS` submenu with hierarchical `Escape` and `BACK` navigation.
- Added composable `MenuView` decorators `TitledCard` and `VersionedView` (`crates/client/src/menu/view.rs`) alongside profile-persistence decorators `SavedControls` (`crates/client/src/menu/settings/saved.rs`) and `SavedWardrobe` (`crates/client/src/menu/studio/saved.rs`) over narrow Tell-Don't-Ask `BindingSink` / `AttireSink` interfaces.

### Changed
- Renamed Main Menu buttons `RESUME GAME` to `RESUME` and `QUIT GAME` to `QUIT` in `PausePanel` (`crates/client/src/menu/pause_panel.rs`), encapsulated all `StudioPanels` fields privately, replaced static layout helper functions with `ControlLayout`, `BlackBoxLayout`, `PortalBounds`, and `StudioBounds` instances, and renamed `-er`-suffixed `StudioPointer` to `StageGrip`.

### Removed
- Removed `ControlStudio::current_bindings` and `WardrobeStudio::current_attire` getters, `(SettingsDirective, bool)` change-flag tuples, hardcoded `FileBindings` / `FileAttire` disk writes inside UI studios, and seven duplicated `VersionStamp` / five duplicated dialog-frame emissions across menu panels.

## [0.76.0] - 2026-11-17

### Added
- Added zero-allocation Flight Log (`BlackBoxAvatar`, `FlightTape`, `LiveTape`, `SilentTape`, `IncidentMedium`, `DiskDossier`, `MemoryDossier`, `FileBlackBox`, and `IncidentReplay` in `crates/client/src/black_box/`), in-game `LOG` menu (`PausePanel::log_button` and `BlackBoxPanel` in `crates/client/src/menu/black_box_panel.rs`) switching polymorphically between a 15-second (1,800-tick @ 120 Hz) ring buffer and a zero-overhead `SilentTape` Null Object, TOML bug-report export (`reports/incident_<millis>.toml` + clipboard path copy), and tilted ruled clipboard `IncidentSheet` HUD overlay (`PrintSubject::LogSheet` in `crates/client/src/photo_plate/exposure.rs` and `log_sheet.rs`).
- Added windowless `HeadlessSession` and `HeadlessSurface` Fake-Object harness (`crates/client/src/session/headless/`), flora-free `BiomedCubeWorld::barren` / `barren_beveled` terrain factories (`crates/world/src/biome.rs`), and getter-free `PresenceProbe` / `Avatar::report_presence` kinematics inspection (`crates/client/src/black_box/avatar.rs`), documented in `AGENTS.md` and `docs/DEVELOPMENT.md`.

### Fixed
- Fixed falling through the planetary surface into the solid corner wedge when sprinting and jumping at shallow grazing angles across `ClimateZone` seams by checking cardinal zone normals alongside smoothed `SeamBevel` normals in `SurfaceClearance::between` (`crates/client/src/threshold.rs`), carrying destination-normal clearance in `SurfaceClearance::FoldedSeam`, encapsulating `reference_up` inside `AirborneFlight` (`crates/client/src/phase/mod.rs`, `aerial.rs`, `context.rs`) so `FlightContext::reorient_edge_flight` projects 3D velocity across seam bevels, and enforcing fail-fast handshake completion in `RemotePlanet::connect` (`crates/sanctuary/src/remote_planet.rs`).

## [0.75.2] - 2026-11-16

### Changed
- Replaced passive `RenderPass` bags and repeated `tron_active: bool` branching in `SessionDisplay` (`crates/client/src/session/display/`) with the `ScenePass` trait, `ShadedScenePass` (`shaded.rs`), `TronScenePass` (`tron.rs`), and `PolaroidScenePass` shutter decorator (`pass.rs`), and simplified `GameSession::new` (`crates/client/src/session/mod.rs`) to accept `(SessionStage, SessionDisplay)` without `#[allow(clippy::too_many_arguments)]`.
- Decomposed `PlanetaryAvatar` into the zero-allocation `PlanetaryAvatar(SatchelAvatar(EmbodiedAvatar))` `Avatar` decorator chain (`crates/client/src/avatar/planetary.rs`, `satchel_guard.rs`, `embodied.rs`, `condition.rs`), removing repeated procedural `perished` / `satchel_open` / `lethal_at` guard checks from core living avatar mechanics.
- Extracted `ShaftLook` (`crates/client/src/orientation/mod.rs`) as a `ViewOrientation` decorator between `SensitivityLook` and a 2-field `MouseLook`.

## [0.75.1] - 2026-11-16

### Fixed
- Fixed getting trapped inside a newly placed block when looking down in a 1-block-wide vertical shaft, falling through the solid shaft floor into the lethal core / teleporting onto the highest rim or spawn block when digging out, and sluggish post-respawn movement in multiplayer by introducing the `PlacementTarget` decorator `StanceClearance` (`crates/world/src/relief/sight_ray/clearance.rs`) and `StepCrossing` (`crates/world/src/relief/sight_ray/contact.rs`) to reject underfoot or body-overlapping placements without stacking phantom blocks onto the shaft rim, clamping pitch immediately upon entering `ShoulderArc::Wedged` (`MouseLook::constrain_shoulders` in `crates/client/src/orientation/mod.rs` and `CrustEnvelope::axis_wedged` in `crates/client/src/envelope/crust.rs`), gating 1x1 rim bridging in `ShaftColumn::support_height` (`crates/world/src/relief/column_clearance/decorators.rs`) on `foot_elev >= rim_h - BLOCK_STEP`, and clearing held `DigOperation` / `KeyMotion` triggers across blackout respawns and modal transitions (`crates/client/src/avatar/planetary.rs`, `crates/client/src/session/stage.rs`, and `crates/client/src/session/mod.rs`).

### Changed
- Split `VehiclePilot` (`crates/client/src/drive/pilot.rs`) from `DrivenCar` (`crates/client/src/drive/mod.rs`) and `orientation` unit tests (`crates/client/src/orientation/tests.rs`) from `MouseLook` (`crates/client/src/orientation/mod.rs`), decoupled `[profile.dev]` (`opt-level = 1` for fast live terrain meshing) from `[profile.test]` (`opt-level = 0` for ~1s incremental test builds) in `Cargo.toml`, pre-sized 262k-quad terrain upload buffers in `SessionDisplay` (`crates/client/src/session/display.rs`), and documented the Reproduce-First `--offline` rule in `AGENTS.md`.

## [0.75.0] - 2026-11-15

### Added
- Added quantized 120 Hz monotonic world heartbeat (`EpochPulse`, `PulseBeat`, `PulseCadence`, `ElasticHorizon`, and `WakeGate` in `crates/client/src/session/pulse/`), sub-tick pedestrian camera interpolation (`BodySpan` in `crates/client/src/avatar/span.rs`), elastic far-horizon wildlife visibility (`SightCone::scaled` in `crates/fauna/src/lib.rs`), reusable actor scratch batches (`CpuMeshBatch::clear`, `CpuLineBatch::clear`, `CpuTronBatch::clear` in `crates/engine/src/`), and kernel `ControlFlow::WaitUntil` + input-urgent waking (`Session::pace_event_loop` and `PlatformSurface::present_frame` / `request_redraw`) so office laptops and high-end GPUs share identical simulation speed with late-latched gaze and zero idle spin.

### Fixed
- Fixed supersonic wildlife flight, accelerated avatar physics, and P2P `GuestWard` anti-cheat/timeout drift on >240 FPS hardware by replacing frame-clamped floating-point delta summation with exact rational nanosecond ticks since `epoch`, measuring active pre-swap hardware work duration before `swap_buffers`, and configuring `SwapInterval::Wait(1)` on `GlPlatformSurface`.

### Removed
- Removed legacy `FramePace` (`crates/client/src/session/display.rs`), duplicate `dt.clamp(1.0 / 240.0, 0.10)` and procedural `(total_dt * 45.0).ceil()` substepping (`crates/client/src/session/stage.rs`), busy-spinning `ControlFlow::Poll` (`crates/client/src/main.rs`), unconditional `window.request_redraw()` inside `GlPlatformSurface::render_frame` (`crates/client/src/surface.rs`), per-frame actor `Vec` heap reallocations in `SessionDisplay::render_scene`, boolean `directional` branching in `SightCone`, and `-er`-suffixed `DiurnalChronometer` (`crates/client/src/diurnal_pipeline.rs`).

## [0.74.0] - 2026-11-15

### Added
- Added `WardedPlanet` (`crates/sanctuary/src/warded_planet.rs`) and `ChronicledPlanet` / `FactJournal` (`crates/sanctuary/src/chronicled_planet.rs`) `PlanetaryWorld` decorators, replacing procedural `match fact` anti-cheat branching in `SanctuaryHost` and duplicated `WorldFact` synthesis across `HostedPlanet` and `RemotePlanet` with composable runtime delegation.
- Added `ModifiedSlices` (`crates/world/src/planet/slices.rs`), `PlanetaryWorld::pending_slices`, and `PlanetaryWorld::drain_modified_slice` so concurrent local and remote world mutations across multiple terrain and flora slices queue in a unified bitmask and upload together in `SessionDisplay::refresh_modified_face` (`crates/client/src/session/display.rs`).

### Fixed
- Fixed simultaneous P2P excavation across `HostedPlanet` and `RemotePlanet` by replacing the global single-cell `DigStageFocus` with per-cell `ExcavationCracks` and self-healing `LayerCrack` leases (`crates/world/src/relief/turf_scars.rs`), broadcasting concurrent per-author `PeerCrack` notices (`crates/sanctuary/src/cohort.rs` and `crates/sanctuary/src/envelope.rs`) without echoing a guest's own stale crack stages back onto itself, eliminating `||` short-circuit starvation of `PlanetaryWorld::step_surface` during active digging in `SessionStage::step_frame` (`crates/client/src/session/stage.rs`), and gating tripod instant-pickup probes on `PlanetaryWorld::placed_camera_tripod` (`crates/client/src/earthworks/reach.rs`).

### Removed
- Removed legacy single-cell `DigStageFocus`, `ReliefElevation::clear_dig_stage`, `(i32, i32)` previous-cell tuple plumbing across `ReliefElevation` / `TerracedRelief` / `OverhangMolds` / `ShaftExcavation`, half-born `CrackNotice::Quiet`, and duplicated `Cell<u16>` `DirtySlices` (`crates/sanctuary/src/chronicle_log.rs`).

## [0.73.0] - 2026-11-14

### Added
- Added huntable Temperate wild boar sounder (`BoarSounder`, `WildBoar`, `BoarActivity`, and `BoarFigure` in `crates/fauna/src/wild_boar/`) inhabiting `ClimateZone::Temperate` oak woods with sculpted 3D anatomy (humped shoulder crest, dorsal bristles, snout disc, curved ivory tusks, ears, tail, and cloven hooves), behavioral phases (`Rooting`, `BristledWarning`, `TuskCharge`, `Trotting`, `Fallen`), weapon-scaled melee reach (`HeldRelic::hunting_reach`), sounder threat alerting (`WildBoar::alert_to_threat`), tusk knockback, dark-crimson first-person shock vignette with FOV tunnel-vision and vertigo sway (`VitalStrain` in `crates/client/src/posture/vital.rs`), 3D spatialized guttural wild-boar vocalizations (`BoarSnort` in `crates/acoustics/src/boar_snort.rs`), and `SurfaceFind::BoarMeat` / `HeldRelic::BoarMeat` drops that heal bruises and grant a temporary sprint vigor boost when eaten (`HeartyBite` in `crates/acoustics/src/hearty_bite.rs`).
- Added camera `SightCone` view-cone and distance culling (`crates/fauna/src/lib.rs`) for woodland sparrows, roe deer, and wild boars.

### Changed
- Extracted standalone `acoustics` (`crates/acoustics`) and `fauna` (`crates/fauna`) workspace crates out of `crates/client`, replacing `crate::cardiac` and scattered `#[cfg(test)]` blocks with `SoundStage` live-playback gating (`SoundStage::live().activate()`), centralizing 3D stereo distance/ITD RIFF encoding in `SpatialStage` (`crates/acoustics/src/spatial.rs`), caching Linux audio backends with non-zombie child reaping, eliminating `HeldRelic` / `SatchelTray` magic-number duplication via `HeldRelic::COUNT` and `HeldRelic::TRAY_ORDER`, refactoring `SessionStage` / `SessionDisplay` / `PortalInvite` to strict "Tell, Don't Ask", and extending `SourceFileAudit` (`crates/client/src/architecture_tests.rs`) across all 6 workspace crates.

## [0.72.0] - 2026-11-14

### Added
- Added European roe deer herd (`RoeHerd`, `RoeDeer`, `RoeActivity`, `TemperateGlade`, `RoeFigure`, and composite `WoodlandFauna` in `crates/client/src/roe_deer/`) native to `ClimateZone::Temperate` oak glades and meadows, featuring sculpted 3D anatomy (snow-white rump mirror, articulated grazing neck, ears, slender cloven-hoofed legs, and forked roebuck antlers), crouch-aware stealth thresholds (`Avatar::step_fauna`), and alternating 3D spatialized stereo bounding hoofbeats with grass rustling (`RoeHoofbeats` in `crates/client/src/cardiac/hoofbeats.rs`) and hoarse laryngeal alarm barks (`RoeBark` in `crates/client/src/cardiac/roe_bark.rs`).

## [0.71.0] - 2026-11-13

### Added
- Added double-tap `Q` / `E` item tossing onto the ground (`TossCadence` in `crates/client/src/earthworks/toss.rs`, `PalmEarthworks::try_cast_relic` in `crates/client/src/earthworks/mod.rs`, and `Avatar::cast_triggered_relic`) so single-tapping `Q` / `E` continues to stow held items into `BeltSatchel` while double-tapping within `0.38s` casts any held block, ore, mechanical part, camera, tool, or weapon onto the planetary floor for later click-to-retrieve pickup (`SurfaceFind`, `FloorLitter`, `GroundToolFigure`, `LitterLanding` in `crates/world/src/tree/`, and P2P `WorldFact::LitterCast` sync across `crates/world/src/chronicle.rs` and `crates/sanctuary/src/`).

## [0.70.0] - 2026-11-13

### Changed
- Grouped the two `CO-OP` actions (`OPEN CO-OP PORTAL` / `COPY INVITE TICKET` and `JOIN CO-OP / SERVER` / `PUNCH NAT / JOIN`) into a dedicated `MULTIPLAYER` submenu (`ModalPhase::MultiplayerHub`, `PortalLayout::emit_multiplayer_hub`, and `PortalStudio::press_hub_pointer` in `crates/client/src/menu/portal/`) and positioned the `MULTIPLAYER` button directly above `QUIT GAME` on the Main Menu (`PausePanel` in `crates/client/src/menu/pause_panel.rs`).

## [0.69.0] - 2026-11-13

### Added
- Added bottom-right compile-time release version watermark (`VersionStamp` in `crates/client/src/menu/version_stamp.rs`, `PixelStamp::emit_bottom_right` in `crates/client/src/menu/glyph.rs`, and `StudioPanels::emit_ui` in `crates/client/src/menu/escape_menu/panels.rs`) rendered with a 1-pixel contrast shadow across all open Escape menu views while remaining hidden during gameplay.

## [0.68.1] - 2026-11-13

### Fixed
- Upgraded `PixelStamp` (`crates/client/src/menu/glyph.rs`) from a 3x5 bitmap (where `'S'` and `'5'` shared an identical pixel mask) to a 5x7 glyph matrix with distinct silhouettes across all 36 alphanumeric characters, title-scale portal code rendering in `PortalLayout` (`crates/client/src/menu/portal/layout.rs`), automatic `XXXX-XXXX-XXXX` 3x4-block input grouping in `PortalDraft` (`crates/client/src/menu/portal/draft.rs`), `'U' -> 'V'` Crockford normalization and `CODE:` prefix stripping in `CrockfordStamp` (`crates/sanctuary/src/wormhole/crockford.rs`), unspecified-IP fallback in `PortalCode::from_endpoint` (`crates/sanctuary/src/wormhole/phrase.rs`), and `logical_key` text fallback in `GameSession` (`crates/client/src/session/mod.rs`).

## [0.68.0] - 2026-11-13

### Added
- Added serverless 12-character P2P Co-op Portal & FRITZ!Box NAT Counter-Punch codes (`XXXX-XXXX-XXXX`, e.g. `K7M9-P2X4-R8W3`) authenticated via RFC 9382 SPAKE2 over Ristretto255 + ChaCha20-Poly1305 (`CrockfordStamp`, `PortalCode`, `PunchCode`, `WormholePledge`, `WormholeGate` in `crates/sanctuary/src/wormhole/`), combined with cross-platform pure-Rust RFC 5389 STUN and local router UPnP IGD / NAT-PMP discovery (`NatAperture`, `StunProbe`, `GatewayLease` in `crates/sanctuary/src/nat/`) and live non-blocking Ping-Pong NAT hole-punching (`PortalDial` in `crates/client/src/portal_config.rs`, `PortalStudio` & `PortalLayout` in `crates/client/src/menu/portal/`).

### Removed
- Removed `RelayBeacon`, `BeaconRoute`, `--relay`, `--relay-only`, and `LOWCUBE_RELAY` across `crates/sanctuary`, `crates/client`, and `server.toml` so multiplayer operates 100% peer-to-peer without any intermediate relay server.

### Fixed
- Replaced external CLI-only Linux clipboard shelling (`wl-copy`/`xclip`/`xsel`) with in-process Wayland display seat binding (`WaylandClipboard` via `smithay-clipboard` attached to `GlPlatformSurface`) and persistent X11/data-control selection ownership (`SelectionClipboard` via `arboard`), keeping `PipeClipboard` and `ScratchClipboard` as fallbacks (`crates/client/src/menu/clipboard/`) and reporting accurate clipboard state in `PortalStudio` (`crates/client/src/menu/portal/mod.rs`).

## [0.67.0] - 2026-11-13

### Added
- Added in-game `JOIN CO-OP / SERVER` studio (`PortalStudio` in `crates/client/src/menu/portal.rs`, `PausePanel` in `crates/client/src/menu/pause_panel.rs`, and `EscapeMenu` in `crates/client/src/menu/escape_menu.rs`) with cross-platform clipboard copy/paste (`SystemClipboard` in `crates/client/src/menu/clipboard.rs`), keyboard address/ticket entry, live target validation, and hot world-swap connection (`PortalConfig::join_live_portal` and `Avatar::land_on_world`).
- Added zero-flag `server.toml` configuration support (`SanctuaryToml`, `SanctuaryEndpoint`, `SanctuaryPlan` in `crates/sanctuary/src/config.rs`) with `128`-block (`size = 1`) default scale and default `:27777` port resolution on bare hostnames in `JoinTarget::parse` (`crates/sanctuary/src/ticket.rs`).
- Added pre-built standalone dedicated server executables (`low-cube-server-linux` and `low-cube-server-windows.exe`) alongside the game executables in `.github/workflows/release.yml`.

### Changed
- Separated player-facing single-file & server documentation (`README.md`, `docs/MULTIPLAYER.md`, `docs/SETTINGS.md`, `server.toml`) from developer `cargo run` workflows (`docs/DEVELOPMENT.md`).

## [0.66.0] - 2026-11-13

### Added
- Added zero-knowledge `crates/sanctuary` multiplayer crate and standalone server/relay binary (`SanctuaryHost`, `HostedPlanet`, `RemotePlanet`, `RemoteLink`, `ChronicleLog`) decorating `PlanetaryWorld` so singleplayer, live co-op hosting, and remote guest play share identical gameplay logic (`crates/sanctuary/src/`).
- Added pure-Rust end-to-end encryption (`HandshakeIdentity`, `SessionSeal` via X25519 + ChaCha20-Poly1305), shareable `LC1-...` invite tickets (`PortalTicket`), UDP hole-punching with encrypted blind relay fallback (`RelayBeacon`), and authoritative reach/cadence/block-credit anti-cheat (`GuestWard` in `crates/sanctuary/src/ward.rs`).
- Added stratified subterranean mineral veins (`LivingVeins`, `HollowVeins`, `EntropyWell`, `ExposedStratum` in `crates/world/src/relief/mineral_vein/`) with air-exposure X-ray culling, geological elevation bands, finite vein bud propagation, and per-band `16x16` sector saturation, plus sculpted 3D `MineralSeam` nuggets for `CopperOre`, `IronOre`, and `GoldOre` (`crates/world/src/block/mineral_seam.rs`).
- Added `CompanionPresence` (`crates/world/src/presence.rs`), binary `WorldFact` chronicle serialization (`crates/world/src/chronicle.rs`), 3D companion avatar rendering in gameplay and Polaroid exposures (`CompanionFigure` in `crates/client/src/companion.rs`), CLI `--host` / `--join` / `--relay` (`PortalConfig` in `crates/client/src/portal_config.rs`), and a live `OPEN CO-OP PORTAL` button in `EscapeMenu` (`crates/client/src/menu/mod.rs`).
- Added player multiplayer guide (`docs/MULTIPLAYER.md`) and architectural decision record (`docs/adr/0004-zero-knowledge-sanctuary-multiplayer.md`).

### Changed
- Extended `HeldRelic`, `RelicLexicon`, `SlotBurden`, and `ArtisanTome` (`crates/client/src/attire/`) with `CopperOre`, `IronOre`, and `GoldOre` handheld nuggets, weight grades (`MODERATE` / `WEIGHTY` / `HEAVY`), and ore-forged crafting recipes for `SteeringWheel`, `FieldCamera`, `AircraftEngine`, and `RocketEngine`.

## [0.65.2] - 2026-11-13

### Fixed
- Aligned the handheld and showcase cantilever crane geometry (`RelicFigure::emit_cantilever` in `crates/client/src/attire/relic/mesh.rs`) with the placed crane (`RigGeometry::emit`) via `SquarePlinth` and `SlopedBeam` (`crates/client/src/attire/relic/plinth.rs`), replacing the octagonal baseplate, hexagonal collar, and round steel rod with a compact square stone baseplate, square forged-iron pedestal ring, mast/crossbar iron shoes, and a rectangular timber-and-iron sloped knee brace (`crates/client/src/attire/tests/mod.rs`).

## [0.65.1] - 2026-11-13

### Fixed
- Suspended `HangingSack` (`crates/client/src/attire/sack.rs`) from the top-right slot corner via a clearly visible taut hemp cord (`corner_hitch -> cinch_knot`) with an outward-tilted mouth tuft so the cord is never occluded, and aligned its octagonal left flank (`SLOT_WALL_X = 0.1508`) in direct physical contact against the right slot border (`x = 0.1512`) across all burden weights (`crates/client/src/attire/tests/burden.rs`).

## [0.65.0] - 2026-11-13

### Added
- Added intermediate `WeightGrade::Weighty` (`WEIGHTY`) tier between `Moderate` and `Heavy` (`LIGHT -> MODERATE -> WEIGHTY -> HEAVY`) and `WeightGrade::escalated` (`crates/client/src/attire/burden.rs`).
- Added unframed 3D `HangingSack` (`crates/client/src/attire/sack.rs`) hanging directly from the top-right slot corner tip of the `3x5` satchel palette (`SatchelTray::emit_with_burden` in `crates/client/src/attire/tray.rs`), progressively bulging outward to the right and sagging lower along the palette as `SatchelBurden` increases.

### Changed
- Updated `SlotBurden::summed_grade` and `SlotBurden::corner_pigment` (`crates/client/src/attire/burden.rs`) so stacked items in a slot escalate one weight tier every 10 items (`1..=9` base grade, `10..=19` `+1` tier, `20..=29` `+2` tiers, `30+` `+3` tiers), and updated `docs/BLOCKS.md`, `docs/CHARACTER.md`, and unit tests in `crates/client/src/attire/tests/burden.rs`.

## [0.64.0] - 2026-11-12

### Added
- Added composable `AcousticWave` decorators (`InflectedWave`, `CrumbledWave`, and `CadencedWave` in `crates/client/src/cardiac/dig_decorators.rs`) that warp pitch/duration, overlay delayed soil-clod crumble textures, and cycle across 8 distinct excavation contours while keeping `PalmScrape` and `SpadeStrike` untouched.
- Added polymorphic `StrokeResonance` interface (`crates/client/src/earthworks/debris.rs`) implemented for `HeldRelic` plus `#[cfg(test)] DigWaveTrace` and unit test `test_sustained_digging_varies_palm_and_spade_sound_via_decorators` (`crates/client/src/earthworks_tests.rs`).

### Changed
- Updated `DigOperation` (`crates/client/src/earthworks/debris.rs`), `SurfaceReach` (`crates/client/src/earthworks/reach.rs`), and `PalmEarthworks` (`crates/client/src/earthworks/mod.rs`) to advance stroke cadence across sustained digging without boolean tool flags or raw progress getters.

## [0.63.0] - 2026-11-12

### Added
- Added 3-tier English item and block weight system (`WeightGrade::Light`, `Moderate`, `Heavy`), stack weight summation (`SlotBurden`), composable `CarryLoad` / `JoinedLoad` mass folding, and total carried inventory burden (`SatchelBurden`) in `crates/client/src/attire/burden.rs`, plus unit tests in `crates/client/src/attire/tests/burden.rs`.
- Added composable `SlotPlaque` compartment decorators (`CompartmentBackplate`, `WeightedCompartment`, `ShowcasedCompartment`, `TalliedCompartment`, `LegendCompartment` in `crates/client/src/attire/compartment.rs`) and `LimbFrame::projected_toward` (`crates/client/src/attire/limb.rs`) rendering an unframed weight-colored backplate corner triangle behind the 3D item and a white foreground stack count badge.
- Added locomotion and jump encumbrance scaling (`SatchelBurden::scale_pace`, `SatchelBurden::scale_leap`, `GravitationalFlight::jump_burdened` in `crates/client/src/flight.rs`, `order.rs`, `avatar/body.rs`, and `avatar/planetary.rs`) so carrying heavier blocks and tools across hands and stowed slots progressively reduces movement speed and peak jump height.
- Documented all 21 block and item weight grades in `docs/BLOCKS.md` and `docs/CHARACTER.md`, and recorded remaining Elegant Objects refactoring steps in `YEGOR_CHECKLIST.md`.

### Changed
- Updated `SatchelTray` (`crates/client/src/attire/tray.rs`) and `BeltSatchel::transfer_hand` (`crates/client/src/attire/pouch/mod.rs`) to always display all 15 satchel compartments (`3x5` grid) whether occupied or empty, support free `WASD` cursor navigation across empty and occupied slots, and stow held items directly into unoccupied slots.

## [0.62.0] - 2026-11-12

### Added
- Added pending seat-perch grip flags (`DigGrip::with_pending_perch`, `pending_perch_any`, `pending_perch_on`, `pending_click_any`, `pending_click_on` in `crates/client/src/grip.rs`), `DigOperation::ready_to_perch` (`crates/client/src/earthworks/debris.rs`), and `DrivenCar::seat_intact` (`crates/client/src/drive.rs`) so holding down a mouse button on a `CabinSeat` dismantles the seat—both while standing and while seated looking down—and automatically transitions `VehiclePilot::Perch` back to `Pedestrian`, whereas a short click perches on the seat.
- Added `SurfaceReach` (`crates/client/src/earthworks/reach.rs`) to encapsulate 3D eye ray, view direction, stance origin, and perch eligibility for surface strikes, sustained excavation, and release actions, plus `#[cfg(test)] HandGesture` (`crates/client/src/avatar/gesture.rs`) and polymorphic `CodebaseAudit` (`SourceFileAudit`, `SourceTreeAudit` in `crates/client/src/architecture_tests.rs`).

### Changed
- **Pure OOP Modularization (Elegant Objects)**:
  - Decomposed `crates/client/src/avatar/` into focused collaborators (`mod.rs`, `planetary.rs`, `body.rs`, `sight.rs`, `bearing_gear.rs`, `gesture.rs`) with a clean 4-field primary constructor on `PlanetaryAvatar`.
  - Extracted `SatchelIntake` (`crates/client/src/attire/pouch/intake.rs`) from `BeltSatchel` (`crates/client/src/attire/pouch/mod.rs`) and split `crane_drive_tests.rs` out of `posture_tests.rs`.
  - Split `crates/world/src/block/{earth_clods,mold,timber_grain,vehicle_parts}` into dedicated submodule directories (`earth_clods/patterns.rs`, `mold/archetype.rs`, `timber_grain/medium_shape.rs`, `vehicle_parts/steering.rs` with `SteeringColumn`) and extracted subterranean shaft and multi-story tests into `crates/world/src/relief/shaft_story_tests.rs` so every source file across the workspace stays well below the 300-line ceiling.

## [0.61.4] - 2026-11-12

### Added
- Added the `TurfCoat` `VoxelShape` decorator and immutable `TurfSod` biological state (`crates/world/src/block/elevated_shape/coat.rs` and `sod.rs`), plus `SodColumn` (`crates/world/src/relief/molded_structure/sod.rs`) and the `TurfHabitat` / `CanopyHabitat(NaturalHabitat)` 3D turf-contact decorator chain (`crates/world/src/relief/turf_habitat.rs`) so static `WorldBlocks` earth blocks start bare on placement and only regrow grass through real `±0.5m` 3D neighbor contact with living turf.
- Added `CachedCell` (`crates/world/src/relief/face/cache.rs`) to partition cached planetary ground quads from molded overhang quads per cell and `ReliefElevation::ground_bare_at` (`crates/world/src/relief/elevation.rs` and `facet.rs`) to distinguish scarred or smothered ground from overhanging blocks.

### Changed
- Split `crates/world/src/block/elevated_shape.rs` into `crates/world/src/block/elevated_shape/{mod.rs,coat.rs,sod.rs,niche.rs}` and reduced `ElevatedShape` to a pure 2-field spatial decorator (`shape`, `elevation_offset`) where uncoated `VoxelShape` soil is bare by nature (`!self.shape.verdant_turf()`).

### Fixed
- Fixed earth blocks (`BlockMedium::TurfEarth`) placed on cantilever crane arms (`CraftCluster`), driven vehicles (`VehicleAssembly`), and parked cars (`ParkedFleet`) rendering with an immediate green grass cap and grass tufts; all harvested and placed earth blocks now always start bare and never grow grass on cranes or vehicles.
- Fixed floating cranes, bridges, and parked cars stripping 3D meadow tufts from unscarred planetary ground underneath (`CachedCell`), unexcavated smothered ground rendering green instead of `EarthClods::LOAM_CROWN` (`ReliefFacet`), and crane dismantling (`CraftCluster::dismantle_into`) dropping remaining non-engine blocks onto per-cell ground height as bare `TurfCoat` soil.

## [0.61.3] - 2026-11-12

### Fixed
- Advanced the cantilever crane boom (`RigArm::advance_toward_craft`, `RigArm::reset_reach`, `CantileverRig::remove_craft_layer`, `RigRegistry::remove_craft_layer` in `crates/world/src/relief/cantilever_rig/`) forward to the next remaining craft block along the boom axis whenever the block directly gripped by the crane arm is removed, and refreshed the `4`-cell neighborhood in `FacetedReliefFace::harvest` and `FacetedReliefFace::place_molded_block` (`crates/world/src/relief/face/mod.rs`) so mast geometry updates immediately.
- Expanded the ground stone pedestal in `RigGeometry::emit` (`crates/world/src/relief/cantilever_rig/mesh.rs`) to span the full `0.5m x 0.5m` cell bounds (`cell.u_bounds()` and `cell.v_bounds()`).

## [0.61.2] - 2026-11-11

### Changed
- Parameterized `MediumShape<S = Box<dyn VoxelShape>>` (`crates/world/src/block/timber_grain.rs` and `crates/world/src/block/shape.rs`) over generic `S: VoxelShape` to eliminate double-boxing in `BlockMold::shape_for` and execute `BlockMold::emit_miniature` (`crates/world/src/block/mold.rs`) entirely on the stack with zero heap allocations.
- Cached `lod = 0` meadow tuft quads per cell (`cached_tufts`) and replaced the hardcoded 81-cell (`9x9`) invalidation window with action-scoped radii (`0`, `1`, `4`) in `ReliefCellCache` and `FacetedReliefFace` (`crates/world/src/relief/face/cache.rs` and `mod.rs`), while gating `SessionDisplay::refresh_modified_and_meadow` (`crates/client/src/session/mod.rs`) on actual world modification.
- Precomputed the `6x6` `depth_tier_at` grid on the stack in `MeadowCrown::emit_contour_terraces`, skipped `depth_tier_at` on distant `lod = 0` tufts (`crates/world/src/mesh_target/crown.rs` and `mod.rs`), skipped clod/terrace/tuft emission on actively cratered blocks (`stage > 0`) in `ElevatedShape::emit_terraced` (`crates/world/src/block/elevated_shape.rs`), and removed 51 buried inner clod quads per dirt block in `EarthClods` (`crates/world/src/block/earth_clods.rs`).
- Optimized `FRAGMENT_SHADER_SRC` (`crates/engine/src/shader.rs`) with edge-gated neighbor `contour_tier` sampling and early `discard` on faded grass blades, added `CpuMeshBatch::with_quad_capacity` (`crates/engine/src/buffer.rs` and `crates/client/src/session/display.rs`), and switched single-frame `CpuMeshBatch::draw` and `CpuLineBatch::draw` uploads to `glow::STREAM_DRAW`.

## [0.61.1] - 2026-11-11

### Removed
- Removed all legacy remnants of the spherical magma core and hollow-world weightlessness concept: deleted `MagmaCore` (`crates/world/src/relief/magma_core.rs`), `InvertedShape` (`crates/world/src/block/inverted_shape.rs`), and `hollow_world_tests.rs` (`crates/world/src/relief/hollow_world_tests.rs`).
- Removed hollow-dome ceiling geometry (`VaultCeilingTarget`, `emit_ceiling`, `emit_skirt`, `emit_upward_shaft`), upward crust-breach tracking (`ceilings`, `upward_at`, `breached_at`, `ContactFace::CrustCeiling`), and weightlessness queries (`weightless_at`) across `CrustVault`, `VaultGeometry`, `ShaftExcavation`, `ColumnClearance`, `SurfaceSightRay`, `TerracedRelief`, `FacetedReliefFace`, `PlanetaryWorld`, and `Face` while preserving the 4-layer `BlockMedium::CrustRock` crust and indestructible lethal core floor (`floor_steps_at`, `emit_crust_floor`, `emit_crust_flank`, `emit_crust_sole`).
- Removed `WeightlessTrajectory`, `weightless_ascent`, `in_upward_shaft`, `LeapIntent::Descend`, `calm_beneath_crust`, `GravitationalFlight::descend`, `VitalStrain::advance_core_hover`, `BodyPose::with_hover_pose`, and `StudioAnimation::WeightlessHover` across `crates/client` while preserving the lethal core-floor heart-attack blackout and surface respawn sequence (`VitalStrain::perish`, `advance_perish`, `MonitoredPulse::clinical`).

## [0.61.0] - 2026-11-10

### Added
- Added `CraftCluster` and `ParkedFleet` (`crates/world/src/relief/craft_cluster/mod.rs` and `fleet.rs`) so everything built onto a `CantileverRig` is encapsulated as an independent multi-block object separate from World-Cube blocks, allowing any block medium on both World-Cube buildings and crane crafts without turning decorative wheels, seats, or steering wheels on houses into vehicles.
- Added the `MoldedStructure` trait, strictly private `VoxelCells`, and the `RiggedStructure(ParkedStructure(&WorldBlocks))` decorator chain (`crates/world/src/relief/molded_structure/mod.rs`, `cells.rs`, and `decorators.rs`) so support, collision, raycast hits, perching, and mesh emission compose polymorphically in `OverhangMolds` (`crates/world/src/relief/overhang_molds.rs`).

### Changed
- Updated `CantileverRig`, `RigArm`, and `RigRegistry` (`crates/world/src/relief/cantilever_rig/mod.rs`, `arm.rs`, and `registry.rs`) to accept any initial block medium on an empty crane, forbid sitting when no `CabinSeat` is installed in the crane object, and keep the object attached to the crane whenever no `SteeringWheel` is installed.
- Removed the obsolete `"ON CRANE FIRST"` steering-wheel notice (`TrayState::SteeringNotice`) from `SatchelTray` (`crates/client/src/attire/tray.rs`) and `BeltSatchel` (`crates/client/src/attire/pouch/mod.rs`), and restored `ControlAction::MoldForm` quarter-turn palm rotation (`HeldRelic::turnable_in_palm`, `BeltSatchel::turn_mold`) in `PalmEarthworks::handle_trigger` (`crates/client/src/earthworks/mod.rs`).

## [0.60.0] - 2026-11-10

### Added
- Added 5-tier 3D contour grass terraces (`MeadowCrown::emit_contour_terraces`, `MeadowCrown::depth_tier_at`, `MeadowCrown::tier_style` in `crates/world/src/mesh_target/crown.rs`), overhanging 3D green turf rims and stepped sod fringes (`EarthClods::emit_turf_flank`, `EarthClods::emit_wall_turf`, `TURF_FRINGE_PATTERN` in `crates/world/src/block/earth_clods.rs`), and distance-graded local meadow emission (`NaturalMeadowTarget::with_lod`, `ReliefCellCache::emit_meadow_around` in `crates/world/src/mesh_target/mod.rs` and `crates/world/src/relief/face/cache.rs`) paired with 30°-splayed 3D grass tufts (`emit_splayed_cross_fan`, `emit_hybrid`, `emit_natural_patch`) across both placed verdant earth blocks and the World-Cube surface.
- Added world-space 5-tier contour terrace shading (`contour_tier`, `tier_scale`) in `FRAGMENT_SHADER_SRC` (`crates/engine/src/shader.rs`) and `glow = 0.02` verdant surface tagging in `ReliefCorner`, `ReliefFacet`, `BlockMedium::emit_box`, and `TurfMeshTarget` so multi-depth grass contour lines continue seamlessly to the planetary horizon at zero extra polygon cost.

### Changed
- Split `crates/world/src/mesh_target.rs` into `crates/world/src/mesh_target/mod.rs` and `crates/world/src/mesh_target/crown.rs` and unified soil/turf clod geometry in `EarthClods` (`crates/world/src/block/earth_clods.rs`) and `ReliefWall` (`crates/world/src/relief/wall.rs`).

## [0.59.3] - 2026-11-10

### Changed
- Updated `SparrowChirp::spatial`, `SparrowChirp::channel_gains`, and `SparrowChirp::stereo_riff_wave` (`crates/client/src/cardiac/birdsong.rs`) along with `SparrowFlock::step_spatial` (`crates/client/src/sparrow/mod.rs`) and `PlanetaryAvatar::step_fauna` (`crates/client/src/avatar/mod.rs`) so woodland sparrow chirps attenuate with 3D listener distance, pan across left/right stereo channels with interaural delay according to the listener's gaze direction, vary in pitch per bird, and use a longer variable cooldown (`4.2s–8.3s`).

## [0.59.2] - 2026-11-10

### Fixed
- Fixed `VehicleAssembly::wheel_ground_lift`, `VehicleAssembly::park_into_blocks`, `VehiclePart::park_into`, `VehiclePart::wheel_floor_drop` (`crates/world/src/relief/vehicle_assembly/mod.rs` and `part.rs`), `CantileverRig::adjust_assembly` (`crates/world/src/relief/cantilever_rig/mod.rs`), and `ElevatedShape::wheel_ground_lift` (`crates/world/src/block/elevated_shape.rs`) so the entire vehicle assembly is elevated above ground by the full lugged wheel clearance while driving, parked, or lowered on a crane instead of sinking the lowered wheel hubs into the terrain.
- Fixed `ContactFace::classify` (`crates/world/src/relief/sight_ray/contact.rs`), `CantileverRig::retract_for_placed_block`, `CantileverRig::connected_cluster` (`crates/world/src/relief/cantilever_rig/mod.rs`), and `RigRegistry::hit_top` (`crates/world/src/relief/cantilever_rig/registry.rs`) so lateral block placement against an underslung pedal cluster beneath a hoisted steering wheel attaches wheels at the pedal layer and retracts the crane arm cleanly instead of triggering the crane steering-wheel notice.

## [0.59.1] - 2026-11-10

### Fixed
- Fixed `AvatarBody::interact_vehicle` and `AvatarBody::step_driving` (`crates/client/src/avatar/body_sight.rs`) so head yaw rotation remains constrained to `±1.42 rad` (`constrain_shoulders(true, 1.42)`) whenever the avatar is seated with an immobilized torso (`VehiclePilot::Perch` as well as `VehiclePilot::Driving`), instead of unlocking 360-degree head rotation upon releasing the steering wheel.

## [0.59.0] - 2026-11-10

### Added
- Added `SparrowFlock`, `Sparrow`, `SparrowActivity`, and `SparrowFigure` (`crates/client/src/sparrow/mod.rs`, `crates/client/src/sparrow/activity.rs`, and `crates/client/src/sparrow/mesh.rs`) for 3D sculpted house sparrows that fly between clearings and trees, peck seeds on the ground, perch in tree crowns, and flee away when a player approaches within `4.5m`.
- Added `SparrowChirp` (`crates/client/src/cardiac/birdsong.rs`) implementing `AcousticWave` to synthesize 24 kHz three-syllable woodland birdsong chirps.
- Added `TreeSpecies::perch_offset` (`crates/world/src/tree/species.rs`), `Arboreal::crown_perch` (`crates/world/src/tree/arboreal.rs` and `standing.rs`), and `PlanetaryWorld::arboreal_perch_near` / `Face::arboreal_perch_near` (`crates/world/src/lib.rs` and `crates/world/src/tree/forested_face.rs`) so sparrows discover standing tree crown perches across all biomes and take flight when a perched tree is felled.

### Changed
- Split `SessionStage` and `StageOverlay` into `crates/client/src/session/stage.rs` to orchestrate per-frame sparrow flock simulation (`Avatar::step_fauna`) and dynamic mesh emission alongside the player avatar.

## [0.58.0] - 2026-11-10

### Added
- Added stack count badges to each stowed slot in `SatchelTray::emit_on_waist` (`crates/client/src/attire/tray.rs`) and stack count headers (`X<count>`) in `RelicFigure::emit_legend` (`crates/client/src/attire/relic/mesh.rs`).

### Fixed
- Fixed `BeltSatchel::collect_excavated_block` (`crates/client/src/attire/pouch/mod.rs`) and `DigGrip::bare_hand_digging` (`crates/client/src/grip.rs`) so excavating a block with a hand that already holds a placeable block keeps the excavated block in the active hand (stowing extra copies in the satchel) instead of placing it into the opposite empty hand.

## [0.57.1] - 2026-11-10

### Changed
- Updated `SatchelTray::emit_on_waist` (`crates/client/src/attire/tray.rs`) to display a concise English crane notice (`STEERING WHEEL` / `ON CRANE FIRST`) accompanied by the 3D `HeldRelic::SteeringWheel` block icon.
- Added `cross_axle:4` (`HeldRelic::CrossAxle`) to the default starting inventory in `server.toml`.
- Updated `VehicleAssembly::board_from_blocks` and `VehicleAssembly::nearest_seat` (`crates/world/src/relief/vehicle_assembly/mod.rs`) so clicking any block of a connected vehicle assembly from outside seats the player on its nearest `CabinSeat`, and split `vehicle_assembly` into `mod.rs` and `part.rs`.

### Fixed
- Fixed `LoosenedPartTarget` shrinking `PedalCluster` to 86% scale after block placement by resetting probe dig stages in `PalmEarthworks::act_with_hand` (`crates/client/src/earthworks/mod.rs`) and `OverhangMolds::stack_molded` (`crates/world/src/relief/overhang_molds.rs`).
- Fixed clicking a `CantileverRig` while seated in a ground vehicle (`VehiclePilot::Perch`) re-grabbing the steering wheel instead of hoisting the vehicle onto the crane by prioritizing `RigRegistry::find_aimed_rig` and `RigRegistry::reconnect_seated_car` (`crates/world/src/relief/cantilever_rig/registry.rs` and `crates/world/src/relief/vehicle_assembly/mod.rs`).
- Fixed inverted steering wheel rotation direction in `VehicleBlockMesh::emit_steering_wheel` (`crates/world/src/block/vehicle_parts.rs`).
- Fixed equipping and using handheld items (such as `FieldCamera`) while seated in `VehiclePilot::Perch` by returning `None` from `AvatarBody::interact_vehicle` (`crates/client/src/avatar/body_sight.rs`) when no steering wheel or crane is hit and clearing held movement triggers in `SessionStage::dispatch_avatar_or_control` (`crates/client/src/session/display.rs`).

## [0.57.0] - 2026-11-10

### Added
- Added `BlockMedium::CrossAxle` (`crates/world/src/block/timber_grain.rs`), `VehicleBlockMesh::emit_cross_axle` (`crates/world/src/block/vehicle_parts.rs`), and `HeldRelic::CrossAxle` (`crates/client/src/attire/relic/mod.rs`, `SatchelTray`, `BeltSatchel`, `TestKitSatchel`, and `ArtisanTome`) for connecting lateral wheels to a vehicle chassis.
- Added `TrayState::SteeringNotice` (`crates/client/src/attire/tray.rs`) and crane placement validation (`OverhangMolds::stack_molded` in `crates/world/src/relief/overhang_molds.rs` and `BeltSatchel::place_from_hand` in `crates/client/src/attire/pouch/mod.rs`) requiring `SteeringWheel` as the first block mounted on an empty `CantileverRig` and displaying a HUD notice when another block is attempted first.
- Added `RigRegistry::cluster_hoisted`, `RigRegistry::find_aimed_rig`, and `RigRegistry::reconnect_seated_car` (`crates/world/src/relief/cantilever_rig/registry.rs`) together with `OverhangMolds::car_locked_at` (`crates/world/src/relief/overhang_molds.rs`) so parked vehicles not connected to a hoisted crane cannot be modified, and clicking a crane while seated reconnects and hoists the vehicle.
- Added `PalmScrape`, `SpadeStrike`, and `EngineRumble` (`crates/client/src/cardiac/work_waves.rs`) implementing `AcousticWave` for distinct bare-hand digging, iron-spade digging, and vehicle driving sounds.

### Changed
- Updated `VehiclePilot` and `DrivenCar` (`crates/client/src/drive.rs`) and `AvatarBody::interact_vehicle` / `AvatarBody::step_driving` (`crates/client/src/avatar/body_sight.rs`) so clicking while driving releases hands from the steering wheel back to the seated perch state, and pressing movement keys (`W`/`A`/`S`/`D`) while seated stands the player up next to the vehicle in the pressed direction.
- Updated `EarthClodfigure` (`crates/world/src/block/earth_clods.rs`) and `LimbFrame::emit_upright_block` (`crates/client/src/attire/relic/mesh.rs`) to apply the full-cube soil clod and pebble relief across all 16 block forms and render handheld dirt blocks with bare soil on top instead of grass.
- Removed handheld block rotation on `WheelClick` (`crates/world/src/block/mold.rs` and `crates/client/src/earthworks_tests.rs`) so block orientation follows player movement and sight direction.
- Expanded `WalnutFloorPan` and `PedalClusterMesh` (`crates/world/src/block/pedal_cluster.rs`) to fill the full `0.5m × 0.5m` cell footprint and raise the rear wall to `y = 0.50m`, and updated `VehicleBlockMesh::emit_wheel` (`crates/world/src/block/vehicle_parts.rs`) to remove the protruding axle, orient the wheel rim toward the player, shift the wheel laterally to dock flush with `CrossAxle`, and lower the wheel hub (`0.11m`) so vehicles sit above the world cube.

## [0.56.1] - 2026-11-08

### Changed
- Changed `VehicleBlockMesh::emit_wheel` (`crates/world/src/block/vehicle_parts.rs`) to emit a single-sided connecting steel axle rod and brass mounting collar oriented toward the adjacent vehicle chassis block instead of protruding from both sides of the wheel.
- Refactored `VehiclePart` and `VehicleAxes` (`crates/world/src/relief/vehicle_assembly.rs`) and `DriveKinematics` (`crates/client/src/drive.rs`) to encapsulate part parking, driven part emission, cardinal grid rotation, yaw stepping, and displacement via message passing ("Tell, Don't Ask").
- Added `pedal_cluster:2` to default starting satchel items in `server.toml`.

### Fixed
- Fixed mid-air `SteeringWheel` mounting onto `CantileverRig` by intercepting sight rays against empty crane boom cells (`CantileverRig::occupies_empty_boom_layer` and `RigRegistry::hit_top` in `crates/world/src/relief/cantilever_rig/mod.rs` and `registry.rs`).
- Fixed wheelless vehicle assemblies altering their block layout on dismount by requiring at least one `CarWheel` before `VehicleAssembly::board_from_blocks` (`crates/world/src/relief/vehicle_assembly.rs`) extracts and lowers blocks when `engage_engine` is triggered, gating chassis yaw rotation on non-zero speed (`DriveKinematics::yaw_step` in `crates/client/src/drive.rs`), rotating parked block shapes by `quarter_turns` (`VehiclePart::park_into`), aligning adjacent crane block placement to the lowest neighbor floor (`CantileverRig::attach_elevation`), and refreshing cell caches around the boarded vehicle seat (`FacetedReliefFace::board_vehicle` in `crates/world/src/relief/face/mod.rs`).

## [0.56.0] - 2026-11-08

### Added
- Added `FootwellPlinth` interface, `WalnutFloorPan`, and `PedalClusterMesh` decorator (`crates/world/src/block/pedal_cluster.rs`) alongside `BlockMedium::PedalCluster` (`crates/world/src/block/timber_grain.rs`) and `HeldRelic::PedalCluster` (`crates/client/src/attire/relic/mod.rs`, `SatchelTray`, `BeltSatchel`, `TestKitSatchel`, and `ArtisanTome`), providing a placeable and craftable gas-and-brake pedal block with an angled firewall toe-board, center transmission tunnel rib, red brake pedal, and green organ-style throttle pedal.
- Added full-width connecting steel axle rods with brass mounting collars to `VehicleBlockMesh::emit_wheel` (`crates/world/src/block/vehicle_parts.rs`) so lateral wheels visibly anchor to the vehicle body instead of floating beside it.
- Added sculpted walnut and leather dashboard cowl with brass trim and three glowing instrument dials to `VehicleBlockMesh::emit_steering_wheel` (`crates/world/src/block/vehicle_parts.rs`).

### Changed
- Extended `CabinSeatMesh::emit` (`crates/world/src/block/cabin_seat.rs`) to emit `WalnutFloorPan` across the full `0.5m × 0.5m` `GridCell` beneath the recessed half-block seat cushion, bridging the lower chassis gap between the front steering column block and the cabin seat while keeping legroom open above the floor pan.
- Added `HeldRelic::blocks_perch` (`crates/client/src/attire/relic/mod.rs`) and updated `AvatarGear::holding_placeable_on` (`crates/client/src/avatar/bearing_gear.rs`) so holding `FieldCamera` still permits sitting on a `CabinSeat` when aiming at the seat cushion.

## [0.55.0] - 2026-11-07

### Added
- Added `NaturalMeadowTarget` decorator (`crates/world/src/mesh_target.rs`) implementing `MeshTarget` and world-anchored natural meadow patch emission (`MeadowCrown::emit_natural_patch`) with precomputed 8-orientation 3-wing star fans (`WING_DIRS`), organic clearings, and tall-grass islands across an expanded 88-cell (`44m` radius / `88m` diameter) horizon meadow (`ReliefCellCache::emit_meadow_around` in `crates/world/src/relief/face/cache.rs`).
- Added `FramePace` and `MonotonicTime` / `SystemClock` (`crates/client/src/session/mod.rs` and `crates/client/src/session/display.rs`) for real-time frame delta stepping (`SessionStage::step_frame`) decoupled from VBO slice refreshes.

### Changed
- Refactored `MeadowCrown` (`crates/world/src/mesh_target.rs`) to Pure OOP / Elegant Objects with a code-free primary constructor (`with_cell`) and zero static methods, coarsened `PlanetaryAvatar::meadow_anchor` (`crates/client/src/avatar/mod.rs`) to a `4.0m` grid, and optimized grass fragment cutout shading (`VoxelShaderSource` in `crates/engine/src/shader.rs`) with trig-free algebraic operations and a `42.0m` distance fade.

### Fixed
- Fixed sun jitter when walking and jumping towards the sun by normalizing the depth row of the view-projection matrix before inversion (`DiurnalPipeline::render` in `crates/client/src/diurnal_pipeline.rs`), computing translation-free ray directions in `FIRMAMENT_VERTEX_SRC` (`crates/client/src/firmament_glsl.rs`), and using `Mat4::look_to_rh` in `CameraHorizon::view_matrix_at` (`crates/client/src/horizon.rs`).

## [0.54.0] - 2026-11-07

### Changed
- Replaced single-blade grass quads with $30^\circ$-splayed 3-wing Cross-Billboard fans (`MeadowCrown::emit_splayed_cross_fan` in `crates/world/src/mesh_target.rs`) and `gl_VertexID`-derived procedural 7-blade silhouette cutout shading (`VoxelShaderSource` in `crates/engine/src/shader.rs`), cutting grass quad count by $3.5\times$ (`18` quads per block / `~2,100` quads in the local $O(1)$ meadow ring) while increasing visible grass density to `126` curved blades per block across 3 fan size variants.

## [0.53.0] - 2026-11-07

### Added
- Added 3-variant 3D low-poly grass blade geometry (`MeadowCrown` in `crates/world/src/mesh_target.rs`) combining tall 2-segment arching blades (`emit_arched_blade`), medium 3-blade fan tufts (`emit_fan_tuft`), and short wide ground sprouts (`emit_broad_sprout`) atop verdant earth blocks (`EarthClods::emit_terraced`, `BlockMold::emit_placed`, `TurfMeshTarget`, and `RelicFigure::emit_in_palm`).
- Added $O(1)$ camera-centered, 3-tier LOD local meadow streaming (`ReliefCellCache::emit_meadow_around` in `crates/world/src/relief/face/cache.rs`) and cell-anchored GPU VBO caching (`SessionDisplay::refresh_meadow_slice` in `crates/client/src/session/display.rs` and `PlanetaryAvatar::meadow_anchor` in `crates/client/src/avatar/mod.rs`) so dense 3D grass renders in constant time independent of world-cube scale (`--size`).
- Added planetary horizon and opposite-hemisphere flora slice culling (`PlanetaryMesh::select_visible` in `crates/engine/src/composite_mesh.rs` and `CubePlanet::visible_slice_mask` in `crates/world/src/planet.rs`) to skip occluded cube faces and distant flora slices during surface traversal.

## [0.52.0] - 2026-11-07

### Added
- Added 3-legged tripod field camera object (`FieldCameraTripod` in `crates/world/src/block/field_camera.rs`) with three splayed wooden legs at $120^\circ$ angles, brass feet, central tripod hub, and 8-way lens rotation (`ForwardAngledSight`, `ProfileSight`, `AngledSight`, `LookingGlassSight`, `LeftAngledSight`, `LeftProfileSight`, `LeftForwardAngledSight` in `crates/client/src/attire/sight.rs` and `MoldArchetype::palm_camera_yaw` / `placed_camera_yaw` in `crates/world/src/block/mold.rs`).
- Added world-space tripod lens projection (`TripodMount::lens_ray` in `crates/client/src/attire/sight.rs` and `AvatarSight::project_lens_view` in `crates/client/src/avatar/body_sight.rs`) so self-timer exposures from a placed camera capture the scene from the placed tripod's actual 3D world position and lens direction while keeping the live player view in first-person (`PlanetaryAvatar::view_projection_matrix`).
- Added single-click portable item pickup (`BlockMedium::instant_pickup` in `crates/world/src/block/timber_grain.rs`, `TerracedRelief::mark_dig_layer`, and `PalmEarthworks::act_with_hand`) so clicking a placed camera picks it up immediately into the player's hand without crack stages or digging animations.

### Changed
- Extracted `CameraExposure` and `PolaroidPrint` into `crates/client/src/photo_plate/exposure.rs`, `PalmStack` into `crates/client/src/attire/pouch/palms.rs`, `NicheColumn` into `crates/world/src/block/elevated_shape.rs`, and `CubePlanet::try_aimed_faces` into `crates/world/src/planet.rs` to keep domain modules well below the 300-line ceiling.
- Restricted `CameraMount` self-timer countdowns (`6` down to `1` in white digits) exclusively to placed cameras (`CameraMount::Placed`) while handheld cameras (`CameraMount::Handheld`) snap immediately on `InputTrigger::WheelClick`.

### Fixed
- Fixed stacked dirt blocks beneath a placed `FieldCamera` losing their horizontal top quads by gating `flank_only` in `OverhangMolds::emit_molded_top` (`crates/world/src/relief/overhang_molds.rs`) on `ElevatedShape::full_cube_envelope`.
- Fixed middle-clicking (`InputTrigger::WheelClick`) on a placed camera rotating the camera via `ControlAction::MoldForm` instead of starting the 6-second self-timer countdown (`PalmEarthworks::try_adjust_rig` and `PalmEarthworks::handle_trigger` in `crates/client/src/earthworks/mod.rs`).

## [0.51.0] - 2026-11-07

### Added
- Added 6-way camera rotation (`SightMode::AngledSight` and `SightMode::LeftAngledSight` alongside `ProfileSight` and `LeftProfileSight` in `crates/client/src/attire/sight.rs`, `BlockMold::next_camera_turn` in `crates/world/src/block/mold.rs`, and `RelicFigure::emit_in_palm` in `crates/client/src/attire/relic/mesh.rs`) so the player can take angled (`_angled.png`) as well as side (`_side.png`), front (`_front.png`), and landscape (`_landscape.png`) photos.
- Added block-mountable field camera (`BlockMedium::FieldCamera` in `crates/world/src/block/timber_grain.rs`, `CabinSeatMesh::emit_camera` in `crates/world/src/block/cabin_seat.rs`, and `HeldRelic::FieldCamera` in `crates/client/src/attire/relic/mod.rs`) allowing the player to place the camera onto a block and rotate its direction.
- Added mouse-wheel-click shutter trigger (`InputTrigger::WheelClick`) with an animated 6-second center-screen countdown (`6` down to `1`) where each digit shrinks from large to small before firing the shutter (`CameraExposure` in `crates/client/src/photo_plate.rs` and `AvatarGear::dispatch_trigger` in `crates/client/src/avatar/bearing_gear.rs`).

### Changed
- Shortened the `CabinSeat` block front by half (`FRONT_RECESS = 0.25` in `CabinSeatMesh::seat_bounds`, `crates/world/src/block/cabin_seat.rs`) so the seat occupies a half block and the seated player's legs hang freely in front of the cushion without clipping into the seat.
- Consolidated camera trigger and inventory delegation inside `AvatarGear::dispatch_trigger` (`crates/client/src/avatar/bearing_gear.rs`) and expanded the `AGENTS.md` Domain & Decorator Map for faster navigation.

## [0.50.0] - 2026-11-07

### Added
- Added rotatable hand-held field camera item (`HeldRelic::FieldCamera` and `RelicFigure::emit_camera` in `crates/client/src/attire/relic/`) that captures PNG screenshots to `screenshots/` (`PhotoPlate` in `crates/client/src/photo_plate.rs`) of the landscape in front (`_landscape.png`), the player from the side (`_side.png`), or the player from the front (`_front.png`) depending on camera rotation (`SightMode::from_camera_turn` in `crates/client/src/attire/sight.rs`), including when seated on a `CabinSeat`.
- Added 2D-rotated instant Polaroid print overlay (`PolaroidPrint`, `CameraExposure`, and `SquareScreenTarget` in `crates/client/src/photo_plate.rs`), dynamic camera viewpoint elevation tracking the held camera height (`BodyPose::held_camera_elevation` and `SightMode::view_matrix`), and broadband mechanical shutter acoustics (`ShutterClick` in `crates/client/src/cardiac.rs`).
- Added server-configurable starting inventory provisioning (`StartingSatchel` decorator chain in `crates/client/src/server_satchel.rs` and `server.toml`) supporting `--inventory`, `LOW_CUBE_START_ITEMS`, and `server.toml`.
- Added left/right mouse button item selection and mouse-wheel navigation inside the open item satchel (`PalmEarthworks::act_with_hand` and `PalmEarthworks::handle_trigger` in `crates/client/src/earthworks/mod.rs`).

### Changed
- Decoupled seated posture (`BodyPose::seated_factor` in `VehiclePilot::Perch`) from steering-wheel hand grip (`BodyPose::driving_factor` in `VehiclePilot::Driving`) so sitting on a `CabinSeat` keeps the player's hands free until clicking the `SteeringWheel`, and updated `DrivenCar::dismount_position` (`crates/client/src/drive.rs`) so dismounting places the player in front of the seat rather than on top of it.

## [0.49.0] - 2026-11-06

### Changed
- Renamed `CarEngine` to `SteeringWheel` across `BlockMedium`, `HeldRelic`, `BeltSatchel`, `SatchelTray`, and `VehicleAssembly`, and replaced the engine console mesh with an angled 3D steering column, beveled directional pedestal, 8-segment steering wheel rim with 12-o'clock red marker, 3 chrome spokes, and glowing brass horn hub (`VehicleBlockMesh::emit_steering_wheel` in `crates/world/src/block/vehicle_parts.rs`).
- Redesigned `CabinSeat` (`CabinSeatMesh` in `crates/world/src/block/cabin_seat.rs`) as a richly upholstered flat-square-top seat block restricted to `MoldArchetype::FullCube` (`0.5m`) and `MoldArchetype::HalfSlab` (`0.25m`) via `MoldArchetype::flat_square_top` and `BlockMold` (`crates/world/src/block/mold.rs`), featuring a dark walnut plinth, gold-brass welt trim, padded side bolsters, four 2-tier beveled leather cushion pillows, five glowing brass tufting buttons, and a front thigh-bolster roll with directional brass crest so players can custom-build their own seat and cockpit around the block while keeping it immediately recognizable in the world cube.

### Added
- Added in-game 3D geometry variant sheet `seat_and_steering_variants.png` showcasing `SteeringWheel` (neutral and turned), upholstered `CabinSeat` (`FullCube` and `HalfSlab`), player-designed cockpit assembly, and world-cube terrain recognition.

## [0.48.0] - 2026-11-06

### Added
- Added four selectable 3D seat profiles (`SeatProfile::Bucket`, `ParkBench`, `WorkshopStool`, and `RegalThrone` via `CabinSeatMesh` in `crates/world/src/block/cabin_seat.rs`) usable both on vehicles and anywhere on the planetary world cube.
- Added two-stage vehicle boarding (`VehiclePilot::Perch` and `VehiclePilot::Driving` in `crates/client/src/drive.rs` and `VehicleAssembly::board_from_blocks` in `crates/world/src/relief/vehicle_assembly.rs`) where clicking a `CabinSeat` seats the player while leaving a hoisted car on the crane, and clicking the `CarEngine` steering wheel while seated lowers the crane and enters driving mode.
- Added multi-angle driver proportion turnaround sheet `fahrer_im_auto.png`.

### Fixed
- Fixed mined `CarWheel`, `CabinSeat`, and `CarEngine` blocks vanishing at dig stage 1 and reappearing on mouse release by routing `rig_mountable()` shapes through `LoosenedPartTarget` (`crates/world/src/block/elevated_shape.rs`) so all sculpted facets contract and darken progressively during stages `1..=3` instead of being discarded by `CrateredMeshTarget`'s `min_span < 0.22` soil-clod filter.
- Fixed `PalmEarthworks::act_with_hand` (`crates/client/src/earthworks/mod.rs`) re-arming `w_place` (`pending_place`) while a mouse button remains held during or immediately after harvesting a block, preventing `release_hand` from re-placing a harvested wheel on mouse release.

## [0.47.0] - 2026-11-06

### Added
- Added selectable wheel profiles (`WheelProfile::Compact`, `Touring`, `Offroad`, and `Monster` in `crates/world/src/block/vehicle_parts.rs`) via form switching (`MoldArchetype`), visibly rotating 8-sided polygonal wheels (`emit_driven_part`), and wheel-diameter chassis lift (`VehicleAssembly::wheel_ground_lift`).
- Added human-like seated head look bounds (`anchor_vehicle_heading` and `steer_yaw` in `crates/client/src/orientation.rs` and `crates/client/src/avatar/body_sight.rs`) where mouse look rotates only the driver's head while the torso and legs remain anchored to the car seat.
- Added steering-wheel hand kinematics (`BodyPose::with_seated_steer` and `ArmKinematics` in `crates/client/src/attire/pose/`) and reverse driving after full braking when holding `S` (`DriveKinematics` in `crates/client/src/drive.rs`).
- Added cabin seat backrest lateral placement resolution (`seat_backrest_cell` in `crates/world/src/relief/overhang_molds.rs` and `ContactFace::classify` in `crates/world/src/relief/sight_ray/contact.rs`) so blocks aimed at a seat backrest attach laterally behind the seat instead of stacking on top of the seat.

### Fixed
- Fixed mining of placed `CarWheel` blocks across all wheel profiles by decoupling vehicle voxel raycast bounds (`MediumShape::elevation_at` at `GridCell::SPAN`) from visual wheel radii and allowing `SurfaceSightRay::trace_contact` to hit occupied vehicle blocks in front of the avatar's standing cell.

### Changed
- Optimized workspace incremental build profiles in `Cargo.toml` (`debug = false`, `incremental = true`, dependency-scoped `opt-level = 1`) and expanded the `AGENTS.md` Domain & Decorator Map for direct vehicle and crane navigation.

## [0.46.0] - 2026-11-05

### Added
- Added 4-way rotatable cabin seat (`BlockMedium::CabinSeat` / `HeldRelic::CabinSeat`) and car wheel (`BlockMedium::CarWheel` / `HeldRelic::CarWheel`) blocks (`VehicleBlockMesh` in `crates/world/src/block/vehicle_parts.rs`) with artisan tome recipes and startup satchel stock.
- Added crane lowering and interactive car driving (`VehicleAssembly` in `crates/world/src/relief/vehicle_assembly.rs`, `DrivenCar` and `VehiclePilot` in `crates/client/src/drive.rs`) allowing the player to lower a hoisted car onto the ground by clicking the crane mast and board, accelerate (`W`), steer (`A`/`D`), and brake (`S`/`Space`) by clicking the cabin seat.
- Added seated driving posture and steering kinematics (`BodyPose::with_seated` in `crates/client/src/attire/pose/mod.rs` and `ArmKinematics` in `crates/client/src/attire/pose/arms.rs`) and integrated `StudioAnimation::SeatedDrive` with an interactive animation toggle button into the Character Studio (`crates/client/src/menu/stage.rs`, `crates/client/src/menu/studio/mod.rs`, and `crates/client/src/menu/studio/layout.rs`).

### Changed
- Redesigned `BlockMedium::CarEngine` (`VehicleBlockMesh::emit_car_steering`) as a 4-way rotatable steering wheel and directional front console block that defines the assembled vehicle's forward driving axis (`WedgeHeading::forward_vector`), and removed the steering wheel from `CabinSeat`.
- Evaluated `CrustVault::corners_at` directly from corner drops without boxing `InvertedShape` and deferred `NaturalStrata` grid memoization to mesh generation so `PlanetaryAvatar::step_motion` maintains zero runtime heap allocations.

## [0.45.0] - 2026-11-05

### Added
- Added polyhedral non-block cantilever crane (`CantileverRig`, `RigRegistry`, and `RigGeometry` in `crates/world/src/relief/cantilever_rig/`) with continuous 45-degree diagonal knee brace, auto-retracting telescopic gripper, vertical hoisting, 90-degree vehicle cluster rotation, and artisan tome recipe (`HeldRelic::CantileverCrane`).
- Added three vehicle engine blocks (`CarEngine`, `AircraftEngine`, and `RocketEngine` in `BlockMedium` and `HeldRelic`) that snap into the deployed crane's lateral clamping jaw and dismantle together with the crane when packed up.
- Added unified 3D block model features: gravity-settling `BlockMedium::Sand`, interior horizontal face culling (`FlankMeshTarget` in `crates/world/src/block/elevated_shape.rs`), and concept sheet `hebebuehne_varianten.png`.

### Changed
- Unified subterranean crust floor with the indestructible lethal Magma/Core block floor (`MagmaCore` / `VaultGeometry`), removing the hollow subterranean cavity and weightlessness while preserving multi-story surface building confinement (`MoldedColumn`).
- Seeded player startup gear (`SessionLaunch::assemble` and `BeltSatchel::with_test_engines`) with the cantilever crane in the right hand and all three vehicle engines in the belt satchel for immediate testing.

## [0.44.5] - 2026-11-05

### Added
- Added `VerticalColumn` decorator chain (`NaturalColumn`, `ShaftColumn`, `MoldedColumn` in `crates/world/src/relief/column_clearance/decorators.rs`) and `AerialTrajectory` decorator chain (`ParabolicTrajectory`, `CliffReboundTrajectory`, `WeightlessTrajectory` in `crates/client/src/phase/aerial.rs`).
- Added dedicated decorator objects `VaultGeometry` (`crates/world/src/relief/crust_vault/mesh.rs`), `CrustEnvelope` (`crates/client/src/envelope/crust.rs`), `ArmKinematics` (`crates/client/src/attire/pose/arms.rs`), `RelicFigure` (`crates/client/src/attire/relic/mesh.rs`), and `StudioLayout` (`crates/client/src/menu/studio/layout.rs`).

### Changed
- Replaced split inherent `impl` blocks across files with dedicated decorator objects and extended `test_workspace_architecture_guard_enforces_line_and_field_limits` (`crates/client/src/earthworks_tests.rs`) to forbid cross-file inherent `impl` blocks.
- Short-circuited `ShaftExcavation::in_upward_shaft` and `ColumnClearance::weightless_at` before vault floor evaluation to preserve zero heap allocations during `PlanetaryAvatar::step_motion`.

## [0.44.4] - 2026-11-05

### Added
- Added `ColumnClearance` (`crates/world/src/relief/column_clearance.rs`) and `TurfScars` (`crates/world/src/relief/turf_scars.rs`) to unify 3D vertical column support, subterranean ceiling confinement, weightlessness, ray-hit resolution, and surface turf regrowth.
- Added automated Elegant Objects architecture guard (`test_workspace_architecture_guard_enforces_line_and_field_limits` in `crates/client/src/earthworks_tests.rs`) enforcing the 300-line file limit and 4-field struct limit across all workspace crates.

### Changed
- Modularized terrain relief, avatar kinematics, attire, and session subsystems into focused sub-modules (`relief/sight_ray/`, `relief/crust_vault/`, `relief/face/`, `avatar/`, `envelope/`, `earthworks/`, `attire/relic/`, `attire/rhythm/`, `attire/pose/`, `posture/`, `session/`, and `menu/studio/`).

### Fixed
- Fixed subterranean Dirt ceiling contact during `Space` ascent from the magma core (`ColumnClearance::confine`, `OverhangMolds::confine_niche_elevation`, `ShaftExcavation::excavate_layer`, `AirborneFlight::advance`, `FlightContext::constrain_surface`) so head contact against unbreached upward shaft ceilings or placed niche roofs clamps vertical motion beneath the block without ejecting the avatar to the surface or stripping surface grass.

## [0.44.3] - 2026-11-04

### Changed
- Unified camera viewpoint and sight-ray brow origin in `CameraHorizon::view_matrix_at` (`crates/client/src/horizon.rs`) by delegating directly to `SurfaceSightRay::brow_origin`.
- Updated `RayProbe::step_hit_top` and `ContactFace::classify` (`crates/world/src/relief/sight_ray.rs`) to evaluate full 3D cell-entry coordinates `(entry_u, entry_v, entry_elev)` and attach lateral wall placements at `block_top - CELL_SPAN` instead of the 2D-heightmap column top.
- Updated `DigStageFocus::layer_stage` (`crates/world/src/relief/overhang_molds.rs`) and `ShaftExcavation::confine` (`crates/world/src/relief/shaft_excavation.rs`) to evaluate exact 3D layer indices and overhang niche ceilings without 2D-heightmap top-layer fallbacks.

### Removed
- Removed legacy 2D-heightmap methods across `ReliefElevation`, `TerracedRelief`, `OverhangMolds`, `CrustVault`, `SurfaceSightRay`, `PlanetaryWorld`, and `Face` (`mark_dig_stage`, `excavate_cell`, `stack_cell`, `top_layer_at`, `target_cell`, `excavated_steps`, `place_surface_block`, `place_block`).

## [0.44.2] - 2026-11-04

### Fixed
- Eliminated downward crosshair parallax against close block walls in `SurfaceSightRay::trace_contact` and `RayProbe` (`crates/world/src/relief/sight_ray.rs`) by removing the `0.28m` backward ray-origin retreat and interpolating the exact vertical cell-boundary entry elevation (`RayProbe::step_hit_top`), ensuring the head-height block is targeted until the crosshair crosses below its lower seam.

## [0.44.1] - 2026-11-04

### Fixed
- Restored live palm-molded block shape preview in `BeltSatchel::emit_in_palm` (`crates/client/src/attire/pouch.rs`) while resetting the palm mold to `BlockMold::cube()` whenever a newly excavated block type is equipped into an empty hand or replaces a different block type.

## [0.44.0] - 2026-11-04

### Added
- Added `HeldRelic::CrustBlock` (`crates/client/src/attire/relic.rs`, `SatchelTray` in `crates/client/src/attire/tray.rs`) and polymorphic full-cube crust rock emission (`BlockMedium::emit_crust_rock_cube`, `MediumShape::emit_terraced_shape` in `crates/world/src/block/timber_grain.rs`) so excavated planetary crust layers equip, stow, and render as distinct crust blocks.
- Added atomic "Tell, Don't Ask" excavation and loosening messages returning the affected `Option<BlockMedium>` (`PlanetaryWorld::harvest_surface`, `PlanetaryWorld::loosen_surface`, `Face::harvest`, `Face::loosen`, `ReliefElevation::harvest_layer`, `ReliefElevation::mark_dig_layer`).

### Changed
- Updated `BeltSatchel::collect_excavated_block` and `BeltSatchel::emit_in_palm` (`crates/client/src/attire/pouch.rs`) so excavating a different block medium with a tool in one hand and a placeable block in the other hand stows the previously held block into the satchel, replaces it in-hand with the newly harvested block reset to `BlockMold::cube()`, and always renders palm-held blocks in cuboid form.
- Delegated `OverhangMolds::emit_molded_top` (`crates/world/src/relief/overhang_molds.rs`) polymorphically through `ElevatedShape::emit_terraced` and `MediumShape` instead of inspecting `BlockMedium::CrustRock`.

### Removed
- Removed "Ask, Then Tell" medium query methods (`PlanetaryWorld::surface_medium`, `Face::surface_medium`, `ReliefElevation::layer_medium_at`, `TerracedRelief::layer_medium_at`, `CrustVault::emit_crust_block`).

## [0.43.1] - 2026-11-04

### Changed
- Unified downward, lateral, and upward excavation layer addressing (`ShaftExcavation::crust_hit_top_at`, `ShaftExcavation::diggable_at`, `ShaftExcavation::excavate_layer`, `ShaftExcavation::deepened_steps`, `ContactFace::classify`) so every `0.5m` column block layer `step` is targeted, cracked (`stage = 1..=3`), and excavated via its true midpoint `step * 0.5 + 0.25` regardless of direction.
- Preserved full-width `0.035m` block mortar seams and subterranean glow during active digging (`Quad::emit_crater`), added 3D digging craters to upward shaft ceilings (`ReliefStrata::emit_upward_shaft`), sealed dome-to-shaft ceiling steps (`CrustVault::emit_skirt`), and maintained weightlessness throughout upward shafts (`TerracedRelief::weightless_at`, `ShaftExcavation::in_upward_shaft`).

### Removed
- Removed legacy layerless digging stage methods (`ReliefElevation::dig_stage_at`, `TerracedRelief::dig_stage_at`, `OverhangMolds::dig_stage_at`, `DigStageFocus::ground_stage`).

## [0.43.0] - 2026-11-04

### Added
- Introduced `BlockMedium::CrustRock` and `BlockMedium::debris_tones` (`crates/world/src/block/timber_grain.rs`, `HeldRelic::from_medium`, `DebrisSplash` in `PalmEarthworks`) so excavating planetary crust layers emits dark rock debris and preserves `TurfEarth` identity on player-placed dirt blocks inside crust cavities.
- Added `NaturalCrown` (`crates/world/src/relief/corner.rs`) unifying 3x3 neighborhood slope, ridge, and valley classification across `ZonedRelief::surface_height_at`, `NaturalStrata::crown_shape_at`, and `ReliefFacet::emit_with_vault`.
- Added `Arboreal::settle_onto` (`StandingTree`, `SolidArboreal`, `ClearedPlot`, `FloorLitter`, `ForestedFace::settle_flora`) so trees, fallen boughs, and flint nodules settle onto the newly exposed block when the ground beneath them is excavated.

### Changed
- Replaced whole-row mesh caching in `FacetedReliefFace` with per-cell 3x3 neighborhood caching (`ReliefCellCache`, `FacetedReliefFace::refresh_neighborhood`), rebuilding at most 9 cells per block edit instead of 768 cells across 3 rows.
- Extracted `RayProbe` in `SurfaceSightRay` (`crates/world/src/relief/sight_ray.rs`), added cross-seam target resolution in `CubePlanet::aimed_indices`, and accelerated shaft wall probing in `BodyEnvelope::wall_distance` and `BodyEnvelope::in_upward_shaft`.

### Fixed
- Eliminated residual dirt layers, soil clods, and brown digging craters on top of the 4-layer planetary crust (`CrustVault::crust_top_step_at`, `CrustVault::crust_layer_at`, `CrustVault::crust_surface_at`, `ReliefFacet::emit_with_vault`, `OverhangMolds::retain_overhang`, `OverhangMolds::emit_molded_top`).
- Fixed double-counted `half_size` in `SolidArboreal::constrain_surface` and datum-relative canopy height evaluation in `ForestedFace::canopy_drag`.

### Removed
- Removed 2D-heightmap legacy parameters and unused aliases (`ElevatedShape::ceiling_above`, `ReliefElevation::stack_molded_cell` `_full_flat` parameter, `OverhangMolds::hit_top_at` `ground_h` parameter, `TerracedRelief::pristine`, `FoldedFace::on_cube`, `CubeFace`).

## [0.42.7] - 2026-11-03

### Fixed
- Sealed triangular see-through gaps in the subterranean crust vault ceiling (`CrustVault::corners_at`, `CrustVault::emit_ceiling`) by splitting crossed neighbor edge steps at their midpoint into inward-facing triangles, correcting East/West step-wall winding, and clamping outer pyramid-seam corners directly to the spherical shell surface.

## [0.42.6] - 2026-11-03

### Fixed
- Restored gravitational descent back to the hollow-world Earth's core when digging upward above the spherical 4-layer crust by evaluating weightlessness against the spherical crust boundary (`CrustVault::encloses` in `TerracedRelief::weightless_at`) and re-engaging gravity immediately above the crust shell (`FlightContext::gravitational_acceleration`, `PlanetaryAvatar::step_motion`, `SurfaceFlight::advance`).

## [0.42.5] - 2026-11-03

### Fixed
- Restored 3D excavation craters on all sides and bottoms of blocks and terrain walls by suppressing occluding micro-clod facets during active digging (`Quad::emit_crater`), correcting inward pyramid face normals (`PyramidPeak::emit_shape`), and evaluating per-layer digging stages across terrain walls and upward shafts (`ReliefElevation::layer_dig_stage_at`, `OverhangMolds::layer_dig_stage_at`, `ReliefWall::emit_side`, `ReliefStrata::emit_upward_shaft`).

## [0.42.4] - 2026-11-03

### Fixed
- Prevented blocks and sloped terrain facets from expanding outward during or after excavation (`Quad::emit_crater`, `ShaftExcavation::crust_hit_top_at`, `NaturalStrata::crown_shape_at`, `OverhangMolds::retain_overhang`, `ReliefFacet::emit_dig_crater`, `ReliefWall::emit_walls`).

## [0.42.3] - 2026-11-03

### Changed
- Eliminated all remaining procedural patchwork ("Flickwerk") across `world` and `client` in strict adherence to Elegant Objects principles:
  - Replaced raw raycast contact flags (`is_side`, `is_bottom`, `is_ceiling`) with polymorphic `ContactFace` (`Top`, `Underside`, `Lateral`, `CrustCeiling`) in `SurfaceSightRay`.
  - Encapsulated shaft excavation, overhang preservation, and grass regrowth inside `ShaftExcavation` and `OverhangMolds` (`DigStageFocus`), and unified shaft-rim physics (`BodyEnvelope::in_deep_shaft`, `BodyEnvelope::catch_shaft_rim`, `BodyEnvelope::chest_reach`).
  - Decomposed `AvatarPosture` (`LeapSpring`, `VitalStrain`), `FlightContext` (`FlightStep`, `FlightBody`, `FlightDynamics`, `FlightTerrain`), and `MiniatureTarget` (`MiniatureBasis`, `MiniatureCoat`) to strictly obey the 4-field limit, and hardened `test_workspace_architecture_guard_enforces_line_and_field_limits` to count colons across every struct field.
  - Unified CLI flag and TOML configuration parsing via `CommandFlags` (`crates/client/src/command_flags.rs`) and `TomlDocument` (`crates/client/src/toml_document.rs`), eliminating duplicated parsers across `argument_*`, `file_*`, `FileBindings`, and `FileAttire`.
  - Replaced ad-hoc RGB/color-equality heuristics and setters with explicit domain contracts (`ReliefPalette::inspect_subterranean`, `CraftingEntry::batch_yield`, `SatchelTray::with_selected`, `HeldRelic::emit_in_satchel_slot`, `VoxelShape::emit_terraced`, `TimberYield::OakTimber`, `ClearedPlot::vacant`, `SolidArboreal::retain_if_standing`, `SessionDisplay::step_and_render`).

### Removed
- Removed all unused first-generation prototype modules and dead scaffolding (`chunk/*`, `landscape/*`, `block/material/*`, `block/form.rs`, `trait Block`, `face.rs`, `cluster.rs`, `elevated_face.rs`, `tinted_face.rs`, `mountain.rs`, `centripetal_world.rs`, `gallery.rs`, `relief/cavity.rs`, `examples/export_blocks.rs`) along with all `#[allow(dead_code)]` suppressions.

## [0.42.2] - 2026-11-02

### Fixed
- Prevented avatar head and camera clipping into overhanging ceiling blocks when jumping inside 3-block-high subterranean tunnels (`ElevatedShape::constrain_support`, `OverhangMolds::constrain_support`, `OverhangMolds::confine_niche_elevation`, `ShaftExcavation::confine`, `BodyEnvelope::constrain_with_height`, `BodyEnvelope::overhead_obstructed`, `PlanetaryAvatar::step_motion`).

## [0.42.1] - 2026-11-02

### Fixed
- Prevented the arm from slapping inward across the screen when picking up or cradling items (`MartialStyle::external_drive`, `MartialStyle::external_wrist_orient`, `MartialStyle::first_person_swing`, `MartialStyle::first_person_wrist_orient`, `PalmEarthworks::act_with_hand`) by giving `MartialStyle::PalmCradle` dedicated neutral kinematics instead of falling back to the diagonal `LumberjackCleave` swing and omitting the cleave strike trigger when foraging surface items.

### Changed
- Accelerated incremental compilation and test execution (`Cargo.toml`, `AGENTS.md`) via `debug = "line-tables-only"`, `opt-level = 1` for `world` in `dev`/`test` profiles, and targeted module test verification rules.

## [0.42.0] - 2026-10-28

### Added
- Sculpted 3D fractured deep-rock crags and glowing quartz/amber mineral veins (`CrustVault::emit_crag_relief`, `CrustVault::emit_crust_floor`, `CrustVault::emit_shaft_ceiling`, `CrustVault::emit_crust_flank`, `CrustVault::emit_crust_block`) across all floors, ceilings, and side flanks of the 4-layer planetary crust.

### Fixed
- Replaced soil clods and dirt wall coloring on lateral walls of the 4-layer planetary crust with crust rock flanks (`ReliefWall::emit_side`, `ReliefWall::emit_walls`, `OverhangMolds::emit_molded_top`, `ReliefFacet::emit_facet`, `TerracedRelief`).
- Grounded `Full-Cube` staircase jump-climbing when holding Spacebar (`AirborneFlight::advance`, `AerialTrial::passable`, `BodyEnvelope::unobstructed_aerial`) by landing on each block top and pausing horizontal ledge overshoot until the feet reach step height instead of flying continuously over multiple steps.

## [0.41.0] - 2026-10-28

### Added
- Acoustic waveform interface and clinical pulse-monitor decorator (`AcousticWave`, `CardiacPulse`, `MonitoredPulse` in `crates/client/src/cardiac.rs`) overlaying a `1046.5 Hz` pulse-monitor beep onto slowing heartbeats during the death animation (`AvatarPosture::advance_perish`, `AvatarPosture::perish_cycles`, `PlanetaryAvatar::step_perish`).
- Bidirectional zero-gravity vertical movement (`LeapIntent::Descend`, `LocomotionOrder::calm_beneath_crust`, `GravitationalFlight::descend`, `SurfaceFlight::advance`) and distance-weighted core strain (`AvatarPosture::advance_core_hover`, `PlanetaryAvatar::step_motion`) accelerating FOV constriction and cardiac failure closer to the magma core.
- Multi-layer stacked Earth's crust block rendering in upward shafts (`CrustVault::emit_shaft_ceiling`, `CrustVault::emit_crust_flank`, `CrustVault::emit_crust_floor`, `ReliefStrata::emit_upward_shaft`, `ReliefFacet::emit_facet`) displaying 4 distinct beveled crust block layers before transitioning to clodded soil.

### Fixed
- Preserved gravity and floor support inside lateral shaft niches excavated from upward shafts (`OverhangMolds::solid_wall_at`, `OverhangMolds::confine_niche_elevation`, `OverhangMolds::constrain_support`, `ShaftExcavation::confine`, `TerracedRelief::excavate_layer`, `BodyEnvelope::confine_beneath_crust`, `BodyEnvelope::unobstructed_aerial`, `AerialTrial::passable`, `AirborneFlight::advance`) so floating sideways into a niche lands on the niche floor instead of teleporting back to the core, while preventing camera wall clipping (`0.21m` lateral wall clearance) and softened the central blackout aperture feather (`AvatarPosture::emit_reticle`).

## [0.40.3] - 2026-10-28

### Fixed
- Eliminated frame stuttering when chopping and felling trees with an axe (`ForestedFace`, `CubePlanet`, `SessionLaunch`, `GameSession`) by caching quad geometry per individual tree specimen (`Vec<Vec<Quad>>`) and isolating terrain and flora GPU mesh slices (`0..face_count` vs `face_count..face_count * 2`) so tree strikes and shudder animations never rebuild or re-upload the terrain mesh.
- Preserved held tools in both hands when excavating a block with no free palm (`BeltSatchel::collect_excavated_block`), stowing the excavated block directly into the belt satchel instead of replacing the held spade.

## [0.40.2] - 2026-10-28

### Changed
- Enforced strict Elegant Objects / Pure OOP architecture across `crates/world`, `crates/engine`, and `crates/client` ($\le 4$ encapsulated fields per struct, $\le 300$ lines per file, polymorphic dispatch, and zero raw bitmask state), guarded automatically by `test_workspace_architecture_guard_enforces_line_and_field_limits`.
- Decomposed `TerracedRelief`, `PlanetaryAvatar`, `BodyPose`, and `PalmEarthworks` into focused domain collaborators (`StrataShell`, `ShaftExcavation`, `OverhangMolds`, `NaturalStrata`, `AvatarBody`, `AvatarSight`, `AvatarBearing`, `AvatarGear`, `StrideRhythm`, `MartialStyle`, and `DigGrip`) and encapsulated coordinate/grid arithmetic inside `SurfaceFrame` and `GridCell`.

### Fixed
- Preserved turf clod and river-pebble structure on the top, sides, and overhanging underside of natural dirt blocks (`EarthClods::emit_sole`, `OverhangMolds::preserve_above`, `CuboidShape::emit_shape`) when excavating laterally into a soil wall.
- Enabled targeting and excavating all four block layers of a vertical dirt wall from head height down to foot height (`SurfaceSightRay::brow_origin`, `SurfaceSightRay::target_excavation`, `SurfaceSightRay::target_placement`) without shifting the ray origin when looking down.

## [0.40.1] - 2026-10-27

### Fixed
- Preserved all overlying blocks when mining a single side block from inside a shaft (`TerracedRelief::excavate_layer`, `TerracedRelief::emit_molded_top`, `ElevatedShape::floating_above`, `ReliefFacet::emit_facet`) by converting the unmined upper column layers `(next_elev + 1)..elev` into `ElevatedShape` blocks in `TerracedRelief` instead of collapsing the entire column down to the shaft floor.

## [0.40.0] - 2026-10-27

### Added
- Circular alpha-gradient tunnel-vision blackout and surface respawn (`AvatarPosture::emit_reticle`, `AvatarPosture::advance_perish`, `AvatarPosture::blackout_complete`, `PlanetaryAvatar::step_perish`, `VoxelShaderSource::embedded`) with per-vertex alpha blending (`GL_SRC_ALPHA`, `GL_ONE_MINUS_SRC_ALPHA`), replacing the rectangular `"YOU DIED"` banner box with a smoothly closing dark-red-to-black circular aperture that awakens the player at the surface spawn once vision is completely black.
- Lateral shaft-widening excavation in both deep downward shafts and upward core-to-surface shafts (`TerracedRelief::hit_top_at`, `TerracedRelief::adjacent_upward_at`, `TerracedRelief::mark_dig_layer`, `TerracedRelief::excavate_layer`, `SurfaceSightRay::trace_hit`, `PalmEarthworks::act_with_hand`, `PalmEarthworks::continue_digging`) allowing the player to dig sideways from inside a 2×2 downward shaft or an upward shaft.

### Changed
- Raised the inner planetary crust vault 12 blocks (`6.0 m`) higher and further away from the magma core (`CrustVault::beneath`, `CrustVault::core` at radius `54.0 m` vs core radius `40.0 m` on the baseline 128-block moonlet, expanding the hollow world clearance to 28 blocks / `14.0 m`).
- Extended the core-proximity survival time to 2 minutes (`120 s`) in `AvatarPosture::advance_hover` and arrested the player safely in weightless hover when excavating the final crust block above the hollow world (`FlightContext::gravitational_acceleration`, `SurfaceStride::resolve_ground`, `SurfaceFlight::advance`) instead of plummeting into the magma core.
- Suppressed sprinting / fast floating while weightless beneath the planetary crust (`LocomotionOrder::calm_beneath_crust`, `PlanetaryAvatar::step_motion`).

### Fixed
- Sealed see-through triangle gaps in the hollow-world crust ceiling (`CornerHeading::valley_elevation`, `GentleValley::emit_shape`, `ValleyCorner::emit_shape`, `DiagonalSlope::emit_shape`, `CrustVault::emit_ceiling`, `CrustVault::emit_skirt`) by aligning `SouthEast` and `SouthWest` valley corner elevations and diagonal splits with `InvertedShape::ceiling_corners`, splitting seam-straddling quads along their radial diagonal, and extending shaft skirts down to the beveled ceiling corners.
- Prevented false shoulder-wedging yaw locks inside 2×2 shafts and upward shafts (`BodyEnvelope::wedged_in_shaft`, `FlightContext::weightless_ascent`) by evaluating full-body pair support (`BodyEnvelope::constrain_with_height`) against shoulder-height (`>= 1.25 m`) walls.

## [0.39.0] - 2026-10-24

### Added
- Core-proximity survival timer, progressive FOV constriction, dark-red tunnel-vision vignette, and accelerating heartbeat pulse (`CardiacPulse`, `AvatarPosture::advance_hover`, `AvatarPosture::cardiac_constriction`, `AvatarPosture::cardiac_fov`, `AvatarPosture::emit_reticle`) when lingering in the hollow world near the magma core, recovering once the player retreats into or above the planetary crust (`TerracedRelief::in_upward_shaft`, `TerracedRelief::excavate_layer`).
- Character-proportioned 3D body dimensions (`YouthPhysique::stature_height`, `YouthPhysique::upper_body_span`, `YouthPhysique::body_depth`, `YouthPhysique::shaft_yaw_play`, `BodyEnvelope::for_physique`, `BodyEnvelope::with_crouch_and_facing`) and upper-body yaw clamping (`ShoulderArc`, `ViewOrientation::constrain_shoulders`) governing doorway clearance, crouch retention under low lintels (`BodyEnvelope::overhead_obstructed`, `ElevatedShape::constrain_support`), and shaft turning limits (`BodyEnvelope::wedged_in_shaft`).

### Changed
- Evaluated excavated pit support across adjacent 2×1 cell pairs (`TerracedRelief::pair_elevation`, `TerracedRelief::body_elevation_at`) and measured shaft wall spans against the character's upper-body width (`BodyEnvelope::axis_wedged`, `BodyEnvelope::wall_distance`), preventing the player from falling into a 1×1 hole while allowing falling into 2×1 and 2×2 holes, restricting 360° rotation in a 2×1 shaft, and permitting full 360° rotation in a 2×2 shaft when the upper body is narrower than 2 blocks.

### Fixed
- Aligned downward digging rays with the exact first-person camera eye origin and look direction of the crosshair (`PlanetaryAvatar::hand_ray`, `SurfaceSightRay::trace_hit`) and forbade excavating the ground block directly supporting the player's feet.

## [0.38.0] - 2026-10-24

### Added
- Configurable server world-cube scale factor (`PlanetaryScale`, `MoonletScale`, `ScaledCube`, `FileScale`, `EnvironmentScale`, `ArgumentScale`, `ServerScale` in `crates/client/src/server_scale.rs`) supporting `[world] size` in `server.toml`, `--size` / `--scale`, and `LOW_CUBE_SIZE` (`size >= 1`, baseline factor `1 = 128` blocks) with fail-fast validation and documentation in `docs/SETTINGS.md`.
- Weightless horizontal hover posture and swimming limb animation (`AvatarPosture::advance_hover`, `BodyPose::with_hover`, `BodyPose::neck_frame`, `CrownDetails`, `Headgear`, `PreviewStage::step`) with torso-anchored neck articulation and a live `FLOATING` / `STANDING` preview toggle in the Character Studio menu.
- Lethal magma core death state and `"YOU DIED"` HUD banner (`AvatarPosture::perish`, `AvatarPosture::perished`, `AvatarPosture::emit_reticle`) arresting the player in place upon touching the magma core instead of teleporting back to the surface.

### Changed
- Rendered the planetary magma core as a glowing incandescent sphere (`MagmaCore`, `Quad::glowing`, `Quad::radiate`) tiled seamlessly across all six faces with molten orange-to-yellow swirls, drawn self-luminous by a new per-vertex glow attribute in the voxel shader; breached crust shafts now open straight into the hollow world instead of ending on a flat magma tile.
- Made magma contact, weightlessness, and hollow-world support follow the spherical core surface (`ReliefElevation::lethal_at`, `ReliefElevation::weightless_at`, `MagmaCore::engulfs`, `MagmaCore::crest_at`) rather than a flat per-face plane.
- Scaled the inner 4-layer (`2.0 m`) planetary block crust (`CrustVault`, `InvertedShape`) and central magma core (`MagmaCore`) to half of the world-cube radius (`CrustVault::beneath`, `CrustVault::core`, `CrustVault::circumference`), separated by a 12-block (`6.0 m`) weightless hollow space, with the hollow-world ceiling sculpted from inverted beveled block shapes (`CuboidShape`, `GentleWedge`, `GentleRidge`, `GentleValley`, `RampWedge`, `RidgeCorner`, `ValleyCorner`, `DiagonalSlope`) and earthy magma-lit rock veins (`CrustVault::emit_ceiling`, `CrustVault::emit_skirt`).
- Dressed excavated terrain soil (bare pit floors and dug walls) in the same clods, pebbles, and loam tones as the hand-held earth block (`EarthClods`, `ReliefWall`).

### Fixed
- Prevented jumping or repeated head collisions in the hollow world from lifting the camera through the crust or catapulting the player to the outer surface (`PlanetaryWorld::confine_beneath_crust`, `CrustVault::confine`, `SurfaceFrame::confine_radius`, `TerracedRelief::support_height_at`).
- Enabled upward ceiling excavation from inside the weightless hollow world to remove the targeted ceiling block and breach an exit shaft back to the outer surface (`SurfaceSightRay::trace_hit`, `TerracedRelief::hit_top_at`, `TerracedRelief::excavate_layer`, `PalmEarthworks::act_with_hand`, `SurfaceFlight::advance`).

## [0.37.0] - 2026-10-23

### Added
- Sculpted 3D clodded-loam and river-pebble geometry for `EARTH BLOCK` (`BlockMedium::TurfEarth`, `BlockMedium::emit_turf_earth_cube`, `MediumShape::emit_terraced_shape`, `BlockMold::emit_miniature`) rendering protruding soil clods and embedded stones identically in the avatar's hand, satchel slot, and placed world blocks.

### Fixed
- Preserved the material medium of placed blocks (`WOOD PLANKS`, `WOOD BLOCK`, `STONE BLOCK`, `EARTH BLOCK`) during excavation (`VoxelShape::block_medium`, `ElevatedShape::block_medium`, `TerracedRelief::layer_medium_at`, `PlanetaryWorld::surface_medium`, `HeldRelic::from_medium`, `BeltSatchel::collect_excavated_block`, `PalmEarthworks::act_with_hand`) so mining a placed wooden plank returns `WOOD PLANKS` into the hand instead of `EARTH BLOCK`.

## [0.36.0] - 2026-10-22

### Added
- Interactive medieval Crafting Book (`ArtisanTome`, `CraftingEntry`, `ControlAction::CraftingBook` bound to `KeyC`) displaying craftable tools, weapons, torches, and building blocks with live resource tallies across both hands and stowed satchel slots (`BeltSatchel::resource_tally`, `BeltSatchel::consume_and_equip`).
- Gatherable forest-floor resources (`FloorLitter`, `SurfaceFind::DeadwoodBough`, `SurfaceFind::FlintStone`, `HeldRelic::DeadwoodBough`, `HeldRelic::FlintStone`) spawning branched deadwood boughs with snapped heartwood ends and leaf buds exclusively within `1.0m` of standing trees, and multi-faceted conchoidal flint nodules with chalky cortex bases and sharp cleavage shards near rocky slopes (`ZonedForest::populate_face`).
- Tree-felling timber harvest (`CleaveOutcome`, `BeltSatchel::collect_felled_timber`, `HeldRelic::TimberBlock`) awarding 3 `WOOD BLOCK`s into the player's free hand and belt satchel upon felling a tree (`StandingTree::cleave_at`), accompanied by axe-cleave trunk shudder, progressive heartwood V-notches, flying woodchips, and falling leaves (`HewnTimberTarget`).
- Sculpted 3D wood and plank building blocks (`BlockMedium`, `MediumShape`, `HeldRelic::TimberBlock`, `HeldRelic::PlankBlock`) rendering identical geometry in the hand, satchel, and placed world blocks: `WOOD BLOCK` (`BlockMedium::RawTimber`) with vertical bark ridges and concentric annual rings on top, and craftable `WOOD PLANKS` (`BlockMedium::PlanedPlanks`, yielding 4 planks per wood block) with 4 alternating honey-oak floorboards separated by deep shadow grooves.

### Fixed
- Doorframe jump-catapulting physics defect (`BodyEnvelope::resolve_step`, `SurfaceSightRay::placement_cell`, `ElevatedShape::constrain_support`, `TerracedRelief::hit_top_at`, `FoldedFace::constrain`) by rejecting sub-`0.30m` horizontal step-ups as vertical walls when squeezing or jumping through 1-block-wide doorframes and enabling underside block attachment to overhead lintels.
- First-person palm orientation for held deadwood boughs and flint stones (`LimbFrame::palm_cradle`) so foraged items rest horizontally atop the upturned open palm instead of piercing through the hand.

## [0.35.0] - 2026-10-22

### Added
- Keybinding Settings screen in the Escape menu (`ControlStudio`, `ActionBindings`, `ControlAction`, `InputTrigger`, `TriggerNotation`, `FileBindings`) enforcing strictly one key or mouse-wheel trigger per action with conflict swapping, `RESET KEYS`, and persistent storage in `controls.toml`.
- Mouse wheel and middle-click input triggers (`InputTrigger::WheelClick`, `InputTrigger::WheelUp`, `InputTrigger::WheelDown`) bound by default to `MOLD SHAPE` (`WHEEL CLICK`), `MOLD TURN L` (`WHEEL UP`), and `MOLD TURN R` (`WHEEL DOWN`).
- Studio-lit satchel slot decorator (`LitSlotTarget`) and per-tool count storage (`BeltSatchel`) so multiple tools of the same type (such as dual axes) are preserved in separate satchel slots and rendered with bright, angle-independent facet lighting.

### Changed
- Framed the open satchel menu (`SightMode::arm_joints`, `BeltSatchel::emit_on_waist`, `HeldRelic::emit_in_satchel_slot`, `HeldRelic::emit_satchel_legend`) in front of outward-positioned held tools and weapons without screen-space overlap, locked player movement and camera look while the satchel is open, hid the gameplay crosshair during satchel inspection, allowed closing the satchel with `Escape`, and updated the satchel legend to English.

### Removed
- Showcase spawn figures (`SpawnTroupe`, `TroupeAvatar`) and ground items (`FieldAxe`) from the player spawn area (`LaunchPad::launch_session`).
- Camera axis inversion (`InvertibleLook`, `AxisPolarity`), `Mirror View` (`KeyM`), and direct hand-relic cycle keys (`KeyG`, `KeyH`).

## [0.34.0] - 2026-10-22

### Added
- Hold-to-dig excavation while holding a block in hand (`PalmEarthworks::act_with_hand`, `PalmEarthworks::continue_digging`, `PalmEarthworks::release_hand`) so a short click places the held block on release while holding the mouse button down transitions into bare-hand excavation.
- Framed inventory slot background plaques (`LimbFrame::emit_framed_plaque`, `HeldRelic::emit_in_satchel_slot`) and a framed 3D pixel-font legend plaque underneath the satchel grid (`HeldRelic::emit_satchel_legend`, `PixelStamp::emit_on_frame`, `BeltSatchel::emit_on_waist`) displaying the selected item title and key controls (`WASD`, `Q`/`E`, `R`/`V`).
- Pure OOP Decorator pattern classes (`CrateredMeshTarget`, `TurfMeshTarget`, `ElevatedShape`, `SatchelWardrobe`) composing 3D digging crater subdivision, canopy/wall coloration with underside sealing, elevated voxel block layers, and bare-hand digging attire.

### Changed
- Enlarged held blocks cradled in the avatar's palm (`LimbFrame::emit_cradled_block`) while hiding the held block during active digging strikes (`SatchelWardrobe::emit`).
- Unified stacked block storage in `TerracedRelief` around `Vec<ElevatedShape>` per cell without mutating terrain `deltas` on placement so placed and natural blocks never alter their shape when additional blocks are stacked on top.

### Fixed
- Rendered 3D digging craters and debris spray directly on laterally attached floating blocks and targeted intermediate stack layers (`Quad::emit_crater`, `CrateredMeshTarget`, `TerracedRelief::hit_top_at`, `TerracedRelief::excavate_layer`) rather than on the ground below or the topmost block.
- Preserved natural terrain `RidgeCorner` slopes when stacking molded blocks on top and corrected upward slope normals in `ReliefCorner::emit_ridge`.
- Colored side walls exposed by excavation (`ReliefFacet::emit_walls`) in earth brown instead of slate cliff grey.

## [0.33.0] - 2026-10-22

### Added
- Synchronized dual-hand block molding and rotation (`BeltSatchel::target_mold_sides`, `BeltSatchel::collect_excavated_block`, `BeltSatchel::toggle_pocket`, `BeltSatchel::reconcile_attire`) so scrolling or rotating modifies both held blocks simultaneously when both hands hold blocks, and newly excavated or drawn blocks inherit the opposite hand's form and rotation regardless of material.
- Lateral side-wall block attachment without ground contact (`SurfaceSightRay::placement_cell`, `TerracedRelief::stack_molded_cell`, `TerracedRelief::emit_molded_top`, `ReliefFacet::emit_facet`) allowing blocks (including `FullCube` and `RidgeCorner`) to be attached horizontally to the topmost block of an elevated pillar with empty air underneath.

### Changed
- Sculpted `RidgeCorner` and `GentleRidge::lower` with a true triangular base (`Dreieckbodenfläche`) omitting the opposite half-cell triangle while preserving the underlying biome surface color (`surf_col`) on the uncovered ground half when placed on terrain.
- Removed the redundant third `Latest Build` download button from `README.md`.

### Fixed
- Corrected inverted cross-product normal on the diagonal slope face of `RidgeCorner::emit_shape` and `GentleRidge::emit_shape` so the diagonal slope points upward and renders on placed blocks.
- Aligned left and right palm roll and forearm yaw compensation (`BodyPose::external_arm`, `BodyPose::first_person_arm`, `LimbFrame::emit_cradled_block`) so blocks held in both hands display identical 3D orientation.

## [0.32.0] - 2026-10-22

### Changed
- Updated bare-hand digging animation (`BodyPose::external_arm`, `BodyPose::first_person_arm`, `BodyPose::finger_curl`) to stretch both arms forward while pronating the hands inward so the palms face downward, followed by a downward wrist flexion and inward finger curl during the digging strike.
- Rebuilt palm geometry in `VambraceStyle::emit` as a flat-faced 8-sided anatomical `TaperedPrism` with synchronized `arm_girth`-scaled finger and thumb knuckle roots and scaled-up masculine hand proportions (`hs = 1.18`, `hg = 1.16`).

### Fixed
- Eliminated the protruding central triangular ridge in the palm that caused cradled blocks to appear sunken into the hand (`VambraceStyle::emit`).
- Preserved held blocks and tools in `PlanetaryAvatar::apply_attire`, `SpawnTroupe::apply_attire`, and `WardrobeStudio::press_pointer` when switching between male and female or selecting archetypes in the Character Studio.

## [0.31.0] - 2026-10-21

### Added
- Shared invisible satchel inventory and 3D camera-aligned inspection grid (`BeltSatchel::toggle_open`, `BeltSatchel::navigate`, `BeltSatchel::toggle_pocket`, `HeldRelic::emit_in_satchel_slot`, `BodyPose::view_frame`) toggled with `Tab` / `B` (including when empty), navigated with `W`/`A`/`S`/`D` or arrow keys while pausing locomotion, and drawn into the left (`Q`) or right (`E`) hand.
- Hollow 4-bar slot border frames (`AxialLimb`) with upright isometric 3D block previews (`LimbFrame::emit_upright_block`) and proportionally scaled tool/weapon previews (`LimbFrame::scaled`) centered inside each satchel slot.
- Direct 1-click download badges for Windows (`.exe`) and Linux (`x86_64`) at the top of `README.md` with `make_latest: true` in `.github/workflows/release.yml`.

### Changed
- Concealed stowed blocks and tools completely while the satchel is closed (`BeltSatchel::emit_on_waist`) instead of rendering external pouches on the avatar's waist.

## [0.30.0] - 2026-10-21

### Added
- Persistent Palm Molding system (`BlockMold`, `MoldArchetype`, `CardinalTurn`) allowing held earth and stone blocks to be sculpted across all 16 base block forms (`MouseWheel` / `KeyR`) and rotated across all 4 cardinal directions (`MouseButton::Middle` / `KeyV` / `KeyC`) while facing the player's placement direction by default.
- Dual-hip belt satchels and tool holsters (`HipPouch`, `BeltSatchel`, `PalmEarthworks::toggle_hand_pocket`) bound to `KeyQ` (left hand) and `KeyE` (right hand) to stow held blocks, spades, and weapons into visible 3D hip pouches that swell with stowed block count, display miniature molded block previews, auto-refill the hand while building, and retain independent left/right block molds.
- Upturned hollow-palm cradling posture (`MartialStyle::PalmCradle`, `LimbFrame::emit_cradled_block`, `BodyPose::finger_curl`) supinating the forearm and cupping the palm, thumb, and fingers underneath held blocks.
- Minecraft-authentic grass loss and neighbor-driven regrowth (`TerracedRelief::step_grass`, `TerracedRelief::bare_dirt_at`, `PlanetaryWorld::step_surface`) stripping grass from excavated/placed dirt blocks and regrowing grass over `40s–95s` (at 8 adjacent grass blocks within `|dh| <= 1`) scaled by neighbor count.

### Fixed
- Completed watertight 14-quad geometry for convex and concave stair corners (`StairCorner::emit_quads`) across all 4 corner headings and aligned diagonal crease triangulation on ridge and valley corners (`RidgeCorner`, `ValleyCorner`, `GentleRidge`, `GentleValley`).

## [0.29.0] - 2026-10-21

### Added
- Hold-to-dig excavation lifecycle (`PalmEarthworks`, `Avatar::release_hand`, `Avatar::step_digging`, `PlanetaryWorld::loosen_block`, `TerracedRelief`) requiring continuous mouse button hold across strikes and aborting excavation progress upon early release.
- True 3D sunken excavation crater (`ReliefFacet::emit_dig_crater`) with 4 outer rim trapezoids, 4 shaded inner walls, and 1 sunken floor scaling across digging stages (1/4 to 3/4) with automatic material depth shading.
- Ground-anchored outward ballistic soil clod splash (`PalmEarthworks::emit_debris`) emitted directly above the targeted block during active excavation strikes.
- Dedicated spade and bare-hand digging martial kinematics (`MartialStyle::SpadeDig`, `MartialStyle::Unarmed`, `BodyPose::external_arm`, `BodyPose::first_person_arm`) with outward-then-inward elbow motion, forward-reaching forearm arcs, inward wrist rotation plunging the spade tip diagonally forward-downward toward the target block, and dual-button bare-hand scooping that brings both palms together in a diagonal outward cup.
- Two additional spawn showcase figures (`GaitFigure::digging_at`, `GaitFigure::palm_digging_at`, `SpawnTroupe`) and Character Studio preview animations (`StudioAnimation::SpadeDig`, `StudioAnimation::PalmDig`) demonstrating spade and bare-hand digging.

### Changed
- Renamed `HeldRelic::IronShovel` to `HeldRelic::IronSpade` (`HeldRelic::spade_blade`) and doubled excavation thresholds (`EXCAVATION_THRESHOLD = 8`: 2 strikes with iron spade, 4 strikes with both empty hands when holding both mouse buttons, 8 strikes with a single empty hand).
- Replaced fixed distance ray sampling in `SurfaceSightRay::aimed_cell` with discrete 3D grid traversal (18 steps up to `1.5m`) constrained to a maximum reach of 3 blocks (`|du| <= 3`, `|dv| <= 3`) around the avatar.

## [0.28.0] - 2026-10-21

### Added
- Block excavation and placement system (`SurfaceSightRay`, `TerracedRelief`, `PalmEarthworks`, `HeldRelic::IronShovel`, `HeldRelic::ExcavatedEarth`) allowing players to dig terrain and placed blocks with bare hands (4 strikes with 1 free hand, 2 strikes with 2 free hands) or an iron shovel (1 strike) and place collected earth blocks back into the world using the hand holding the block.
- Ground iron shovel relic (`FieldAxe`) stationed near the planetary spawn point alongside the woodman's axe and knightly sword.
- Multi-face planetary GPU mesh (`PlanetaryMesh`, `PlanetaryWorld::face_count`, `PlanetaryWorld::emit_face`, `PlanetaryWorld::emit_modified_face`, `GlPlatformSurface::replace_mesh`, `Mesh::release`) uploading each of the 6 planetary faces as an independent GPU buffer and replacing only the single modified face slice upon block excavation or placement.
- Row-level relief quad cache (`FacetedReliefFace`), `O(1)` natural elevation grid cache (`TerracedRelief`), and static flora quad cache (`ForestedFace`) so block updates recompute only the 3 affected cell rows (`cv - 1 ..= cv + 1`) on the modified face without re-evaluating untouched terrain or trees.

### Fixed
- Preserved natural slope, ridge, and valley geometry and collision heights on neighboring cells when excavating sloped blocks (`ReliefFacet::emit_facet`, `ReliefFacet::emit_walls`, `TerracedRelief::surface_height_at`).

## [0.27.0] - 2026-10-21

### Added
- Rendered 12 high-resolution 3D character portraits (`docs/images/character/{knight,valkyrie,ranger,huntress,woodsman,herbalist,blacksmith,weaver,sentinel,shieldmaid,noble,lady}.png`) and added a complete male and female **Character Archetypes Gallery** with descriptions in `docs/CHARACTER.md`.

### Changed
- Scaled all full-cube and non-full-cube block shapes (`CuboidBlock`, `WedgeBlock`, `GentleWedgeBlock`, `GentleRidgeBlock`, `GentleValleyBlock`, `RidgeCornerBlock`, `ValleyCornerBlock`, `PyramidBlock`, `StairBlock`, `StairCornerBlock`, `GridCell`, `FormBlock`) to half size (`0.5m × 0.5m × 0.5m` with `0.25m` slabs/steps and `0.125m` quarter-height increments) and adapted planetary relief (`ElevatedFacet`, `ElevationStrata`, `SeamCorner`, `SubterraneanCavity`), biome factory (`BiomedWorld`), forestry ground checks (`FullCubeGround`, `MeridianGround`, `SpawnGround`, `SurfaceGround`, `WoodlandGround`, `ForestedFace`), and mountain/biome landscapes accordingly.
- Refined the female bust in `TunicStyle::emit` and `PolygonAccoutrement::emit_tunic_details` into a clearly recognizable, non-pointy, continuous clothed 3-tier bust with a subtle center indentation (`x = 0.0`) and underbust bodice framing across all archetypes.
- Removed the `Architectural Design` section from `docs/CHARACTER.md` and updated the `create git message` workflow in `AGENTS.md` to stage all files and execute the commit via CLI.

## [0.26.0] - 2026-10-21

### Added
- Fifth spawn showcase character with continuous sneaking animation (`GaitFigure::sneaking_at`, `StrideRhythm::sneaking`, `SpawnTroupe`) placed alongside standing, walking, running, and jumping figures.
- Gender switch (`MALE` / `FEMALE` buttons and hotkey `F`) in the 3D Character Studio (`WardrobeStudio`, `YouthPhysique::feminine`, `YouthAttire::feminine`) with 6 dedicated female character archetypes (`AvatarArchetype::gender_title`, `AvatarArchetype::female_attire`: `VALKYRIE`, `HUNTRESS`, `HERBALIST`, `WEAVER`, `SHIELDMAID`, `LADY`).
- Distinctly feminine 3D character anatomy and wardrobe (`TunicStyle::emit`, `Accoutrement::emit`, `CrownStyle::emit`, `PolygonHeadgear::emit`, `FootwearStyle::emit`) featuring narrower shoulders, hourglass corset waist, flared A-line skirt/dress hem, sculpted 3D bust and Valkyrie breastplate, softer facial jawline with rosy lips and eyelashes, long flowing rear hair with twin front shoulder braids, and open-faced feminine headgear (winged diadem, huntress hood, floral wreath, high ribbon bun, shieldmaiden circlet, lady tiara and veil).

### Changed
- Deepened sneaking posture kinematics (`SneakGait::crouching`, `BodyPose::hip_elevation`, `BodyPose::leg_joints`, `BodyPose::torso_frame`, `BodyPose::external_arm`, `SightMode::arm_joints`) with deeper knee flexion (`elevation = 0.36`), forward-leaning upper torso hinged from the hips, forward-bent elbows, slightly inward-angled forearms, and diagonally inward-rotated hands.

## [0.25.0] - 2026-10-20

### Added
- Independent left-hand and right-hand weapon strike routing (`MouseButton::Left` for left offhand, `MouseButton::Right` for right relic, `StrideRhythm::trigger_left_strike`, `StrideRhythm::trigger_right_strike`, `BodyPose::with_strikes`) with centered HUD aiming crosshair.
- Calibrated 3-phase axe and sword strike kinematics (`BodyPose::axe_joints`, `BodyPose::sword_joints`, `BodyPose::wrist_snap`, `HeldRelic::hand_roll`) featuring downward wrist hinge flexion, overhead semicircular axe cleave (`wind` -> `bow` -> `strike`) with outward hand/axe-head roll touching the crosshair at apex, and tucked-elbow diagonal sword cross-slash aligning the sword tip directly with the crosshair.
- Four-phase two-legged jumping animation (`BodyPose::with_leap`, `AvatarPosture::advance_leap`, `GaitFigure::jumping_at`, `StudioAnimation::JumpingLeap`) modeling deep takeoff knee crouch, airborne full straight-leg backward extension, landing knee absorption, upright recovery, and $C^1$-continuous arm swing across the player avatar, the 4th spawn showcase figure (`SpawnTroupe`), and the 7-animation 3D Character Studio turntable (`TurntableStage`).
- Character Studio showcase screenshot (`docs/images/character/character-studio.png`) and expanded English documentation in `docs/CHARACTER.md` and `README.md`.

### Changed
- Decoupled First-Person camera `eye_origin` from `leap` posture flex (`BodyPose`) and anchored anatomical torso/head vertical proportions above `hip_elevation()` (`TunicStyle::emit`, `GirdleStyle::emit`, `PolygonAccoutrement::emit`) so crouching and jumping bend only the knees without squashing the torso or detaching the head.
- Shortened Character Studio right-hand panel title (`CharacterStudio`) to `"BODY PARTS"` so the header fits cleanly inside the border box.

### Fixed
- Resolved sprint-jump slope collision hang (`BodyEnvelope::unobstructed_aerial`, `BodyEnvelope::unobstructed_ground`, `AerialTrial::destination`) by evaluating radial clearance in two `0.16 m` half-steps and letting `AirborneFlight::advance` detect and land cleanly on inclined terrain up to `2:1` (`63°`).

## [0.24.0] - 2026-10-20

### Added
- Interactive in-game `Esc` modal menu (`EscapeMenu`, `SessionModal`, `ModalDirective`) with `RESUME GAME`, `CHARACTER`, and `QUIT GAME` buttons, keyboard shortcuts, and cursor release/re-capture.
- Full-height 3D Character Studio (`WardrobeStudio`, `TurntableStage`, `StudioAnimation`, `PixelStamp`, `ScreenPoint`, `ScreenBounds`) rendering a single centered character figure on a stone pedestal at `z = 4.65` with 360-degree mouse-drag and button/key turntable rotation (`< ROTATE`, `FRONT`, `ROTATE >`) and automatic sequential demonstration of all 6 animations (`StandingIdle`, `WalkingStride`, `RunningSprint`, `CrouchingSneak`, `AxeCleave`, `SwordSlash`).
- Six fundamentally distinct Polygon-City-style character body archetypes (`AvatarArchetype`: `Knight`, `Ranger`, `Woodsman`, `Blacksmith`, `Sentinel`, `Noble`) with bespoke 3D silhouette architecture (`PolygonAccoutrement`, `PolygonHeadgear`) including sallet visors, hooded cloaks, blacksmith aprons, tower shields, and velvet capes.
- Independent per-body-part `SIZE` (length/height) and `THICKNESS` (width/girth) customization (`BodyRegion`: `Head`, `Torso`, `Arms`, `Legs`, `PartProportion`, `YouthPhysique`) with real-time 3D joint chain and bone-length scaling (`BodyPose`).
- Expanded skin complexion palette (`ComplexionSwatch`) with 8 selectable skin tones (`Birch`, `Meadow`, `Amber`, `Umber`, `Porcelain`, `Olive`, `Bronze`, `Ebony`) and dedicated clothing selection controls (`TUNIC`, `SLEEVES`, `BELT`, `BOOTS`, `HEADGEAR`, `DYE COLOR`) alongside a single `BACK` navigation button.

### Changed
- Extended `FileAttire` (`character.toml`) and deterministic 16-byte wire serialization to persist per-region `SIZE` and `THICKNESS` proportions and expanded wardrobe variants.

## [0.23.0] - 2026-10-20

### Added
- Spawn showcase figure troupe (`FigureTroupe`, `SpawnTroupe`, `GaitFigure`, `TroupeAvatar`) placing three animated character instances in front of the spawn point facing the player to demonstrate standing (*Stehen*), walking (*Gehen*), and running (*Laufen*) gaits with zero runtime heap allocations.
- Interactive medieval ground weapons near spawn (`RelicSite`, `FieldAxe`, `FieldRelic`) featuring an authentic medieval bearded felling axe (`HeldRelic::WoodmanAxe`) and a medieval knightly sword (`HeldRelic::KnightlySword` with wheel pommel, leather grip, cruciform crossguard, and double-edged blade with fuller).
- Dual-hand item holding (`YouthAttire` with `relic` for right hand and `offhand` for left hand, hotkeys `8`/`H` and `9`/`G`, plus `E` pickup and `Q` ballistic throw) allowing the avatar to hold and wield items in both hands simultaneously in First-Person and Third-Person views.
- Weapon-specific mouse-click martial animations (`MartialStyle::LumberjackCleave` and `MartialStyle::SwordplaySlash`, `StrideRhythm::trigger_strike`, `cleave_arc`, `cleave_surge`) featuring a full-arm lateral woodcutter felling swing (*seitlicher Schwung*) with forward upper-arm surge and inward-and-forward axe shaft inclination stopping at the body center line (`x >= 0.0`), alongside a distinct diagonal sword cross-slash.
- Dedicated `SightMode` module (`crates/client/src/attire/sight.rs`) and troupe/weapon test suite (`crates/client/src/troupe_tests.rs`).

### Changed
- Refined 3D anatomical character proportions: broad-shouldered V-taper torso (`TunicStyle::emit`), 3-tier sculpted head with full rear-cranium hair/headgear coverage (`CrownStyle::emit`), hip sockets aligned cleanly at the tunic skirt hem (`y = 0.78 m`) to prevent thigh-torso clipping, and low running elbows (`>= 0.20 m` below shoulders) with slight lateral abduction.
- Seamless wrist-to-palm transition and tapered hand geometry (`VambraceStyle::emit`) with wristbands (`RawhideWraps`, `StuddedVambraces`) terminating cleanly behind the wrist joint (`z = -0.018 m`), tapered palm and 4-cm thumb, and demand-driven First-Person hand raising (`SightMode::arm_joints`) so empty hands hang naturally at the sides.
- Pulled First-Person camera eye position slightly forward (`CameraHorizon`, `BodyPose::eye_origin`) and expanded `BodyEnvelope` radius (`0.30 m`) for clear downward foot visibility without chest occlusion.
- Updated `docs/CHARACTER.md` with spawn showcase troupe, dual-wielding controls, interactive ground weapons, and martial kinematics documentation.

## [0.22.0] - 2026-10-19

### Added
- Faceted low-poly medieval youth / squire avatar system (`Wardrobe`, `YouthAttire`, `BodyPose`, `StrideRhythm`, `LimbFrame`, `FacetedSolid`, `TaperedPrism`, `AxialLimb`) with full First-Person Body Awareness and Multiplayer / Looking-Glass Mirror (`M`) rendering (`SightMode`).
- Articulated 5-finger hands with 3 phalanges per finger, natural anatomical wrist inflection, collinear forearm-to-wrist frame alignment, and gait-responsive finger curl (`VambraceStyle`, `BodyPose::finger_curl`).
- Constant-length 3D trigonometric leg joint chain (`BodyPose::leg_joints`) with human swing-through flexion, forward step extension, 3D knee perspective flare, wide hip sockets, sculpted thighs, knee garters, muscular calf bellies, and medieval boots (`FootwearStyle`).
- Live wardrobe customization hotkeys (`1`–`8`, `H`, `M`) across upper tunics (`TunicStyle`), forearm bracers (`VambraceStyle`), waist girdles (`GirdleStyle`), footwear (`FootwearStyle`), hair and headgear (`CrownStyle`), natural cloth dyes (`PigmentSwatch`), skin complexions (`ComplexionSwatch`), stature builds (`YouthPhysique`), and held relics (`HeldRelic`).
- Persistent TOML character profile storage (`character.toml`) and deterministic 9-byte multiplayer wire codec (`AttireProfile`, `FileAttire`).
- Octagonal physical body collision envelope (`BodyEnvelope`) preventing First-Person camera and upper torso clipping into solid block walls while preserving smooth wall sliding.
- Dynamic per-frame avatar mesh compositing over static world geometry (`CompositeMesh`, `GpuBuffer::render_combined`, `LineBuffer::render_combined`).
- Comprehensive character concept and wardrobe customization documentation in `docs/CHARACTER.md` with showcase illustrations in `docs/images/character/`.

### Changed
- Forward-sloped torso geometry (`TunicStyle::emit`, `GirdleStyle::emit`) with recessed collarbone yoke and wide flared hip skirt so the upper chest never occludes the waist or hips when looking down from First-Person.
- Partitioned forearm and bracer geometry (`BareForearms`, `RawhideWraps`, `SplintedVambraces`) into non-overlapping axial segments to eliminate skin Z-fighting through leather wraps.
- Completed Pure OOP / Elegant Objects refactoring across `client`, `engine`, and `world` crates and resolved all compiler and Clippy warnings.

## [0.21.0] - 2026-10-16

### Added
- Nocturnal Arctic polar lights (auroras) with dynamic waving green and violet curtains constrained to the Arctic face (`AuroraCanopy`, `CalibratedCanopy`, `VoidCanopy`).
- Server-configurable aurora scheduling via `[aurora]` section in `server.toml`, `--aurora` CLI flag, and `LOW_CUBE_AURORA` env var, supporting `nightly`, `off`, `rare`, `normal`, and `frequent` modes (`AuroraCadence`, `NightlyAuroraCadence`, `DormantAuroraCadence`, `StochasticAuroraCadence`, `ServerAurora`, `FileAurora`, `ArgumentAurora`, `EnvironmentAurora`).
- Server-configurable day/night cycle duration in minutes via `cycle = <minutes>` in `server.toml`, `--cycle <minutes>` CLI flag, and `LOW_CUBE_CYCLE=<minutes>` env var (`CycleDuration`, `ServerCelestial`, `FileCelestial`, `ArgumentCelestial`, `EnvironmentCelestial`).
- In-game screenshot capturing nocturnal polar lights over the Arctic face in `docs/images/climates/arctic-night.png`.

### Changed
- Refactored `TunedCadence` into a pure `CelestialCadence` decorator delegating sun trajectory and sky coloring to inner origin.
- Refactored `StochasticAuroraCadence` into a pure `AuroraCadence` decorator modulating base nocturnal activity.
- Refactored `FileCelestial`, `ArgumentCelestial`, and `EnvironmentCelestial` to evaluate configuration once at startup, eliminating runtime disk I/O, regex/string scanning, and per-frame heap allocations in the 60 FPS animation loop.
- Eliminated static utility functions in favor of encapsulated domain objects (`StochasticHash`) and message passing (`Vec4::lerp`).
- Updated `docs/SETTINGS.md` and `server.toml` to document minute-based cycle durations and polar light settings.

## [0.20.0] - 2026-10-16

### Added
- Dynamic diurnal day/night cycle and celestial vault rendering pipeline (`DiurnalPipeline`, `CelestialCadence`, `TerrestrialCadence`, `TunedCadence`).
- Celestial sky vault shader (`GlFirmament`, `firmament_glsl.rs`) rendering a fiery sun with atmospheric extinction and corona, twinkling starfield, and an opaque, cratered lunar sphere.
- High-contrast multi-scale procedural crater field generator (`crater_field`, `lunar_basin`) modeling large dark basalt maria (Oceanus Procellarum, Imbrium, Serenitatis, Tranquillitatis, Crisium), medium impact craters with raised rims, dense micro-craterlets, and Tycho impact ejecta rays.
- Complete star occlusion behind solid celestial bodies (moon and sun) and solid lunar body compositing, preventing stars from shining through the moon.
- Stable celestial pole alignment (`u_celestial_pole`) preventing gimbal flips, rotation snaps, or singularity distortions as the moon traverses the zenith.
- Server-configurable day/night cadence supporting CLI flags (`--day-cycle <seconds>`, `--day-cycle-seconds=<seconds>`), environment variable (`LOW_CUBE_DAY_CYCLE`), and configuration file (`server.toml`), defaulting to 1200 seconds (20 minutes).
- Principle of Static Configuration on Startup documented in `docs/SETTINGS.md`.
- In-game day and night comparison gallery across all 6 biomes in `docs/CLIMATES.md` and `README.md`.

### Changed
- Extended `GraphicPipeline::render` signature and `Avatar::surface_rotation` to pass current planetary surface rotation for horizon and celestial alignment.
- Split firmament GLSL shader sources into dedicated `firmament_glsl.rs` to maintain strict adherence to the 200-line ceiling rule.

### Added
- Climate-adapted procedural low-poly tree and forestry system (`Arboreal`, `TreeSpecies`, `StandingTree`, `ClearedPlot`) across all six planetary biomes (`TemperateOak`, `ArcticPine`, `JungleEmergent`, `DesertSucculent`, `SteppeAcacia`, `VolcanicSpire`).
- Solid physical trunk collision and direct timber harvesting lifecycle with haptic strikes, wood particle bursts, and instant zero-stump vanishing to direct inventory deposit.
- Forestry decorator pipeline (`PlantingGround`, `FullCubeGround`, `WoodlandGround`, `MeridianGround`, `SpawnSanctuaryGround`, `ForestedFace`) providing flat-cube ground support checks, climate density modulation, and spawn sanctuary protection.
- Aerodynamic canopy drag physics (`CanopyBraking`) softening vertical falls and enabling high-altitude tree canopy navigation.
- Asymmetric slope pacing (`InclinePace`) modulating stride vectors across uphill inclines (-25% penalty) and downhill declines (+35% boost) across all gaits (walking, sneaking, and sprinting).
- Comprehensive planetary climate zones documentation in `docs/CLIMATES.md` with in-game photography, hazard profiles, and geological breakdown.
- Tree catalog and forestry reference documentation in `docs/TREES.md`.
- Interactive Python tree visualization script in `scripts/tree_catalog.py`.
- In-game climate zone screenshots in `docs/images/climates/` (Temperate, Arctic, Desert, Jungle, Steppe, Volcanic).

### Changed
- Refactored material blocks (`DirtBlock`, `SandBlock`, `SnowBlock`, `StoneBlock`) from fake inheritance wrapper structs into clean factories producing the canonical `MaterialBlock` decorator directly.
- Refactored landscape entities (`VolcanicCaldera`, `DesertDunes`, `GlacialMassif`, `JungleKarst`, `SteppeTerrace`, `VoxelMountain`, `BlockGallery`) from simulated subclasses into direct `VoxelCluster` decorator factories, eliminating double-boxing and forwarding boilerplate.
- Refactored `BiomedCubeWorld` into a factory assembling `CubePlanet` and its surrounding decorators directly.
- Embedded 6-climate showcase slideshow grid and documentation links into `README.md`.
- Renamed internal decorator implementation files (`decorator.rs` -> `modulation.rs`, `trial.rs`) to eliminate design pattern jargon.

## [0.18.0] - 2026-10-15

### Added
- Flush 2-row wedge seam alignment across all 12 planetary cube seams, terminating slope coordinates at the symmetric midpoint boundary line ($u \pm 0.5, v \pm 0.5$) with zero gap and zero polygon overlap.
- 3-way tri-chamfer corner cap convergence across all 8 cube vertices, joining orthogonal edge wedge rows at the shared triple point ($H - 0.5, H - 0.5, H - 0.5$).
- Seam wedge verification test suite in `crates/world/src/relief/wedge_seam_tests.rs` testing flush edge contact, corner cap convergence, and biome-differentiated palette preservation.
- Active Milestone 3 roadmap in `docs/ROADMAP.md` tracking flush 2-row wedge seams, corner caps, and edge-inward terrain heightfield anchoring.

### Fixed
- Eliminated geometry interpenetration and nested polygon clipping between adjacent planetary cube faces along 90-degree seams.

## [0.17.0] - 2026-10-14

### Added
- Diagonal slope block form (`DiagonalSlope`) and typed constructors (`diagonal_slope_ne`, `diagonal_slope_nw`, `diagonal_slope_se`, `diagonal_slope_sw`) across all material blocks (`DirtBlock`, `SandBlock`, `SnowBlock`, `StoneBlock`) providing 45-degree diagonal slopes with watertight boundary triangles.
- Watertight wall sealing regression test suite in `crates/world/src/relief/watertight_tests.rs` and corner seam test suite in `crates/world/src/relief/corner_seam_tests.rs`.

### Fixed
- Watertight relief mesh sealing between Full-Cube blocks and all non-flush adjacent shapes (ramp wedges, valley corners, ridge corners, and seam chamfers), preventing see-through gaps into cube interiors.
- Preserved sub-second client launch latency with zero-cost plateau early-exit and short-circuited boundary seam elevation lookups.

### Changed
- Updated gameplay demonstration screenshot in `docs/images/gameplay.png` and `README.md`.
- Documented diagonal slope block forms in `docs/BLOCKS.md`.

## [0.16.0] - 2026-10-14

### Added
- Diagnostic Tron wireframe rendering mode (`--tron` CLI argument and in-game toggle key `T`) rendering terrain and block geometries purely as glowing unlit neon vector lines against a pitch-black cosmic void.
- Emissive diagnostic signature colors for all 15 block shapes (`VoxelShape::shape_color`) enabling immediate visual classification of cuboids, ramps, diagonal ridge/valley corners, stairs, pyramids, and gentle slopes.
- Elevation discontinuity and seam mismatch diagnostics: non-matching adjoining blocks and vertical cliff steps emit high-contrast warning red lines (`#FF1744`).
- Tron GPU rendering pipeline (`TronPipeline`, `TronShaderSource`, `TronProgram`) with black occlusion face pass via polygon offset preventing see-through backfaces, paired with emissive line mesh drawing (`glow::LINES`).
- CPU wireframe line batching (`CpuLineBatch`, `CpuTronBatch`, `GlLineMesh`, `TronMesh`).
- Faceted relief Tron palettes in `ReliefPalette::tron()` and `FacetedReliefFace::tron` for planetary cube terrain.
- Decoupled OpenGL host environment initialization into `GlHostEnvironment` (`crates/client/src/host_env.rs`) maintaining strict single-responsibility and file line limits.
- Comprehensive Tron wireframe color and mismatch reference documentation in `docs/BLOCKS.md`.

## [0.15.0] - 2026-10-14

### Added
- Complete shape constructors across all typed material blocks (`DirtBlock`, `SandBlock`, `SnowBlock`, `StoneBlock`) supporting gentle lower/upper wedges, gentle ridge/valley corners, and stair corners (`stair_corner_convex`, `stair_corner_concave`).
- Concave stepped inner corner mesh generation in `StairCorner::emit_shape`.
- Block geometry exporter in `crates/world/examples/export_blocks.rs`.
- Interactive block catalog and snapshot CLI in `scripts/block_catalog.py` providing listing (`list`, `list --forms`, `list --materials`), single-block rendering (`render`), and batch rendering (`render-all`).
- Comprehensive block catalog and geometry reference documentation in `docs/BLOCKS.md` with visual matrix across all materials and shapes.
- High-resolution (512x512 with 2x SSAA and alpha transparency) snapshot images for all 105 available blocks in `docs/images/blocks/`.
- Markdown-only commit workflow exception and line-count exemption in `AGENTS.md`, alongside CI `paths-ignore` filter in `.github/workflows/release.yml`.

### Changed
- Linked block catalog documentation in `README.md`.

### Added
- Procedural planetary relief generation in `crates/world/src/relief/` (`ReliefElevation`, `ZonedRelief`, `ReliefFace`, `ReliefFacet`, `ReliefCorner`, `ReliefPalette`, `ReliefStrata`, `ReliefCavity`) generating continuous, watertight terrain facets and slopes.
- Coherent gradient noise engine in `crates/world/src/noise/` (`SpatialFrequency`, `CoherentNoise`, `DomainWarp`, `FractalHarmonics`) providing continuous multi-octave 3D frequency sampling.
- Distinct biome landscape topologies in `crates/world/src/landscape/`:
  - `GlacialMassif` (Alpine horns, arêtes, and cirque basins in Arctic)
  - `VolcanicCaldera` (Summit crater, caldera rim, and basalt lava flows in Volcanic)
  - `KarstTowers` (Limestone tower karst pillars and river valleys in Jungle)
  - `SteppeTerrace` (Stratified mesas and terraced plateaus in Steppe)
  - `DesertDunes` (Barchan sand dunes, slip faces, and interdune corridors in Desert)
- Slope and corner block forms in `crates/world/src/block/`:
  - `GentleWedge` (half-pitch gentle slope along primary axes)
  - `GentleRidge` (diagonal ridge corner connecting gentle slopes)
  - `GentleValley` (concave diagonal valley corner connecting gentle slopes)
  - `StairCorner` (stepped corner block)
- Ridge and valley diagonal corner generation in `ReliefCorner` with closed manifold facets and slope surface collision height in `ZonedRelief`.
- Test suites in `relief/tests.rs`, `noise/tests.rs`, and `landscape/tests.rs`.

### Changed
- Refactored `crates/client/src/launch.rs` and `main.rs` to launch directly with zoned procedural relief planet mesh assembly.
- Streamlined `Biome` in `crates/world/src/biome.rs` to configure distinct biome-specific relief surfaces and stratum color palettes.
- Purged `crates/client/src/world_choice.rs`.

## [0.13.0] - 2026-10-13

### Added
- 3D volume-partitioned cubic planet architecture (`VolumePlanet`, `SparsePlanetaryGrid`, `VoxelVolume`, `StratifiedVolume`, `MonolithVolume`, `VoidVolume`) with multi-layer surface voxel crusts across all 6 faces.
- Impenetrable bedrock containment shell (`BedrockShell`, `VoxelBarrier`, `PlanetaryCrust`) sealing the 0-byte hollow core void.
- Greedy meshing (`GreedyMesh`, `emit_volume_mesh`) with cross-volume boundary occlusion culling across internal and external seams.
- Integrated `PlanetaryTopology` seam coordinate folding into `VolumePlanet::constrain_to_surface`, enabling seamless 90° edge traversal and continuous 360° circuits on voxel worlds.
- Sneak cliff-arrest policy (`permits_seam_fold`) at 90° planetary cube seams preventing accidental edge crossings and camera flips, while enabling lateral sliding along edges.
- Polymorphic world assembly (`WorldAssembly`, `VolumeAssembly`, `BiomedAssembly`, `ArgumentWorld`), setting `VolumePlanet` as the default world when launching the client.
- Integration test suite in `crates/client/src/chunk_traversal_tests.rs` verifying voxel planet locomotion, jumping, edge crossing, and sneak cliff arrest.

### Changed
- Purged all legacy aliases and shims (`VoidChunk`, `UniformChunk`, `StratifiedChunk`, `FacetedChunk`, `ChunkCoord`, `Chunk`, etc.), strictly standardizing the domain model around `VolumePlanet`, `VoxelVolume`, `VolumeCoord`, and `VoxelCoord`.
- Replaced vague constructor `BedrockShell::standard_shell` with expressive domain constructor `BedrockShell::boundary`.

### Added
- Completed Roadmap Phases 4, 5, and 6: Decoupled camera horizon convergence, ballistic flight, 360° circuit verification, visual continuity, and performance audit.
- Decoupled physical locomotion surface reorientation (`surface_rot`) and visual camera horizon convergence (`camera_rot`) in `PlanetaryAvatar`.
- Time-damped smooth camera horizon convergence (`converge_camera`) with gradual exponential slerp upon reaching and traversing seams.
- Automated end-to-end 360° planetary circuit test in `crates/client/src/circuit_tests.rs` verifying continuous 4-face traversal and steady-state horizontal view pitch.
- Ballistic edge leap and mid-air trajectory continuity unit tests in `crates/client/src/ballistic_tests.rs`.
- Visual regression tests in `crates/client/src/visual_tests.rs` for directional lighting, shadow umbra continuity, and quad alignment across all 12 seams.
- Hot simulation loop zero-runtime-allocation audit in `crates/client/src/allocation_audit.rs`.
- Architectural rule "Fail Fast (Never Fail Safe)" added to `AGENTS.md`.

### Fixed
- Fixed centripetal altitude calculation in `CentripetalWorld` using cubic $L_\infty$ distance (`max(|x|, |y|, |z|) - half_size`) instead of Euclidean length, eliminating false radial gravity and forced downward tilt on cube surfaces.
- Restored sharp 90° voxel planetary world in `launch.rs` (`BiomedCubeWorld::of_size`), ensuring the avatar's gaze remains level and fully under player control before reaching edges.
- Fixed avatar edge sticking on subsequent seams by ensuring immediate physical gravity frame reorientation and accurate seam clearance classification in `threshold.rs`.

### Changed
- Modularized flight phases by splitting monolithic `phase.rs` into `crates/client/src/phase/` submodules (`grounded.rs`, `airborne.rs`, `context.rs`, `resolution.rs`), enforcing strict 200-line limits across all source files.
- Consolidated gait decorators into `crates/client/src/gait/mod.rs` and streamlined locomotion order dispatch in `crates/client/src/order.rs`.

## [0.11.0] - 2026-10-12

### Added
- Phases 3 & 4 of 90° Edge Traversal Roadmap: Continuous Locomotion Kinematics & Horizon Viewport Orientation.
- `SurfaceClearance` and `StepThreshold` in `crates/client/src/threshold.rs` distinguishing open-air drops from continuous 90° corner transitions without procedural boolean flags.
- `ViewFrame` trait in `crates/client/src/horizon.rs` providing pure Tell-Don't-Ask viewport view-matrix generation.
- `CameraHorizon` encapsulating surface-aligned look direction and `HorizonTilt` decorator applying camera roll without vector leaks.
- `SurfaceHeading` in `crates/client/src/heading.rs` encapsulating tangential locomotion steering.
- `ProjectionLens` trait and `MagnifiedLens` decorator in `crates/client/src/lens.rs`.
- `CentripetalWorld` decorator in `crates/world/src/centripetal_world.rs` (Phase 5 of Roadmap) implementing altitude-dependent radial gravity well falloff toward the planetary core.
- `ClimateZone` domain enum in `crates/world/src/topology/climate_zone.rs` replacing `CubeFace` with expressive domain naming.
- Ken Perlin's $C^2$ quintic smootherstep and elevation-invariant surface delta in `SeamBevel` for seamless, jerk-free edge transitions.
- Comprehensive unit test suites in `tilt_tests.rs`, `seam_clearance_tests.rs`, and `centripetal_tests.rs`.

### Changed
- Purged `basis.rs`, splitting responsibilities into cohesive domain objects `horizon.rs` and `heading.rs` according to Yegor Bugayenko's declarative naming principles.
- Replaced framerate-dependent camera slerp and snap threshold with exponential decay time-damped slerp ($1 - e^{-1.5 \Delta t}$) and widened bevel transition corridor.
- Purged all dead code, eliminated all `#[allow(dead_code)]`, and removed all `is_` / `has_` method prefixes repository-wide.
- Updated `AGENTS.md` to enforce the 200-line limit immediately upon each code edit and streamlined the `create git message` workflow.
- Updated `docs/ROADMAP.md` tracking progress across Phases 3, 4, and 5.

## [0.10.0] - 2026-10-12

### Added
- Phase 2 of 90° Edge Traversal Roadmap: Seam Surface Geometry & Coordinate Folding (`world` crate).
- `SeamBevel` and `BevelTransition` interface in `crates/world/src/topology/bevel.rs` providing continuous spherical slerp normal transitions across 90° edge seams.
- `BeveledWorld` decorator in `crates/world/src/beveled_world.rs` decorating any `PlanetaryWorld` with smooth seam normal interpolation.
- `FoldedFace` decorator in `crates/world/src/topology/folded_face.rs` wrapping any underlying `Face`, cleanly separating seam folding from planar math.
- Comprehensive unit tests in `bevel_tests.rs` and `corner_tests.rs` verifying seamless 4-edge coordinate folding and C1 normal continuity without corner pinning.
- `MeshTarget` decorators in `crates/world/src/mesh_target.rs`: `ColoredMeshTarget`, `TintedMeshTarget`, `OffsetMeshTarget`, and `CountedMeshTarget`.
- `ElevatedFace` and `TintedFace` decorators in `crates/world` for normal offsets and atmospheric surface tints.
- `ElevatedShape` decorator in `crates/world/src/block/elevated_shape.rs` for vertical voxel shape stacking.
- `SensitivityLook` in `crates/client/src/orientation.rs` scaling mouse rotation deltas.
- `InvertibleLook` with polymorphic `AxisPolarity` (`DirectPolarity`, `ReversePolarity`), eliminating primitive boolean flags for view inversion.
- `DampedLocomotionTarget`, `GroundedLocomotionTarget`, `TallyLocomotionTarget`, and `MutedControls` in `crates/client/src/controls/decorator.rs`.
- `OffsetClock` and `FrozenClock` decorators in `crates/client/src/clock.rs` for monotonic timeline manipulation.

### Changed
- Refactored material blocks (`DirtBlock`, `StoneBlock`, `SandBlock`, `SnowBlock`) into living objects implementing `Block` and decorating `MaterialBlock`.
- Refactored `MaterialBlock` to polymorphically wrap any `Box<dyn Block>`, using `ColoredMeshTarget` for surface emission.
- Eliminated procedural `emit_form` from `FormBlock`; mesh emission now runs uniformly through `Block::emit_surface`.
- Eliminated all boolean flags (`invert_x`, `invert_y`) from `MouseLook`.
- Removed passive getters `source_face`, `source_edge`, `destination_face`, and `destination_edge` from `SeamFold`, replacing them with Tell-Don't-Ask methods (`bevel`, `destination`, `reciprocal_on`).
- Removed passive getters `edge` and `overshoot` from `PerimeterCrossing` in favor of polymorphic `dispatch`.
- Removed unused `color` getter from `MaterialBlock`.
- Split `gait.rs` into `gait/mod.rs` and `gait/decorator.rs`, and `controls.rs` into `controls/mod.rs` and `controls/decorator.rs`, enforcing the 200-line limit across all files.
- Updated `docs/ROADMAP.md` marking Phase 2 as completed (4/4).

## [0.9.0] - 2026-10-12

### Added
- Phase 1 of 90° Edge Traversal Roadmap: Planetary Topology & Edge Neighborhoods (`world` crate).
- `PlanetaryTopology` connecting the 6 faces of a cubic world across 12 seam neighborhoods (24 directed seam folds) with $O(1)$ constant-time canonical array indexing.
- `SeamFold` encapsulating 90° rotation, seam axis, center translation, and continuous coordinate/vector folding across seams.
- `FoldedPoint` domain entity encapsulating 3D spatial position on destination frames with Tell-Don't-Ask transfer.
- `FacePerimeter` and `PerimeterCrossing` domain objects classifying tangential boundary states (`Interior`, `Crossing`) and clamping without procedural data extraction.
- `CubeFace` domain enum modeling the 6 canonical cubic planetary faces with normal, tangent, bitangent, and surface frame construction.
- Comprehensive unit test suite in `crates/world/src/topology/tests.rs` verifying coordinate continuity, seam reciprocity, 90° normal switches, and 360° closed planetary circuit.
- Updated gameplay demonstration screenshot in `docs/images/gameplay.png`.

### Changed
- Refactored `BiomedCubeWorld::of_size` to construct all 6 biome faces declaratively via `PlanarFace::on_cube` and `CubeFace` domain entities, eliminating vector duplication.
- Updated `docs/ROADMAP.md` marking Phase 1 (Planetary Topology & Edge Neighborhoods) as completed (4/4).

## [0.8.0] - 2026-10-12

### Changed
- Simplified server physics configuration to a single intuitive `gravity` parameter using clean whole integers without decimals.
- Derived avatar jump height dynamically and naturally from planetary gravity ($h = \frac{5.12}{g}$), preserving constant human leap muscle impulse.
- Replaced verbose Server & World Configuration section in `README.md` with direct link to the dedicated settings guide.
- Streamlined Windows pre-built binary launch instructions in `README.md` to direct double-clicking.
- Rewrote `docs/SETTINGS.md` into a dedicated settings guide featuring integer planetary presets (`10`, `5`, `2`, `1`, `20`), configuration template, and precedence order.
- Updated `FilePhysics`, `ArgumentPhysics`, and `EnvironmentPhysics` to parse only `gravity` and eliminate redundant `jump_height` parameters.
- Updated `CONCEPT.md` and `server.toml` to align with the single-parameter integer gravity model.

### Added
- Decoupled planetary jump height and gravity from world-cube dimensions across all world scales.
- Server and world physics configurability with Earth gravity ($g = 9.81\,\text{m/s}^2$) and realistic human vertical leap ($h \approx 0.522\,\text{m}$) as standard baseline.
- `server.toml` configuration template supporting custom `jump_height` and `gravity` parameters.
- Command-line argument support (`--jump-height <METERS>`, `--gravity <M/S2>`) and environment variable overrides (`LOW_CUBE_JUMP_HEIGHT`, `LOW_CUBE_GRAVITY`).
- Pure OOP `PlanetaryPhysics` interface with `EarthPhysics` and `TunedPhysics` implementations.
- Pure OOP decorator pipeline for physics resolution: `ArgumentPhysics` $\to$ `EnvironmentPhysics` $\to$ `FilePhysics` $\to$ `EarthPhysics`.
- Dedicated player and server-host documentation in `docs/SETTINGS.md` detailing presets, configuration options, and evaluation precedence.
- Comprehensive unit test suite in `crates/client/src/physics_tests.rs` covering all physics decorators, file configuration parsing, and CLI flag evaluation.

### Changed
- Refactored `SurfaceFlight` with `calibrated` secondary constructor computing initial vertical velocity from target peak jump height and gravitational acceleration.
- Refactored `PlanetaryAvatar` with `with_flight` secondary constructor for polymorphic flight injection.
- Updated `CONCEPT.md` Section 2.1 and Section 5.2 to document world-size independence and server physics configuration.
- Updated `README.md` with Server & World Configuration section and links to `docs/SETTINGS.md`.

## [0.6.0] - 2026-10-12

### Added
- Double-tap forward sprint locomotion: tapping `W` twice within 300 ms triggers high-speed sprinting at 160% walking pace.
- High-speed dynamic FOV expansion: `PerspectiveLens` smoothly interpolates field of view from 65° to 78° during sprint locomotion with zero GPU overhead.
- Pure OOP `SprintCadence` state pattern (`RestingCadence`, `EngagedCadence`, `PrimedCadence`, `SprintingCadence`, and `TransitioningCadence`) encapsulating timeout duration without procedural branching, primitive flags, or half-born objects.
- Pure OOP `MonotonicTime` trait decoupling clock reads from static platform calls, with `SystemClock` for hardware time and `ControllableClock` for deterministic microsecond-precision testing.
- Pure OOP `SprintGait` implementing `LocomotionGait` alongside refactored `WalkGait` and `SneakGait`, encapsulating pacing factor, elevation, and target FOV as private fields with expressive secondary constructors (`WalkGait::upright`, `SneakGait::crouching`, `SprintGait::swift`).
- Sprint cancellation upon backward key press (`KeyCode::KeyS`).
- Comprehensive unit tests verifying cadence transitions, timeout rejection, lens expansion/restoration, and deterministic mock clock control.

### Added
- Sneak locomotion activated by `Shift` (`ShiftLeft` / `ShiftRight`) with 35% movement pacing.
- Edge cliff protection restraining the avatar from stepping off elevated blocks while sneaking, while allowing smooth lateral sliding along ledge perimeters.
- Crouch kinematics with smooth avatar viewpoint lowering from standing eye level (0.8) to crouched eye level (0.45).
- Pure OOP `LocomotionGait` trait (`WalkGait` and `SneakGait`) encapsulating gait-specific pacing, posture shaping, and precipice resolution without getters, setters, or procedural branching.
- Pure OOP `AvatarPosture` encapsulating eye-level transitions and orientation basis view matrix generation.
- Polymorphic `LeapIntent` (`RestingLeap` and `ActiveLeap`) eliminating primitive boolean jump request flags from `LocomotionOrder`.
- Comprehensive test coverage for sneak edge restraint, posture transitions, crouch view matrices, and shift key controls.
- Updated gameplay demonstration screenshot in `docs/images/gameplay.png`.

## [0.4.0] - 2026-10-12

### Added
- Multi-form voxel geometry objects in `crates/world/src/block/`: `CuboidShape` (full cube and half slab), `StairFlight` (two-tiered stepped stair), `RampWedge` (45° climbable incline), `RidgeCorner` (convex edge corner), `ValleyCorner` (concave valley corner), and `PyramidPeak` (45° pyramidal apex).
- Pure OOP material blocks derived from `FormBlock` via decorator composition: `StoneBlock`, `DirtBlock`, `SandBlock`, and `SnowBlock` in dedicated `crates/world/src/block/material/` submodules.
- `BlockGallery` exhibition in `crates/world/src/gallery.rs` showcasing all materials in parallel lines with 1-block spacing across all geometric block shapes.
- Alpine `VoxelMountain` massif in `crates/world/src/mountain.rs` with smooth rock slab entrance shelf, continuous 45° slope, and crystalline snow pyramid summit.
- Wall-sliding mechanics preserving lateral strafe motion when running diagonally against obstacle walls in `crates/client`.
- Pure OOP `SurfaceStride`, `WalkTrial`, and `AerialTrial` in `crates/client/src/stride.rs` encapsulating obstacle deflection and step resolution without getters or control-flow `Option`.
- `SurfaceClearance` in `crates/client/src/threshold.rs` encapsulating altitude clearance relative to surface up vectors with precipice, floor contact, and cliff impact semantics.
- `FlightContext` and `FlightProgress` in `crates/client/src/phase.rs` encapsulating simulation environment and phase transitions, reducing `FlightPhase::advance` to a single context parameter.
- `GridCell::corner_points` in `crates/world/src/block/cell.rs` constructing quad vertices directly without coordinate data extraction.
- Comprehensive unit tests verifying block form constraints, material derivation, gallery exhibition, mountain elevation profile, and obstacle wall-sliding traversal.

### Removed
- Legacy `BlockGallery::single_row` compatibility constructor and `VoxelBlock` type alias.
- Redundant `Face` parameter from `Block::emit_surface`, `FormBlock::emit_form`, and `VoxelShape::emit_shape`.
- Dummy `let _ = face.pull_direction()` statements in `StairFlight` and `PyramidPeak`.

## [0.3.0] - 2026-10-12

### Added
- Authentic Earth gravity baseline ($g = 9.81\,\text{m/s}^2$) and realistic human vertical leap ($h \approx 0.5\,\text{m}$, $v_0 \approx 3.2\,\text{m/s}$) in `CONCEPT.md` and `crates/client`.
- Micro-Topology & Multi-Form Voxel Shapes in `CONCEPT.md` Section 2.6 specifying climbable $45^\circ$ Wedges, $0.5\,\text{m}$ rock shelves (Slabs), and vertical precipices.
- Variable World Cube dimensions ($128^3$ to $2048^3$) with $1024^3$ as Earth baseline, scale-derived jump dynamics, and hollow core specification with an indestructible 1-block bedrock shell in `CONCEPT.md` Section 2.1.
- Pure OOP `ReliefFace` and `RidgeElevation` in `crates/world` introducing 3D faceted mountain terrain with continuous $45^\circ$ slope climbing and rock shelves.
- Pure OOP `GravitationalFlight` and polymorphic `FlightPhase` (`GroundedFlight`, `AirborneFlight`) eliminating boolean state flags in `crates/client`.
- Unified `LocomotionTarget` messaging in `crates/client` eliminating primitive boolean bit-bags from `KeyMotion`.
- Steerable Mounts & Domestication in `CONCEPT.md` Sections 3.2 and 4.5 specifying craftable saddles, pack bags, mounted combat, and 90° edge traversal for large fauna.
- Architecture & Interactive Fixtures section to `CONCEPT.md` specifying openable doors and windows with environmental sealing and access controls.
- Updated gameplay screenshot in `README.md` and `docs/images/gameplay.png`.

## [0.2.0] - 2026-10-11

### Added
- GitHub Actions workflow (`.github/workflows/release.yml`) building standalone Linux (`low-cube-linux`) and Windows (`low-cube-windows.exe`) executables published as GitHub Releases and workflow artifacts.
- Dedicated developer guide in `docs/DEVELOPMENT.md` covering setup, dependencies, execution, and controls.
- Pre-built binary download and direct execution instructions in `README.md`.
- Workflow rules for version increments and changelog updates upon `create git message` prompt in `AGENTS.md`.

### Changed
- **Pure OOP Refactoring (Elegant Objects / Alan Kay)**:
  - Eliminated `normal()` getter on `Face` trait in favor of behavioral methods (`surface_up()`, `pull_direction()`, `constrain()`, `emit_surface()`).
  - Encapsulated `half_size` and `divisions` directly within `PlanarFace`, eliminating procedural parameter passing.
  - Replaced procedural data fields in `CubePlanet`, making it a pure composite of `Face` objects.
  - Removed `aspect_ratio()` getter from `PlatformSurface` trait; `render_frame` now takes `avatar: &dyn Avatar` directly ("Tell, Don't Ask").
  - Removed procedural matrix orchestration and data extraction in `GameSession::handle_window_event`.
  - Added `OrientationBasis::aligned` secondary constructor and `view_matrix_at` method to encapsulate coordinate alignment logic.
  - Renamed `KeyState` to `KeyMotion` to avoid passive data container semantics.
  - Added `ForwardPipeline::daylight()` secondary constructor to encapsulate sunlight direction and clear color without magic numbers.

## [0.1.0] - 2026-10-11

### Added
- Minimal runnable prototype for planetary voxel exploration in Rust and OpenGL 3.3.
- `crates/world`: Spherical-cubic planetary world body with multi-biome faces (`BiomedCubeWorld`).
- `crates/engine`: Forward graphics pipeline, shader compilation, and mesh batching.
- `crates/client`: First-person avatar navigation with surface-relative orientation, mouse look, and keyboard controls.
- Architecture and codebase rules enforced via Alan Kay and Yegor Bugayenko (*Elegant Objects*) principles in `AGENTS.md`.
