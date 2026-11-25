# Architektur-Reflexion, Strikte Horizont-Isolation & Out-of-the-Box-Ideen (`v0.112.0+`)

> **Kontext:** Tiefenprüfung aller Subsysteme (`crates/world`, `crates/sanctuary`, `crates/fauna`, `crates/client`) gegen das oberste Skalierungs-Axiom von Low-Cube. Die lokalen Horizont- & Klimazonen-Isolations-Decorators aus §3 (`SectorForest`, `PlanetaryMesh::with_horizon`, `present_vessels_near`, `SectorGridAudience`, `VelocityScaledSight`, `SpatialFactIndex`/`HorizonStream`, `SectorCracks`, `VaultStream`, `OffworldVault`) sind in **`v0.113.0`** umgesetzt; §5 zeigt die verbleibenden Ausbaustufen.

---

## 0. Das Leit-Axiom: Strikte Horizont- & Klimazonen-Isolation

> **Grundsatz:** Einen Spieler (z. B. in `Temperate` auf der Erde) darf technisch **weder im Netzwerk (`0 Bytes`), noch im RAM (`0 Heap-Objekte`), noch auf der CPU (`0 Ticks / 0 Raycasts`), noch im GPU-VRAM (`0 Meshes`)** interessieren, was weiter weg auf seiner eigenen Würfelseite, in einer anderen `ClimateZone` (`Arctic`, `Jungle`, `Desert`, `Steppe`, `Volcanic`) oder auf einem anderen Himmelskörper passiert!
> * Nur wenn er sich bis auf `<= 160 m` einer Würfelkante (`BeveledBoundary`) nähert, öffnet sich nahtlos der angrenzende Randsektor der Nachbar-Klimazone.
> * Nur wenn er mit einem Schiff in die Höhe / in den Orbit aufsteigt, werden grobe `OrbitalShell`-Makro-Patches der sichtbaren Würfelseiten geladen.

---

## 1. Gehen wir hier den richtigen Weg?

**Ja – strategisch, technisch und spielerisch ist dieser Weg goldrichtig.**

1. **Seed + Sparse-Deltas statt Gigabyte-Chunk-Streaming:** Server und Client erzeugen die Grundwelt deterministisch aus demselben Seed; unveränderte Landschaft kostet null Netzwerkbandbreite.
2. **Physischer Horizont (`0 Hz` Fernfeld) statt globalem `O(N²)`-Zwang:** Weit entfernte Spieler kosten `0 Hz / 0 Bytes` (`ProximityAudience`), Kommunikation ist strikt lokal (`<= 96 m`).
3. **Spielbare Hard-Sci-Fi (`< 1 h` Flip-&-Burn + endliche Heimat-Erze):** `1 AU <= 48,09 Mio. m` garantiert unter einer Stunde Flugzeit bis zum äußeren Asteroidengürtel (`3,20 AU`), während erschöpfliche Erze (`LivingVeins`) den wirtschaftlichen Aufbruch ins All erzwingen.
4. **Yegor Bugayenkos Pure OOP & Decorator-Komposition:** Alle Erweiterungen bleiben gekapselte, austauschbare Decorators (`<= 4` Felder pro Struct, `< 300` Zeilen pro Datei, keine Getter/Setter, keine globalen Locks).

---

## 2. Erneute Tiefenprüfung aller Punkte gegen die strikte Horizont- & Klimazonen-Isolation

Beim erneuten Abklopfen des **tatsächlichen Ist-Codes (`v0.112.0`)** und unserer bisherigen Planpunkte gegen die Regel *„nichts Weites und keine andere Klimazone darf den Spieler technisch belasten“* sind neben den bekannten Kosmos-Punkten **vier massive lokale Lecks auf der Erde selbst** aufgetaucht:

### 2.1 NEUFUND: Bäume & Bodenfunde (`ZonedForest` / `ForestedFace`) laden und ticken noch alle 6 Klimazonen!
* **Befund im Code (`crates/world/src/tree/zoned_forest.rs` & `forested_face/mod.rs`):**
  1. `ZonedForest::populate_face` erzeugt beim Weltstart **sofort alle Bäume, Äste und Feuersteine der gesamten Würfelseite** für alle **6 Klimazonen**. Auf einer `4096³`-Erde (`half_size = 2.048 m`) sind das `~6,26 Mio.` Gitterproben und **`~1,8 Millionen` `Box<dyn Arboreal>`-Objekte im RAM** – obwohl der Spieler nur in einem `160 m`-Waldstück in `Temperate` steht!
  2. `ForestedFace::constrain` (`forested_face/mod.rs:67`) iteriert bei *jeder* Bodenkollision linear über **alle Bäume der gesamten Klimazone** (`for tree in &self.trees`).
  3. `CubePlanet::step_surface` (`planet/mod.rs:159`) ruft `120`-mal pro Sekunde `step_flora(dt)` auf **allen 6 Klimazonen** gleichzeitig auf (`Arctic`, `Jungle`, `Desert`, `Steppe`, `Volcanic` inklusive!).
  4. `SessionDisplay::refresh_world_mesh` (`session/display/mod.rs:67`) baut und lädt beim Beitritt sofort **alle 6 Terrain-Faces UND alle 6 kompletten Baum-Meshes aller Klimazonen** in den GPU-VRAM – obwohl der Spieler durch den festen Planeten hindurch die anderen 5 Klimazonen überhaupt nicht sehen kann!
* **Konsequente Lösung (`HorizonFlora` / `SectorForest`):**
  * Bäume und `FloorLitter` werden wie `ReliefCellCache` **ausschließlich on-demand pro Sektor `(BodyId, ClimateZone, SectorU, SectorV)` im lokalen Sichtring (`<= 160 m`)** erzeugt, kollidiert (`3×3`-Sektoren statt `300.000` Bäume!), getickt und in einen lokalen Flora-Ring auf der GPU geladen.
  * Andere Klimazonen und ferne Wälder derselben Klimazone belegen **`0` RAM, `0` CPU-Ticks und `0` GPU-VRAM**! Auch die `OrbitalShell`-Terrain-Slices der 5 abgewandten Klimazonen werden erst gestreamt, wenn die Höhe des Spielers über die Horizontkante steigt.

### 2.2 NEUFUND: Geparkte Fahrzeuge (`present_vessels`) und Grab-Risse (`PeerCrack`) lecken über Klimazonen hinweg!
* **Befund im Code:**
  1. `CubePlanet::present_vessels` (`crates/world/src/planet/facets.rs:94`) iteriert in **jedem Render-Frame** über alle 6 `ClimateZone`-Faces und übergibt jedes geparkte Auto/Raumschiff des gesamten Planeten an die GPU (`VesselStage`) – ein Rover in `Volcanic` wird für einen Spieler in `Temperate` jeden Frame mitgeprüft!
  2. `HostAura::record_crack` (`crates/sanctuary/src/aura.rs:168`) speichert Grab-Risse (`PeerCrack`) aller Spieler des gesamten Servers in einem **einzigen globalen `16`-Elemente-Puffer** (`if len >= 16 { remove(0) }`). Graben 20 Spieler in `Desert`, `Arctic` oder auf dem Mars gleichzeitig, überschreiben sie die Grab-Risse direkt vor den Füßen eines Spielers in `Temperate`!
* **Konsequente Lösung:**
  * Geparkte Schiffe und `PeerCrack`-Ereignisse hängen strikt an ihrem Sektor-Schlüssel **`SectorKey = (BodyId, ClimateZone, SectorU, SectorV)`**. Was in `Volcanic` oder `500 m` weiter in `Temperate` parkt oder gräbt, existiert weder im Puffer noch im Render-Pass von `Temperate`.

### 2.3 Chronik-Architektur: `SectorChronicle` pro `(BodyId, ClimateZone, Sector)` + On-Demand `VesselCatalog`
* **Geprüft & bestätigt:** Weder ein globales `ChronicleLog` noch eine bloße `BodyId`-Partitionierung genügt (da auf einer `4096³`-Erde sowohl die anderen 5 Klimazonen als auch `3 km` entfernte Täler derselben `Temperate`-Zone für den Spieler irrelevant sind).
* **Richtiger Schnitt:**
  1. **`SectorChronicle` (`(BodyId, ClimateZone, SectorU, SectorV)`):** Speichert den **aktuellen Netto-Zellzustand (`SectorSnapshot`)** statt historischer Raycast-Events. Der Client synchronisiert nur seinen lokalen `HorizonSectors`-Ring (`~5×5` Sektoren um seine Position; über eine Würfelkante hinweg nur dann, wenn er `< 160 m` an der Fase steht). Andere Klimazonen = **`0 Bytes, 0 Raycasts`**.
  2. **`VesselCatalog` (On-Demand im Horizont):** Reisende Schiffe (`CraftBlueprint` + aktuelle Pose) sind vom Boden-Log getrennt und werden **ausschließlich dann** an einen Client übertragen, wenn das Schiff tatsächlich in seinen lokalen Sichthorizont eintritt.

### 2.4 Lokale Hill-Sphären & Klimazonen-Koordinaten (Keine `16 m` `f32`-Sprünge im Gürtel)
* **Geprüft:** Bei `3,20 AU = 153,88 Mio. m` beträgt `1 ULP` in heliozentrischem `f32` **`16,0 Meter`** – Gehen (`0,03 m/Tick`) und `0,5 m`-Blöcke frieren dort ein.
* **Feinschliff:** Jede Spieler- und Schiffsposition ankert lokal an `(BodyId, LocalOffset: Vec3)` (und am Boden zusätzlich implizit in ihrer `ClimateZone`-`SurfaceFrame`). Im freien Raum zwischen den Planeten dient ein lokaler Sektor-Ursprung (`DVec3`) als Anker. Damit herrscht an jedem Punkt des Sonnensystems Sub-Millimeter-`f32`-Präzision, ohne dass ein Spieler auf der Erde jemals Koordinaten-Updates aus dem Gürtel erhält (`0 Hz`).

### 2.5 Server-Simulation & Sharding: Nach `(BodyId, ClimateZone, SectorCluster)` statt nur nach `BodyId`!
* **Wichtige Korrektur unseres eigenen Plans:** Zuvor stand im Plan, die Server-Simulation pro `BodyId` auf CPU-Kerne zu verteilen. **Fehler:** Wenn sich `900` von `1.000` Spielern gleichzeitig auf `BodyId::EARTH` aufhalten (`200` in `Temperate`, `150` in `Jungle`, `150` in `Arctic`, `200` in `Volcanic`, `200` in `Desert`), würde ein einziger `Earth`-Thread die gesamte Last tragen, während 7 CPU-Kerne für leere Monde brachliegen!
* **Richtiger Schnitt:** Weil ein Spieler in `Temperate` nichts mit `Volcanic` oder einem `1 km` entfernten Tal zu tun hat, sharded der Server die Welt nach **aktiven `(BodyId, ClimateZone, SectorCluster)`-Inseln**!
  * Alle 6 Klimazonen der Erde laufen parallel auf getrennten CPU-Kernen.
  * Unbewohnte Klimazonen, unbesuchte Sektoren und leere Monde/Asteroiden sind **komplett passiv (`Dormant`, `0 %` CPU, `0` RAM)** und wachen nur in genau den Sektoren auf, in denen sich tatsächlich ein Spieler befindet!

### 2.6 Wildtiere (`SectorFauna`), Multi-Crew (`AboardVessel`) & Server-Inventar (`SurvivorLedger`)
* **Wildtiere (`SectorFauna`):** `HostSounder` darf nicht global im `HostPulse` an jeden Spieler geschickt werden (`crates/sanctuary/src/cohort.rs:243`). Wildtiere leben pro aktivem Sektor ihrer jeweiligen `ClimateZone`; verlässt der letzte Spieler den Sektor, schläft die Fauna ein (`0` Bytes an andere Zonen, `0` CPU-Ticks).
* **Multi-Crew (`AboardVessel`):** Mitfahrer auf einem `VesselDeck` übertragen `(CraftIdentity, DeckOffset: Vec3)` statt Weltkoordinaten – nur relevant für Beobachter im Sichthorizont genau dieses Schiffs.
* **`SurvivorLedger`:** Serverseitige Speicherung von `(BodyId, LocalPos)` + Satchel pro X25519-Spieler-Key beim Logout, damit endliche Erze geschützt sind und niemand per lokaler Client-`server.toml` Items ercheaten kann.

---

## 3. Zusammenfassung der konkreten Architektur-Feinjustierungen (Yegor-Decorators)

| # | Bereich | Ist-Zustand (`v0.112.0`) | Feinjustierung (Strikte Horizont- & Klimazonen-Isolation) | Technischer Effekt |
|---|---|---|---|---|
| **1** | **Flora & GPU-Slices** (`ZonedForest`, `ForestedFace`, `SessionDisplay`) | Erzeugt, kollidiert, tickt und lädt beim Start Bäume & Meshes **aller 6 Klimazonen** (`~1,8 Mio.` Objekte bei `4096³`). | **`SectorForest` + `HorizonSliceLoader`**：Generiert Bäume, Litter & Flora-VRAM-Patches lazy nur in den `<= 160 m`-Sektoren der eigenen `ClimateZone` (Nachbar-Zone nur `< 160 m` vor Würfelkante; ferne `OrbitalShell`s erst ab Steigflug). | Startzeit `< 50 ms`, **`99 %` weniger RAM/VRAM & CPU** auf `4096³`-Erde. |
| **2** | **Chronik & Flotte** (`ChronicleLog`, `CubePlanet::present_vessels`) | Globales `Vec<WorldFact>` + planetenweites Rendern aller geparkten Fahrzeuge über alle 6 Faces. | **`SectorChronicle` + `HorizonVessels`**：Netto-Zellzustand (`SectorSnapshot`) pro `(BodyId, ClimateZone, Sector)` + On-Demand `CraftBlueprint`/Park-Sync nur bei Eintritt in den Sichtring. | **Login `< 0,2 s`** nach Jahren Laufzeit; `0 Bytes` & `0 Drawcalls` für andere Klimazonen. |
| **3** | **Audience & Cracks** (`WorldAudience`, `HostAura::record_crack`) | `O(N²)`-CPU-Scan in `select_for` + globaler `16`-Slot-Puffer für `PeerCrack`s über das ganze Universum. | **`SeamAwareSectorAudience`**：Indiziert Spieler, `PeerCrack`s und `WildlifePulse` direkt in `(BodyId, ClimateZone, Sector)`-Buckets (faltet nur am Kantenrand über die 90°-Fase). | Server-CPU bei 2.500 Spielern **`< 0,15 ms/Tick` (`O(N)`)**, null Übersprechen zwischen Zonen. |
| **4** | **Server-Kerne & Welten** (`VicinityCache`, Server-Tick) | `2`-Slot-LRU-Cache für das ganze Sonnensystem + ein einziger Main-Thread für ganz `Earth`. | **`ActiveSectorWorlds` + Cluster-Sharding**：Server hält nur besuchte Sektoren auf Erde, Monden & Asteroiden wach (mit Idle-TTL) und rechnet Klimazonen/Cluster parallel. | Kein `2`-Slot-Thrashing; auch **1.000 Spieler allein auf der Erde** skalieren über alle CPU-Kerne. |
| **5** | **Koordinaten, WAL & Anflug** (`SpaceStep`, `DiskVaultChronicle`, `MtuBudgetAudience`) | Heliozentrisches `f32` (`16 m` ULP im Gürtel), `open()` pro WAL-Fact, starre `48 m`/`160 m`-Schiffssicht. | **`HillBubbleFrame` + `BufferedVaultChronicle` + `VelocityScaledSight`**：Lokales `(BodyId, Vec3)`, Tick-geflushter `BufWriter<File>` und geschwindigkeitsskalierte Sicht für landende Raketen *im eigenen Himmel*. | Sub-Millimeter-Physik überall, **100× schnelleres SSD-WAL**, sanft sichtbare Landungen vor Ort. |

---

## 4. Geprüfte Out-of-the-Box-Ideen (100 % kompatibel mit Horizont-Isolation!)

Auch kreative Ideen müssen sich dem Gesetz beugen: **Keine globalen Server-Broadcasts, keine Last durch unsichtbare Klimazonen!**

1. **Physisches Erz-Gewicht (`CargoBallast` auf `a = F / m`) — *100 % lokal*:**
   Abgebautes Erz wird als physische Frachtblöcke auf das Schiffsdeck gesetzt und erhöht direkt die Schiffsmasse in `HullBallast`. Leer fliegt der Prospektor mit `9 G` zum Gürtel, vollbeladen bremst er träge mit `1,8 G` zurück. Null zusätzlicher Netzwerk-Overhead außerhalb des Sichthorizonts!
2. **Akustisches Vakuum & „Helm-an-Helm“-Kontaktfunk (`MediumChat`) — *100 % lokal*:**
   In Atmosphäre (`Earth`, `Mars`, `Titan`) trägt der lokale Chat `96 m`. Im Vakuum (`Luna`, `BeltRock`, EVA) trägt offener Schall `0 m`: Kommunikation läuft über Bord-Intercom (gleiches `VesselDeck`), lokalen Kurzstrecken-Anzugfunk (`<= 96 m`) oder – bei Funkstille – durch direktes **Helm-an-Helm-Berühren (`<= 1,8 m`)**.
3. **Forensische Spuren vor Ort (Schmelzbasalt, Geisterwracks & Blackbox-Block) — *100 % lokal im Sektor*:**
   * Heiße Fusionsdüsen hinterlassen im getroffenen `SectorSnapshot` einen dunklen Ring aus verglastem **Schmelzbasalt**.
   * Zerschellt ein Schiff, bleibt im jeweiligen Sektor ein unzerstörbarer orangefarbener **Blackbox-Voxelblock** liegen, aus dem nur ein Finder *direkt vor Ort* die letzten 15 Sekunden Flugschreiber-Telemetrie (`LiveTape`) bergen kann.
4. **Teleskop-Brennsterne (`DriveStar`) — *Strikt nur auf Anfrage / Sichtkegel im Nacht-/Raumhimmel, niemals global!*:**
   Einen Spieler am Tag, unter Tage oder auf der abgewandten Erdseite (`Temperate` zeigt von Mars weg) geht ein fremdes Triebwerk **nichts an (`0 Bytes`)**. Nur wenn ein Beobachter im Weltraum ist oder nachts aktiv durch ein `6×/24×`-Teleskop in seine **offene Himmelshalbkugel (`dot(local_up, dir) > 0.15`)** blickt, kann der Server maximal 4 helle Fusions-Brennsterne genau dieses Himmelssektors einblenden – null Last für alle normalen Boden-Spieler!
5. **Optische Sichtlinien-Leuchtfeuer statt Global-Chat — *Strikt physikalische Sichtlinie*:**
   Kein magischer Planeten-Chat: Ein gebautes Laser-/Licht-Leuchtfeuer ist nur für denjenigen sichtbar, der eine **echte freie geometrische Sichtlinie** auf die Bake hat (z. B. ein anfliegendes Schiff im Orbit über genau dieser Klimazone). Wer in einer anderen Klimazone hinter dem Horizont steht, empfängt `0 Bytes`.

---

## 5. Umsetzungsstand (`v0.118.0` — Roadmap vollständig umgesetzt)

Sowohl die strikte Horizont-/Klimazonen-Isolation (`v0.113.0`) als auch alle fünf Ausbaupakete (**Paket 1–5**, `v0.114.0`–`v0.118.0`) sowie die Kosmos-Konfigurationsbereinigung (`COSMOS_PLAN.md` Phase 2) sind vollständig umgesetzt und verifiziert:

1. ✅ **Erledigt in `v0.114.0` — Lokale Hill-Sphären (`CosmicFrame::Body`, `BeltAsteroid`, `4,096 m` `DeepSpace`) + `LWEL` `epoch_secs`-Sync:**
   * Sub-Millimeter-`f32`-Präzision (`< 0,13 mm` ULP) bis `30,07 AU` (`Triton`), kontinuierlich weiterlaufende Planetenbahnen ohne `if !locked`-Sperre sowie `MtuBudgetAudience(CosmicCraftSight(CosmicBubbleAudience))`-Isolation im Multiplayer.
2. ✅ **Erledigt in `v0.115.0` — `VesselBerth`-Relativkoordinaten & `VesselDeckCurve` für Multi-Crew-Raumschiffe:**
   * Hält Piloten, sitzende Beifahrer (`PassengerBerth`) und frei über das Deck laufende Crew-Mitglieder (`DeckFooting::Aboard` inkl. Startrampe-zu-Orbit-Äquivalenzprinzip & Kabinen-Proximity-Chat) selbst bei `240.000 m/s` und `180°` Flip-&-Burn mit `0,000 m` Schlupf exakt im Schiffsrumpf (`0` Extra-Bytes für Bodenspieler).
3. ✅ **Erledigt in `v0.116.0` — Serverseitiger `SurvivorStore` (`DiskVaultSurvivors(MemorySurvivors)`) pro X25519-Key:**
   * Bindet `SatchelManifest` (alle `32` `HeldRelic`-Bestände + `20` `ArtisanTome`-Rezepte) und den letzten `(CosmicFrame, Vec3)`-Aufenthaltsort (`Earth`, `Mars`, `BeltAsteroid`) an persistente X25519-Spielerschlüssel (`survivor.key` / `<journal>.survivors`), verhindert lokale `server.toml`-Starterkit-Cheats und stellt Bergleute selbst nach stundenlanger Orbital-Drift und Server-WAL-Neustarts millimetergenau auf ihrem Asteroiden wieder her.
4. ✅ **Erledigt in `v0.117.0` — Netto-Zellzustands-Snapshots (`SectorSnapshot` / `SnapshottedChronicle`) + Multi-Core-Klimazonen-Sharding (`ZoneShards` & `ParallelInbox`):**
   * Faltet gealterte `4 m × 4 m`-Sektoren sowohl live (`3 × 3`-Nachbarschafts-Snapshot + `ChronicleEntry::Superseded`) als auch atomar beim WAL-Serverneustart (`DiskVaultChronicle::replay_and_open`) auf ihren raycast-freien Netto-Zellzustand (`ShaftExcavation`, `64`-Bit-`bare_mask`, offengelegte Erzadern, `WorldBlocks` inkl. `NaturalOverhang` & `8`-Wege-`FieldCamera` sowie `CantileverRig`-Kräne), hält unbewohnte `ClimateZone`s strikt auf `0 %` CPU (`Dormant`) und skaliert aktive Klimazonen, warme `OffworldVault`-Himmelskörper und eingehende ChaCha20-Poly1305-Pakete (`ParallelInbox`) über alle CPU-Kerne.
5. ✅ **Erledigt in `v0.118.0` — Emergente Hard-Sci-Fi-Sandbox (`CargoBallast`-Erzmasse, `ChatMedium`-Vakuumfunk & lokale Forensik mit `VitrifiedBasalt` / `BlackBoxPod`) + gekoppelte `[world] size`-Kosmos-Konfiguration:**
   * Physische Erzdichte (`GoldOre > IronOre > CopperOre > QuarriedStone`) und `CargoBallast<S>` (`a = F / m` inkl. `SatchelBurden::ballast_tonnage`) skalieren Beschleunigung, Trägheit und den `FlipBurnReadout`-Bremspunkt (`s = v² / 2a`) vollbeladener Frachter automatisch.
   * `SuitTransceiverMedium(AtmosphericVoiceMedium(CabinIntercomMedium(HelmetContactMedium)))` (`crates/sanctuary/src/chat_medium.rs`) erzwingt `0 m` offenen Schall im Vakuum (`<= 32 m` EVA-Funk, `~`-Funkstille nur über `<= 1,85 m` Helm-an-Helm-Visierkontakt, `240 km/s` Bord-Intercom auf demselben `VesselBerth::Aboard`).
   * `FusionTorch`-Triebwerke verglasen Startrampen lokal zu glühendem `VitrifiedBasalt`, harte Abstürze (`v_norm < -26 m/s`) hinterlassen bergbare orangefarbene `BlackBoxPod`-Kapseln (`1`-Klick-Bergung + `WheelClick`-`IncidentSheet`-Telemetrie-Auswertung vor Ort), und alle freien Verzerrungs-Overrides (`body_scale`, `sun_scale`, `distance_scale`) sind zugunsten von `[world] size = 1..=32` + `asteroid_density` bereinigt.
