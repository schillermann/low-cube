# Umsetzungsplan: Kuratierter Kosmos-Maßstab & Skalierbare Multiplayer-Heliosphäre

Dieses Dokument beschreibt den detaillierten Architektur- und Schritt-für-Schritt-Umsetzungsplan, um:
1. die fehleranfälligen separaten Kosmos-Verzerrungsfaktoren (`body_scale`, `sun_scale`, `distance_scale`, `[cosmos] scale`) zu entfernen und das gesamte Sonnensystem sauber an den **einen World-Cube-Skalierungsfaktor `size = 1..=32` (`128³` bis `4096³` Blöcke, Standard `size = 8` = `1024³`)** mit **garantierter `< 1 Stunde` Epstein-Flugzeit bis zum äußeren Asteroidengürtel (`3,20 AU`)** zu koppeln,
2. den **Raketen-Abprall-Bug** beim Rückflug zur Heimatwelt sowie auf Gasriesen strukturell zu beseitigen,
3. **Sektor-Streaming (Quadtree-LOD)**, Aufhebung des `512`-Block-Deckels (bis `4096³`) und **echte begeh-/abbaubare Voxel-Asteroiden im Asteroidengürtel (`BeltRock`)** einzuführen, und
4. **lokale Hill-Sphären-Referenzsysteme („Gravity Bubbles“)** einzuführen, damit große Planeten bis `4096³`, Hunderte verteilte Multiplayer-Spieler und äußere Planeten bis Neptun ohne RAM-Explosion, ohne eingefrorene Umlaufbahnen und ohne `f32`-Jitter funktionieren.

---

## 1. Diagnose der aktuellen Engpässe (Warum ein Gesamtkonzept nötig ist)

Alle beobachteten Symptome im aktuellen Stand hängen technisch direkt zusammen:

1. **Zu kleine `128³`-Testwelt für Multiplayer:**
   Der aktuelle Fallback `MoonletScale` (`128` Blöcke Kantenlänge, `earth_half = 64 m`) bietet pro Klimazone nur `64 m × 64 m` Fläche – für mehrere Spieler, Basen, Kräne und Fahrzeuge viel zu eng. Für echten Multiplayer ist laut `CONCEPT.md` eine **`1024 × 1024 × 1024`-Heimatwelt** (`earth_half = 512 m`) erforderlich.
2. **Unsichtbare Raketen-Abprallwand über dem World-Cube:**
   `TouchdownStep::resolve` (`crates/client/src/drive/space_step.rs`) prüft auf Bodenkontakt, sobald die Gravitation `> 0,2 m/s²` beträgt (`r ≈ 448 m` bei `128³`, bzw. **`r ≈ 3.584 m` bei `1024³`**). `HeliosphereWorld::near_earth` (`crates/world/src/celestial/heliosphere/mod.rs`) leitet `constrain_to_surface` jedoch nur bis `earth_half + 64 m` an die Erde weiter und gibt darüber im Vakuum stumm den um `1,15 m .. 3,65 m` angehobenen Sondenpunkt (`FaceContact::ground`) selbst zurück. `HullClearance` interpretiert das als „Rakete steckt `2,5 m` unter der Erde“ (`alt < 0`) und stößt sinkende Schiffe (`v_norm < 0`) endlos ins All zurück. Auf Gasriesen passiert bei `~7,8 km` exakt dasselbe.
3. **`f32`-Präzisionskollaps bei `1024³`-Weltgröße:**
   Bei einer `1024³`-Erde wächst `1 AU` auf `~12,02 Mio. m` und Neptuns Bahn (`30,07 AU`) auf **`~361,5 Mio. m`**. In erdzentrierten `f32`-Koordinaten (`Vec3`) beträgt die Schrittweite (`1 ULP`) schon bei Mars/Venus `~1 m` und bei Neptun **`32 m`** (unbrauchbar für `0,5 m`-`GridCell`-Blöcke) – zusätzlich liegt Neptun jenseits der `100 Mio. m` Outerra-Depth-Far-Plane.
4. **RAM-Limit (`512`-Block-Deckel) & 2-Slot-Engpass im Multiplayer:**
   `SolarProportions::voxel_blocks` klemmt Planeten heute hart bei `512` Blöcken ab, weil `FacetedReliefFace` ganze Würfelseiten am Stück im Speicher aufbaut. Zudem hält `VicinityCache` global nur **2 Slots** (`[ActiveVicinity; 2]`) vor und hält in `HeliosphereWorld::step_surface` die gesamten Planetenbahnen an (`if !locked`), sobald ein einziger Mond geladen ist. Verteilen sich im Multiplayer 4 Spieler auf Erde, Mond, Mars und Europa, verdrängen sie sich gegenseitig im Sekundentakt die Planeten unter den Füßen.
5. **Zerschossenes Antriebs-Balancing durch freie Faktoren:**
   Bei `1024³` (`R = 512 m`, `g = 9,81 m/s²`) steigt die Kreisbahngeschwindigkeit für einen niedrigen Erdorbit auf **$v_{\text{orb}} = \sqrt{g \cdot R} = 70{,}9\,\text{m/s}$** und die Fluchtgeschwindigkeit auf **$v_{\text{esc}} = \sqrt{2gR} = 100{,}2\,\text{m/s}$**. Mit dem bisherigen `128³`-Limit `MAX_COSMIC_SPEED = 45,0 m/s` käme keine chemische Rakete mehr in den Orbit; umgekehrt zerschießen freie Spieler-Faktoren jedes verlässliche Fahrzeug-Blueprint.

---

## 2. Verbindliche Architektur-Leitplanken (Pure OOP / Elegant Objects)

Jeder Schritt dieses Plans unterliegt ausnahmslos den Regeln aus `AGENTS.md`:
* **Maximal 4 private Felder pro `struct`** – größere Zustände werden in fokussierte Kollaborateure zerlegt.
* **Maximal 300 Zeilen pro `.rs`-Datei** (ohne einzeilige Code-Quetschung); sofortige Prüfung nach jeder Änderung.
* **Deklarative Domänennamen ohne `-er`/`-or`-Suffixe** (kein `Manager`, `Resolver`, `Loader`, `Streamer`, `Controller`, `Provider`).
* **Keine vagen/generischen Namen** (`Standard`, `Default`, `Common`, `Base`, `Simple`, `Generic`, `Abstract`, `Data` sind verboten $\rightarrow$ stattdessen z. B. `TerrestrialCosmos`, `PocketCosmos`, `CrustContact`, `VacuumMedium`).
* **Kein `is_`- oder `has_`-Präfix** bei booleschen Methoden (stattdessen `landable()`, `within_well()`, `subdivided()`, `airborne()`).
* **Keine Getter/Setter & keine stummen Dummy-Fallbacks** (*Tell, Don't Ask* & *Fail Fast*); kein `Option<T>` für uninitialisierte Objektzustände.

---

## 3. Zielwerte des skalierbaren Kosmos-Maßstabs (`size = 1..=32`)

Die separaten Verzerrungsfaktoren (`body_scale`, `sun_scale`, `distance_scale`) entfallen. Es bleibt **genau ein ganzzahliger Skalierungsfaktor `size` (`earth_blocks = 128 × size`, Standard `size = 8` = `1024³`, Großserver-Maximum `size = 32` = `4096³`)**, mit dem Planeten und Sonnensystem mitwachsen – nach oben begrenzt durch die Spielregel **„Epstein-Drive erreicht den äußeren Asteroidengürtel (`3,20 AU`) immer in `< 1 Stunde`“**:

| Kenngröße / Himmelskörper | `size = 1` (`128³`) | `size = 8` (`1024³` Standard) | `size = 32` (`4096³` Großserver-Max) | Bemerkung |
| :--- | :--- | :--- | :--- | :--- |
| **Erde (`EARTH`, Fläche `6×D²`)** | `128` Blöcke (`0,10 km²`) | **`1024` Blöcke (`6,29 km²`)** | **`4096` Blöcke (`100,7 km²`)** | `f32`-Präzision selbst bei `4096³`: `0,24 mm` |
| **Erdmond (`LUNA`, `0,273×`)** | `36` Blöcke | **`280` Blöcke** | **`1118` Blöcke** | Echte Kraterlandschaften & Mondbasen |
| **Mars (`MARS`, `0,532×`)** | `68` Blöcke | **`544` Blöcke** | **`2180` Blöcke** | Über bisherigem `512`-Cap $\rightarrow$ Phase 3 |
| **Venus (`VENUS`, `0,949×`)** | `122` Blöcke | **`972` Blöcke** | **`3888` Blöcke** | Nahe Erdgröße mit dichter Atmosphäre |
| **Phobos / Deimos** | `12` / `10` Blöcke | **`92` / `76` Blöcke** | **`368` / `308` Blöcke** | Von Haus aus groß & begehbar |
| **Gürtel-Asteroiden (`BeltRock`)** | `16..32` Blöcke | **`32..96` Blöcke** | **`64..128` Blöcke** | Echte abbaubare Voxel-Erzwürfel (`2,2..3,2 AU`) |
| **Astronomische Einheit (`1 AU`)** | `~1,50 Mio. m` | **`~12,02 Mio. m`** | **`~48,09 Mio. m` (`AU`-Deckel)** | Garantiert `< 1 h` zum äußeren Gürtel (`~41–55 Min.`) |
| **Erdorbit / Fluchtgeschw.** | `25,1` / `35,4 m/s` | **`70,9` / `100,2 m/s`** | **`141,8` / `200,5 m/s`** | Abgeleitet aus $g = 9{,}81\,\text{m/s}^2$ ($\propto \sqrt{\text{size}}$) |
| **Chemische Raketen-Obergrenze** | `58,0 m/s` | **`164,0 m/s`** | **`328,0 m/s`** | Skaliert automatisch mit $\sqrt{\text{size}}$ |
| **Epstein-Drive (`FusionTorch`)** | `8 G` (`>= 200 km/s`) | **`8 G` (`>= 200 km/s`)** | **`8 G` (`>= 200 km/s`)** | Äußerer Gürtel (`3,2 AU`) in `~7` / `~21` / `~41–55 Min.` |

> **Garantie:** Durch den oberen `AU`-Deckel (`max. ~48,09 Mio. m`) und das geöffnete Epstein-Geschwindigkeitslimit (`>= 200 km/s` statt `48 km/s` + Log-Depth-Far-Plane `10 Mrd. m` in `shader.rs`) bleibt der äußere Asteroidengürtel zwischen Mars und Jupiter (`3,20 AU`) auf **jeder** Weltgröße in unter einer Stunde (Flip & Burn inkl. Abbremsen) erreichbar.

---

## 4. Übergreifende Master-Checkliste (Vollständig abgeschlossen in `v0.112.0`–`v0.118.0`)

- [x] **Phase 1: Polymorpher Krusten- & Raumkontakt (Fix des Raketen-Abprall-Bugs)**
  - [x] Domänenobjekt `SurfaceContact` (`Vacuum`, `GasEnvelope`, `SolidCrust`) in `crates/world` einführen
  - [x] `PlanetaryWorld::surface_contact` ohne Dummy-`position`-Fallback implementieren (`CubePlanet`, `TranslatedWorld`, `BinaryPlanet`, `HeliosphereWorld`, `OrbitalFleetWorld`)
  - [x] `HeliosphereWorld::near_earth` von starren `+ 64 m` auf den echten Gravitations-/Kontaktbereich umstellen
  - [x] `HullClearance` und `TouchdownStep` auf `SurfaceContact` umstellen (kein Lande-Snap im Vakuum oder in Gasriesen-Wolken)
  - [x] Atomare Reproduktions- und Regressionstests für Rückflug aus `1.000 m` / `5.000 m` Höhe (senkrecht & über 45°-Kanten) sowie Gasriesen-Eintauchen grün
- [x] **Phase 2: Gekoppelter Kosmos-Maßstab (`size = 1..=32`) & `< 1h` Epstein-Gürtel-Garantie**
  - [x] Separate Verzerrungsfaktoren (`[cosmos] scale`, `body_scale`, `sun_scale`, `distance_scale`, `--*-scale`) entfernen; `[world] size` (`1..=32`, bis `4096³`) als einzigen Maßstabs-Parameter beibehalten
  - [x] `SolarProportions::astronomical_unit` mit dem `< 1h`-Epstein-Deckel (`<= 48,1 Mio. m`) versehen und Log-Depth-Far-Plane in `crates/engine/src/shader.rs` auf `10 Mrd. m` erweitern
  - [x] `BeltDensity` (`crates/client/src/cosmos_scale.rs`) und `ServerCosmosScale` (`crates/sanctuary/src/config/cosmos.rs`) verschlanken (nur optionale `asteroid_density`, kein toter Code!)
  - [x] Chemische Antriebsgrenze (`MAX_COSMIC_SPEED`) mit $\sqrt{\text{size}}$ skalieren und künstliche `48 km/s`-Drossel (`MAX_FUSION_SPEED`) für den Epstein-Drive öffnen (`240 km/s`)
  - [x] Multiplayer-Handshake (`seal.rs` / `remote_link.rs`) und Dokumentation (`server.toml`, `docs/SETTINGS.md`) aktualisieren
- [x] **Phase 3: On-Demand Sektor-Streaming, `4096³`-Freigabe & Begehbarer Asteroidengürtel**
  - [x] `ReliefCellCache` (`crates/world/src/relief/face/cache.rs`) von Eager-Vollflächen-Generierung auf bedarfsgesteuerte Sektor-Materialisierung umstellen
  - [x] Grobes Orbital-Facetten-Mesh (`OrbitalShell`) für ferne/ungeladene Sektoren großer Planeten (`512..=4096` Blöcke) ergänzen
  - [x] `512`-Block-Clamp für Felsplaneten/Monde auf `4096` anheben (`EARTH` bis `4096`, `VENUS` bis `3888`, `MARS` bis `2180`)
  - [x] Angeflogene Gürtel-Asteroiden (`BeltRock` / `BodyId::belt_rock` in `2,20..3,20 AU`) on-demand als echte begeh- und abbaubare Voxel-Würfel (`16³..128³` mit reichen `LivingVeins`) materialisieren
  - [x] Server-Welthaltung für alle Körper & aktiven Asteroiden von GPU-Mesh-Slots entkoppeln (`OffworldVault` + `SpatialFactIndex` pro `(BodyId, ClimateZone, Sector)` ohne 2-Slot-Verdrängung im Multiplayer)
  - [x] Startzeit-, RAM- und `allocation_audit`-Tests für `1024³`- und `4096³`-Heimatwelten grün
- [x] **Phase 4: Mitbewegte „Gravity Bubbles“ (`CosmicFrame`) gegen `f32`-Jitter & Orbit-Freeze**
  - [x] Lokale Referenz-Koordinate `CosmicFrame` (`Body`, `BeltAsteroid`, `DeepSpace`) und `GravityBubble` (Hill-Sphären-Radien in `f64`) definieren
  - [x] Nahtlosen Referenzwechsel (`FrameHandoff`) für `RocketDrive` (`SpaceState`) und Fußgänger-Avatar beim Durchfliegen einer Hill-Sphäre implementieren
  - [x] Erd-Sonderrolle in `HeliosphereWorld` auflösen: Erde nutzt denselben `BodyId::EARTH`-Frame-Mechanismus wie Mond, Mars und Titan
  - [x] Ephemeriden-Sperre (`if !locked` in `HeliosphereWorld::step_surface`) entfernen $\rightarrow$ kontinuierliche Live-Planetenbahnen und echte Eigenrotation auch während Landungen
  - [x] Kamera- und Rendering-Pipeline (`CosmicBodyCube`, `GiantCloudShell`, `TranslatedWorld`) relativ zum aktiven lokalen `CosmicFrame`-Ursprung rendern (Sub-Millimeter-`f32`-Präzision bis Neptun/Triton)
- [x] **Phase 5: Multiplayer-Protokoll, Navigations-HUD & Endabnahme**
  - [x] Sanctuary-Netzwerkprotokoll (`CompanionPresence`, `DrivenCraft`, `WorldFact`, `WardedPlanet`) auf `CosmicFrame`- und `VesselBerth`-Verankerung erweitern
  - [x] Anti-Cheat (`WardedPlanet`) pro aktivem Himmelskörper-Frame validieren statt globalem `heliosphere_span`
  - [x] `HelmChart`, `TransferArc` und `OrreryFrame` auf `1024³`-Maßstab, bewegte Referenzblasen und `CargoBallast`-Masse abstimmen
  - [x] Vollständiger Testlauf (`cargo test --workspace`), Zeilenlimit-Prüfung (`<= 300` Zeilen) und Pure-OOP-Audit

---

## 5. Detaillierter Schritt-für-Schritt-Plan

### Phase 1: Polymorpher Krusten- & Raumkontakt (Sofort-Fix für das Abprallen)

**Ziel:** Raketen dürfen im freien Weltraum und in Gasriesen-Atmosphären niemals fälschlich Bodenkontakt gemeldet bekommen; Rückkehr und Landung auf dem World-Cube funktionieren aus jedem Anflugwinkel (auch über Kanten/Ecken) und bei jeder Planetengröße zuverlässig.

1. **Domänenvertrag `SurfaceContact` anlegen (`crates/world/src/surface_contact.rs`, `< 90` Zeilen):**
   * Polymorphes Enum ohne `Option`:
     * `SurfaceContact::Vacuum` (freier Weltraum außerhalb jeder festen Kruste),
     * `SurfaceContact::GasEnvelope { drag: f32 }` (Gasriesen-Wolkenschichten),
     * `SurfaceContact::SolidCrust { ground: Vec3, normal: Vec3 }` (echter Voxelboden).
   * Tell-Don't-Ask-Verhalten auf `SurfaceContact` kapseln (z. B. Höhen- und Aufsetzberechnung über Closure-/Delegations-Methoden statt Getter).
2. **`PlanetaryWorld` erweitern (`crates/world/src/contracts.rs` & Implementierungen):**
   * Methode `surface_contact(&self, probe: Vec3, clearance: f32) -> SurfaceContact` ergänzen.
   * In `CubePlanet`: prüft, ob `probe` innerhalb der realen Anflug-Hülle (`chebyshev <= half_size + approach_band`) liegt; falls außerhalb, gibt es `SurfaceContact::Vacuum` zurück, sonst den gefalteten Oberflächenpunkt als `SolidCrust` – **wobei `PerimeterFold` nur greift, wenn `probe` sich tatsächlich auf Krustenhöhe befindet**, nicht `50 m` diagonal im All über einer Würfelkante!
   * In `HeliosphereWorld`: prüft `earth` über eine dynamische, an `earth_half` gekoppelte Anflug-Schale (größer als der `TouchdownStep`-Aktivierungsradius), prüft geladene Felswelten im `VicinityCache`, liefert für `SurfaceNature::GasGiant` `GasEnvelope` und im restlichen All ehrlich `SurfaceContact::Vacuum`.
3. **`HullClearance` & `TouchdownStep` umstellen (`crates/client/src/drive/clearance.rs` & `space_step.rs`):**
   * `HullClearance::local_contact` fragt `world.surface_contact(...)` statt des Fußgänger-Hacks `FaceContact::ground` $\rightarrow$ bei `Vacuum` oder `GasEnvelope` kehrt `TouchdownStep` sofort unverändert mit `s` zurück.
   * Kein künstlicher `SURFACE_PROBE_LIFT`-Scheinboden mehr im Vakuum.
4. **Tests & Verifikation (Phase 1):**
   * Neuer Unit-Test: Senkrechter und 45°-schräger Sinkflug einer Rakete aus `800 m` bzw. `4.000 m` Höhe bis zum Stillstand auf der Erdoberfläche (`alt ≈ 0`, kein Vorzeichenwechsel von `v_norm` vor Krustenkontakt).
   * Neuer Unit-Test: Eintauchen in Jupiter (`BodyId::JUPITER`) erfährt `canopy_drag` / `GasEnvelope`, prallt aber an keiner unsichtbaren Orbit-Schale ab.

---

### Phase 2: Gekoppelter Kosmos-Maßstab (`size = 1..=32`) & `< 1h` Epstein-Gürtel-Garantie

**Ziel:** Den World-Cube skalierbar halten (`size = 1..=32` $\rightarrow$ `128³` bis `4096³` Blöcke, Standard `size = 8` = `1024³`), fehleranfällige Einzel-Verzerrungsfaktoren restlos entfernen und mathematisch garantieren, dass der äußere Asteroidengürtel (`3,20 AU`) mit dem Epstein-Drive auf **jeder** Weltgröße in `< 1 Stunde` erreichbar ist.

1. **`SolarProportions` konsolidieren (`crates/world/src/celestial/proportions.rs`):**
   * Freie Verzerrungs-Mutatoren (`with_body_scale`, `with_sun_scale`, `with_distance_scale`, `with_system_scale`) entfernen.
   * Maßstab direkt aus `earth_blocks = 128 × size` (`size = 1..=32`, Standard `8` = `1024³`) ableiten, wobei `astronomical_unit()` nach oben bei **`AU_CAP ≈ 48,1 Mio. m`** gedeckelt wird (garantiert `< 55 Min.` selbst im ungünstigsten `4,4 AU`-Konjunktionsflug mit `8 G` Flip & Burn).
   * Physikalisch konsistente Antriebskennwerte (`chemical_speed_cap()` $\propto \sqrt{\text{size}}$, `fusion_speed_cap() >= 200_000.0`) direkt aus dem Maßstab ableiten, sodass gespeicherte Raketen-Blueprints auf jeder Servergröße (`128³..4096³`) zuverlässig Erdorbit und Mond erreichen.
2. **Client-, Server- und Shader-Konfiguration bereinigen:**
   * `PlanetaryScale` (`crates/client/src/server_scale.rs`, `[world] size` / `--size`, validiert auf `1..=32` mit Default `8` für Server) beibehalten; separate Flags `--cosmos-scale`, `--body-scale`, `--sun-scale`, `--distance-scale` und die korrespondierenden TOML-Schlüssel aus `crates/client/src/cosmos_scale.rs` und `crates/sanctuary/src/config/cosmos.rs` entfernen (nur optionale `belt_density` verbleibt).
   * In `crates/engine/src/shader.rs` die Outerra-Log-Depth-Far-Plane von `100 Mio. m` auf `10 Mrd. m` (`F_COEF = 0.060206`) anheben, damit auch bei `4096³` alle äußeren Planeten bis Neptun (`~1,45 Mrd. m`) ohne Far-Clipping gerendert werden.
3. **Antriebs- & HUD-Kopplung (`crates/client/src/drive/space_step.rs` & `orrery_hud/`):**
   * Starre Konstanten `MAX_COSMIC_SPEED = 45.0` und `MAX_FUSION_SPEED = 48_000.0` durch die maßstabsgekoppelten Kennwerte ersetzen.
4. **Tests & Verifikation (Phase 2):**
   * Bestehende `cosmos_scale_tests.rs` und Sanctuary-TOML-Tests auf die neue Invariante umstellen.
   * Physik-Test: Start einer chemischen Rakete erreicht auf `128³`, `1024³` und `4096³` stabilen Erdorbit; ein Epstein-Schiff (`8 G` Flip & Burn) erreicht `3,20 AU` (und selbst `4,40 AU` Worst-Case) auf `size = 32` (`4096³`) in `< 3.600 s`.

---

### Phase 3: On-Demand Sektor-Streaming, `4096³`-Freigabe & Begehbarer Asteroidengürtel

**Ziel:** Planeten bis `4096³` (`100,7 km²` Erdoberfläche), Dutzende gleichzeitig besuchte Monde und abbaubare Gürtel-Asteroiden starten in Millisekunden, brauchen wenig RAM und erzeugen beim Anflug keinerlei Frame-Spikes.

1. **Bedarfsgesteuerte Sektor-Materialisierung (`crates/world/src/relief/face/cache.rs` & `patch_grid.rs`):**
   * Aktuell iteriert `ReliefCellCache` beim ersten `emit_surface` über **alle** Zellen einer kompletten Würfelseite.
   * Umbau in **Sektor-Kacheln (`16 × 16` oder `32 × 32` Blöcke)**:
     * Reine Physik-Abfragen (`support_height_at`, `confine_beneath_crust`, `surface_contact`) werten `TerracedRelief` mathematisch in `O(1)` aus – **komplett ohne Mesh-Cache-Aufbau**!
     * Feine `0,5 m`-Voxel-Sektoren werden auf dem Client **nur innerhalb eines Sichtrings um Kamera/Schiff** generiert und inkrementell (z. B. max. 2–4 Sektoren pro Frame) in GPU-Patch-Slots hochgeladen.
2. **Grobes Orbital-Relief (`crates/world/src/relief/orbital_shell.rs`, `< 180` Zeilen):**
   * Für Sektoren außerhalb des Nahbereichs sowie für den Anflug aus dem Orbit erzeugt jede Würfelseite ein leichtgewichtiges **Makro-Gitter** (z. B. `32 × 32` abgetastete Stützpunkte pro Würfelseite unabhängig von der Blockanzahl), das echte Gebirge, Krater und Küstenlinien zeigt und nahtlos an die feinen Nah-Sektoren anschließt.
3. **`voxel_blocks`-Deckel auf `4096` öffnen, Voxel-Asteroiden & Multiplayer-Weltpool entkoppeln (`crates/world/src/celestial/`):**
   * In `proportions.rs`: Clamp von `4..=512` auf `4..=4096` erhöhen (`EARTH` bis `4096`, `VENUS` bis `3888`, `MARS` bis `2180`).
   * Angeflogene `BeltRock`-Asteroiden (`2,20..3,20 AU`, `BodyId::belt_rock`) on-demand als echte begeh- und abbaubare Voxel-Würfel (`16³..128³` mit reichen `LivingVeins`) bereitstellen.
   * Auf der autoritativen Weltseite (`HeliosphereWorld` / neuer Kollaborateur `SolarWorlds`) wird das 2-Slot-Mesh-Limit von der Physik getrennt: Leichte prozedurale `CubePlanet`-Instanzen (ohne vorberechnete GPU-Meshes!) plus `Chronicles` stehen für **beliebig viele gleichzeitig von verschiedenen Spielern besuchte Himmelskörper und Asteroiden** parallel bereit. Nur das Client-GPU-Mesh-Budget verwaltet lokale Sicht-Slots.
4. **Tests & Verifikation (Phase 3):**
   * Benchmark-/Startzeit-Test: Initialisierung von `CelestialSystem::heliosphere` für `1024³` und `4096³` dauert `< 50 ms` (keine Vorab-Vollgenerierung ganzer Würfelseiten).
   * Multiplayer-Simulationstest: Spieler stehen gleichzeitig auf Erde (`4096³`), Luna, Mars, Titan und einem `BeltRock`-Asteroiden und bauen Erze ab – keine Slot-Verdrängung, alle `WorldFact`-Änderungen bleiben konsistent.

---

### Phase 4: Mitbewegte „Gravity Bubbles“ (`CosmicFrame`) gegen `f32`-Jitter & Orbit-Freeze *(Umgesetzt in `v0.114.0`)*

**Ziel:** Sub-Millimeter-`f32`-Präzision auf jedem Himmelskörper bis Neptun (`361 Mio. m`) und kontinuierlich weiterlaufende Umlaufbahnen/Eigenrotationen, auch wenn Spieler auf fremden Monden gelandet sind.

1. **Lokale Referenz-Domäne (`crates/world/src/celestial/bubble.rs`, `< 220` Zeilen):**
   * `FrameAnchor`: kapselt `(BodyId, Vec3)` (Position und Geschwindigkeit immer relativ zum Zentrum und zur Rotationsphase des aktuell dominierenden Himmelskörpers; im interplanetaren Raum `BodyId::SOL` oder nächster Planet mit Kamera-Floating-Origin).
   * `GravityBubble`: berechnet aus Masse und großem Halbmesser den physikalischen **Hill-Radius** $r_{\text{Hill}} = a \cdot \sqrt[3]{GM_{\text{body}} / (3 \, GM_{\text{parent}})}$ in `f64`.
2. **Nahtloser Sphären-Übergang (`FrameHandoff`):**
   * Durchfliegt ein Raumschiff (`SpaceState`) oder Avatar im EVA die Grenze einer Hill-Sphäre (mit kleiner Hysterese gegen Flattern an der Grenze), transformiert `FrameHandoff` Orts- und Geschwindigkeitsvektor exakt in `f64` in das Kind- bzw. Eltern-Bezugssystem (`p_local = (p_helio - body_helio) -> Vec3`, `v_local = (v_helio - body_vel) -> Vec3`).
   * Geparkte Fahrzeuge (`ParkedFleet` & `OrbitalFleetWorld`) ruhen automatisch im lokalen Frame ihres Himmelskörpers.
3. **Aufhebung der Erd-Sonderrolle & des Orbit-Freezes (`crates/world/src/celestial/heliosphere/`):**
   * Die Erde (`BodyId::EARTH`) ist nicht mehr starr `(0, 0, 0)` des gesamten Kosmos, sondern einfach die Start-Bubble des Spielers.
   * In `HeliosphereWorld::step_surface`: die Sperre `let locked = self.vicinity.borrow().any_active(); if !locked { ... }` **ersatzlos streichen** – `ephemeris` läuft immer in Echtzeit weiter! Steht ein Spieler auf Europa, wandern Jupiter und die Sonne live am Himmel weiter.
4. **Kamerazentrierte Kosmos-Projektion (`cosmic_sky.rs` & `rayleigh_shell/`):**
   * Alle Himmelskörper und Atmosphären-Schalen berechnen ihren Relativvektor in `f64` direkt gegen `(anchor_body_helio + viewer_local_dvec3)` und projizieren erst die Differenz nach `Vec3`.
5. **Tests & Verifikation (Phase 4):**
   * Präzisions-Test auf Triton (`BodyId::TRITON` bei `~361 Mio. m` Sonnenabstand): Ein Spieler geht `0,5 m`-Schritte auf `GridCell`-Blöcken und landet eine Rakete – lokaler `Vec3`-Wert bleibt `< 200 m`, `f32`-Fehler `< 0,0001 m` (null Jitter!).
   * Orbit-Kontinuitäts-Test: Während ein Spieler auf `BodyId::LUNA` steht und ein geparktes Fahrzeug neben ihm ruht, schreitet `step_surface` um 600 Sekunden voran; das Fahrzeug bleibt millimetergenau auf seinem Kraterplatz, während sich die Erde am Mondhimmel weiterbewegt.

---

### Phase 5: Multiplayer-Protokoll, Navigations-HUD & Endabnahme *(Umgesetzt in `v0.114.0`)*

**Ziel:** Vollständige Integration in das P2P-/Dedicated-Server-Protokoll (`crates/sanctuary`), das Raumschiff-HUD und alle Qualitäts-Gates.

1. **Sanctuary-Synchronisation (`crates/sanctuary/src/`):**
   * `CompanionPresence`, `DrivenCraft` und `WorldFact`-Ortsangaben tragen das `BodyId`-Bezugssystem (`u16`), sodass `CompanionGlide` Mitspieler auf demselben Mond ruckelfrei im lokalen `Vec3`-Raum interpoliert und Mitspieler auf fernen Planeten keine Bandbreite verschwenden.
   * `WardedPlanet` prüft Reichweiten- und Geschwindigkeits-Guards lokal im jeweiligen `BodyId`-Frame statt gegen eine riesige globale `heliosphere_span`-Box.
2. **Navigations-Instrumente (`crates/client/src/drive/helm_chart.rs`, `transfer_arc/` & `menu/orrery_hud/`):**
   * `HelmChart` und `TransferArc` lesen Schiffsort und -geschwindigkeit über `FrameAnchor` in heliozentrischem `f64` aus, sodass Kurslinien, Flip-&-Burn-Punkte und `TGT`/`ALT`-Laser-Skalen auf die `1024³`-Welt und bewegte Monde abgestimmt sind.
3. **Abschluss-Qualitätssicherung:**
   * `cargo test --workspace` (alle Unit-, Physik-, Replay- und Multiplayer-Tests grün).
   * `allocation_audit` prüfen (null Heap-Allokationen im 120-Hz-Physik- und Flug-Hotpath).
   * Zeilenanzahl aller geänderten/neuen `.rs`-Dateien prüfen (`wc -l`, strikt `<= 300` Zeilen) und auf Einhaltung aller Pure-OOP-Regeln (`<= 4` Felder, keine Getter/Setter, keine `-er`-Namen, kein toter Code) kontrollieren.
