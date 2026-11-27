# Zero-Legacy Umbauplan (`REBUILD_PLAN`): Schritt-für-Schritt zur neuen Raumfahrt-Engine

Dieser Plan beschreibt den **kompromisslosen Komplett-Umbau** des bestehenden Repositories hin zur Zielarchitektur aus [`ENGINE_CONCEPT.md`](file:///workspace/ENGINE_CONCEPT.md) — **ohne Altlasten, ohne Rückwärtskompatibilitäts-Brücken und ohne tote Übergangs-Layer**.

---

## 1. Grundregeln für den Umbau („Zero Legacy“)

1. **Kein Mitschleppen alter Formate oder Epochen:** Alte Savegames/Chronicles, OpenGL-Wrapper (`glow`/`glutin`), Mittelalter-Gegenstände, Wildschwein-Jagd und Einzelblock-Heap-Strukturen (`Box<dyn VoxelShape>`) werden nicht „deprecated“, sondern **restlos gelöscht**.
2. **Nur bewährte Raumfahrt-Perlen übernehmen:** Erhalten und direkt auf das neue Fundament portiert werden ausschließlich:
   * `crates/sanctuary` (Pure-Rust P2P, ChaCha20-Poly1305 AEAD, STUN & 12-Zeichen-NAT-Punching, Sprach-/Funk-Medien),
   * `crates/acoustics` (Linux/Windows-PCM-Streaming, Synthese & NASA-Foley-Samples),
   * Himmelsmechanik (Kepler-Ellipsen, Hill-Sphären, Ephemeriden) & das diegetische Phosphor-Bordkamera-Design (`ShipOpticHud`),
   * die Code-Qualitätsregeln aus [`AGENTS.md`](file:///workspace/AGENTS.md) (Pure OOP an den Domänengrenzen, `< 250–300` Zeilen pro Datei, deterministische Headless-Tests).
3. **Referenz-Hardware bei jedem Schritt:** Linux (Wayland/X11, Vulkan) & Windows (DX12/Vulkan) mit Zielmarke **`60 FPS @ 1080p` und `< 250 MB` VRAM** auf einem **ThinkPad T480 (Core i5/i7-U + GeForce MX150 `2 GB` / Intel UHD 620)**.

---

## 2. Abriss- & Erhaltungs-Matrix (Was fliegt raus, was wird neu?)

| Bereich | Altbestand (wird gelöscht!) | Zielzustand nach Umbau |
| :--- | :--- | :--- |
| **Epoche & Fauna** | Komplettes Crate `crates/fauna`, `host_sounder.rs`, `remote_wildlife.rs`, `BakingOven`, `BloomeryForge`, `SmithyAnvil`, `KnightlySword`, `WoodenMallet`, `TimberTorch`, Knappen-Kleidung | **100 % Raumfahrt ab Sekunde 1:** EVA-Druckanzug, Helmlampe, Montage-/Abbau-Werkzeug, Lichtbogen-Schmelze, Fabrikator, Elektrolyse, Luftschleuse, Koje |
| **Voxel-Speicher** | `VoxelCells` (`HashMap<(i32,i32), Vec<ElevatedShape>>` + `Box<dyn VoxelShape>`) | **Flache `BrickArena` (`8×8×8` à `0,5 m` + `u64`-Bitmasken):** `0` Einzel-Heap-Allokationen; Roh-Atome + Mehrblock-`Anchor`/`Proxy`-Zellen |
| **Bauteil-Größen** | Alles in `1×1×1`-Einheitswürfel (`0,5 m`) gequetscht | **Feste Mehrblock-Maschinen (`2×2×3` etc.) + im `0,5 m`-Raster skalierbare Thruster, Tanks & Radiatoren (`B × H × T`)** |
| **Grafik-Crate** | `crates/engine` auf OpenGL (`glow`, `glutin`, `184 Bytes/Quad` in `CpuMeshBatch`) | **`wgpu` (Vulkan/DX12):** CPU `u64` Binary Greedy Mesher (`~20 µs`) + **`8-Byte` Packed Quads** (Vertex-Pulling, `23×` kleiner), Optimus-Schalter, `1,5 MB/Frame` Upload-Cap |
| **Welt-Topologie** | `CubePlanet` mit 90°-Würfelkanten (`BeveledBoundary`, `PerimeterFold`) & `f32`/`f64`-Mix | **Deterministischer `i64`-Raum (`0,1 mm`)** + runde **`Quad-Sphere`-Planeten** mit GPU-Höhen-Clipmap (`0 Bytes` Brick-RAM für unberührtes Gelände) |
| **Schiff vs. Welt** | 4 getrennte Welten: `WorldBlocks`, `CantileverRig`, `ParkedFleet`, `WorldCraft` (fester Sitz-Zwang im Flug) | **Ein einziger Typ `GridBody`:** Haus, Raumstation, Asteroid und Schiff sind identisch; freies Gehen im Flug unter `1 g` Schub & in rotierenden Stations-Ringen |
| **Umwelt-Physik** | Statische Blöcke ohne Gasdruck oder nachrutschendes Schüttgut | **Sleep-by-Default Solver (`0 %` Idle-CPU):** Luftdichte Räume, Schleusen, Lecks, Rohr-/Kabel-Blöcke + **nachrutschender Sand/Regolith entlang $\vec{g}_{\text{lokal}}$** |

---

## 3. Technische Funktionsweise: Nachrutschender Sand & Regolith

Schüttgut (`Sand`, `Mond-Regolith`, `Eisgeröll`, `Erzschutt`) fügt sich ohne Sonder-Engine direkt in die `8×8×8`-Bricks und geschachtelten Schwerkraft-Räume ein:
1. **`0,0 %` CPU im Ruhezustand (*Sleep-by-Default*):** Liegender Sand wird niemals gepollt.
2. **Aufwecken durch Nachbar-Änderung:** Wird ein Block abgebaut oder weggesprengt, prüft die Engine einmalig die 6 Nachbarzellen. Liegt darüber (relativ zum lokalen Schwerkraft-Vektor $\vec{g}_{\text{lokal}}$) ein `Granular`-Block, wandert dessen Koordinate in die `ActiveSettlingQueue` des `GridBody`.
3. **Deterministischer Rutsch-Takt (`15 Hz`, gedeckelt auf `max. 256 Zellen/Tick`):**
   * **Senkrecht frei?** Ist die Zelle in Richtung $\vec{g}_{\text{lokal}}$ leer (`1` Bit-Test in der `u64`-Maske), rückt der Sandblock nach und weckt die Zelle über seiner alten Position auf.
   * **Unten voll, aber schräg-unten frei?** Der Sand rutscht im `45°`-Schüttwinkel seitlich ab — es bilden sich von selbst natürliche Dünen und Kraterhänge statt unnatürlicher Sand-Säulen.
   * **Schwerelosigkeit (`0 g`):** Ist $|\vec{g}_{\text{lokal}}| \approx 0$ (freier Fall im All), bleibt loses Geröll schweben — bis ein Triebwerk zündet und den Schutt Richtung Heckboden drückt!
4. **Null Extra-Netzwerklast:** Da der Rutsch-Algorithmus auf beiden Seiten deterministisch aus demselben `RemoveBlock`-Ereignis folgt, muss kein einziger rutschender Sandblock einzeln übers Netzwerk geschickt werden, und das Neu-Meshen des `8×8×8`-Bricks dauert auf dem ThinkPad nur `~20 µs`.

---

## 4. Schritt-für-Schritt-Checkliste für den Umbau (Stufe 1 bis 9)

Jede Stufe schließt mit grünem Build und schnellen Headless-Tests auf Linux und Windows ab.

### Stufe 1: Tabula Rasa — Radikale Entfernung aller Mittelalter- & Legacy-Altlasten
- [x] Crate `crates/fauna` vollständig aus dem Workspace (`Cargo.toml`, `client`, `sanctuary`) entfernen; `host_sounder.rs`, `remote_wildlife.rs` und `wild_boar_tests.rs` löschen.
- [x] Alle Mittelalter-Medien, Werkbänke und Waffen löschen (`BakingOven`, `BloomeryForge`, `SmithyAnvil`, `KnightlySword`, `WoodenMallet`, `IronSpade`, `WoodmanAxe`, `TimberTorch`, mittelalterliches Ringbuch `ArtisanTome` & Knappen-Garderobe).
- [x] Spieler-Startausrüstung auf **100 % Raumfahrtzeitalter** umstellen: Druckanzug mit integrierter Helmlampe, Montage-/Demontage-Werkzeug und Funkgerät ab Sekunde 1.
- [x] Alte Chronicle-/Snapshot-Kompatibilitätspfade bereinigen (sauberer Schnitt ohne Legacy-Decoder).

### Stufe 2: Neues Speicherherz — `BrickArena` (`8×8×8` à `0,5 m`) & Mehrblock-Anker
- [ ] `Brick8` (`512` Zellen à `0,5 m` = `4 m` Würfel + `8 × u64` Belegungsmasken) und kontiguierliche `BrickArena` in `crates/world` anlegen.
- [ ] Zell-Repräsentation ohne Heap-Pointer bauen: `Empty`, `RawAtom(Material, Orientation, GranularFlag)`, `ModuleAnchor(ModuleSpec)` und `ModuleProxy(AnchorOffset)`.
- [ ] **Feste Mehrblock-Maschinen (`> 1 Block`)** auf dem neuen Anker/Proxy-System anlegen: Lichtbogen-Schmelze (`2×2×3`), Fabrikator (`2×1×2`), Elektrolyse-Raffinerie (`2×2×3`), Luftschleuse (`2×2×4`), Pilotensitz (`2×2×3`), Phosphor-Flugkonsole (`2×2×2`), Andockring (`2×2×1`) und Spawn-Koje (`2×4×2`).
- [ ] **Im Raster skalierbare Funktionsblöcke (`B × H × T`)** anlegen: Skalierbarer Thruster, skalierbarer Treibstofftank und skalierbarer Radiator (inkl. Zieh-Hologramm und atomarem Abbau über jede `ModuleProxy`-Zelle).
- [ ] `VoxelCells`, `ElevatedShape` (`Box<dyn VoxelShape>`) und `TurfCoat`-Einzelblock-Wrapper vollständig löschen.
- [ ] Inkrementellen `O(1)`-Massentensor (Masse $M$, Schwerpunkt $\vec{c}$, $3\times 3$-Trägheitstensor $\mathbf{I}$ via Satz von Steiner) direkt in den Brick-Container integrieren.

### Stufe 3: Grafik-Neubau — `glow`/`glutin` löschen $\rightarrow$ `wgpu` + `8-Byte` Quads (ThinkPad-Kern)
- [ ] `glow`, `glutin` und `glutin-winit` aus `crates/engine` und `crates/client` entfernen; `CpuMeshBatch` (`184 Bytes/Quad`) und `GlMesh` löschen.
- [ ] `wgpu`-Backend (Vulkan unter Linux, DX12/Vulkan unter Windows) mit expliziter **GPU-Auswahl (`HighPerformance` MX150 vs. `LowPower` Intel iGPU)**, `60 Hz` Frame-Pacing und **gedrosseltem PCIe-Upload-Ringbuffer (`<= 1,5 MB/Frame`)** aufbauen.
- [ ] **CPU Binary Greedy Mesher** auf den `u64`-Bitmasken von `Brick8` implementieren (`~20 µs` pro geändertem Brick inkl. vorberechneter 2-Bit Eck-AO).
- [ ] **`8-Byte` Packed Quad Pipeline (`u64` GPU Vertex-Pulling)** in `WGSL` schreiben und VRAM-Budget (`< 250 MB`) verifizieren.
- [ ] Low-Poly Hardware-Instancing (`< 150` Dreiecke pro Typ, per Matrix auf `B × H × T` skaliert) für Mehrblock-Maschinen und skalierte Thruster/Tanks einbinden.
- [ ] Kompakte `1024²` Sonnen-Shadow-Map + max. 16 Punktlichter + schnelle **`128×64` Atmosphären-LUT (`2 Hz` Update)** statt Pro-Pixel-Rayleigh-Shader einbauen.
- [ ] Diegetische Phosphor-Bordmonitore (`NoseFlightHud`, `BellyFlightHud`, Docking-Cam) auf die neue `wgpu`-Pipeline umziehen — mit striktem **Round-Robin-Update (`max. 1 Monitor pro Frame` bei `240×160`)**.

### Stufe 4: Deterministischer `i64`-Kosmos & Runde `Quad-Sphere`-Planeten
- [ ] Kosmische Koordinaten durchgehend auf **`CosmicPos` (`[i64; 3]` in `0,1 mm`-Schritten)** umstellen; kamera-relative Subtraktion vor der `f32`-Übergabe an `wgpu` verankern.
- [ ] Kepler-Bahnen und Hill-Sphären (`CosmicFrame`) auf plattformunabhängigen Cross-OS-Determinismus (Linux == Windows Bit-Gleichheit) umstellen.
- [ ] `CubePlanet`, `BeveledBoundary` und `PerimeterFold` (90°-Würfelkanten) ablösen durch **`Quad-Sphere`-Planeten** (6 sanft auf eine echte Kugel projizierte Gitter-Schalen, am Boden lokal rechtwinklig für nahtlosen Hausbau wie in Minecraft).
- [ ] **Sparse-Planet-Edits** durchsetzen: Unberührte Planetenoberflächen rendern rein über eine schlanke GPU-Höhen-Clipmap (`0 Bytes` Brick-RAM) und instanziieren `Brick8`-Blöcke erst bei realem Abbau, Hausbau oder Einschlagskratern.

### Stufe 5: Vereinheitlichung zum `GridBody` — Begehbare Schiffe & Raumstationen
- [ ] Die vier getrennten Strukturen `WorldBlocks`, `CantileverRig`, `ParkedFleet` und `WorldCraft` zu **einem einzigen universellen Typ `GridBody`** verschmelzen (Haus auf der Erde, Raumstation im Orbit, Asteroid und Raumschiff nutzen 100 % denselben Code).
- [ ] Kran-Zwang auflösen: Neuer Bau im Orbit oder am Boden startet an einem **magnetischen Montage-Anker / Docking-Flansch** und wird beim Entriegeln sofort ein freier `GridBody`.
- [ ] `VesselBerth` zu **geschachtelten Trägheits-Räumen (`Nested Inertial Grids`)** ausbauen: Sitz-Zwang im Flug aufheben, sodass Spieler während des Flugs frei durch Schiff und Raumstation laufen und bauen können.
- [ ] Lokale Scheinkräfte einspeisen: Linear-Beschleunigung ($-\vec{a}$ = künstliche Schwerkraft unter `1 g` Schub!) sowie Zentrifugal- und Corioliskraft für rotierende Raumstations-Ringe + Magnetstiefel für `0 g`.
- [ ] Physisches **Andocken (`2×2×1` Andockring)** zwischen zwei `GridBody`s (z. B. Landefähre ans Mutterschiff oder neues Modul an eine Raumstation koppeln) implementieren.

### Stufe 6: Skalierbare Antriebs-Physik & Echtzeit-Stufen-Absprengung
- [ ] Antriebs-Physik an die `B × H × T`-Skalierung knüpfen: Düsen-Querschnitt bestimmt Maximalschub (Planetenstart), Düsen-Länge bestimmt Effizienz/$I_{sp}$ (Deep-Space), Position relativ zum `O(1)`-Schwerpunkt bestimmt echtes Drehmoment.
- [ ] Dynamische Tankmasse (`B × H × T`) und Radiator-Wärmehaushalt im Vakuum aktivieren.
- [ ] **Inkrementellen `u64`-Connectivity-Check & Stufen-Trennung** einbauen: Bei Zerstörung tragender Blöcke oder Auslösen von Trenn-Bolzen spalten sich unverbundene Teile in `< 0,1 ms` als eigenständiger `GridBody` ab.

### Stufe 7: Sleep-by-Default Innenraum- & Schüttgut-Solver (`Sand` / `Luftdruck` / `Leitungen`)
- [ ] **Deterministische Schüttgut-Queue (`Sand`, `Regolith`, `Eisgeröll`)** implementieren: `0,0 %` Idle-CPU, Aufwachen beim Abbau darunterliegender Blöcke, Nachrutschen entlang $\vec{g}_{\text{lokal}}$ + `45°`-Böschungswinkel (`max. 256 Zellen/Tick`, Schweben bei `0 g`).
- [ ] Inkrementellen **Raum-Druck- & Temperatur-Graphen** (Flood-Fill nur über geänderte Bricks) einbauen: Beliebig geformte, geschlossen gemauerte Häuser (Mars/Mond), Stationen und Schiffsrümpfe halten automatisch Atemluft.
- [ ] Luftschleusen-Zyklus, Hüllenlecks (inkl. Vakuum-Ausström-Rückstoß) und Triebwerks-Bodenverglasung über die ereignisgesteuerte Aktiv-Liste steuern.
- [ ] Physische **Rohr- & Datenkabel-Blöcke** im `0,5 m`-Raster aktivieren (unterbrochene Leitung kappt Treibstoffzufuhr bzw. schaltet betroffene Bordkamera auf Schnee/Rauschen).

### Stufe 8: Kopplung der Dual-Pfad-Akustik (Luftdruck + Stiefel-Körperschall)
- [ ] `crates/acoustics` an den lokalen Druck des Raum-Graphen anbinden (stufenloses Ausblenden beim Abpumpen der Schleuse bis zu `0,0 %` Luftschall im Vakuum).
- [ ] **Körperschall-Pfad** aktivieren: Solange Füße, Hände oder Sitz Kontakt zum `GridBody` haben, werden Triebwerksgrollen, Struktur-Ächzen, nachrutschendes Geröll im Silo und Andock-Stöße über einen `< 220 Hz`-Tiefpass übertragen.
- [ ] Sprach-Routing (`chat_medium.rs`) dynamisch mit dem Raum-Druck-Graphen koppeln (normales Sprechen ohne Helm in jedem unter Druck stehenden Raum — egal ob Erde, Mondbunker, Raumstation oder fliegendes Schiff).

### Stufe 9: Zero-Difference `SimKernel`, Kojen-Drop-In & ThinkPad-Abnahme
- [ ] Singleplayer (`SavedPlanet`-Direktpfad entfernen!) und Multiplayer auf **einen einzigen `120 Hz` `SimKernel`-Nachrichtenstrom** (lokaler Zero-Copy-Ringbuffer vs. `sanctuary`-P2P-Kanal) vereinheitlichen.
- [ ] Chronicle & Horizont-Streaming auf komprimierte `LZ4`-Brick-Deltas und `GridBody`-Zustände umstellen.
- [ ] **Drop-In-Kojen-Spawn** fertigstellen: Beitretende Freunde wachen direkt in der nächsten freien Spawn-Koje (`2×4×2`) der Raumstation, Mars-Basis oder des Expeditionsschiffs auf.
- [ ] **Abschluss-Verifikation (Linux + Windows):** Automatisierte Determinismus- & Replay-Suite (`--headless`) sowie Live-Benchmark auf ThinkPad-T480-Profil (`1920×1080`, `< 250 MB` VRAM, stabile `60 FPS` auf MX150 und flüssiger Akkubetrieb auf Intel iGPU).
