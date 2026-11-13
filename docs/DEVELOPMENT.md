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

- **Local Multiplayer, Dedicated Sanctuary & Relay Development**:
  ```bash
  # Start headless authoritative World-Cube server on UDP 27777 (default: 128 blocks/edge, reads server.toml)
  cargo run -p sanctuary -- --port 27777 --size 1

  # Custom block resolution for fast integration testing
  cargo run -p sanctuary -- --port 27777 --blocks 64

  # Start lightweight NAT Rendezvous & Encrypted Relay node
  cargo run -p sanctuary -- --relay-only --port 27778

  # Launch client directly into co-op host or remote join mode
  cargo run -p client -- --host
  cargo run -p client -- --join 127.0.0.1:27777
  cargo run -p client -- --join LC1-...
  ```

---

## Running Tests

Run the test suite across all workspace crates (`crates/world`, `crates/engine`, `crates/client`):

```bash
cargo test
```

---

## Controls Reference

| Input | Action |
|---|---|
| `W`, `A`, `S`, `D` | Move character forward, left, backward, right |
| `Mouse` | Look / orient camera |
| `I` | Toggle inverted vertical mouse look (pitch) |
| `O` | Toggle inverted horizontal mouse look (yaw) |
| `Esc` | Open Main Menu (Character Studio, Keybindings, Co-op Portal & Join) |

---

## Workspace Structure

- `crates/world`: Cubic planetary world, face orientations, gravity mechanics, biome voxel generation, subterranean veins, and vehicles.
- `crates/engine`: OpenGL 3.3 forward pipeline, VBO/VAO batching, and embedded GLSL shaders.
- `crates/sanctuary`: Authoritative E2EE multiplayer World-Cube server (`SanctuaryHost`), zero-knowledge client decorator (`RemotePlanet`), `LC1-...` tickets, and NAT `RelayBeacon`.
- `crates/client`: Application lifecycle, `winit` event loop, `glutin` surface handling, input mapping, 3D avatar kinematics, and menus.
