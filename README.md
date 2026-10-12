# Low-Cube

> *Survive. Build. Defy gravity across the faces of cubic worlds.*

![Low-Cube Gameplay](docs/images/gameplay.png)

**Low-Cube** is a voxel survival sandbox set on freestanding, cubic planetary worlds adrift in space. Step onto a world where gravity bends around 90-degree edges, each face hosts a unique climate, and survival demands wit, shelter, and resourcefulness.

---

## The World of Low-Cube

### Six Faces, Six Worlds
Every cube world features six distinct faces, each with its own climate, terrain, and hazards. From frozen tundras and scorched deserts to dense jungles and volcanic crags, each side presents unique survival challenges and raw resources.

### Defy Gravity
Walk right up to the edge of the world and step over. Watch the horizon tilt as gravity seamlessly pivots 90 degrees beneath your feet. Throw arrows, pour water, or drop boulders off the edge and see them curve around the planetary corner into the adjacent gravitational field.

### Outrun the Night
As the sun circles the planet, light only touches a few faces at a time while the rest are plunged into darkness. When the shadows fall, aggressive predators emerge from the caves and skies to hunt. If your shelter isn't ready, your best defense might be to grab your gear, sprint to the nearest edge, and leap into the dawn of a sunlit face.

### Realistic Building & Structural Integrity
Say goodbye to floating islands and gravity-defying dirt towers. Every block in *Low-Cube* relies on structural support. Build load-bearing pillars, sturdy walls, and reinforced roofs—push your architecture too far without a solid foundation, and your fortress will collapse under its own weight.

### Railroads Across the Edges
Tired of walking? Lay down rail networks that snake across mountain ranges and curve directly around 90-degree planetary edges. Hop into a minecart to transport mined diamonds from the depths of a dark face to your foundry on a sunny face in seconds. Build rovers and aerial craft to scout uncharted faces.

### Mysterious Ring Gates
Scattered across the stars are ancient Ring Gates. Step through the shimmering portal into a wormhole tunnel to discover entirely new cubic worlds—some with low gravity where you can leap across canyons, others with crushing gravity and untamed riches.

### Survive, Forge & Defend
- **Manage Your Vitals**: Keep your hunger, thirst, oxygen, and body temperature in balance as you brave freezing peaks, blistering dunes, and airless high altitudes.
- **Master the Forge**: Dig deep toward the indestructible planetary core to unearth copper, iron, gold, and diamonds. Forge stronger weapons, durable tools, and reinforced armor.
- **Secure Your Base**: Place light sources to keep monsters at bay, build secure lockable vaults to safeguard your hard-earned loot, and place a **Life Capsule** to anchor your respawn point.
- **Play Solo or with Friends**: Brave the worlds alone or team up with up to 50 players to build sprawling trans-planetary civilizations.

---

## Quick Play (Pre-built Binaries)

Ready-to-run executables for Linux and Windows are built automatically via GitHub Actions and published under [Releases](../../releases):

- **Linux**: Download `low-cube-linux`, mark as executable, and run:
  ```bash
  chmod +x low-cube-linux
  ./low-cube-linux
  ```
- **Windows**: Download `low-cube-windows.exe` and launch directly by double-clicking or from PowerShell / CMD:
  ```powershell
  .\low-cube-windows.exe
  ```

---

## Server & World Configuration

Jump height is completely independent of the world-cube size. By default, **Earth gravity** ($9.81\,\text{m/s}^2$) and an authentic human jump height of $\approx 0.52\,\text{m}$ (ideal for mounting $50\,\text{cm}$ rock shelves) are enabled.

Players hosting a local game or dedicated server can customize jump height and gravity in three ways:

### 1. Configuration File (`server.toml`) — Recommended
Place a `server.toml` file in the same folder as your executable:
```toml
[physics]
# Vertical jump height in meters (default: 0.522)
# Set to 1.0 to leap directly onto full 1-meter blocks
jump_height = 0.522

# Planetary gravity in m/s^2 (default: 9.81 for Earth gravitation)
gravity = 9.81
```

### 2. Launch Flags
Launch the executable from your terminal or command prompt:
- **Linux**:
  ```bash
  ./low-cube-linux --jump-height 1.0 --gravity 9.81
  ```
- **Windows**:
  ```powershell
  .\low-cube-windows.exe --jump-height 1.0 --gravity 9.81
  ```

### 3. Environment Variables
```bash
LOW_CUBE_JUMP_HEIGHT=1.0 LOW_CUBE_GRAVITY=9.81 ./low-cube-linux
```

For detailed options and presets, see the [Server & Game Settings Guide](docs/SETTINGS.md).

---

## Development

For environment setup, building, running, and testing instructions, see the [Developer Guide](docs/DEVELOPMENT.md).

---

## Documentation

- [Game Concept](CONCEPT.md)
- [Edge Traversal Roadmap](docs/ROADMAP.md)
- [Server & Game Settings Guide](docs/SETTINGS.md)
- [Developer Guide](docs/DEVELOPMENT.md)
- [Architecture Decision Records](docs/adr/README.md)

---

## License

This project is open source and licensed under the [MIT License](LICENSE).
