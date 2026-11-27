mod crust_ops;
mod docking;
mod flange_hoist;
mod flange_ops;
mod fleet;
mod fleet_docking;
mod fleet_surface;
mod forces;
mod hull_edit;
mod motion;

use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::brick_arena::{BrickStructure, SteinerMass, SymmetricInertia};
use crate::frame::SurfaceFrame;
use crate::relief::{
    CraftBlueprint, CraftCluster, CraftIdentity, CraftPose, DeckFrame, MoldedStructure,
    NaturalStrata, PosedHull, RigArm, SanctuaryPlinth, ShaftExcavation, TurfScars,
    VehicleAssembly,
};
pub use self::crust_ops::{CrustMason, CrustSurvey};
pub use self::docking::{DockingClamp, DockingPort};
pub use self::flange_hoist::FlangeHoist;
pub use self::flange_ops::{FlangeMason, FlangeSurvey};
pub use self::fleet::GridFleet;
pub use self::fleet_docking::FleetDockyard;
pub use self::fleet_surface::SurfaceFleetView;
pub use self::forces::{
    CentrifugalRing, CoriolisDeflection, DeckDynamics, InertialForce, MagBootGrip, ThrustGravity,
    ZeroGravityField,
};
pub use self::hull_edit::HullMason;
pub use self::motion::{BodyMooring, BodyMotion, DockedLink};

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Universal `0.5 m` voxel & multi-block entity uniting planetary structures, magnetic flanges, rovers, stations, and spacecraft (`4` fields).
pub struct GridBody {
    grid: BrickStructure,
    blueprint: CraftBlueprint,
    motion: BodyMotion,
    mooring: BodyMooring,
}

impl GridBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        grid: BrickStructure,
        blueprint: CraftBlueprint,
        motion: BodyMotion,
        mooring: BodyMooring,
    ) -> Self {
        Self { grid, blueprint, motion, mooring }
    }

    pub fn crust_anchored() -> Self {
        Self::new(
            BrickStructure::untouched(),
            CraftBlueprint::new(CraftIdentity::unassigned(), Vec::new()),
            BodyMotion::planetary_identity(),
            BodyMooring::crust_anchored(SanctuaryPlinth::absent()),
        )
    }

    pub fn flange_deployed(ground_top: f32, turn: u8) -> Self {
        Self::new(
            BrickStructure::untouched(),
            CraftBlueprint::new(CraftIdentity::unassigned(), Vec::new()),
            BodyMotion::planetary_identity(),
            BodyMooring::MagneticFlange(RigArm::deployed(ground_top, turn)),
        )
    }

    pub fn flange_from_snapshot(gsu: i32, gsv: i32, snap: &crate::RigSnapshot) -> ((i32, i32), Self) {
        let (mu, mv) = crate::SectorSnapshot::unpack_uv(snap.local_uv(), gsu, gsv);
        let craft = CraftCluster::from_relative_snapshot(mu, mv, snap.craft_cells());
        let body = Self::new(
            craft.into_structure(),
            CraftBlueprint::new(CraftIdentity::unassigned(), Vec::new()),
            BodyMotion::planetary_identity(),
            BodyMooring::MagneticFlange(RigArm::from_posture(snap.arm())),
        );
        ((mu, mv), body)
    }

    pub fn from_free_assembly(assembly: VehicleAssembly, linear_vel: Vec3, angular_vel: Vec3, proper_accel: Vec3) -> Self {
        let (raw_bp, raw_pose) = assembly.into_blueprint_pose();
        let bp = if raw_bp.identity().assigned() || raw_bp.part_count() == 0 {
            raw_bp
        } else {
            CraftBlueprint::commission(raw_pose.origin(), raw_bp.into_parts())
        };
        let local_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let grid = bp.to_face_cluster(&local_frame).into_structure();
        let motion = BodyMotion::new(raw_pose, linear_vel, angular_vel, proper_accel)
            .with_proper_accel(proper_accel);
        Self::new(grid, bp, motion, BodyMooring::free_flight())
    }

    pub fn settled_on_surface(
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        assembly: VehicleAssembly,
    ) -> (Self, (i32, i32)) {
        let (raw_bp, raw_pose) = assembly.into_blueprint_pose();
        let bp = if raw_bp.identity().assigned() || !raw_bp.drivable() {
            raw_bp
        } else {
            CraftBlueprint::commission(raw_pose.origin(), raw_bp.into_parts())
        };
        let (u0, v0) = frame.decompose(raw_pose.origin());
        let half_cells = (frame.height_of(frame.point_at(0.0, 0.0, 0.0)) * 2.0).round() as i32;
        let (u, v) = if half_cells >= 8 { frame.clamp_interior(u0, v0, 0.35) } else { (u0, v0) };
        let seat_cell = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let ground_top = (shafts.elevation_at(strata, seat_cell.0, seat_cell.1) as f32) * BLOCK_STEP;
        let cur_elev = frame.elevation_of(raw_pose.origin());
        let settled_elev = if (cur_elev - ground_top).abs() <= 0.65 { ground_top } else { cur_elev.max(ground_top) };
        let settled_origin = frame.point_at(u, v, settled_elev);
        let settled_pose = raw_pose.relocated(settled_origin, frame.normal()).with_motion(0.0, 0.0);
        let local_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let grid = bp.to_face_cluster(&local_frame).into_structure();
        let body = Self::new(grid, bp, BodyMotion::stationary(settled_pose), BodyMooring::SurfaceSettled(*frame));
        (body, seat_cell)
    }

    pub const fn identity(&self) -> CraftIdentity { self.blueprint.identity() }
    pub const fn structure(&self) -> &BrickStructure { &self.grid }
    pub const fn blueprint(&self) -> &CraftBlueprint { &self.blueprint }
    pub const fn motion(&self) -> BodyMotion { self.motion }
    pub const fn pose(&self) -> CraftPose { self.motion.pose() }
    pub const fn mooring(&self) -> BodyMooring { self.mooring }
    pub fn allocated_brick_bytes(&self) -> usize { self.grid.allocated_brick_bytes() }
    pub fn steiner_mass(&self) -> SteinerMass { self.grid.bricks().steiner() }
    pub fn total_mass_kg(&self) -> f32 { self.grid.bricks().total_mass_kg() as f32 }
    pub fn center_of_mass_local(&self) -> Vec3 { self.grid.bricks().center_of_mass().as_vec3() }
    pub fn inertia_tensor_local(&self) -> SymmetricInertia { self.grid.bricks().steiner().inertia_about_com() }
    pub fn into_assembly(self) -> VehicleAssembly { VehicleAssembly::from_blueprint(self.blueprint, self.motion.pose()) }
    pub fn crown_origin_plinth(&mut self) { self.mooring = BodyMooring::CrustAnchored(SanctuaryPlinth::origin_crown()); }
    pub fn crust(&self) -> CrustSurvey<'_> { CrustSurvey::new(&self.grid, self.sanctuary_plinth()) }
    pub fn crust_mut(&mut self) -> CrustMason<'_> { CrustMason::new(&mut self.grid) }
    pub fn flange(&self) -> Option<FlangeSurvey<'_>> { self.rig_arm().map(|a| FlangeSurvey::new(&self.grid, a)) }
    pub fn hull_mason(&mut self) -> HullMason<'_> { HullMason::new(&mut self.grid, &mut self.blueprint, self.motion.pose()) }

    pub fn sanctuary_plinth(&self) -> SanctuaryPlinth {
        match self.mooring {
            BodyMooring::CrustAnchored(p) => p,
            _ => SanctuaryPlinth::absent(),
        }
    }

    pub fn rig_arm(&self) -> Option<RigArm> {
        match self.mooring {
            BodyMooring::MagneticFlange(arm) => Some(arm),
            _ => None,
        }
    }

    pub fn with_flange_mut<R>(&mut self, f: impl FnOnce(FlangeMason<'_>) -> R) -> Option<R> {
        let BodyMooring::MagneticFlange(ref mut arm) = self.mooring else { return None };
        Some(f(FlangeMason::new(&mut self.grid, arm)))
    }

    pub fn with_flange_hoist<R>(&mut self, f: impl FnOnce(FlangeHoist<'_>) -> R) -> Option<R> {
        let BodyMooring::MagneticFlange(ref mut arm) = self.mooring else { return None };
        Some(f(FlangeHoist::new(&mut self.grid, arm)))
    }

    pub fn into_face_cluster(self, strata: &NaturalStrata, shafts: &ShaftExcavation) -> CraftCluster {
        let BodyMooring::SurfaceSettled(face_frame) = self.mooring else {
            return CraftCluster::new(self.grid);
        };
        let lift = self.blueprint.wheel_ground_lift();
        let raw_seat = face_frame.cell_at(self.pose().origin()).feed_coordinates(|cu, cv| (cu, cv));
        let basis = self.pose().axes().grid_basis(&face_frame);
        let half_cells = (face_frame.height_of(face_frame.point_at(0.0, 0.0, 0.0)) * 2.0).round() as i32;
        let seat = self.blueprint.clamped_seat(raw_seat, basis, half_cells);
        let mut craft = CraftCluster::untouched();
        for part in self.blueprint.into_parts() {
            part.park_into(seat, basis, lift, &mut craft, strata, shafts);
        }
        craft
    }

    pub fn deck_dynamics(&self) -> DeckDynamics {
        let inv_rot = self.pose().hull_rotation().inverse();
        let local_a = inv_rot * self.motion.proper_accel();
        let local_w = inv_rot * self.motion.angular_vel();
        let burn_y = DeckFrame::thrust_from_pose(self.pose());
        let eff_a = if local_a.length_squared() > 1e-4 { local_a } else { Vec3::new(0.0, burn_y, 0.0) };
        DeckDynamics::new(eff_a, local_w, self.center_of_mass_local())
    }

    pub fn collect_docking_ports(&self, out: &mut Vec<DockingPort>) {
        DockingPort::collect_from(self.identity(), &self.grid, self.pose(), out);
    }

    pub fn apply_motion(&mut self, motion: BodyMotion) { self.motion = motion; }
    pub fn lock_to_port(&mut self, link: DockedLink, synced: BodyMotion) {
        self.mooring = BodyMooring::DockedPort(link);
        self.motion = synced;
    }

    pub fn undock(&mut self, impulse_vel: Vec3) -> bool {
        if self.mooring.docked_link().is_none() {
            return false;
        }
        self.mooring = BodyMooring::FreeFlight;
        self.motion = self.motion.with_velocity(self.motion.linear_vel() + impulse_vel, self.motion.angular_vel());
        true
    }

    pub fn step_free(&mut self, dt: f32, gravitational_dv: Vec3) {
        if self.mooring.free_moving() {
            self.motion = self.motion.step_six_dof(dt, gravitational_dv);
        }
    }

    pub fn covers_ground_at(&self, u: i32, v: i32, ground_top: f32) -> bool { self.grid.covers_ground_at(u, v, ground_top) }
    pub fn bare_surface_at(&self, u: i32, v: i32, ground_bare: bool) -> bool { self.grid.bare_surface_at(u, v, ground_bare) }
    pub fn verdant_near_step(&self, u: i32, v: i32, target_step: i32) -> bool { self.grid.verdant_near_step(u, v, target_step) }
    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta: i32) -> Option<(i32, i32)> { self.grid.step_camera_turn(u, v, delta) }
    pub fn confine_niche_elevation(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        self.grid.confine_niche(u, v, ground_h, elev)
    }
    pub fn settle_gravity(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, scars: &mut TurfScars) -> Vec<(i32, i32)> {
        self.grid.settle_gravity(|u, v| (shafts.elevation_at(strata, u, v) as f32) * BLOCK_STEP, scars)
    }
    pub(crate) fn posed_hull(&self) -> PosedHull<'_> { PosedHull::new(&self.blueprint, self.pose()) }

    pub(crate) fn surface_cell_center(&self, u: i32, v: i32, elev: f32) -> Option<(SurfaceFrame, Vec3)> {
        let BodyMooring::SurfaceSettled(ff) = self.mooring else { return None };
        let (uc, vc) = GridCell::new(u, v).center();
        Some((ff, ff.point_at(uc, vc, elev)))
    }

    pub(crate) fn occupies_surface_layer(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        let BodyMooring::SurfaceSettled(ff) = self.mooring else { return false };
        let (uc, vc) = GridCell::new(u, v).center();
        [(0.0, 0.0), (0.14, 0.14), (-0.14, 0.14), (0.14, -0.14), (-0.14, -0.14)]
            .into_iter()
            .any(|(du, dv)| self.posed_hull().medium_at_world(ff.point_at(uc + du, vc + dv, layer_elev)).is_some())
    }
}

impl MoldedStructure for GridBody {
    fn vacant(&self) -> bool { matches!(self.sanctuary_plinth(), SanctuaryPlinth::Absent) && self.grid.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.sanctuary_plinth().active_at(u, v) || self.grid.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        let floor = if self.sanctuary_plinth().active_at(u, v) { floor_steps.max(0) } else { floor_steps };
        self.grid.top_steps_at(u, v, floor)
    }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        let (u, v) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let floor = if self.sanctuary_plinth().active_at(u, v) && span.0 >= -0.55 { h.max(0.0) } else { h };
        self.grid.constrain_support(cell, planar, floor, span)
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.sanctuary_plinth().in_layer(u, v, layer_elev) || self.grid.layer_occupied_at(u, v, layer_elev)
    }
    fn locked_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> { self.grid.seat_backrest_cell(u, v, layer_elev) }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.grid.medium_at(u, v, layer_elev).or_else(|| self.sanctuary_plinth().medium_at(u, v, layer_elev))
    }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.grid.hit_top_at(cu, cv, u, v, ray_elev).or_else(|| self.sanctuary_plinth().hit_top(cu, cv, ray_elev))
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.grid.perch_at(u, v, layer_elev, eu, ev, frame)
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.grid.tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let p = self.sanctuary_plinth().emit_at(cell, frame, target);
        let g = self.grid.emit_at(cell, frame, colors, scars, target);
        p || g
    }
}
