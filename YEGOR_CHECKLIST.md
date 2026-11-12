# Yegor Bugayenko (Elegant Objects) — Refactoring & Prüf-Checkliste

## Bereits umgesetzt

- [x] **`SlotPlaque`-Decorator-Kette für Beutel-Slots** (`crates/client/src/attire/compartment.rs`):
  - Schmale Schnittstelle `SlotPlaque` (`emit_slot`) statt prozeduraler `if item.gripped()`-UI-Logik in `SatchelTray::emit_on_waist`.
  - Komponierbare Decorators mit jeweils `<= 4` privaten Feldern: `TalliedCompartment(ShowcasedCompartment(WeightedCompartment(CompartmentBackplate)))` sowie `LegendCompartment`.
- [x] **`CarryLoad`-Interface & `JoinedLoad`-Decorator** (`crates/client/src/attire/burden.rs`):
  - Polymorphe Gewichtssummierung über `HeldRelic`, `SlotBurden`, `SatchelTray` und `JoinedLoad<A, B>` ohne Herausreichen roher Masse-Zahlen.
- [x] **Keine Daten-Getter / statischen Hilfsfunktionen für Legende & Titel**:
  - `std::fmt::Display` auf `WeightGrade` und `SlotBurden` statt `english_title()` / `legend_heading()`.
  - Statische Methode `RelicFigure::emit_legend` entfernt und durch das Objekt `LegendCompartment` ersetzt.
- [x] **Sauberer Primärkonstruktor & Kapselung in `SatchelTray`** (`crates/client/src/attire/tray.rs`):
  - `SatchelTray::new` weist Argumente direkt zu (`TrayState` statt `bool`-Verzweigung).
  - Öffentliche Getter `cursor()`, `active_side()` und `packed_slots()` entfernt; ersetzt durch Verhalten (`selected_relic()`, `open_if_populated()`, `focused_sides()`).

---

## Geplante Änderungen (Nächster Schritt)

- [ ] **`BurdenedGait<G: LocomotionGait>`-Decorator für Bewegung & Sprung** (`crates/client/src/gait/mod.rs`):
  - `LocomotionGait` um `fn pace_leap(&self, leap_speed: f32) -> f32` erweitern.
  - Stack-allokierten Decorator `BurdenedGait<G>` (`origin: G`, `burden: SatchelBurden`, 2 Felder) einführen, der `pace_distance` und `pace_leap` skaliert und alle übrigen Gait-Nachrichten an `origin` delegiert.
- [ ] **Entkopplung von `SurfaceFlight`, `LeapIntent` und `AvatarBody`** (`crates/client/src/order.rs`, `flight.rs`, `avatar/body.rs`):
  - `LocomotionOrder` kapselt `gait: BurdenedGait<ActiveGait>` (`with_burden(SatchelBurden)`).
  - `GravitationalFlight::jump_burdened` und `PendingLeap` in `flight.rs` zurückbauen — `SurfaceFlight::advance` fragt ausschließlich `gait.pace_leap(self.state.jump_strength)` ab und kennt `SatchelBurden` nicht mehr.
  - `LeapIntent::apply` und `AvatarBody::prepare_stride` wieder vollständig frei von `SatchelBurden` halten.
- [ ] **Pass-Through-Getter `carried_burden()` eliminieren ("Tell, Don't Ask")** (`crates/client/src/avatar/bearing_gear.rs`, `earthworks/mod.rs`, `avatar/planetary.rs`):
  - `carried_burden()` auf `AvatarGear` und `PalmEarthworks` durch `encumber_order(&self, order: LocomotionOrder) -> LocomotionOrder` ersetzen, sodass `PlanetaryAvatar::step_motion` nur `let order = self.gear.encumber_order(order);` aufruft.
- [ ] **Unveränderlichkeit (`Immutability by Default`) auf `SatchelTray` prüfen** (`crates/client/src/attire/tray.rs`, `pouch/intake.rs`, `pouch/mod.rs`):
  - Mutierende Methoden (`close`, `focus_side`, `stow_relic`, `take_stowed`, `toggle_open`, `navigate`) auf dem `Copy`-Objekt `SatchelTray` in unveränderliche Übergänge überführen.
- [ ] **Weitere Kapselung auf `SlotBurden` prüfen** (`crates/client/src/attire/burden.rs`, `attire/tests/burden.rs`):
  - Prüfen, ob `unit_grade()`, `summed_grade()` und `corner_pigment()` privat (`const fn`) werden können und die Unit-Tests ausschließlich über `Display`, `CarryLoad` und `SlotPlaque`-Geometrie/Farben testen.

---

## Noch zu prüfen (Verifikation nach Umsetzung)

- [ ] **Zeilenlimits (`wc -l`)**: Alle neuen und geänderten `.rs`-Dateien unter `250` Zeilen Headroom (strikt `<= 300` Zeilen).
- [ ] **Compiler-Sauberkeit**: `cargo check -p client` ohne Warnungen, ohne ungenutzte Imports und ohne Dead Code.
- [ ] **Architektur-Wächter**: `cargo test -p client --bin client architecture_tests` (max. 4 Felder pro `struct`, max. 300 Zeilen, lokale `impl`-Blöcke).
- [ ] **Zero-Allocation-Garantie im Hot-Loop**: `cargo test -p client --bin client allocation_audit` (0 Heap-Allokationen über 600 Frames Gehen, Sprinten und Springen mit `BurdenedGait`).
- [ ] **Fachliche Ziel-Tests**:
  - `cargo test -p client --bin client attire::tests`
  - `cargo test -p client --bin client earthworks_tests`
  - `cargo test -p client --bin client troupe_tests`
  - `cargo test -p client --bin client ballistic_tests`
