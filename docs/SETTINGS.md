# Server & Game Settings Guide

This guide explains how players and server operators can customize world physics and server settings for **Low-Cube**.

---

## 1. Overview

In *Low-Cube*, world physics is kept deliberately simple and intuitive: **only planetary `gravity` is configured**.

Players and server operators never need to solve math problems or calculate decimal numbers. The player's jump height is automatically and naturally derived from the world's gravity:
- **Constant Jump Impulse**: The human character always exerts the same muscular jump impulse.
- **Natural Scaling**: Lower gravity automatically lets players leap higher and stay airborne longer; higher gravity pulls players down quickly with short, heavy hops.

By default, every server runs with **Earth Gravity (`10`)**:
- Allows vaulting onto natural $50\,\text{cm}$ rock shelves and slabs.
- Full 1-meter cliffs require constructed steps, ramps, or lower gravity.

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
```

---

## 3. Alternative Configuration Methods

Settings can also be adjusted via command-line flags or environment variables without editing files.

### Evaluation Precedence
1. **Command-Line Arguments** (highest priority)
2. **Environment Variables**
3. **`server.toml` Configuration File**
4. **Earth Gravity Baseline (`10`)** (fallback)

### Command-Line Arguments
Pass the `--gravity` flag when launching the executable:

- **Linux**:
  ```bash
  # Standard Earth gravity
  ./low-cube-linux --gravity 10

  # Low gravity world (high jumps onto 1m blocks)
  ./low-cube-linux --gravity 5
  ```

- **Windows**:
  ```powershell
  # Standard Earth gravity
  .\low-cube-windows.exe --gravity 10

  # Low gravity world (high jumps onto 1m blocks)
  .\low-cube-windows.exe --gravity 5
  ```

### Environment Variables
Set the `LOW_CUBE_GRAVITY` environment variable before starting the executable:

- **Linux**:
  ```bash
  LOW_CUBE_GRAVITY=5 ./low-cube-linux
  ```

- **Windows (PowerShell)**:
  ```powershell
  $env:LOW_CUBE_GRAVITY = "5"
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
