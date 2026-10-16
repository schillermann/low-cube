# Server & Game Settings Guide

This guide explains how players and server operators can customize world physics and server settings for **Low-Cube**.

---

## 1. Overview

In *Low-Cube*, world parameters are kept deliberately simple and intuitive:
- **Planetary Gravity (`gravity`)**: Governs jump height and airborne trajectory.
- **Day/Night Cycle Duration (`day_cycle_seconds`)**: Sets the length of a complete planetary day and night rotation (default: `1200` seconds / 20 minutes).

### The Principle of Static Configuration & Server Restart

All server configurations follow the **Principle of Static Configuration on Startup**:
- **Mandatory Restart**: Any modification to `server.toml`, command-line flags, or environment variables **strictly requires a server restart** to take effect.
- **Why Restart is Required**:
  1. **Strict Immutability & Zero Runtime Overhead**: Parameters are read once at startup and encapsulated into immutable domain objects. This eliminates locks (`Mutex`/`RwLock`), cache misses, and continuous disk I/O in the hot simulation and rendering loops.
  2. **Multiplayer Determinism & Desync Prevention**: Clients receive fixed session parameters upon joining. Mid-game modifications would cause visual glitches, sudden lighting teleportation, or client-server desynchronization.
  3. **Fail-Fast Safety**: Configuration errors, invalid numbers, or malformed syntax are intercepted immediately at boot time, preventing corrupt state or unexpected crashes during active gameplay.

---

## 2. Configuration File: `server.toml` (Recommended)

Place a `server.toml` file in the same directory as the game executable (`low-cube-linux` or `low-cube-windows.exe`).

### Example `server.toml`

```toml
# Low-Cube Server Configuration

[physics]
# Planetary gravity (default: 10 for Earth gravity)
# Lower values increase jump height; higher values reduce jump height.
gravity = 10

[world]
# Duration of a full planetary day/night rotation in seconds.
# Default: 1200 (20 minutes: 10 min daylight, 7 min night, 3 min twilight transitions).
# Set to 0 to freeze the sun at permanent noon.
day_cycle_seconds = 1200
```

---

## 3. Alternative Configuration Methods

Settings can also be adjusted via command-line flags or environment variables without editing files.

### Evaluation Precedence
1. **Command-Line Arguments** (highest priority)
2. **Environment Variables**
3. **`server.toml` Configuration File**
4. **Terrestrial Baselines** (fallback: `gravity = 10`, `day_cycle_seconds = 1200`)


### Command-Line Arguments
Pass configuration flags when launching the executable:

| Flag | Format | Default | Description |
| :--- | :--- | :--- | :--- |
| `--day-cycle`<br>`--day-cycle-seconds` | `--day-cycle <seconds>`<br>`--day-cycle-seconds=<seconds>` | `1200` (20 min) | Duration of a complete day/night cycle in seconds. Set to `0` to freeze the sun at permanent noon. |
| `--gravity` | `--gravity <value>` | `10` | Planetary surface gravity. Lower values increase jump height; higher values reduce jump height. |

- **Linux**:
  ```bash
  # Standard Earth gravity and 20-minute day cycle
  ./low-cube-linux --gravity 10 --day-cycle 1200

  # Low gravity world with fast 10-minute day cycle
  ./low-cube-linux --gravity 5 --day-cycle 600

  # Permanent daylight (frozen sun at noon)
  ./low-cube-linux --day-cycle 0
  ```

- **Windows**:
  ```powershell
  # Standard Earth gravity and 20-minute day cycle
  .\low-cube-windows.exe --gravity 10 --day-cycle 1200

  # Low gravity world with fast 10-minute day cycle
  .\low-cube-windows.exe --gravity 5 --day-cycle 600

  # Permanent daylight (frozen sun at noon)
  .\low-cube-windows.exe --day-cycle 0
  ```

### Environment Variables
Set configuration environment variables before starting the executable:

- **Linux**:
  ```bash
  LOW_CUBE_GRAVITY=5 LOW_CUBE_DAY_CYCLE=600 ./low-cube-linux
  ```

- **Windows (PowerShell)**:
  ```powershell
  $env:LOW_CUBE_GRAVITY = "5"
  $env:LOW_CUBE_DAY_CYCLE = "600"
  .\low-cube-windows.exe
  ```

---

## 4. Planetary Presets

No decimals required—simply pick an integer that matches your desired world experience:

| Preset | `gravity` | Derived Jump Height | Description |
|---|---|---|---|
| **Earth Standard (Default)** | `10` | $\approx 0.5\,\text{m}$ | Authentic human leap; clears 50cm slabs, requires ramps for 1m cliffs. |
| **High Mobility** | `5` | $\approx 1.0\,\text{m}$ | Direct leaps onto full 1-meter blocks without requiring ramps. |
| **Lunar Low-G** | `2` | $\approx 2.5\,\text{m}$ | High, floaty jumps suitable for crater and canyon exploration. |
| **Asteroid / Microgravity** | `1` | $\approx 5.0\,\text{m}$ | Extreme low gravity; soaring leaps into the sky. |
| **Heavy World** | `20` | $\approx 0.25\,\text{m}$ | Heavy steps; climbing even low ledges demands stairs and ladders. |
