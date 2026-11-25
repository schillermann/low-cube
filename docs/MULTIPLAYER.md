# Multiplayer & Subterranean Mining Guide

Play **Low-Cube** solo offline, open a spontaneous co-op portal for friends directly from the in-game menu without touching router settings, or run a persistent 24/7 dedicated server.

---

## 1. Playing Together (One Game File — No Terminal Needed)

For normal solo and co-op sessions, every player only needs the game executable (`low-cube-windows.exe` or `low-cube-linux`).

### Host a Co-op World
1. Launch `low-cube-windows.exe` (or `./low-cube-linux`).
2. Press **`Esc`** to open the Main Menu and click **`OPEN CO-OP PORTAL`**.
3. Share the **12-Character Portal Code** shown in the menu (for example **`CODE: K7M9-P2X4-R8W3`**), or paste (`Ctrl+V`) the automatically copied **`LC1-...` Invite Ticket** in chat.

### Join a Friend's World or Public Server
1. Launch `low-cube-windows.exe` (or `./low-cube-linux`).
2. Press **`Esc`** and click **`JOIN CO-OP / SERVER`**.
3. Type the **12-Character Portal Code** (`K7M9-P2X4-R8W3` — hyphens and uppercase are optional), click **`PASTE (CTRL+V)`** for an `LC1-...` ticket, or enter a server address such as `play.example.com` or `192.168.1.50:27777`.
4. Press **`Enter`** (`JOIN WORLD`). On Wi-Fi/LAN or routers with UPnP/NAT-PMP enabled, you land immediately!
5. **Closed Routers (e.g. FRITZ!Box with UPnP disabled — Zero-Server Ping-Pong Punch)**: If the host's router blocks unsolicited packets, your screen displays **`NAT LOCKED - TELL HOST: XXXX-XXXX-XXXX`**. Once the host enters that 12-character reply code under **`PUNCH NAT / JOIN`**, both routers punch a direct P2P UDP tunnel without any external relay server!

---

## 2. Hosting a 24/7 Dedicated Server (`low-cube-server`)

If you want a permanent world running on a home server or headless Linux VPS (no GPU or display required), download the standalone server executable from [Releases](../../releases):

- **Windows**: Double-click `low-cube-server-windows.exe`.
- **Linux**:
  ```bash
  chmod +x low-cube-server-linux
  ./low-cube-server-linux
  ```

By default, the server starts a `128 × 128 × 128` World-Cube (`size = 1`) on UDP port **`27777`** and prints both its direct address and an `LC1-...` invite ticket.

### Customizing via `server.toml` (No Command-Line Flags Required)
Place an optional `server.toml` file in the same folder as `low-cube-server-windows.exe` / `low-cube-server-linux` and restart the server:

```toml
[server]
port = 27777

[storage]
journal = "sanctuary.wal"   # Persistent SSD write-ahead chronicle + <journal>.survivors vault

[world]
size = 1                    # 1..=32 (1 = 128^3 blocks, 8 = 1024^3, 32 = 4096^3 = 100.7 km^2 Earth)
cycle = 20

[cosmos]
asteroid_density = 1.0      # Mars-Jupiter Asteroid Belt (2.20..3.20 AU) density (0.0..=8.0)
```

When `journal` is configured, the server automatically persists both terrain/fleet changes (via raycast-free `SectorSnapshot` compaction) and each player's inventory, equipped palms, and Hill-bubble logout position (keyed by their X25519 identity in `<journal>.survivors`). Uninhabited off-world moons and asteroids fold into cold `SectorSnapshot`s after `180 s` idle so server RAM stays flat across the entire solar system.

See the [Server & Game Settings Guide](SETTINGS.md) for all world, gravity, and aurora options.

---

## 3. Optional Launch Shortcuts (Advanced)

Players and server operators who prefer desktop shortcuts or startup scripts can also pass optional flags to the executables:

| Executable & Flag | Purpose |
|---|---|
| *(no flags)* | Launch offline singleplayer (`low-cube`) or default UDP `27777` server (`low-cube-server`). |
| `--host` | Automatically open a co-op portal on game launch and copy the `LC1-...` ticket. |
| `--port <udp>` | Bind a specific UDP port (`--host --port 27777` or on `low-cube-server`). |
| `--join <XXXX-XXXX-XXXX \| LC1-... \| host[:port]>` | Join a friend's 12-char code, ticket, or server on startup (`LOWCUBE_JOIN`). |

*(Compiling and launching from Rust source via `cargo` is documented in the [Developer Guide](DEVELOPMENT.md).)*

---

## 4. Subterranean Geology & Fair Mining

All six cube faces can be worked solo or cooperatively on the same face. Beneath the surface turf lies a stratified mantle above the indestructible molten core:

| Ore | Geological Stratum | Weight Grade | Primary Crafting Uses (`C`) |
|---|---|---|---|
| **Copper Ore** | Upper-to-mid mantle (`2–4` layers below surface) | `WEIGHTY` | **Steering Wheel**, **Field Camera**, **Rocket Camera**, **Flight Console**, **Blackbox Pod**, **Laser Beacon** |
| **Iron Ore** | Mid-to-deep mantle (`5+` layers below surface) | `WEIGHTY` | **Plane Engine**, **Rocket Nozzle**, **Fusion Torch**, **Blackbox Pod** |
| **Gold Ore** | Deep core boundary (`0–3` layers above core vault) | `HEAVY` | **Field Camera**, **Flight Console**, **Rocket Nozzle**, **Fusion Torch**, **Rocket Camera**, **Laser Beacon** |

### Fair-Play Protections & Hard-SciFi Space Telemetry
- **X-Ray Proof & Horizon Isolation**: Hidden ores do not exist on client machines until an adjacent shaft wall or floor block is physically excavated, and terrain/companion packets are strictly isolated to each player's `160 m` local horizon (`0 Hz` across other `ClimateZone`s and planets).
- **Anti-Strip-Mining**: Mineral veins grow organically with finite yield budgets, and heavily strip-mined `16×16` sectors saturate—rewarding genuine exploration across mountains, moons, and belt asteroids.
- **Physical Voice, Cabin Intercom & Visor-Touch Chat (`Enter`)**: Acoustic voice carries `96 m` in atmospheres (`Earth`, `Venus`, `Titan`), short-range suit radio carries `32 m` in vacuum, multi-crew passengers aboard the same spacecraft share a full-cabin intercom at any speed, and prefixing a message with `~` (`~follow me`) restricts transmission to `1.85 m` helmet-touch conduction with zero radio leakage.
- **Telescope Drive Stars & Orbital Laser Beacons**: Distant firing `FusionTorch` ships (`> 2.4 km`) appear as cyan-white drive stars in vacuum or when zooming into the night sky with a `6×/24×` hull camera (`F`), and placed `LaserBeacon` blocks shine up into orbit strictly above their local `ClimateZone` horizon plane (`0` extra wire bytes on the ground).
