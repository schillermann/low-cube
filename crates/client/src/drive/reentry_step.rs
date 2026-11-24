use glam::Vec3;
use world::{PlanetaryWorld, VehicleAssembly};
use super::clearance::HullClearance;
use super::space_step::{SpaceState, SpaceStep};

const REENTRY_CEILING: f32 = 14.0;
const REENTRY_SPEED_THRESHOLD: f32 = 50.0;

/// `SpaceStep` decorator modeling gas-envelope drag and carbon-silicate ceramic aerobraking vs. buffeting.
pub(super) struct AtmosphericEntryStep<'a, S> {
    origin: S,
    assembly: &'a VehicleAssembly,
    world: &'a dyn PlanetaryWorld,
}

impl<'a, S> AtmosphericEntryStep<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        origin: S,
        assembly: &'a VehicleAssembly,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self { origin, assembly, world }
    }

    fn apply_envelope(
        &self,
        pos: Vec3,
        vel: Vec3,
        omega: Vec3,
        drag_rate: f32,
        dt: f32,
    ) -> (Vec3, Vec3) {
        let shielded = self.assembly.thermal_shielded();
        let brake = if shielded { drag_rate * 1.45 } else { drag_rate * 0.65 };
        let next_vel = vel * (1.0 - (brake * dt).clamp(0.0, 0.75));
        let speed = vel.length();
        let next_omega = if shielded {
            omega * (1.0 - (4.5 * dt).clamp(0.0, 1.0))
        } else if speed > 12.0 {
            let flow = vel / speed;
            let ship_up = self.assembly.normal().normalize_or_zero();
            let cross_axis = ship_up.cross(flow);
            let axis = if cross_axis.length_squared() > 1e-4 {
                cross_axis.normalize()
            } else {
                Vec3::new(flow.y, -flow.x, flow.z).normalize_or_zero()
            };
            let buffet = (drag_rate * (speed * 0.025).min(2.2)) * dt;
            let sign = if (pos.x + pos.z) >= 0.0 { 1.0 } else { -1.0 };
            omega + axis * (buffet * sign)
        } else {
            omega
        };
        (next_vel, next_omega)
    }
}

impl<S: SpaceStep> SpaceStep for AtmosphericEntryStep<'_, S> {
    fn resolve(&self, dt: f32) -> SpaceState {
        let stepped = self.origin.resolve(dt);
        stepped.transfer(|pos, vel, omega, rot| {
            let contact = self.world.surface_contact(pos, 0.0);
            if let Some((v_gas, w_gas)) = contact.resolve_gas(None, |drag| {
                Some(self.apply_envelope(pos, vel, omega, drag.max(0.35), dt))
            }) {
                return SpaceState::new(pos, v_gas, w_gas, rot);
            }
            if self.world.apply_pull(pos, 1.0).length_squared() <= 0.04 {
                return SpaceState::new(pos, vel, omega, rot);
            }
            let probe = HullClearance::new(self.assembly, pos, vel, rot);
            let Some((_, _, alt, v_norm)) = probe.crust_contact(self.world) else {
                return SpaceState::new(pos, vel, omega, rot);
            };
            if alt > 0.0 && alt < REENTRY_CEILING && v_norm < -REENTRY_SPEED_THRESHOLD {
                let density = (1.0 - alt / REENTRY_CEILING).clamp(0.0, 1.0);
                let (v_re, w_re) = self.apply_envelope(pos, vel, omega, 2.2 * density, dt);
                return SpaceState::new(pos, v_re, w_re, rot);
            }
            SpaceState::new(pos, vel, omega, rot)
        })
    }
}
