# Notiz: Co-op Beitritt & `NAT LOCKED - TELL HOST` Ablauf

Dokumentation zum beobachteten Verhalten beim Klick auf **`JOIN CO-OP / SERVER`** mit anschließender Meldung:

```text
NAT LOCKED - TELL HOST: Z08G-QYHW-AJ86
```

---

## 1. Ursache & Technischer Hintergrund

Low-Cube nutzt für Co-op-Sitzungen eine komplett serverlose Peer-to-Peer-Verbindung (UDP + SPAKE2 + X25519).

1. Beim Klick auf **`JOIN WORLD (ENTER)`** versucht [`PortalDial::begin`](../crates/client/src/portal_config.rs) zunächst `240 ms` lang, den Gastgeber direkt per UDP (`LPK1`) zu erreichen.
2. Blockiert der Router des Gastgebers (z. B. eine **FRITZ!Box** ohne aktives UPnP/Port-Forwarding) unangeforderte eingehende UDP-Pakete, kommt innerhalb dieser `240 ms` keine Antwort zurück.
3. Daraufhin erzeugt der Gast-Client über [`PunchCode`](../crates/sanctuary/src/wormhole/phrase.rs) seinen eigenen 12-stelligen Gegen-Code (z. B. `Z08G-QYHW-AJ86`), kopiert ihn automatisch in die Zwischenablage ([`PortalStudio::announce_punch_reply`](../crates/client/src/menu/portal/mod.rs)) und zeigt `NAT LOCKED - TELL HOST: ...` an ([`PortalLayout::status_line`](../crates/client/src/menu/portal/layout.rs)).
4. Im Hintergrund hält `PortalDial::Punching` den lokalen STUN-gemappten UDP-Socket für bis zu `180 s` offen und sendet alle `160 ms` weitere Handshake-Pakete ([`RemoteLink::step_handshake`](../crates/sanctuary/src/remote_link.rs)).

---

## 2. Aktueller Ablauf: Wer muss was machen?

### Schritt A — Portal öffnen (Gastgeber)
1. **`ESC`** $\rightarrow$ **`MULTIPLAYER`** $\rightarrow$ **`OPEN CO-OP PORTAL`** klicken.
2. Den angezeigten **`CODE: XXXX-XXXX-XXXX`** (Host-Code) an den Gast schicken.

### Schritt B — Beitreten starten (Gast)
1. **`ESC`** $\rightarrow$ **`MULTIPLAYER`** $\rightarrow$ **`JOIN CO-OP / SERVER`** klicken.
2. Den **Host-Code** einfügen (`PASTE (CTRL+V)`) und **`JOIN WORLD (ENTER)`** drücken.
3. Erscheint **`NAT LOCKED - TELL HOST: XXXX-XXXX-XXXX`**:
   - **Wichtig:** Das Join-Fenster **offen lassen** (nicht `ESC`, `CLEAR` oder `BACK` drücken!).
   - Den neuen **Gast-Rück-Code** (liegt bereits in der Zwischenablage) an den Gastgeber schicken.

### Schritt C — Gegen-Punch auslösen (Gastgeber)
1. **`ESC`** $\rightarrow$ **`MULTIPLAYER`** $\rightarrow$ **`PUNCH NAT / JOIN`** klicken.
2. Den **Gast-Rück-Code** einfügen (`PASTE (CTRL+V)`). Unten erscheint grün: `READY: PUNCH GUEST NAT - PRESS ENTER`.
3. **`PUNCH GUEST NAT (ENTER)`** klicken (bzw. `ENTER` drücken).
4. [`WormholeGate::punch_peer`](../crates/sanctuary/src/wormhole/pake.rs) sendet nun `8 s` lang UDP-Punch-Pakete (`LCPN`) an den Router des Gasts. Dadurch öffnet sich die Firewall des Gastgebers, und der Gast tritt automatisch der Welt bei.

---

## 3. Offene Punkte & Verbesserungsideen für später

Wenn wir später an diesem Thema weiterarbeiten, sind folgende UX- und Netzwerk-Punkte relevant:

- **Unklare In-Game-Anweisung:** Die einzeilige Meldung `NAT LOCKED - TELL HOST: ...` in [`PortalLayout`](../crates/client/src/menu/portal/layout.rs) erklärt nicht, *wo* der Gastgeber den Code eingeben muss (`MULTIPLAYER` $\rightarrow$ `PUNCH NAT / JOIN`) und dass der Gast das Menü **nicht schließen** darf.
- **Abbruch beim Schließen des Menüs:** Drückt der Gast während des Wartens `ESC` oder `BACK`, bricht [`EscapeMenu::toggle_escape`](../crates/client/src/menu/escape_menu/mod.rs) über `dialing_active() == false` den `PortalDial` sofort ab (`self.overlay.dial.cancel()` in [`SessionStage::step_frame`](../crates/client/src/session/stage.rs)). Beim nächsten Versuch bekommt der Socket einen neuen Port und der bisherige Rück-Code wird ungültig. *Idee:* Hintergrund-Dialing auch bei geschlossenem Menü weiterlaufen lassen, inkl. kleinem HUD-Hinweis.
- **Kurzes Initial-Timeout (`240 ms`):** Bei höherer Latenz oder während das asynchrone UPnP-Port-Mapping ([`GatewayLease::request_port_mapping`](../crates/sanctuary/src/nat/mod.rs)) auf dem Router des Gastgebers noch anläuft, springt der Client sehr schnell in `NAT LOCKED`, obwohl ein direkter Connect kurz darauf evtl. geklappt hätte.
- **Kein Unterscheiden zwischen „NAT blockiert“ und „Host offline“:** Da UDP verbindungslos ist, führt auch ein geschlossenes Spiel beim Gastgeber oder ein abgelaufener Code nach `240 ms` zunächst zur Meldung `NAT LOCKED - TELL HOST: ...`.
- **Symmetric NAT / DS-Lite / CGNAT (Mobilfunk, Kabelanschlüsse ohne öffentliche IPv4):** Falls einer der Router für jedes Ziel-IP:Port-Paar einen neuen externen Port vergibt (Symmetric NAT) oder gar keine öffentliche IPv4 bereitstellt, reicht STUN-basiertes UDP-Hole-Punching allein technisch nicht aus (hier wäre optional IPv6-Direktverbindung oder ein leichtgewichtiger Relay-Fallback nötig).

---

## 4. Beteiligte Dateien im Code

- [`crates/client/src/menu/portal/mod.rs`](../crates/client/src/menu/portal/mod.rs) — `PortalStudio` (Zwischenablage, Eingabe- & Punch-Logik)
- [`crates/client/src/menu/portal/layout.rs`](../crates/client/src/menu/portal/layout.rs) — `PortalLayout` (Buttons & Statuszeile `NAT LOCKED - TELL HOST`)
- [`crates/client/src/menu/portal/invite.rs`](../crates/client/src/menu/portal/invite.rs) — `PortalInvite` & `PortalNotice`
- [`crates/client/src/menu/escape_menu/mod.rs`](../crates/client/src/menu/escape_menu/mod.rs) — `EscapeMenu` (`dialing_active` & Abbruch bei `ESC`)
- [`crates/client/src/portal_config.rs`](../crates/client/src/portal_config.rs) — `PortalDial` (`240 ms` Direktversuch, `180 s` Punch-Schleife)
- [`crates/sanctuary/src/nat/mod.rs`](../crates/sanctuary/src/nat/mod.rs) — `NatAperture` (STUN-Abfrage & UPnP/NAT-PMP)
- [`crates/sanctuary/src/wormhole/phrase.rs`](../crates/sanctuary/src/wormhole/phrase.rs) — `PortalCode` & `PunchCode`
- [`crates/sanctuary/src/wormhole/pake.rs`](../crates/sanctuary/src/wormhole/pake.rs) — `WormholePledge` & `WormholeGate::punch_peer`
