# ADR 0004: Zero-Knowledge Sanctuary Multiplayer & Stratified Vein Architecture

- **Status**: Accepted
- **Date**: 2026-11-13
- **Crates**: `crates/world`, `crates/sanctuary`, `crates/client`

---

## 1. Context & Problem Statement

Internet multiplayer on a shared 6-sided World-Cube must satisfy four strict constraints simultaneously:
1. **Defeat Custom X-Ray Clients**: Modified clients inspecting RAM or network traffic must gain zero information about unexposed subterranean resources.
2. **Curb Strip-Mining ("Raubbau") & Bot Exploits**: Neither deterministic seed cracking nor packet-flooding / zig-zag tunnel farming may yield infinite ore.
3. **Zero Duplicate Game Logic**: Offline Singleplayer, Living-Room Co-op Hosts, and Headless Internet Servers must execute the exact same `PlanetaryWorld` domain logic without forcing local sockets onto offline players.
4. **Zero Router Configuration**: Home players behind NAT, CGNAT, or DS-Lite must connect without manual router port-forwarding.

---

## 2. Architectural Design

### 2.1 Decorator Symmetry (`PlanetaryWorld`)
All three runtime topologies program exclusively against [`PlanetaryWorld`](../../crates/world/src/lib.rs):
- **Solitary Singleplayer**: `SessionStage` owns `BiomedCubeWorld::beveled` directly (`0` sockets, `0` extra threads, `0` heap allocations inside `Avatar::step_motion`).
- **Authoritative Host (`crates/sanctuary`)**: [`HostedPlanet`](../../crates/sanctuary/src/hosted_planet.rs) decorates [`SanctuaryHost`](../../crates/sanctuary/src/sanctuary_host.rs), which wraps any `Box<dyn PlanetaryWorld>`—allowing live mid-session upgrades from Singleplayer via `Esc -> OPEN CO-OP PORTAL` as well as headless execution in the `sanctuary` binary.
- **Remote Guest**: [`RemotePlanet`](../../crates/sanctuary/src/remote_planet.rs) decorates a zero-knowledge `BiomedCubeWorld::hollow_beveled` together with [`RemoteLink`](../../crates/sanctuary/src/remote_link.rs).

### 2.2 Zero-Knowledge Geology & Air-Exposure Culling
Subterranean geology is decoupled behind [`SubterraneanVeins`](../../crates/world/src/relief/mineral_vein/mod.rs):
- **Authoritative [`LivingVeins`](../../crates/world/src/relief/mineral_vein/living.rs)**: Evaluates cells lazily using 128-bit keyed SipRound entropy ([`EntropyWell`](../../crates/world/src/relief/mineral_vein/entropy.rs)) only when excavation exposes an adjacent air face (`(u, v, new_elev - 1)` floor and newly uncovered `±1` pit walls).
  - **Height Stratigraphy**: `CopperOre` spawns in upper-mid layers, `IronOre` in deeper mantle layers, and `GoldOre` right above `CrustVault`.
  - **Finite Markov Vein Buds**: Discovered veins propagate a conserved `VeinBud` yield budget across matching strata, preventing zig-zag tunnel exploits.
  - **Per-Band Sector Saturation**: Each `16×16` sector tracks discoveries per ore band, steeply lowering sprout odds under concentrated strip-mining.
- **Client [`HollowVeins`](../../crates/world/src/relief/mineral_vein/mod.rs)**: Contains an empty map until the server transmits [`WorldFact::StratumDisclosed`](../../crates/world/src/chronicle.rs) for air-adjacent pit cells.

### 2.3 Serverless Pure-Rust E2EE Transport, SPAKE2 Codes & NAT Hole-Punching
- **Authenticated Encryption**: [`HandshakeIdentity`](../../crates/sanctuary/src/seal.rs) performs X25519 ECDH pinned to the host public key embedded in [`PortalTicket`](../../crates/sanctuary/src/ticket.rs) (`LC1-...`) or negotiated via RFC 9382 SPAKE2 Ristretto255 over 12-character [`PortalCode`](../../crates/sanctuary/src/wormhole/phrase.rs) invites. [`SessionSeal`](../../crates/sanctuary/src/seal.rs) encrypts every datagram with ChaCha20-Poly1305 AEAD and a 64-packet sliding anti-replay window.
- **Serverless NAT Traversal**: [`NatAperture`](../../crates/sanctuary/src/nat/mod.rs) discovers external endpoints via RFC 5389 STUN ([`StunProbe`](../../crates/sanctuary/src/nat/stun.rs)) plus local UPnP IGD / NAT-PMP ([`GatewayLease`](../../crates/sanctuary/src/nat/router.rs)), and [`PunchCode`](../../crates/sanctuary/src/wormhole/phrase.rs) enables mutual Ping-Pong UDP hole-punching across closed port-restricted routers (such as factory-default FRITZ!Box gateways) with zero external relay servers.

### 2.4 Authoritative Anti-Cheat & Loss-Tolerant Chronicle Sync
- **Anti-Cheat ([`GuestWard`](../../crates/sanctuary/src/ward.rs))**: Enforces body-to-eye reach (`<= 5.2m`), server-side `SurfaceSightRay` line-of-sight occlusion, token-bucket harvest rate-limiting (`5/s`, burst `2`), and earned-block credit accounting before accepting `MoldPlaced`.
- **Ordered UDP Synchronization ([`ChronicleLog`](../../crates/sanctuary/src/chronicle_log.rs))**: Durable [`WorldFact`](../../crates/world/src/chronicle.rs) mutations receive monotonic sequence numbers and stream in MTU-safe batches (`<= 24` facts) based on each guest's `ack_seq`, providing both packet-loss recovery and full late-joiner history replay. `RemotePlanet` deduplicates locally predicted petitions against incoming `HostChronicle` echoes so local actions feel instantaneous (`0ms`) without double-excavating layers.
