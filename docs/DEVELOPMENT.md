# Developer Guide

This guide explains how to set up the development environment, compile, run, and test **Low-Cube**.

---

## Prerequisites

### 1. Rust Toolchain
Low-Cube is written in Rust (2021 edition). Install the official toolchain via [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify your installation:
```bash
cargo --version
rustc --version
```

### 2. Platform Dependencies

#### Linux (X11 / Wayland & OpenGL)
The client relies on `glutin` and `winit` for window management and OpenGL context creation.

- **Ubuntu / Debian**:
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    libx11-dev libxcursor-dev libxrandr-dev libxi-dev \
    libxinerama-dev libgl1-mesa-dev libxkbcommon-dev pkg-config libwayland-dev
  ```

- **Fedora**:
  ```bash
  sudo dnf install libX11-devel libXcursor-devel libXrandr-devel libXi-devel \
    libXinerama-devel mesa-libGL-devel libxkbcommon-devel pkgconf-pkg-config wayland-devel
  ```

- **Arch Linux**:
  ```bash
  sudo pacman -S libx11 libxcursor libxrandr libxi libxinerama mesa libxkbcommon pkgconf wayland
  ```

#### Windows
Install the **Desktop development with C++** workload via the [Visual Studio Installer](https://visualstudio.microsoft.com/downloads/) to provide the MSVC linker and SDK.

---

## Running the Game

Launch directly from the repository root using `cargo`:

- **Debug Run** (faster compilation, lower frame rates):
  ```bash
  cargo run
  ```

- **Release Run** (optimized build, recommended for smooth 60 FPS):
  ```bash
  cargo run --release
  ```

- **Custom Day/Night Cycle & World Settings** (pass flags after `--`):
  ```bash
  # Fast 1-minute day/night cycle for celestial testing
  cargo run -p client -- --cycle 1

  # Low gravity world with double-sized World-Cube
  cargo run -p client -- --gravity 5 --size 2
  ```
  *See the [Server & Game Settings Guide](SETTINGS.md) for full configuration options.*

- **Local Multiplayer & Dedicated Sanctuary Development**:
  ```bash
  # Start headless authoritative World-Cube server on UDP 27777 (default: 128 blocks/edge, reads server.toml)
  cargo run -p sanctuary -- --port 27777 --size 1

  # Custom block resolution for fast integration testing
  cargo run -p sanctuary -- --port 27777 --blocks 64

  # Launch client directly into co-op host or remote P2P join mode
  cargo run -p client -- --host
  cargo run -p client -- --join K7M9-P2X4-R8W3
  cargo run -p client -- --join 127.0.0.1:27777
  cargo run -p client -- --join LC1-...
  ```

---

## Running Tests

Run the test suite across all workspace crates (`crates/world`, `crates/engine`, `crates/sanctuary`, `crates/acoustics`, `crates/fauna`, `crates/client`):

```bash
cargo test
```

### Flight Log (`Flugschreiber`) & Headless Simulation

Low-Cube ships a built-in, zero-allocation **Flight Log** (`crates/client/src/black_box/`) alongside a windowless **Headless Session** harness (`crates/client/src/session/headless/`) following Pure OOP (Elegant Objects):

1. **In-Game Toggle & Tilted Log-Sheet Overlay**:
   - Press **`Esc`** $\rightarrow$ **`LOG`** to switch recording **`ON`** (`LiveTape`, rolling 15-second / 1,800-tick ring buffer at 120 Hz with zero runtime heap allocations) or **`OFF`** (`SilentTape` Null Object, persisted in `blackbox.toml`).
   - Since opening **`Esc`** pauses physics ticks immediately, the buffer stays frozen at the exact moment of an anomaly. Clicking **`CREATE LOG REPORT`** writes `reports/incident_<millis>.toml`, copies the report path to the clipboard, closes the menu, and snaps a tilted **ruled Log-Report clipboard sheet** (`IncidentSheet`) onto the screen center with the same entrance/fade cadence as the field-camera Polaroid animation.
2. **Deterministic Replay & Windowless Agent Testing**:
   - Any `reports/incident_*.toml` dossier can be loaded in tests via `IncidentReplay::from_toml` to replay the recorded input and `dt` stream deterministically against `BiomedCubeWorld::barren_beveled`.
   - `HeadlessSession` + `HeadlessSurface` execute `SessionStage`, `SessionDisplay`, `EscapeMenu`, and localhost UDP `HostedPlanet`/`RemotePlanet` co-op without an X11/Wayland display or GPU (`cargo test --offline -p client --bin client black_box_tests`).
   - Use `BiomedCubeWorld::barren_beveled(blocks, margin)` for `< 1 ms` flora-free physics worlds and `PresenceProbe` (`avatar.report_presence(&mut probe)`) to inspect exact 3D foot positions without getters or matrix inversion.

---

## Controls Reference

| Input | Action |
|---|---|
| `W`, `A`, `S`, `D` | Move character forward, left, backward, right |
| `Mouse` | Look / orient camera |
| `I` | Toggle inverted vertical mouse look (pitch) |
| `O` | Toggle inverted horizontal mouse look (yaw) |
| `Esc` | Open Main Menu (Character Studio, Keybindings, Co-op Portal & Flight Blackbox) |

---

## Workspace Structure

- `crates/world`: Cubic planetary world, face orientations, gravity mechanics, biome voxel generation, subterranean veins, and vehicles.
- `crates/engine`: OpenGL 3.3 forward pipeline, VBO/VAO batching, and embedded GLSL shaders.
- `crates/sanctuary`: Authoritative P2P E2EE multiplayer World-Cube server (`SanctuaryHost`), zero-knowledge client decorator (`RemotePlanet`), 12-char SPAKE2 portal/punch codes (`PortalCode`, `PunchCode`), `LC1-...` tickets, and STUN/UPnP NAT traversal (`NatAperture`).
- `crates/client`: Application lifecycle, `winit` event loop, `glutin` surface handling, input mapping, 3D avatar kinematics, and menus.
