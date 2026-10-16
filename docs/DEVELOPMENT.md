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
  # Fast 60-second day/night cycle for celestial testing
  cargo run -p client -- --day-cycle 60

  # Low gravity world with 10-minute cycle
  cargo run -p client -- --gravity 5 --day-cycle 600
  ```
  *See the [Server & Game Settings Guide](SETTINGS.md) for full configuration options.*

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
| `Esc` | Exit the game |

---

## Workspace Structure

- `crates/world`: Cubic planetary world, face orientations, gravity mechanics, biome voxel generation.
- `crates/engine`: OpenGL 3.3 forward pipeline, VBO/VAO batching, and embedded GLSL shaders.
- `crates/client`: Application lifecycle, `winit` event loop, `glutin` surface handling, input mapping, and camera projection.
