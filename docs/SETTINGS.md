# Server & Game Settings Guide

This guide explains how players and server operators can customize world physics and gameplay settings for **Low-Cube**.

---

## 1. Overview

In *Low-Cube*, jump height and gravitational acceleration are **completely decoupled from the planetary world-cube dimensions**. Whether running a compact $128^3$ outpost or a massive $1024^3$ terrestrial world, players can customize their movement physics.

By default, every server runs with **Earth Gravity**:
- **Gravity**: $9.81\,\text{m/s}^2$
- **Jump Height**: $0.522\,\text{m}$ (initial vertical leap $v_0 \approx 3.20\,\text{m/s}$)

This default allows players to climb natural $50\,\text{cm}$ rock shelves and slabs without being able to leap directly over vertical 1-meter stone cliffs.

---

## 2. Configuration File: `server.toml` (Recommended)

Place a `server.toml` file in the same directory as the game executable (`low-cube-linux` or `low-cube-windows.exe`).

### Example `server.toml`

```toml
# Low-Cube Server & World Settings

[physics]
# Peak vertical jump height in meters (default: 0.522)
jump_height = 0.522

# Planetary gravitational acceleration in m/s^2 (default: 9.81)
gravity = 9.81
```

---

## 3. Alternative Configuration Methods

Settings can also be adjusted via command-line flags or environment variables without editing files.

### Evaluation Precedence
1. **Command-Line Arguments** (highest priority)
2. **Environment Variables**
3. **`server.toml` Configuration File**
4. **Earth Gravity Defaults** (lowest priority fallback)

### Command-Line Arguments
Pass flags when launching the game or server executable:

- **Linux**:
  ```bash
  # Custom jump height (1.0 meter)
  ./low-cube-linux --jump-height 1.0

  # Custom jump height and custom gravity
  ./low-cube-linux --jump-height 1.5 --gravity 9.81
  ```

- **Windows**:
  ```powershell
  # Custom jump height (1.0 meter)
  .\low-cube-windows.exe --jump-height 1.0

  # Custom jump height and custom gravity
  .\low-cube-windows.exe --jump-height 1.5 --gravity 9.81
  ```

### Environment Variables
Set environment variables before starting the executable:

- **Linux**:
  ```bash
  LOW_CUBE_JUMP_HEIGHT=1.0 LOW_CUBE_GRAVITY=9.81 ./low-cube-linux
  ```

- **Windows (PowerShell)**:
  ```powershell
  $env:LOW_CUBE_JUMP_HEIGHT = "1.0"
  $env:LOW_CUBE_GRAVITY = "9.81"
  .\low-cube-windows.exe
  ```

---

## 4. Popular Gameplay Presets

Here are recommended configuration values for different play styles:

| Preset | `jump_height` | `gravity` | Description |
|---|---|---|---|
| **Earth Baseline (Default)** | `0.522` | `9.81` | Authentic human leap; clears 50cm slabs, requires ramps for 1m cliffs. |
| **High Mobility** | `1.0` | `9.81` | Direct jumps onto full 1-meter blocks without requiring slabs or ramps. |
| **Lunar Low-Gravity** | `2.5` | `1.62` | Floaty, high-altitude jumps suitable for moonlet exploration. |
| **Hardcore Realism** | `0.35` | `9.81` | Restricted jump clearance; demands switchbacks and constructed ladders. |
| **Super-Jump Parkour** | `4.0` | `9.81` | Leaping across wide ravines and chasms. |
