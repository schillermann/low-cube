# Sonnensystem-Checkliste (Pure OOP / Yegor Bugayenko)

Diese Checkliste dokumentiert den aktuellen Stand sowie alle verbleibenden Arbeitspakete für das maßstabsgetreue, frei skalierbare Würfel-Sonnensystem inkl. Erd-Atmosphäre, Gravity-Slingshots, interaktiver Systemkarte (`Taste M`), *The Expanse*-Fusionsantrieb und prozeduralem Asteroidengürtel.

---

## Architektur-Prinzipien nach Yegor Bugayenko (*Elegant Objects*)

Alle neuen und zukünftigen Objekte folgen diesen Leitplanken (automatisch geprüft durch `crates/client/src/architecture_tests.rs`):
- **Lebendige Objekte statt Datenbeutel (*Tell, Don't Ask*):** Kein `PlanetData`, keine Getter/Setter (`get_*`, `set_*`), kein `is_*`/`has_*`. Objekte übergeben Zustand per Callback (`transfer`, `observe_from`) oder wirken direkt auf Senken (`MeshTarget`, `VesselSink`).
- **Decorators statt Mega-Structs:** Maximal **4 Felder pro `struct`**, maximal **300 Zeilen pro `.rs`-Datei**, codefreie primäre Konstruktoren (`const fn new`).
- **Polymorphie statt `null` / `Option`-Felder:** Zustände werden durch sprechende Enums modelliert (`ActiveVicinity::Dormant | Awakened`, `ScaleOverride::Inherited | Specified`, `AtlasMode::BinaryPocket | Heliosphere`, `NavigationCourse::Uncharted | Charted`).
- **Deklarative Domänen-Nomen (keine `-er`/`-or`-Suffixe, keine vagen Namen):** Namen beschreiben, *was* ein Objekt ist (`SolarProportions`, `CubicRayleighShell`, `SolarEphemeris`, `FusionTorchStage`, `SolarChartModal`), niemals prozedurale Helfer (`OrbitManager`, `MapController`, `PlanetRenderer`).

---

## Meilenstein 1: Skalierbare Kosmos-Konfiguration & Multiplayer-Sync
- [x] `SolarProportions`, `BodyRatios` & `BeltAndReach` in `crates/world/src/celestial/proportions.rs` (`earth_blocks`, `body_scale`, `sun_scale`, `distance_scale`, `belt_density`)
- [x] Pure-OOP `CosmosScale` Decorator-Kette in `crates/client/src/cosmos_scale.rs` (`ArgumentCosmos > EnvironmentCosmos > FileCosmos(client.toml) > FileCosmos(server.toml) > MeasuredCosmos`)
- [x] Übertragung der `SolarProportions` im `LWEL`-Handshake (`crates/sanctuary/src/host_wire.rs`, `sanctuary_host.rs`, `remote_link.rs`, `remote_planet.rs`) + dynamische `heliosphere_span()`-Grenze
- [x] Anbindung an `SessionLaunch::windowed_cosmos` (`crates/client/src/launch.rs`) und `PortalConfig` (`crates/client/src/portal_config.rs`)
- [x] Unit-Tests in `crates/client/src/cosmos_scale_tests.rs`
- [x] Optional: Dedicated-Server-CLI in `crates/sanctuary/src/config/` ebenfalls um `[cosmos]`-Parameter (`distance_scale`, `body_scale`, `sun_scale`, `asteroid_density`) ergänzen

---

## Meilenstein 2: Eckige Sonne & Sichtbare Erd-Atmosphäre aus dem Weltall
- [x] Analytisches Rayleigh-Scattering `CubicRayleighShell` & `CubicSolarBody` (`crates/client/src/rayleigh_shell.rs`)
- [x] Sky-Shader-Erweiterung in `crates/client/src/firmament_glsl.rs`:
  - [x] Eckiger 3D-Sonnenwürfel mit abgeschrägter Leuchtkante & Korona
  - [x] Aus dem Orbit sichtbare Erd-Atmosphäre (azurblauer Limb-Glow auf der Tagseite, tief orange-roter Terminator, dunkle Nachtseite, nahtloser Übergang zum Taghimmel)
  - [x] Ferne kubische Planeten, quadratischer Saturn-Ring & ekliptisches Asteroidengürtel-Staubband
- [x] Orbitale Rayleigh-Luftperspektive & Dämmerungssaum auf beleuchtetem Erd-Terrain in `crates/engine/src/shader.rs`
- [x] Optische Invarianten-Tests in `crates/client/src/heliosphere_tests.rs`
- [x] Dynamische Übergabe der exakten Echtzeit-Planetenrichtungen aus `SolarEphemeris` (`FirmamentPlanets`) als kompaktes Uniform-Array an `GlFirmament` (`crates/client/src/gl_firmament.rs` & `diurnal_pipeline.rs`), inkl. scharfem quadratischem Mond-/Planeten-Finsternisschatten vor der eckigen Sonne

---

## Meilenstein 3: Bewegte Orbits, `1/r²`-Gravitation (Slingshots) & Alle Planeten/Monde
- [x] Katalog aller 21 Hauptkörper (`SOL`, `MERCURY`, `VENUS`, `EARTH`, `LUNA`, `MARS`, `PHOBOS`, `DEIMOS`, `CERES`, `JUPITER`, `IO`, `EUROPA`, `GANYMEDE`, `CALLISTO`, `SATURN`, `ENCELADUS`, `TITAN`, `URANUS`, `TITANIA`, `NEPTUNE`, `TRITON`) in `crates/world/src/celestial/catalog.rs`
- [x] `SolarEphemeris` (`DVec3` 64-Bit-Präzision, bewegte Kepler-Kreis-/Inklinationsbahnen, weitreichende `1/r²`-Mehrkörpergravitation mit Erd-Gezeitenkompensation) in `crates/world/src/celestial/ephemeris.rs`
- [x] `HeliosphereWorld` & `VicinityCache` (`crates/world/src/celestial/heliosphere/`):
  - [x] On-Demand-Generierung von Gesteins-/Eisplaneten und Monden (`AlienWorld` in `crates/world/src/celestial/alien_worlds.rs`) bei Annäherung (`<= 1` aktives Off-World-`CubePlanet` gleichzeitig im RAM/VRAM)
  - [x] Persistente Speicherung & Wiederherstellung von Bauten/Grabungen pro `BodyId` bei Planetenwechseln
  - [x] Geschachtelte, gegenläufig rotierende Wolken-Würfelschalen für Gasriesen + flacher quadratischer Saturn-Ring (`GiantCloudShell` in `facets.rs`) sowie Atmosphären-Widerstand (`canopy_drag`) und Kerndruck/Sonnenhitze (`lethal_at`)
- [x] Erweiterung von `CelestialAtlas` (`crates/world/src/celestial/atlas.rs`) um `AtlasMode::Heliosphere`
- [x] Physik-Tests für antriebslosen **Jupiter-Gravity-Assist-Slingshot** ($\Delta v$-Gewinn & Hyperbel-Ablenkung) sowie Mars-/Europa-Persistenz in `crates/client/src/heliosphere_tests.rs`
- [x] Multiplayer-`WorldFact` (`WorldFact::OffWorld`) in `crates/world/src/chronicle/` & `crates/sanctuary/` um `BodyId`-Adressierung erweitern, damit Co-op-Bauten auf Mars/Europa auch über Netzwerk-Snapshots synchronisiert werden

---

## Meilenstein 4: Interaktive Sonnensystem-Karte (`Taste M`) & Gekrümmte Flugbahnen
- [x] `ControlAction::SystemMap` (Standardbelegung `KeyCode::KeyM`, umbelegbar in Settings) in `crates/client/src/controls/bindings.rs` (`[InputTrigger; 21]`) und Anzeige in `ControlsSlateHud` (`crates/client/src/menu/controls_slate.rs`) ergänzen
- [x] Symplektischer Mehrkörper-Bahnintegrator (`SymplecticOrbit` `Velocity-Verlet` in `DVec3`) in `crates/client/src/drive/transfer_arc/`:
  - [x] Antriebslose gekrümmte Kepler-Ellipsen & **Slingshot-Hyperbeln** durch die sich weiterbewegenden Planetenfelder berechnen (statt gerader Sehne)
  - [x] Zukünftigen Bahnort des Zielplaneten zum Ankunftszeitpunkt ($t + \tau_{\text{arrival}}$) als Vorhaltepunkt bestimmen
  - [x] Umschaltbare Brachistochrone-Kurve (`BrachistochroneArc` mit markiertem `50 %` *Flip-and-Burn*-Umkehrpunkt) für Fusionsschiffe
- [x] `SolarChartModal` (`SessionModal`-Decorator in `crates/client/src/menu/solar_chart/`):
  - [x] Im Weltall / Cockpit per `Taste M` öffnen & schließen, Mauszeiger freigeben, Zoomen (Mausrad) und Schwenken
  - [x] 3D-/Hologramm-Darstellung von eckiger Sonne, allen 8 Planeten + Monden, Asteroidengürtel, Umlaufbahnen und der eigenen **gekrümmten Live-Flugbahn**
  - [x] Klick-Auswahl eines Himmelskörpers auf der Karte $\rightarrow$ berechnet Abfangkurs und überträgt ihn direkt in `HelmChart` (`crates/client/src/drive/helm_chart.rs`) & `NoseOrreryHud`
- [x] Unit- & Interaktions-Tests (`Taste M` öffnen, Mars/Jupiter anklicken, gekrümmte Bahn & Slingshot-Vorschau verifizieren in `crates/client/src/solar_chart_tests.rs`)

---

## Meilenstein 5: „The Expanse“ Fusionsantrieb (`FusionTorch` & Flip-and-Burn)
- [x] Neues Bauteil `BlockMedium::FusionTorch` (`crates/world/src/block/timber_grain/mod.rs`) & `HeldRelic::FusionTorch` (`crates/client/src/attire/relic/`, herstellbar im Crafting-Buch unter `"SPACE"`)
- [x] Skulptierte Epstein-Magnetdüse `FusionTorchMesh` (`crates/world/src/block/space_parts/fusion_torch.rs`) mit leuchtenden Supraleiter-Spulen und `2.85m` langer blau-weißer Plasma-Lanze
- [x] `FusionTorchStage` Decorator in `ImpulseStage` (`crates/world/src/relief/vehicle_assembly/impulse/stages.rs`) + Aufhebung des `45 m/s`-Limits für Fusionsschiffe (`MAX_FUSION_SPEED = 48_000 m/s`, bis `~8 G` Dauerbeschleunigung in `crates/client/src/drive/space_step.rs`)
- [x] Cockpit-Telemetrie für *Flip-and-Burn* (`TIME TO FLIP` / `RETRO FLIP NOW` / `BRAKE BURN` Anzeige in `NoseOrreryHud` & `ViewfinderHud`)
- [x] Kopplung dauerhafter High-G-Beschleunigung (`> 4 G`) an `AvatarPosture` (`HighGStrain` in `crates/client/src/posture/high_g.rs` für spürbaren High-G-Tunnelblick) + Reaktor-Soundwelle `FusionHum` in `crates/acoustics/src/fusion_hum.rs`
- [x] End-to-End Brachistochrone-Flugtest (Erde $\rightarrow$ Dauer-Burn $\rightarrow$ `RetroFlip` bei 50 % $\rightarrow$ Brems-Burn in den Mars-Orbit in `crates/client/src/solar_chart_tests.rs`)

---

## Meilenstein 6: Prozeduraler Asteroidengürtel & Abschluss-Qualitätssicherung
- [x] Deterministischer 3D-Torus-Gürtel `AsteroidBelt` (`2.20 .. 3.20 AU` zwischen Mars & Jupiter) in `crates/world/src/celestial/belt.rs`:
  - [x] `0 Byte` Grundspeicher durch Sektor-Hashing, konfigurierbar über `asteroid_density`
  - [x] Dynamisches Frustum-geculltes Streaming taumelnder Asteroiden-Würfel um das Schiff (`emit_cosmic_field`)
  - [x] `O(1)` Nachbarschaftsabfrage & On-Demand-Materialisierung einzelner Asteroiden als land- und abbaubares `CubePlanet`
- [x] Allocation-Audit-Test in `crates/client/src/allocation_audit.rs` für `0` Heap-Allokationen beim Durchflug durch den Asteroidengürtel
- [x] Vollständiger Testlauf über alle Crates (`cargo test --workspace`) & finale Prüfung aller Zeilen-/Feld-Limits (`architecture_tests`)
