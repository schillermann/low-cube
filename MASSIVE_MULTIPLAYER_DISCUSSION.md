# Diskussionsplattform: Großserver-Architektur (100–500+ Spieler)

> **Status:** Basis-Architektur in `v0.112.0` umgesetzt; Feinjustierungen für Dauerbetrieb (§5) spezifiziert im Zusammenspiel mit [`COSMOS_PLAN.md`](./COSMOS_PLAN.md) und [`ARCHITECTURE_REFLECTION.md`](./ARCHITECTURE_REFLECTION.md).
> **Ziel:** Festhalten aller technischen Befunde, Rechenwerte, Architektur-Bausteine und beschlossenen Gamedesign-Entscheidungen für Community- und Großserver mit Hunderten gleichzeitigen Spielern.

---

## 1. Ausgangslage: Wo die aktuelle Architektur an Grenzen stößt

Das P2P-/Dedicated-Protokoll in `crates/sanctuary` ist aktuell ideal für **kleine, latenzarme Koop-Gruppen (2–16 Spieler)** ausgelegt. Für **Hunderte gleichzeitige Spieler (100–500+)** auf einer persistenten Welt gibt es fünf konkrete strukturelle Hürden:

### 1.1 Harte `1.500 Byte` UDP-MTU-Grenze & $O(N^2)$-Broadcast (~25-Spieler-Mauer)
* **Ist-Zustand im Code:**
  * `GuestCohort::broadcast` (`crates/sanctuary/src/cohort.rs`) sendet **60-mal pro Sekunde** (`PULSE_INTERVAL_SECS = 1.0 / 60.0`) die vollständige Liste **aller** `CompanionPresence`- und `DrivenCraft`-Einträge der gesamten Welt in einem einzigen `SanctuaryEnvelope::HostPulse`-Datagramm an jeden verbundenen Client.
  * `HostWire::pump_authoritative` (`crates/sanctuary/src/host_wire.rs`) und der Client lesen UDP-Pakete in einen festen `[0u8; 1500]`-Puffer.
* **Folge:**
  * Bei ~50–65 Bytes pro Spieler/Fahrzeug sprengt `HostPulse` bereits ab **ca. 22–25 Spielern** die `1.400–1.500 Byte` MTU-Grenze. Abgeschnittene UDP-Pakete scheitern an der ChaCha20-Poly1305-AEAD-Prüfung $\rightarrow$ Totalausfall der Positions-Updates.
  * Selbst mit Fragmentierung wächst der ausgehende Verkehr quadratisch ($N^2$): `300 Spieler × 300 Einträge × 60 Hz ≈ 5,4 Mio. Einträge/s` (`~300–400 Mbit/s` reiner Positions-Overhead).

### 1.2 Stundenlanger Late-Join-Sync durch lineares `ChronicleLog`
* **Ist-Zustand im Code:**
  * `ChronicleLog` (`crates/sanctuary/src/chronicle_log.rs`) speichert jede dauerhafte Weltänderung (`WorldFact`: Graben, Bauen, Fällen, Fahrzeug-Blaupausen) seit Serverstart in einem einzigen linearen `Vec<WorldFact>`.
  * Neu beitretende Spieler (`ack_seq = 0`) erhalten pro Tick maximal `24 Facts` (`<= 1.100 Bytes`, d. h. maximal `1.440 Facts/s`).
* **Folge:**
  * Bauen und graben 200 Spieler über mehrere Tage oder Wochen, wächst das Log leicht auf **5–20 Millionen Einträge** (inklusive längst wieder abgerissener oder überbauter Blöcke sowie Änderungen auf fernen Planeten).
  * Ein neuer Spieler müsste **1 bis 4 Stunden warten**, bis die aktuelle Welt aufgebaut ist.

### 1.3 Weltfläche, Ressourcen-Erschöpfung & Spawn-Kollision
* **Flächenrechnung:**
  * Ein `1024`-Block-Würfel (`R = 512 m`) besitzt `~1,05 km²` pro Würfelseite (`~6,29 km²` insgesamt) – passend für ca. **20–50 Spieler** (`CONCEPT.md`).
  * Bei **300 gleichzeitigen Spielern** (und >1.000 registrierten Basen auf einem persistenten Server) stünde im Start-Biom (`Temperate`) statistisch **alle ~50–60 Meter** ein fremdes Bauwerk; Bäume und oberflächennahe Erze (`LivingVeins`) wären binnen Stunden erschöpft.
  * Zudem setzt `GuestCohort::admit_guest` alle neuen Spieler exakt auf `(0.0, half, 0.0)` ab.

### 1.4 Reine RAM-Haltung ohne Disk-Persistenz
* Aktuell liegen `ChronicleLog` und alle veränderten Sektoren ausschließlich im Arbeitsspeicher. Für wochenlang laufende Großserver fehlen Crash-Recovery (WAL/Snapshots auf SSD) sowie das Auslagern unbesuchter Sektoren/Monde aus dem RAM.

### 1.5 Serieller `Mutex<HostWire>` für Krypto & Netzwerk
* `SanctuaryHost` (`crates/sanctuary/src/sanctuary_host.rs`) verarbeitet Socket-I/O und AEAD-Ver-/Entschlüsselung unter einem zentralen `Mutex<HostWire>`. Bei `300 Spielern × 30–60 Hz` (`9.000–18.000` verschlüsselte Pakete/s pro Richtung) wird ein einzelner Thread zum Flaschenhals.

---

## 2. Lösungsbausteine für Großserver (kompatibel mit Pure OOP & `COSMOS_PLAN.md`)

Das Sektor-Streaming (Phase 3) und die lokalen `BodyId`-Bezugssysteme (Phase 4) aus [`COSMOS_PLAN.md`](./COSMOS_PLAN.md) legen bereits das ideale Fundament, auf dem vier Server-Erweiterungen aufsetzen können:

### Baustein A: Strikte Horizont- & Klimazonen-Isolation (`0 Hz / 0 RAM / 0 GPU` außerhalb des Sichtrings)
* Einen Spieler (z. B. in `Temperate` auf der Erde) darf technisch **weder im Netzwerk, noch im RAM, noch auf der CPU, noch im GPU-VRAM** belasten, was weiter weg auf seiner eigenen Würfelseite, in einer der anderen 5 `ClimateZone`s (`Arctic`, `Jungle`, `Desert`, `Steppe`, `Volcanic`) oder auf einem fernen Planeten geschieht:
  1. **Nahfeld (`20–30 Hz`):** Nur Spieler, Fahrzeuge, Grab-Risse (`PeerCrack`) und Wildtiere (`SectorFauna`) innerhalb des lokalen Sichtrings (`~160 m`, über `(BodyId, ClimateZone, Sector)` indiziert; Faltung in die Nachbar-Klimazone ausschließlich im `160 m`-Randstreifen einer Würfelkante) – strikt gedeckelt auf `<= 1.240 Bytes` (`max. 16` Nachbarn pro Paket).
  2. **Hotspot-Fairness ohne Flackern (`SmoothRoster`-Lease):** Stehen 80–120 Spieler am selben Raumhafen, werden die nächsten `8` Nachbarn jeden Puls gesendet und die übrigen per Round-Robin rotiert (`0,85 s` Sight-Lease für butterweiches `CompanionGlide`).
  3. **Fernfeld, andere Klimazonen & andere Planeten (`0 Hz / 0 Byte / 0 Objekte`):** Ferne Gebiete derselben Klimazone, die 5 anderen Klimazonen (inkl. ihrer Bäume `SectorForest`, geparkten Fahrzeuge `HorizonVessels` und GPU-Slices am Boden) sowie andere Himmelskörper kosten **null** Bandbreite, RAM und Tick-Zeit.

### Baustein B: Räumliche Sektor-Zustands-Snapshots (`SectorChronicle`) + Entkoppeltes `VesselCatalog`
* **Warum weder ein globales noch ein bloß pro `BodyId` geteiltes Event-Log ausreicht:**
  1. Auf einer `4096³`-Erde (`100,7 km²`) finden `80–90 %` aller Grab- und Bauaktionen auf `BodyId::EARTH` statt – eine reine `BodyId`-Partitionierung ließe die Erde weiterhin auf Millionen Raycast-Events anwachsen, obwohl ein Spieler in `Temperate` Änderungen auf der Rückseite (`Volcanic`, `6 km` entfernt) gar nicht sehen kann.
  2. Raumschiffe (`CraftCommissioned`, `HelmReleased`) reisen zwischen Erde, Mars und Asteroiden und gehören deshalb **keinem** festen Himmelskörper an.
* **Zweigeteilte Zielarchitektur:**
  1. **Ortsfestes Terrain (`SectorChronicle` pro `(BodyId, Face, SectorCoord)`):** Mehrere Änderungen an derselben `GridCell` falten sich sofort zum **aktuellen Netto-Zellzustand (`SectorSnapshot`)** zusammen (null Raycast-Replay für historische Zwischenschritte). Ein Client abonniert wie bei `ProximityAudience` ausschließlich die Sektoren in seinem lokalen Sichtring (`HorizonSectors`) $\rightarrow$ **Login-Sync dauert immer `< 0,2–0,5 Sekunden`**, und ferne Kontinente kosten ebenso `0 Bytes` wie ferne Monde. Bereits geladene Nachbar-Sektoren erhalten Live-Änderungen weiterhin sofort als schlanke Delta-`WorldFact`s.
  2. **Reisende Schiffe (`VesselCatalog`):** Schiffs-Blaupausen (`CraftIdentity -> CraftBlueprint`) und geparkte Posen sind vom Boden-Log entkoppelt. Sobald ein fremdes Schiff (fliegend oder geparkt) erstmals in den Sichthorizont eines Clients eintritt, überträgt der Server einmalig dessen kompaktes `CraftBlueprint` on-demand.

### Baustein C: Disk-Persistenz & Kalte-Sektor-Auslagerung (`VaultArchive`)
* Jeder Sektor schreibt Änderungen in ein schnelles Append-Only-Journal auf SSD und verdichtet sich periodisch zu kompakten Sektor-Snapshots.
* Sektoren und fremde Monde ohne anwesende Spieler werden nach einer Ruhefrist aus dem Server-RAM entladen und bei Annäherung verzögerungsfrei von SSD geladen.

### Baustein D: Parallele Paket-Versiegelung & Klimazonen-/Cluster-Sharding
* **Parallele Sitzungs-Versiegelung (`ParallelSeals`):** Eingehende UDP-Datagramme und ausgehende versiegelte Pakete werden außerhalb des Welt-Ticks parallel über `SessionSeal` entschlüsselt/verschlüsselt.
* **Simulation nach `(BodyId, ClimateZone, SectorCluster)` statt nur nach `BodyId`:** Würde man nur nach `BodyId` parallelisieren, lägen `900` Erd-Spieler auf einem einzigen CPU-Kern! Weil sich Klimazonen und entfernte Sektor-Inseln gegenseitig nicht beeinflussen, rechnet der Server alle 6 Klimazonen der Erde sowie aktive Mond-/Asteroiden-Sektoren parallel auf getrennten Kernen, während unbewohnte Sektoren komplett schlafen (`0 %` CPU).

### Baustein E: Begehbarer Voxel-Asteroidengürtel (`2,20 .. 3,20 AU`)
* `BeltRock` (`crates/world/src/celestial/belt.rs`) nutzt bereits deterministische Sektor-Hashes und `32.768` reservierte `BodyId::belt_rock`-Kennungen (`0x8000 | slot`), wird aktuell aber nur visuell als einzelner Würfel gezeichnet.
* Für die Großserver-Rohstoffwirtschaft werden angeflogene `BeltRock`-Instanzen on-demand als echte, begehbare Mini-`CubePlanet`s (`16³..128³` Blöcke mit reichen `LivingVeins`-Erzadern und schwacher Mikro-Gravitation) materialisiert und über `SectorChronicle` persistent gespeichert.

---

## 3. Entschiedene Produkt- & Gamedesign-Leitplanken

Die 5 zentralen Design-Fragen für Großserver sind wie folgt festgelegt:

### 3.1 Skalierbare World-Cube-Größe (`size = 1..=32`) mit `< 1 Stunde` Epstein-Gürtel-Garantie
* **Ein einziger Skalierungsfaktor `size` (`earth_blocks = 128 × size`):**
  * Der Heimat-World-Cube bleibt über `PlanetaryScale` (`[world] size` / `--size`) von `size = 1` (`128³`, `0,10 km²`) über `size = 8` (`1024³`, `6,29 km²`) bis **`size = 32` (`4096³`, `100,7 km²` allein auf der Erde)** skalierbar; Planeten- und Mondgrößen wachsen proportional mit.
  * Auf einer `4096³`-Welt beträgt die lokale `f32`-Präzision selbst an den äußersten Würfelecken `0,24 mm` (butterweich ohne Kamera-/Kollisions-Jitter).
* **Verbindliche Reisezeit-Obergrenze (`< 1 Stunde` bis zum äußeren Asteroidengürtel):**
  * Das Sonnensystem muss **nicht** starr im `1:1`-Originalverhältnis erzwungen werden – Vorrang hat die Spielregel, dass ein Schiff mit **Epstein-Drive (`FusionTorch`, `~8 G = 78,5 m/s²` im Flip-&-Burn-Flug)** den **äußeren Rand des Asteroidengürtels zwischen Mars und Jupiter (`3,20 AU`) selbst bei Maximalgröße der Welt in unter einer Stunde erreicht**.
  * Dafür wird die künstliche `48 km/s`-Drossel (`MAX_FUSION_SPEED` in `space_step.rs`) für Fusionsantriebe geöffnet (z. B. `>= 200 km/s`), die Log-Depth-Far-Plane in `shader.rs` auf `5–10 Mrd. m` erweitert, und `1 AU` nach oben auf das `< 1h`-Epstein-Budget (`max. ~48 Mio. m`, sodass selbst der ungünstigste Konjunktionsflug `4,4 AU` hinter der Sonne vorbei unter `~55 Minuten` und der Normalflug `2,5 AU` bei `~41 Minuten` bleibt) gedeckelt.
  * **Antriebs-Konsistenz über alle Weltgrößen:** Chemische Raketendüsen (`MAX_COSMIC_SPEED`) skalieren automatisch mit $\sqrt{\text{size}}$ (`45 m/s × √size`), damit jedes Raketen-Blueprint unabhängig von `size` zuverlässig Erdorbit und Mond erreicht ($v_{\text{esc}} = \sqrt{2 g R}$), während der interplanetare Flug zum Asteroidengürtel den Epstein-Drive belohnt:

| `size` | World-Cube ($D$) | Fläche Erde (`6×D²`) | `1 AU` | Epstein-Flug zum äußeren Gürtel (`2,5 AU` Mittel / `4,4 AU` Worst-Case bei `8 G`) |
| :--- | :--- | :--- | :--- | :--- |
| **`1`** | `128³` | `0,10 km²` | `1,50 Mio. m` | **7,3 Min.** / 9,7 Min. |
| **`4`** | `512³` | `1,57 km²` | `6,01 Mio. m` | **14,6 Min.** / 19,4 Min. |
| **`8`** | `1024³` | `6,29 km²` | `12,02 Mio. m` | **20,6 Min.** / 27,4 Min. |
| **`16`** | `2048³` | `25,2 km²` | `24,04 Mio. m` | **29,2 Min.** / 38,7 Min. |
| **`32` (Max)** | **`4096³`** | **`100,7 km²`** | **`48,09 Mio. m`** *(AU-Cap)* | **41,3 Min.** / **54,7 Min. (< 1 h!)** |

*(Falls ein Serverbetreiber jemals `size = 64` / `8192³` wählt, greift derselbe `48 Mio. m`-AU-Deckel automatisch und hält die Flugzeit zum äußeren Asteroidengürtel auch dort strikt unter einer Stunde.)*

### 3.2 Spawn-Strategie: Platzierbare Admin-Spawn-Blöcke
* Neue Spieler spawnen nicht mehr alle starr auf `(0.0, half, 0.0)`: Der Server-Admin kann gezielt spezielle **Spawn-Blöcke** (`BlockMedium`-Erweiterung mit Admin-/Host-Berechtigungsprüfung in `WardedPlanet` / `HostState`) in der Welt platzieren.
* `GuestCohort::admit_guest` verteilt neu beitretende Spieler über die aktiven Admin-Spawn-Blöcke (mit `(0.0, half, 0.0)` lediglich als initialem Fallback, solange noch kein Spawn-Block gesetzt wurde).

### 3.3 Ressourcen-Ökologie: Endliche Heimat-Rohstoffe $\rightarrow$ Expansion ins All
* **Kein künstlicher Erz-Respawn (`LivingVeins`) auf dem Heimat-World-Cube.**
* Sind die oberflächennahen und tiefen Vorkommen der Heimatwelt abgebaut, entsteht ein natürlicher wirtschaftlicher und technologischer Druck: Spieler weichen mit Raumschiffen auf **andere Planeten/Monde** (Mond, Mars, Jupitermonde, Titan) aus oder betreiben **Bergbau im Asteroidengürtel (`2,20 .. 3,20 AU`, Baustein E)**.

### 3.4 Grundstücks-Schutz: Reine Sandbox (vorerst keine Claims)
* Vorerst **keine künstliche Block-Absicherung / Claim-Mechanik** – volle physikalische Sandbox-Freiheit ohne zusätzlichen Protokoll- und Berechtigungs-Overhead.

### 3.5 Kommunikation: Lokaler Proximity-Text-Chat
* Kein globaler Server-Chat für Hunderte Spieler und kein Voice-Streaming-Overhead: Spieler unterhalten sich per **Text-Chat ausschließlich mit Spielern in ihrer räumlichen Nähe** (gleicher `BodyId` innerhalb eines definierten Hör-/Sicht-Radius, direkt gekoppelt an das Nahfeld-Interest-Management aus Baustein A).

---

## 4. Maximale Spieleranzahl & Kapazitätsrechnung (bei `0 Hz` Fernfeld)

Weil weit entfernte Spieler **null** Bandbreite füreinander erzeugen (`0 Hz`), fällt das quadratische $O(N^2)$-Wachstum komplett weg und der Server skaliert **linear ($O(N)$)** mit der Gesamtzahl $N$ der eingeloggten Spieler:

### 4.1 Bandbreite & Paketlast pro Spieler
* **`companion_id` im Protokoll:** Bereits `u32` (`> 4,2 Mrd.` eindeutige IDs $\rightarrow$ kein Protokoll-Limit).
* **Ausgehend pro Spieler (Server $\rightarrow$ Client, `20 Hz`):** Im Schnitt `~600–800 Bytes` (max. `1.240 Bytes` MTU-Deckel für bis zu `~16–18` Nachbarn pro Paket) $\implies$ **`~128–200 kbit/s` pro Spieler**, völlig unabhängig davon, ob 50 oder 2.000 Spieler auf dem Server sind!
* **Eingehend pro Spieler (Client $\rightarrow$ Server, `30 Hz`):** `~100 Bytes` (`GuestPulse`) $\implies$ **`~24 kbit/s` pro Spieler**.

### 4.2 Realistische Maximalwerte nach Server-Hardware (einzelner Dedicated-Prozess)

| Gleichzeitige Spieler ($N$) | Ausgehende Bandbreite | Pakete/s (RX + TX) | Krypto (`SessionSeal` AEAD) & Tick-Last | Benötigte Server-Hardware |
| :--- | :--- | :--- | :--- | :--- |
| **~100–150 Spieler** | `~15–25 Mbit/s` | `~5.000–7.500 pkt/s` | `< 10 %` eines Kerns, `< 1 ms` Tick | Kleiner **2-vCPU VPS** (`4 GB RAM`) |
| **~500 Spieler** *(Komfort-Ziel)* | `~65–100 Mbit/s` | `~25.000 pkt/s` | `< 10 %` auf 4 Kernen (`ParallelSeals`), `~2–3 ms` Tick | Mittelklasse **4–6 Kern Server** (`8–16 GB RAM`, `1 Gbit/s`) |
| **~1.000 Spieler** | `~130–200 Mbit/s` | `~50.000 pkt/s` | `< 15 %` auf 8 Kernen, `~5–6 ms` Tick | **8-Kern Dedicated Server** (`16–32 GB RAM`, `1 Gbit/s`) |
| **~2.000–2.500 Spieler** *(Praktisches Einzel-Prozess-Limit)* | `~300–450 Mbit/s` | `~100.000–125.000 pkt/s` | `~1,5–2 Kerne` Krypto + `BodyId`-Sharding (`~10–14 ms` Tick) | Starker **16-Kern Root-Server** (`32–64 GB RAM`, `1 Gbit/s`, UDP-Socket-Batching) |

### 4.3 Lokales Hotspot-Limit (wenn sich viele Spieler am selben Ort treffen)
* Selbst wenn sich **80–120 Spieler gleichzeitig am selben Raumhafen oder Spawn-Punkt** versammeln, reißt kein einziges UDP-Paket die `1.500 Byte`-MTU-Grenze:
  * Jedes `20 Hz`-Paket enthält die `8–10` unmittelbarsten Nachbarn (`< 25 m`) plus `8` rotierende Nachbarn aus dem weiteren Sichtfeld (`25–150 m`).
  * Durch das `~0,8 s`-Sicht-Lease in `SmoothRoster` bleiben alle `80–120` Spieler vor Ort gleichzeitig sichtbar und gleiten flüssig über den Bildschirm, während Spieler jenseits des Sichtrings sauber ausgeblendet werden (`0 Hz`).

---

## 5. Umsetzungsstand (`v0.113.0`) & Verbleibende Ausbaustufen

### 5.1 In `v0.112.0` bis `v0.114.0` produktiv umgesetzt
* **Mitbewegte lokale Hill-Sphären („Gravity Bubbles“ `CosmicFrame` / `GravityBubble` / `FrameHandoff`) + `LWEL` `epoch_secs`-Sync (`v0.114.0`):**
  * **`CosmicFrame` (`Body(BodyId)`, `BeltAsteroid { id, cell }`, `DeepSpace([i64; 3])` mit `4.096 m`-Sektoren in `crates/world/src/celestial/bubble.rs`):** Sub-Millimeter-`f32`-Präzision (`< 0,13 mm` ULP) von der Erde über den äußeren Asteroidengürtel (`3,20 AU`) bis `Triton` (`30,07 AU`), inklusive `0 m/s` Flip-&-Burn-Wendemanövern; kontinuierlich weiterlaufende Planetenbahnen ohne `if !locked`-Orbit-Freeze (`HeliosphereWorld::step_surface`) und freie Gezeitenkompensation (`-frame_accel`).
  * **`MtuBudgetAudience(CosmicCraftSight(CosmicBubbleAudience))` (`crates/sanctuary/src/bubble_audience.rs`) & `EpochSync` (`link_state/sync.rs`):** Synchronisiert beim `LWEL`-Handshake die live laufende Server-`epoch_secs`, isoliert Mitspieler, `SectorCracks` und `HorizonStream` pro `CosmicFrame` (`0 Hz / 0 Extra-Bytes` für Erdspieler) und projiziert bis `2.400 m` nahe Raumschiffe über Bubble-Grenzen hinweg exakt in `f64`.
* **Strikte Horizont- & Klimazonen-Isolation (`v0.113.0`):**
  * **`SectorForest` & `SectorGrove` (`crates/world/src/tree/sector_forest.rs`, `sector_grove.rs` & `zoned_forest.rs`):** `0` eager erzeugte Bäume beim Start einer `4096³`-Welt; allokationsfreie Stack-Kollision (`constrain_untouched_sector`) und Kronenreibung im `120 Hz`-Hotpath; Bäume und `FloorLitter` entstehen und ticken ausschließlich in aktiven `4m × 4m`-Sektoren und streamen lokal über `MeadowRing`.
  * **Horizont-GPU-Slices (`PlanetaryMesh::with_horizon` in `crates/engine/src/composite_mesh.rs`, `launch.rs`, `ShadedScenePass`) & `present_vessels_near`:** Am Boden von `Temperate` wird auf `4096³` initial nur die eigene `ClimateZone` gebaut und hochgeladen (Nachbar-Klimazone erst `< 180 m` vor der Würfelkante, andere Seiten erst im Orbit); geparkte Fahrzeuge anderer Klimazonen werden im Frame-Loop übersprungen.
  * **`SectorGridAudience` + `VelocityScaledSight` (`crates/sanctuary/src/sector_audience.rs`):** `O(N)`-Raumgitter (`160 m`-Zellen) mit Hemisphären-/Klimazonen-Trennung und geschwindigkeitsskalierter Sichtweite (`bis 2.400 m`) für im eigenen Himmel landende Raumschiffe.
  * **`SpatialFactIndex` & `HorizonStream` (`crates/sanctuary/src/spatial_chronicle.rs`):** Jeder Gast erhält einen persönlichen `HostChronicle`-Strom ausschließlich aus seinem `256 m`-Sichtring derselben (bzw. kanten-angrenzenden) `ClimateZone`, angeflogenen Himmelskörpern und sichtbaren Schiffs-Blueprints (`0 Bytes` für andere Klimazonen und ferne Planeten).
  * **`SectorCracks` (`crates/sanctuary/src/aura.rs`), `0 Byte` ruhiges `WildlifePulse` (`envelope.rs`), `VaultStream` (`chronicle_decorators.rs`), `RingEvicted` `ReliefCellCache` & `OffworldVault` (`crates/world/src/celestial/heliosphere/vault.rs`):** Sektor-isolierte Grab-Risse, null Wildtier-Traffic außerhalb von `160 m`, Tick-gepuffertes `BufWriter<File>`-SSD-WAL, ringförmige Zell-Cache-Verdrängung und ein warmer `32`-Körper-Serverpool gegen `2`-Slot-LRU-Thrashing.
* **Basis-Großserver & Kosmos (`v0.112.0`):** `MtuBudgetAudience` + `SmoothRoster` (`0,85 s` Sight-Lease), `AdminBlockWard` + `SpawnBeacons` (`SpawnStone`), `ParallelSeals`, `EPSTEIN_HOUR_AU_CAP` (`< 1 h` bis `3,20 AU`), `10 Gm` Log-Depth, `SurfaceContact` und `96 m` `ProximityChatModal`.

### 5.2 Was als Nächstes umgesetzt werden kann (Priorisierte Ausbaustufen)
1. **`AboardVessel`-Relativkoordinaten (`PresenceFrame`) für Multi-Crew-Raumschiffe:** Überträgt Mitfahrer auf fliegenden `VesselDeck`s relativ zum Schiffsrumpf, damit Besatzungen auch bei `50–240 km/s` frei durch die Kabine laufen können.
2. **Serverseitiger `SurvivorLedger` (Inventar & Auslogg-Ort pro X25519-Key):** Schützt die endliche Erz-Ökonomie vor lokalen `server.toml`-Manipulationen und bewahrt Position + Beute bei Verbindungsabbrüchen im Asteroidengürtel.
3. **Netto-Zellzustands-Snapshots (`SectorSnapshot`) bei Langzeit-WAL-Kompaktierung:** Faltet hundertfach umgebaute Sektoren auf ihren reinen Zell-Endzustand zusammen (null historische Raycasts beim Betreten alter Städte).
4. **Klimazonen-/Cluster-paralleler Server-Tick & Physikalische Sandbox-Vertiefung (`CargoBallast`-Erzmasse, Vakuum-Kontaktfunk, bergbare `LiveTape`-Blackboxes):** Multi-Core-Simulation der 6 Erd-Klimazonen plus emergentes Hard-Sci-Fi-Gameplay ohne globale Broadcasts.
