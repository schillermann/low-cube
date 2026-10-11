# Architecture Decision Record: Programming Language Selection (Rust)

- **Status**: Accepted
- **Date**: 2026-10-11
- **Deciders**: Core Development Team
- **Technical Context**: Core Engine, Client, and Dedicated Server for *Low-Cube*

---

## 1. Context and Problem Statement

*Low-Cube* is a custom voxel game and engine built from scratch. It features a $256 \times 256 \times 256$ cubic world topology, 90-degree continuous edge physics, a limited structural integrity model, modern Core OpenGL rendering, and an authoritative dedicated server architecture supporting up to 50 concurrent players per world.

The baseline hardware is a ThinkPad T480 (Intel UHD 620 integrated GPU / NVIDIA GeForce MX150), running on Linux and Windows. To maintain a rock-solid 60 FPS on low-power mobile hardware while sustaining heavy background tasks (voxel meshing, cellular automata fluids, network packet handling), a programming language must be chosen that satisfies stringent performance and safety criteria.

---

## 2. Decision Drivers

1. **Deterministic 60 FPS Performance (No GC Pauses)**: The baseline hardware operates within a 15W thermal envelope. Garbage collection pauses or background GC thread contention can induce micro-stutters and frame drops.
2. **Fearless Concurrency for Voxel Pipelines**: Chunk generation, Greedy Meshing, and fluid propagation must execute concurrently across multiple threads without risking data races or undefined behavior.
3. **Cross-Platform Parity (Linux & Windows)**: Seamless compilation, dependency management, and execution across Linux and Windows.
4. **OpenGL Interoperability**: Direct, zero-overhead Foreign Function Interface (FFI) to C libraries (OpenGL, GLFW).
5. **Unified Codebase**: Ability to share game simulation, physics, and networking code identically between client, local singleplayer loopback, and headless dedicated server instances.
6. **Alignment with Core Architectural Principles**:
   - Absence of null pointer exceptions (`Option<T>`).
   - Immutability by default (variables and references are immutable unless explicitly marked `mut`).
   - Strict encapsulation and trait-based polymorphism (interfaces everywhere, composition over inheritance).

---

## 3. Considered Options

* **Option 1: Rust**
* **Option 2: Modern C++ (C++20)**
* **Option 3: Zig**
* **Option 4: C# (.NET 8 / 9 with Silk.NET)**
* **Option 5: Java 21+ (LWJGL 3)**
* **Option 6: Go (Golang)**

---

## 4. Evaluation Summary

| Criterion | Rust | C++20 | Zig | C# / Java | Go |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Garbage Collector** | **None** | **None** | **None** | Present (GC pauses) | Present (GC pauses) |
| **Memory & Thread Safety** | **Compile-time enforced** | Manual / Undefined Behavior risk | Manual | Memory safe, race conditions possible | Memory safe, race conditions possible |
| **OpenGL FFI Overhead** | **Zero** (`glow`/`gl`) | **Zero** (Native) | **Zero** (Native C) | Low (P/Invoke, JNI) | Measurable (`cgo` context switch) |
| **Multi-threading (Rayon / Tokio)** | **Industry-leading** | OpenMP / std::thread (Manual sync) | Manual threads | Task / Loom | Goroutines |
| **Cross-Build (Linux ↔ Windows)** | **Trivial** (`cargo`) | Complex (CMake / MinGW) | **Trivial** (`zig build`) | Easy (.NET SDK) | **Trivial** (`GOOS`) |
| **Package Management** | **Cargo (First-class)** | Fragmented (vcpkg/Conan) | Emerging | NuGet / Maven | Go Modules |

---

## 5. Decision Outcome

**Chosen Option: Rust**.

### Rationale:
1. **Zero-Cost Abstractions & Zero GC**: Rust provides predictability down to the microsecond. The lack of a garbage collector guarantees that frame times remain stable at 16.6ms on the Intel UHD 620, even during intense chunk remeshing.
2. **Safe Multi-threaded Meshing**: With `rayon`, parallelizing Greedy Meshing and cellular automata updates across CPU cores is trivial and completely free of race conditions.
3. **Native OpenGL Tooling**: Mature crates such as `glow` (safe OpenGL bindings), `glutin` / `winit` (cross-platform windowing), and `glam` (SIMD-accelerated 3D mathematics) provide a lean, robust foundation.
4. **First-Class Server Stack**: The headless server can leverage `tokio` for asynchronous I/O and `bincode` / `serde` for compact binary network serialization, delivering peak network throughput for 50 players.
5. **Architectural Discipline**:
   - **No NULL**: Rust eliminates `null` entirely via `Option<T>`.
   - **Immutability by Default**: All bindings and data structures are immutable unless explicitly opted-in via `mut`.
   - **Traits & Composition**: Rust enforces interface-based design (`traits`) and composition over inheritance, fitting cleanly into modern pure object and entity modeling.

---

## 6. Implementation Guidelines

1. **Workspace Layout**: Utilize a Cargo Workspace:
   - `crates/engine_render`: OpenGL context, shader management, Greedy Meshing.
   - `crates/world`: Voxel storage ($256^3$), biomes, daylight cycle, structural stability checks.
   - `crates/physics`: 90-degree edge gravity, collision detection, continuous projectile & fluid paths.
   - `crates/net`: Network protocol, serialization, client/server message packets.
   - `crates/server`: Headless authoritative server application.
   - `crates/client`: Player controls, camera, audio, and game loop.
2. **Encapsulation & Safety**:
   - Forbid `unsafe` code outside of low-level OpenGL/driver FFI wrappers.
   - Enforce strict private struct fields with message-oriented public trait implementations.
