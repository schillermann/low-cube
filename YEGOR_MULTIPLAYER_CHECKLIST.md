# Yegor Bugayenko (Elegant Objects) — Schrittweise Architektur-Checkliste: Multiplayer, Fahrzeuge & Chunk-Meshing

## Wie Yegor Bugayenko vorgehen würde (Leitprinzipien)

Nach **Alan Kay** und **Yegor Bugayenko (*Elegant Objects*)** wird ein Architekturproblem niemals durch „schnelles Dazubauen von Flags, Caches oder Sonderfällen" in bestehenden großen Klassen gelöst, sondern in kleinen, streng gekapselten Schritten:

1. **Verträge vor Implementierung (*Interfaces First*)**: Bevor eine Zeile Logik entsteht, wird ein schmales `trait` (2–4 Methoden, reines *Tell, Don't Ask*, keine Getter) definiert.
2. **Fake-Objekte & Reproduktions-Tests zuerst (*Fake Objects, Not Mocks*)**: Jede Phase beginnt mit einem deterministischen In-Memory-Test (`< 50 ms`, ohne Fenster/GPU/Echtzeit-Sleep), der das Architekturproblem isoliert beweist.
3. **Lebende Domänen-Objekte mit Identität statt Daten-Beutel**: Ein Fahrzeug behält nach dem Bau seine eigene Identität (`CraftIdentity`) in der Welt – es wird beim Einsteigen **nicht** als rohe Bauteilliste pro Frame kopiert.
4. **Dekoratoren & Komposition statt Klassen-Wachstum**: Kein `struct` überschreitet **4 private Felder**, keine Datei **300 Zeilen** (Ziel `< 250` Zeilen), keine Klasse trägt `-er`/`-or`-Suffixe (`Manager`, `Controller`, `Syncer`, `Uploader`) oder `Option<T>`-Zustandsfelder.
5. **Jeder Schritt hinterlässt einen grünen Build**: Keine halbfertigen Umbauten über mehrere Module hinweg; toter Altcode wird im selben Schritt rückstandslos gelöscht (*No Dead Code*).

---

## Phase 1: Fahrzeuge als echte Welt-Entities mit Identität & Ownership (`crates/world`, `crates/sanctuary`, `crates/client`)

**Ziel:** Fahrzeuge bleiben jederzeit physische Bestandteile von `PlanetaryWorld` (inkl. Kollision, Beifahrer-Sitzen, eindeutiger `CraftIdentity` und Host-autorisiertem Lenkrad-Lock ohne Block-Listen-Kopien pro Netzwerk-Puls).

- [x] **1.1 Unveränderliche Fahrzeug-Identität & Bauplan eingeführt** (`crates/world/src/relief/craft_entity/blueprint.rs`):
  - `CraftIdentity` (`code: u64`, 1 Feld, kanonisch sortierter FNV-1a-Digest) als eindeutige Identität jedes vom Kran gelösten Fahrzeugs, die über Park- und Einsteigezyklen in `ParkedFleet` erhalten bleibt.
  - `CraftBlueprint` (`identity: CraftIdentity`, `parts: Vec<VehiclePart>`, 2 Felder) kapselt Block-Strukturen bis `512` Blöcke ohne Trunkierung.
  - Atomarer Roundtrip-Test `test_large_blueprint_roundtrip_and_80_byte_pose_wire_budget` für 96-Block-Fahrzeuge grün.
- [x] **1.2 Kompakte Bewegungs-Pose & polymorphe Sitzbelegung modelliert** (`crates/world/src/relief/craft_entity/pose.rs`):
  - `CraftPose` (`origin: Vec3`, `axes: VehicleAxes`, `heading: Vec3`, `wheel_motion: (f32, f32)`, 4 Felder) kapselt 68-Byte-Pose, Raddrehung und Lenkeinschlag.
  - Polymorphes Enum `SeatOccupancy` (`Vacant`, `Helmed`, `Crewed`) ohne `Option<u32>`.
- [x] **1.3 Lebendes `WorldCraft`, `BlueprintCatalog` & `HelmLocks` verankert** (`crates/world/src/relief/craft_entity/mod.rs`, `crates/sanctuary/src/fleet_ledger.rs`):
  - `VehicleAssembly` auf 2 komponierte Domänen-Felder (`blueprint: CraftBlueprint`, `pose: CraftPose`) reduziert.
  - Fahrende Fahrzeuge bieten echte physische Kollisions-/Standfläche (`constrain_crafts`) sowie besteigbare Zweitsitze (`VehiclePilot::Passenger`, `board_passenger_seat`, `passenger_seat_frame`).
- [x] **1.4 Autoritativer Ownership-Lock über `WorldFact` & `WardedPlanet`** (`crates/world/src/chronicle.rs`, `crates/sanctuary/src/warded_planet.rs`):
  - `WorldFact` auf `CraftCommissioned`, `HelmClaimed` und `HelmReleased` (77 Byte) umgestellt.
  - Atomarer Host-Lock in `WardedPlanet` + automatischer Rollback (`helm_revoked`) bei gleichzeitigem Doppelklick zweier Gäste im Test `test_multiplayer_vehicle_ownership_lock_passenger_seat_and_smooth_glide` bewiesen.
- [x] **1.5 Netzwerk-Puls auf konstante 80-Byte-`DrivenCraft` verschlankt** (`crates/world/src/presence/craft.rs`, `crates/sanctuary/src/envelope.rs`):
  - `DrivenCraft` ist ein 80-Byte-`Copy`-Wert (`slot + CraftIdentity + CraftPose`) ohne Heap-Allokation im 120-Hz-Hot-Loop.

---

## Phase 2: Räumliches `16×16`-Sektor-Meshing & persistenter Streaming-VBO (`crates/world`, `crates/engine`, `crates/client`)

**Ziel:** Kein einzelner Block-Abbau, Riss (`CrackNotice`), Parkvorgang oder Wiesen-Übergang baut jemals wieder eine komplette Würfelseite oder das gesamte Grasfeld auf dem Main-Thread neu; `O(1)` GPU-Updates unabhängig von `server.toml`-Weltgröße.

- [x] **2.1 Persistentes `StreamingMesh` in `engine` statt VAO-Neuanlage pro Frame** (`crates/engine/src/streaming_mesh.rs`):
  - `StreamingMesh` (`batch: CpuMeshBatch`, `state: RefCell<StreamState>`, 2 Felder) mit polymorphem `StreamState` (`Cold` | `Warm(GlStreamSlot)`) wiederverwendet GPU-Handles per `STREAM_DRAW`-Buffer-Orphaning.
- [x] **2.2 `PatchCoord`, `DirtySectors` & `SectorGridMesh` für `16×16`-Terrain-Sektoren eingeführt** (`crates/world/src/planet/patches.rs`, `crates/world/src/relief/face/patch_grid.rs`, `crates/engine/src/sector_mesh.rs`):
  - Polymorphe `Mesh::patch_at` / `Mesh::store_patch`-Nachrichten auf `PlanetaryMesh` und `SectorGridMesh` ohne Type-Casting.
- [x] **2.3 Sektor-Updates in `ReliefCellCache` & `ShadedScenePass` aktiviert** (`crates/world/src/relief/face/cache.rs`, `crates/client/src/session/display/shaded.rs`):
  - `drain_modified_patch` aktualisiert pro Änderung nur die 1–4 betroffenen `16×16`-Sektoren; ein einzelner Grab-Riss invalidiert exakt `1` Sektor.
- [x] **2.4 Ring-adressiertes `MeadowRing` (`9×9` Torus) für Gras-Teppiche umgesetzt** (`crates/world/src/relief/face/meadow_ring.rs`, `crates/world/src/planet/patches.rs`):
  - Beim Fahren über eine 8-Meter-Sektorgrenze bleiben über 50 der 81 Gras-Patches unangetastet im VRAM; veraltetes `Avatar::emit_surroundings` rückstandslos entfernt.

---

## Phase 3: Entkoppelter Netzwerk-Herzschlag unabhängig vom Render-Loop (`crates/sanctuary`, `crates/client`)

**Ziel:** Fenster-Verschieben, VSync-Drosselung im Hintergrund oder kurze GPU-Spikes halten niemals wieder `SanctuaryHost::pump_network` oder `RemoteLink::pump` an.

- [x] **3.1 Schmale `PortalMailbox`-Statusschnittstelle definiert** (`crates/sanctuary/src/mailbox/mod.rs`):
  - Lock-freies `PortalMailbox` (`alive: AtomicBool`, `surface_dirty: AtomicBool`, 2 Felder) mit atomarem Test `test_portal_mailbox_atomic_latch_and_halt`.
- [x] **3.2 Autonomer `SocketLoop` mit eigenem 120-Hz-Takt umgesetzt** (`crates/sanctuary/src/mailbox/socket_loop.rs`):
  - Polymorphes `SocketThread` (`Detached` | `Running(JoinHandle<()>)`) taktet `pump_portal` streng monoton alle `8,33 ms` und beendet sich sauber via `Drop`.
  - Deterministische Einzel-Thread-Klassen (`HostedPlanet`, `RemotePlanet`) bleiben für `< 1 ms`-Unit-Tests erhalten; `step_local_flora` verhindert Doppel-Taktung.
- [x] **3.3 `ConduitPlanet`-Decorator für Live-Sessions eingehängt** (`crates/sanctuary/src/conduit_planet.rs`, `crates/client/src/portal_config.rs`):
  - Alle Live-P2P-Welten (`LivePortal::open`, `PortalDial`, `PortalConfig::assemble_planet`) laufen über `ConduitPlanet::spawning`.
  - Test `test_conduit_planet_relays_guest_motion_while_host_render_loop_stalls` beweist unterbrechungsfreies Weiterleiten zwischen Gästen trotz komplett pausiertem Host-Render-Loop.

---

## Abschluss-Prüfung (Verifiziert)

- [x] **Zeilenlimits (`wc -l`)**: Alle neuen und geänderten `.rs`-Dateien strikt `<= 300` Zeilen.
- [x] **Elegant-Objects-Wächter**: `cargo test --offline -p client --bin client architecture_tests` grün.
- [x] **Namens- & Zustandsdisziplin**: Keine `-er`/`-or`-Klassennamen, keine generischen Wörter, keine `is_`/`has_`-Präfixe, kein `Option<T>`-Objektzustand, kein Dead Code (`0` Compiler-Warnungen).
- [x] **Zero-Allocation im Hot-Loop**: `cargo test --offline -p client --bin client allocation_audit` sowie `cargo test --offline -p fauna` grün.
- [x] **Zielgerichtete Domänen- & Multiplayer-Tests**: `world`, `engine`, `sanctuary`, `acoustics`, `fauna` und `client` grün.
