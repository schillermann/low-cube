# Multiplayer & Subterranean Mining Guide

Play **Low-Cube** solo offline, open a spontaneous co-op portal for friends without touching router settings, or host a persistent dedicated World-Cube on the internet.

---

## 1. Quick Start: Playing Together

### Option A: Open a Co-op Portal In-Game (No Router Setup)
1. Start the game normally (`./low-cube-linux` or `cargo run -p client`).
2. Press **`Esc`** and click **`OPEN CO-OP PORTAL`** (or launch directly with `--host`).
3. Copy the **`LC1-...` Invite Ticket** printed in the console (and previewed on the Pause Menu).
4. Friends join immediately with:
   ```bash
   cargo run -p client -- --join LC1-...
   ```
   *Neither host nor guest needs to configure router port-forwarding when using a Relay Beacon (`--relay <host:port>` or `LOWCUBE_RELAY=<host:port>`).*

### Option B: Dedicated Public Internet Server (`sanctuary`)
Run a headless 24/7 World-Cube server that anyone with the address or ticket can join:
```bash
# Start authoritative World-Cube server on UDP 27777 (default: 64 blocks/edge)
cargo run -p sanctuary -- --port 27777 --blocks 64

# Players join by IP/domain or by Invite Ticket:
cargo run -p client -- --join play.example.com:27777
```

To operate a lightweight NAT Rendezvous & Encrypted Relay node for home hosts behind CGNAT/DS-Lite:
```bash
cargo run -p sanctuary -- --relay-only --port 27778
```

---

## 2. CLI & Environment Reference

| Command / Flag | Purpose |
|---|---|
| *(no flags)* | Offline Singleplayer (0 sockets, instant `Esc` pause). |
| `--host` | Start as an authoritative Co-op Host and print an `LC1-...` ticket. |
| `--port <udp>` | Bind a fixed UDP port when hosting (`--host --port 27777`). |
| `--join <LC1-... \| host:port>` | Connect to a friend's portal ticket or a public server (`LOWCUBE_JOIN`). |
| `--relay <host:port>` | Assist `--host` / Pause-Menu portals with NAT hole-punching & encrypted relay (`LOWCUBE_RELAY`). |

---

## 3. Subterranean Geology & Fair Mining

All six cube faces can be worked solo or cooperatively on the same face. Beneath the surface turf lies a stratified mantle above the indestructible molten core ([`CrustVault`](../crates/world/src/relief/crust_vault/mod.rs)):

| Ore | Geological Stratum | Weight Grade | Primary Crafting Uses (`C`) |
|---|---|---|---|
| **Copper Ore** | Upper-to-mid mantle (`2–4` layers below surface) | `WEIGHTY` | **Steering Wheel**, **Field Camera**, **Plane Engine** |
| **Iron Ore** | Mid-to-deep mantle (`5+` layers below surface) | `WEIGHTY` | **Plane Engine**, **Rocket Engine** |
| **Gold Ore** | Deep core boundary (`0–3` layers above `CrustVault`) | `HEAVY` | **Field Camera**, **Rocket Engine** |

### Fair-Play Protections
- **X-Ray Proof**: Hidden ores do not exist on client machines until an adjacent shaft wall or floor block is physically excavated.
- **Anti-Strip-Mining**: Mineral veins grow organically with finite yield budgets, and heavily strip-mined `16×16` sectors saturate—rewarding genuine exploration across mountains and planetary faces instead of blind grid-tunneling.
