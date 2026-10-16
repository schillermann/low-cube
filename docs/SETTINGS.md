# Server & Game Settings Guide

This guide explains how players and server operators can customize world physics, day/night cycles, and nocturnal atmospheric phenomena for **Low-Cube**.

---

## 1. Overview

In *Low-Cube*, world parameters are kept deliberately simple and intuitive:
- **Planetary Gravity (`gravity`)**: Governs jump height and airborne trajectory.
- **Day/Night Cycle Duration (`cycle`)**: Sets the length of a complete planetary day and night rotation in minutes (default: `20` minutes; plain number in minutes, set to `0` for permanent noon).
- **Polar Lights Schedule (`[aurora]`)**: Governs the nocturnal appearance of waving green and violet aurora curtains over the Arctic face:
  - `nightly`: Appears every night throughout nocturnal hours (default baseline).
  - `off`: Permanently disabled (starry night sky remains fully active without auroras).
  - `rare`: Appears rarely (approx. 1 in 5 nights / 20% chance).
  - `normal`: Appears regularly (approx. every 2nd night / 50% chance).
  - `frequent`: Appears frequently (approx. 4 in 5 nights / 80% chance).

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
# Duration of a full planetary day/night rotation in minutes.
# Default: 20 (20 minutes: 10 min daylight, 7 min night, 3 min twilight transitions).
# Set to 0 to freeze the sun at permanent noon.
cycle = 20

[aurora]
# Nocturnal polar lights over Arctic.
# Supported modes: "nightly", "off", "rare", "normal", "frequent"
mode = "nightly"
```

---

## 3. Alternative Configuration Methods

Settings can also be adjusted via command-line flags or environment variables without editing files.

### Evaluation Precedence
1. **Command-Line Arguments** (highest priority)
2. **Environment Variables**
3. **`server.toml` Configuration File**
4. **Baselines** (fallback: `gravity = 10`, `cycle = 20`, `aurora = "nightly"`)

### Command-Line Arguments
Pass configuration flags when launching the executable:

| Flag | Format | Default | Description |
| :--- | :--- | :--- | :--- |
| `--cycle` | `--cycle <minutes>` | `20` | Duration of a complete day/night cycle in minutes. Set to `0` to freeze the sun at permanent noon. |
| `--gravity` | `--gravity <value>` | `10` | Planetary surface gravity. Lower values increase jump height; higher values reduce jump height. |
| `--aurora` | `--aurora <mode>`<br>`--aurora=<mode>` | `nightly` | Aurora schedule: `nightly`, `off`, `rare`, `normal`, or `frequent`. |

- **Linux**:
  ```bash
  # Standard Earth gravity and 20-minute day cycle
  ./low-cube-linux --gravity 10 --cycle 20

  # Quick 5-minute day cycle
  ./low-cube-linux --cycle 5

  # Aurora disabled
  ./low-cube-linux --aurora off

  # Rare polar lights
  ./low-cube-linux --aurora rare

  # Frequent polar lights
  ./low-cube-linux --aurora frequent
  ```

- **Windows**:
  ```powershell
  # Standard Earth gravity and 20-minute day cycle
  .\low-cube-windows.exe --gravity 10 --cycle 20

  # Quick 5-minute day cycle
  .\low-cube-windows.exe --cycle 5

  # Aurora disabled
  .\low-cube-windows.exe --aurora off

  # Rare polar lights
  .\low-cube-windows.exe --aurora rare

  # Frequent polar lights
  .\low-cube-windows.exe --aurora frequent
  ```

### Environment Variables
Set configuration environment variables before starting the executable:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `LOW_CUBE_GRAVITY` | `10` | Planetary surface gravity. |
| `LOW_CUBE_CYCLE` | `20` | Duration of full day/night cycle in minutes. |
| `LOW_CUBE_AURORA` | `nightly` | Aurora schedule (`nightly`, `off`, `rare`, `normal`, `frequent`). |

- **Linux**:
  ```bash
  LOW_CUBE_AURORA=rare ./low-cube-linux
  LOW_CUBE_AURORA=frequent ./low-cube-linux
  LOW_CUBE_AURORA=off ./low-cube-linux
  ```

- **Windows (PowerShell)**:
  ```powershell
  $env:LOW_CUBE_AURORA = "rare"
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

---

## 5. Arctic Aurora Modes

| Mode | Frequency | Occurrence | Description |
| :--- | :--- | :--- | :--- |
| `nightly` | Every night | $100\%$ | Active throughout nocturnal hours ($0.50 \dots 1.00$). Intensity smoothly follows the night factor. Stars and lunar features render alongside the polar ribbons. |
| `off` | Disabled | $0\%$ | Aurora curtains are completely dormant. Stars and the moon remain visible at full brilliance. |
| `rare`<br>(*selten*) | Seldom | $\approx 20\%$ ($1\text{ in }5\text{ nights}$) | Polar lights appear rarely on select nights. |
| `normal` | Regular | $\approx 50\%$ ($\text{every }2\text{nd night}$) | Balanced natural frequency. |
| `frequent`<br>(*häufig*) | Often | $\approx 80\%$ ($4\text{ in }5\text{ nights}$) | Polar lights appear on most nights. |
