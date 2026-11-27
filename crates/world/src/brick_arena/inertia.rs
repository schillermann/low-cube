use glam::{DMat3, DVec3};
use crate::block::GridCell;
use super::atom::RawAtom;
use super::brick::BrickCell;
use super::coord::CellCoord;
use super::module_spec::ModuleSpec;

/// Symmetric $3\times 3$ rotational inertia tensor (`kg * m^2`) stored as diagonal + off-diagonal components (`2` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SymmetricInertia {
    diag: DVec3,
    off_diag: DVec3,
}

impl SymmetricInertia {
    /// Primary constructor doing nothing except assigning arguments to fields (`off_diag = (I_xy, I_xz, I_yz)`).
    pub const fn new(diag: DVec3, off_diag: DVec3) -> Self {
        Self { diag, off_diag }
    }

    pub const fn zero() -> Self {
        Self::new(DVec3::ZERO, DVec3::ZERO)
    }

    /// Origin-referenced Steiner contribution of a uniform `0.5 m` cube of `mass_kg` centered at `p = (x, y, z)`.
    pub fn cube_about_origin(p: DVec3, mass_kg: f64) -> Self {
        let s = f64::from(GridCell::SPAN);
        let intrinsic = (1.0 / 6.0) * mass_kg * s * s;
        let ixx = intrinsic + mass_kg * (p.y * p.y + p.z * p.z);
        let iyy = intrinsic + mass_kg * (p.x * p.x + p.z * p.z);
        let izz = intrinsic + mass_kg * (p.x * p.x + p.y * p.y);
        let ixy = -mass_kg * p.x * p.y;
        let ixz = -mass_kg * p.x * p.z;
        let iyz = -mass_kg * p.y * p.z;
        Self::new(DVec3::new(ixx, iyy, izz), DVec3::new(ixy, ixz, iyz))
    }

    pub fn plus(self, rhs: Self) -> Self {
        Self::new(self.diag + rhs.diag, self.off_diag + rhs.off_diag)
    }

    pub fn minus(self, rhs: Self) -> Self {
        Self::new(self.diag - rhs.diag, self.off_diag - rhs.off_diag)
    }

    /// Reverse Parallel-Axis Theorem (Satz von Steiner) shifting $\mathbf{I}_0 \to \mathbf{I}_{\text{CoM}}$ in `O(1)`.
    pub fn shifted_to_com(self, total_mass: f64, com: DVec3) -> Self {
        if total_mass <= 1e-9 {
            return Self::zero();
        }
        let steiner_diag = DVec3::new(
            total_mass * (com.y * com.y + com.z * com.z),
            total_mass * (com.x * com.x + com.z * com.z),
            total_mass * (com.x * com.x + com.y * com.y),
        );
        let steiner_off = DVec3::new(
            -total_mass * com.x * com.y,
            -total_mass * com.x * com.z,
            -total_mass * com.y * com.z,
        );
        Self::new(self.diag - steiner_diag, self.off_diag - steiner_off)
    }

    pub fn to_mat3(self) -> DMat3 {
        let (ixx, iyy, izz) = (self.diag.x, self.diag.y, self.diag.z);
        let (ixy, ixz, iyz) = (self.off_diag.x, self.off_diag.y, self.off_diag.z);
        DMat3::from_cols(
            DVec3::new(ixx, ixy, ixz),
            DVec3::new(ixy, iyy, iyz),
            DVec3::new(ixz, iyz, izz),
        )
    }
}

/// Incremental `O(1)` Parallel-Axis Theorem (Satz von Steiner) mass, CoM, and $3\times 3$ inertia accumulator (`3` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteinerMass {
    mass_kg: f64,
    first_moment: DVec3,
    origin_tensor: SymmetricInertia,
}

impl SteinerMass {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mass_kg: f64, first_moment: DVec3, origin_tensor: SymmetricInertia) -> Self {
        Self { mass_kg, first_moment, origin_tensor }
    }

    pub const fn weightless() -> Self {
        Self::new(0.0, DVec3::ZERO, SymmetricInertia::zero())
    }

    pub fn include_cell(self, coord: CellCoord, cell_mass_kg: f64) -> Self {
        if cell_mass_kg <= 0.0 {
            return self;
        }
        let p = coord.metric_center(0.0);
        Self::new(
            self.mass_kg + cell_mass_kg,
            self.first_moment + p * cell_mass_kg,
            self.origin_tensor.plus(SymmetricInertia::cube_about_origin(p, cell_mass_kg)),
        )
    }

    pub fn exclude_cell(self, coord: CellCoord, cell_mass_kg: f64) -> Self {
        if cell_mass_kg <= 0.0 {
            return self;
        }
        let p = coord.metric_center(0.0);
        let next_mass = (self.mass_kg - cell_mass_kg).max(0.0);
        if next_mass <= 1e-9 {
            return Self::weightless();
        }
        Self::new(
            next_mass,
            self.first_moment - p * cell_mass_kg,
            self.origin_tensor.minus(SymmetricInertia::cube_about_origin(p, cell_mass_kg)),
        )
    }

    pub const fn total_mass_kg(self) -> f64 {
        self.mass_kg
    }

    pub fn center_of_mass(self) -> DVec3 {
        if self.mass_kg <= 1e-9 {
            DVec3::ZERO
        } else {
            self.first_moment / self.mass_kg
        }
    }

    pub fn inertia_about_com(self) -> SymmetricInertia {
        self.origin_tensor.shifted_to_com(self.mass_kg, self.center_of_mass())
    }

    pub fn inertia_matrix(self) -> DMat3 {
        self.inertia_about_com().to_mat3()
    }
}

/// Focused contract for contiguous `8×8×8` brick cell & multi-block module storage, decorated by `InertialBricks`.
pub trait BrickVolume {
    fn vacant(&self) -> bool;
    fn brick_count(&self) -> usize;
    fn cell_at(&self, coord: CellCoord) -> BrickCell;
    fn resolve_module(&self, coord: CellCoord) -> Option<(CellCoord, ModuleSpec)>;
    fn place_atom(&mut self, coord: CellCoord, atom: RawAtom) -> BrickCell;
    fn place_module(&mut self, anchor: CellCoord, spec: ModuleSpec) -> bool;
    fn dismantle_at(&mut self, coord: CellCoord) -> DismantledSpan;
}

/// Outcome of atomically dismantling a single `RawAtom` or an entire `B × T × H` `ModuleAnchor`/`ModuleProxy` span (`2` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DismantledSpan {
    Vacant,
    Atom { coord: CellCoord, atom: RawAtom },
    Module { anchor: CellCoord, spec: ModuleSpec },
}

impl DismantledSpan {
    pub const fn medium(self) -> Option<crate::block::BlockMedium> {
        match self {
            Self::Vacant => None,
            Self::Atom { atom, .. } => Some(atom.medium()),
            Self::Module { spec, .. } => Some(spec.medium()),
        }
    }
}

/// Elegant-Objects Decorator maintaining exact `O(1)` Steiner mass $M$, CoM $\vec{c}$, and $3\times 3$ inertia tensor $\mathbf{I}$ over any `BrickVolume` (`2` fields).
#[derive(Clone)]
pub struct InertialBricks<V> {
    origin: V,
    steiner: SteinerMass,
}

impl<V> InertialBricks<V> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: V, steiner: SteinerMass) -> Self {
        Self { origin, steiner }
    }

    pub const fn wrapping(origin: V) -> Self {
        Self::new(origin, SteinerMass::weightless())
    }

    pub const fn inner(&self) -> &V {
        &self.origin
    }

    pub const fn steiner(&self) -> SteinerMass {
        self.steiner
    }

    pub const fn total_mass_kg(&self) -> f64 {
        self.steiner.total_mass_kg()
    }

    pub fn center_of_mass(&self) -> DVec3 {
        self.steiner.center_of_mass()
    }

    pub fn inertia_matrix(&self) -> DMat3 {
        self.steiner.inertia_matrix()
    }
}

impl<V: BrickVolume> BrickVolume for InertialBricks<V> {
    fn vacant(&self) -> bool { self.origin.vacant() }
    fn brick_count(&self) -> usize { self.origin.brick_count() }
    fn cell_at(&self, coord: CellCoord) -> BrickCell { self.origin.cell_at(coord) }
    fn resolve_module(&self, coord: CellCoord) -> Option<(CellCoord, ModuleSpec)> { self.origin.resolve_module(coord) }

    fn place_atom(&mut self, coord: CellCoord, atom: RawAtom) -> BrickCell {
        if matches!(self.origin.cell_at(coord), BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_)) {
            let _ = self.dismantle_at(coord);
        }
        let displaced = self.origin.place_atom(coord, atom);
        if let BrickCell::RawAtom(old) = displaced {
            self.steiner = self.steiner.exclude_cell(coord, old.mass_kg());
        }
        self.steiner = self.steiner.include_cell(coord, atom.mass_kg());
        displaced
    }

    fn place_module(&mut self, anchor: CellCoord, spec: ModuleSpec) -> bool {
        if !self.origin.place_module(anchor, spec) {
            return false;
        }
        let per_cell = spec.cell_mass_kg();
        let mut next = self.steiner;
        spec.span().visit_offsets(|off| {
            let (du, dv, dw) = off.deltas();
            next = next.include_cell(anchor.shifted(du as i32, dv as i32, dw as i32), per_cell);
        });
        self.steiner = next;
        true
    }

    fn dismantle_at(&mut self, coord: CellCoord) -> DismantledSpan {
        let out = self.origin.dismantle_at(coord);
        match out {
            DismantledSpan::Vacant => {}
            DismantledSpan::Atom { coord: c, atom } => {
                self.steiner = self.steiner.exclude_cell(c, atom.mass_kg());
            }
            DismantledSpan::Module { anchor, spec } => {
                let per_cell = spec.cell_mass_kg();
                let mut next = self.steiner;
                spec.span().visit_offsets(|off| {
                    let (du, dv, dw) = off.deltas();
                    next = next.exclude_cell(anchor.shifted(du as i32, dv as i32, dw as i32), per_cell);
                });
                self.steiner = next;
            }
        }
        out
    }
}
