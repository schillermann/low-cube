use glam::Vec3;
use world::{MachineKind, ScalableKind, SpaceModule};

/// Strict triangle ceiling (`< 150` triangles) per instanced `SpaceModule` archetype.
pub const MAX_ARCHETYPE_TRIANGLES: u32 = 150;

/// GPU vertex for low-poly hardware-instanced `[0..1]³` module archetypes (`3` private fields).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModuleVertex {
    pos_glow: [f32; 4],
    norm_pad: [f32; 4],
    color_pad: [f32; 4],
}

impl ModuleVertex {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pos_glow: [f32; 4], norm_pad: [f32; 4], color_pad: [f32; 4]) -> Self {
        Self { pos_glow, norm_pad, color_pad }
    }

    pub const fn shaded(pos: [f32; 3], norm: [f32; 3], color: Vec3, glow: f32) -> Self {
        Self::new(
            [pos[0], pos[1], pos[2], glow],
            [norm[0], norm[1], norm[2], 0.0],
            [color.x, color.y, color.z, 1.0],
        )
    }
}

/// Index range and base-vertex offset for one `SpaceModule` inside `ModuleArchetypeBank` (`3` private fields).
#[derive(Clone, Copy, Debug)]
pub struct ArchetypeSlice {
    first_index: u32,
    index_count: u32,
    vertex_offset: i32,
}

impl ArchetypeSlice {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(first_index: u32, index_count: u32, vertex_offset: i32) -> Self {
        Self { first_index, index_count, vertex_offset }
    }

    pub const fn first_index(self) -> u32 {
        self.first_index
    }

    pub const fn index_count(self) -> u32 {
        self.index_count
    }

    pub const fn vertex_offset(self) -> i32 {
        self.vertex_offset
    }

    pub const fn triangle_count(self) -> u32 {
        self.index_count / 3
    }
}

/// Shared unit `[0..1]³` low-poly mesh bank for all `11` Space-Age multi-block & scalable modules (`3` private fields).
pub struct ModuleArchetypeBank {
    vertices: Vec<ModuleVertex>,
    indices: Vec<u16>,
    slices: [ArchetypeSlice; 11],
}

impl ModuleArchetypeBank {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        vertices: Vec<ModuleVertex>,
        indices: Vec<u16>,
        slices: [ArchetypeSlice; 11],
    ) -> Self {
        Self { vertices, indices, slices }
    }

    /// Assembles all `11` `SpaceModule` unit `[0..1]³` low-poly archetypes (`< 150` triangles each).
    pub fn assembled() -> Self {
        let mut bank = Self::new(Vec::with_capacity(640), Vec::with_capacity(960), [ArchetypeSlice::new(0, 0, 0); 11]);
        for idx in 0..11u8 {
            let module = SpaceModule::from_code(idx + 1).expect("valid SpaceModule code 1..=11");
            let v_base = bank.vertices.len() as i32;
            let i_start = bank.indices.len() as u32;
            bank.populate_archetype(module);
            let i_count = (bank.indices.len() as u32) - i_start;
            let slice = ArchetypeSlice::new(i_start, i_count, v_base);
            assert!(
                slice.triangle_count() > 0 && slice.triangle_count() < MAX_ARCHETYPE_TRIANGLES,
                "Archetype {:?} exceeded < 150 triangle budget: {}",
                module,
                slice.triangle_count()
            );
            bank.slices[idx as usize] = slice;
        }
        bank
    }

    pub const fn slice_for(&self, module: SpaceModule) -> ArchetypeSlice {
        self.slices[(module.code() - 1) as usize]
    }

    pub fn vertex_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.vertices.as_ptr().cast::<u8>(),
                self.vertices.len() * core::mem::size_of::<ModuleVertex>(),
            )
        }
    }

    pub fn index_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.indices.as_ptr().cast::<u8>(),
                self.indices.len() * core::mem::size_of::<u16>(),
            )
        }
    }

    fn push_quad(&mut self, base_v: usize, p: [[f32; 3]; 4], norm: [f32; 3], col: Vec3, glow: f32) {
        let local = (self.vertices.len() - base_v) as u16;
        for corner in p {
            self.vertices.push(ModuleVertex::shaded(corner, norm, col, glow));
        }
        self.indices.extend_from_slice(&[local, local + 1, local + 2, local, local + 2, local + 3]);
    }

    fn push_box(&mut self, base_v: usize, min: [f32; 3], max: [f32; 3], side: Vec3, cap: Vec3, glow: f32) {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        self.push_quad(base_v, [[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]], [1.0, 0.0, 0.0], side, glow);
        self.push_quad(base_v, [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], [-1.0, 0.0, 0.0], side, glow);
        self.push_quad(base_v, [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]], [0.0, 1.0, 0.0], cap, glow);
        self.push_quad(base_v, [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], [0.0, -1.0, 0.0], side * 0.82, glow);
        self.push_quad(base_v, [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], [0.0, 0.0, 1.0], side, glow);
        self.push_quad(base_v, [[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]], [0.0, 0.0, -1.0], side, glow);
    }

    fn populate_archetype(&mut self, module: SpaceModule) {
        let base_v = self.vertices.len();
        let ti = Vec3::new(0.66, 0.72, 0.80);
        let cb = Vec3::new(0.20, 0.24, 0.30);
        let cu = Vec3::new(0.84, 0.48, 0.22);
        let ph = Vec3::new(0.18, 0.94, 0.68);
        let arc = Vec3::new(0.98, 0.56, 0.16);
        let cy = Vec3::new(0.24, 0.88, 0.98);
        match module {
            SpaceModule::Machine(MachineKind::ArcFurnace) => {
                self.push_box(base_v, [0.04, 0.0, 0.04], [0.96, 0.72, 0.96], cb, ti, 0.0);
                self.push_box(base_v, [0.16, 0.72, 0.16], [0.84, 1.0, 0.84], ti, cu, 0.0);
                self.push_quad(base_v, [[0.20, 0.22, 0.03], [0.20, 0.52, 0.03], [0.80, 0.52, 0.03], [0.80, 0.22, 0.03]], [0.0, 0.0, -1.0], arc, 1.2);
            }
            SpaceModule::Machine(MachineKind::AssemblyBench) => {
                self.push_box(base_v, [0.03, 0.0, 0.05], [0.97, 0.52, 0.95], cb, ti, 0.0);
                self.push_box(base_v, [0.08, 0.52, 0.14], [0.92, 1.0, 0.86], ti, cy, 0.25);
            }
            SpaceModule::Machine(MachineKind::ElectrolysisPlant) => {
                self.push_box(base_v, [0.04, 0.0, 0.04], [0.96, 0.18, 0.96], cb, ti, 0.0);
                self.push_box(base_v, [0.08, 0.18, 0.10], [0.46, 0.96, 0.90], cy, ti, 0.15);
                self.push_box(base_v, [0.54, 0.18, 0.10], [0.92, 0.96, 0.90], cu, ti, 0.0);
            }
            SpaceModule::Machine(MachineKind::AirlockChamber) => {
                self.push_box(base_v, [0.0, 0.0, 0.0], [1.0, 0.08, 1.0], cb, ti, 0.0);
                self.push_box(base_v, [0.0, 0.92, 0.0], [1.0, 1.0, 1.0], cb, ti, 0.0);
                self.push_box(base_v, [0.0, 0.08, 0.0], [0.12, 0.92, 1.0], ti, ti, 0.0);
                self.push_box(base_v, [0.88, 0.08, 0.0], [1.0, 0.92, 1.0], ti, ti, 0.0);
            }
            SpaceModule::Machine(MachineKind::PilotSeat) => {
                self.push_box(base_v, [0.14, 0.0, 0.14], [0.86, 0.34, 0.86], cb, ti, 0.0);
                self.push_box(base_v, [0.16, 0.34, 0.72], [0.84, 0.94, 0.92], cb, cu, 0.0);
            }
            SpaceModule::Machine(MachineKind::PhosphorConsole) => {
                self.push_box(base_v, [0.05, 0.0, 0.05], [0.95, 0.62, 0.95], cb, ti, 0.0);
                self.push_box(base_v, [0.10, 0.62, 0.12], [0.90, 1.0, 0.88], cb, ph, 0.95);
            }
            SpaceModule::Machine(MachineKind::DockingRing) => {
                self.push_box(base_v, [0.02, 0.0, 0.02], [0.98, 1.0, 0.18], ti, cu, 0.0);
                self.push_box(base_v, [0.02, 0.0, 0.82], [0.98, 1.0, 0.98], ti, cu, 0.0);
                self.push_box(base_v, [0.02, 0.0, 0.18], [0.18, 1.0, 0.82], ti, cu, 0.0);
                self.push_box(base_v, [0.82, 0.0, 0.18], [0.98, 1.0, 0.82], ti, cu, 0.0);
            }
            SpaceModule::Machine(MachineKind::CrewBerth) => {
                self.push_box(base_v, [0.04, 0.0, 0.04], [0.96, 0.36, 0.96], ti, Vec3::new(0.84, 0.88, 0.94), 0.0);
                self.push_box(base_v, [0.04, 0.36, 0.84], [0.96, 0.88, 0.98], cb, cy, 0.20);
            }
            SpaceModule::Scalable(ScalableKind::PropulsionBell) => {
                self.push_box(base_v, [0.22, 0.55, 0.22], [0.78, 1.0, 0.78], ti, cu, 0.0);
                self.push_box(base_v, [0.06, 0.0, 0.06], [0.94, 0.55, 0.94], cb, ti, 0.0);
            }
            SpaceModule::Scalable(ScalableKind::PropellantTank) => {
                self.push_box(base_v, [0.06, 0.05, 0.06], [0.94, 0.95, 0.94], Vec3::new(0.86, 0.89, 0.94), cu, 0.0);
                self.push_box(base_v, [0.03, 0.44, 0.03], [0.97, 0.56, 0.97], cu, cu, 0.0);
            }
            SpaceModule::Scalable(ScalableKind::ThermalFin) => {
                self.push_box(base_v, [0.03, 0.0, 0.36], [0.97, 1.0, 0.64], cb, arc, 0.25);
            }
        }
    }
}
