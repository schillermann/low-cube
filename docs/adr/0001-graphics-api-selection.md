# Architecture Decision Record: Graphics API Selection

- **Status**: Accepted
- **Date**: 2026-10-11
- **Deciders**: Core Development Team
- **Technical Context**: Game Engine for *Low-Cube*

---

## 1. Context and Problem Statement

*Low-Cube* is being developed from scratch as a custom voxel game engine targeting Linux and Windows. The baseline hardware target is a ThinkPad T480 equipped with an Intel UHD 620 integrated GPU and an NVIDIA GeForce MX150 dedicated GPU.

A fundamental technical decision is required regarding the graphics API: Should the custom engine use **Modern Core OpenGL (3.3 / 4.3)** or **Vulkan**?

---

## 2. Decision Drivers

1. **Target Hardware Profile**: Must run fluidly at 60 FPS on low-power mobile GPUs (Intel UHD 620).
2. **Development Velocity & Complexity**: As a custom engine built alongside the game, boilerplate and infrastructure complexity must be minimized so focus remains on core gameplay (voxel physics, 90° edge gravity, networking).
3. **Render Workload Characteristics**:
   - World dimensions are bounded to $256 \times 256 \times 256$ blocks per world.
   - The visual style is minimalist and non-textured (flat-shaded, vertex/palette-colored).
   - **Greedy Meshing** merges coplanar adjacent voxel faces into single quads, reducing draw calls to a modest number (estimated 50–200 draw calls per frame).
4. **Platform Scope**: Linux and Windows only (macOS is excluded, removing the requirement for Apple Metal / MoltenVK).
5. **Driver Maturity**: Predictable, robust driver behavior across Mesa (Linux Intel/Nouveau/Proprietary Nvidia) and Windows display drivers.

---

## 3. Considered Options

* **Option 1**: Modern Core OpenGL (Core Profile 3.3 or 4.3)
* **Option 2**: Vulkan (1.2 / 1.3)
* **Option 3**: Direct3D 11/12 (Rejected: Non-portable to Linux without Proton/Wine)
* **Option 4**: WebGPU / wgpu (Viable, but introduces an extra abstraction layer)

---

## 4. Evaluation & Comparison

| Criterion | OpenGL (Core 3.3 / 4.3) | Vulkan |
| :--- | :--- | :--- |
| **Boilerplate & Initialization** | **Low (~150–250 lines)** to initialize a context, window, and render the first triangle. | **Very High (~1,500–2,000 lines)** for Instance, Physical/Logical Device, Swapchain, RenderPass, Pipeline Layout, Semaphores, and Fences. |
| **Memory Management** | Automatic (managed by driver runtime and VRAM manager). | Explicit (manual heap allocations, sub-allocations, alignment, VMA dependency). |
| **Draw Call Overhead** | Higher driver CPU overhead per draw call if tens of thousands of draw calls are issued. | Near-zero CPU driver overhead; multi-threaded command buffer recording. |
| **Applicability to Low-Cube** | **Ideal fit**: Greedy Meshing produces so few draw calls that OpenGL CPU overhead is negligible. | Over-engineered for a $256^3$ flat-shaded voxel world; offers no measurable framerate gain for this workload. |
| **Debugging & Tooling** | Straightforward with `KHR_debug` and RenderDoc. | Excellent validation layers, but requires extensive synchronization debugging (hazards, barriers). |
| **Compute Shader Capabilities** | Supported in OpenGL 4.3 (useful for GPU-accelerated meshing if needed). | Full compute support across all versions. |

---

## 5. Decision Outcome

**Chosen Option: Modern Core OpenGL (Core Profile 3.3 / 4.3)**.

### Rationale:
1. **Low Draw-Call Footprint**: Because *Low-Cube* relies on non-textured flat shading and Greedy Meshing, the scene complexity will not bottleneck the CPU command submission of OpenGL.
2. **Maximum Development Velocity**: OpenGL allows the team to reach an interactive, playable voxel prototype within days rather than spending weeks writing low-level Vulkan plumbing.
3. **Rock-Solid Driver Support on Target Hardware**:
   - On Linux, the open-source **Mesa** drivers provide world-class, mature OpenGL implementations for Intel GPUs.
   - On Windows, Intel and NVIDIA drivers for the T480 hardware support OpenGL 3.3/4.3 flawlessly.
4. **macOS Omission Advantage**: Bypassing macOS completely eliminates the need to cater to Apple's deprecated OpenGL 4.1 limitations or write a proprietary Metal backend.

---

## 6. Consequences & Implementation Guidelines

### Positive Consequences:
- Rapid time-to-first-voxel and early gameplay testing.
- Lean codebase with minimal external dependencies (e.g., GLFW + GLAD / GLEW in C/C++, or `glow`/`gl` in Rust).
- Simple shader authoring pipeline using GLSL.

### Negative Consequences / Risks:
- Less fine-grained control over GPU synchronization and explicit multi-threaded rendering queues (deemed acceptable given the bounded world size).

### Architectural Rules:
- Enforce strict separation between the rendering subsystem and the game/physics simulation tick.
- Restrict OpenGL calls to a dedicated rendering module/thread to allow future backend additions if ever desired.
