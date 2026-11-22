use glam::{DVec3, Vec3};
use world::{BodyId, CelestialAtlas, SolarEphemeris};
use super::{ARC_NODES, CourseProfile, TransferArc};

/// Continuous-thrust *The Expanse* Brachistochrone trajectory solver (`50%` accel + *Flip-and-Burn* + `50%` brake).
#[derive(Clone, Copy, Debug)]
pub struct BrachistochroneArc {
    burn_accel: f64,
}

impl BrachistochroneArc {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(burn_accel: f64) -> Self {
        Self { burn_accel }
    }

    fn simulate_torch(
        &self,
        eph: SolarEphemeris,
        origin: DVec3,
        initial_vel: DVec3,
        thrust_dir: DVec3,
        duration: f64,
    ) -> [DVec3; ARC_NODES] {
        let mut out = [origin; ARC_NODES];
        let sub = 4usize;
        let total = (ARC_NODES - 1) * sub;
        let half_step = total / 2;
        let dt = (duration / (total as f64)).max(1e-3);
        let (mut p, mut v, mut t) = (origin, initial_vel, 0.0_f64);
        let mut step_idx = 0usize;

        for seg in 1..ARC_NODES {
            for _ in 0..sub {
                let sign = if step_idx < half_step { 1.0 } else { -1.0 };
                let thrust = thrust_dir * (self.burn_accel * sign);
                let a0 = eph.gravity_accel_at(p, t) + thrust;
                let next_p = p + v * dt + a0 * (0.5 * dt * dt);
                let next_t = t + dt;
                let a1 = eph.gravity_accel_at(next_p, next_t) + thrust;
                v += (a0 + a1) * (0.5 * dt);
                p = next_p;
                t = next_t;
                step_idx += 1;
            }
            out[seg] = p;
        }
        out
    }

    /// Computes a gravity-coupled Brachistochrone transfer to `body`'s moving arrival point with marked `50%` flip node.
    pub fn intercept_body(
        &self,
        origin: DVec3,
        velocity: DVec3,
        body: BodyId,
        atlas: CelestialAtlas,
    ) -> Option<TransferArc> {
        let eph = atlas.ephemeris()?;
        let accel = self.burn_accel.max(6.0);
        let (snap0, pt0, norm0) = atlas.target_on_body(body, origin, 0.0)?;
        let mut tau = (2.0 * ((pt0 - origin).length().max(1.0) / accel).sqrt()).clamp(1.5, 7_200.0);
        let (mut lead_pt, mut lead_norm, mut target_vel) = (pt0, norm0, snap0.velocity());

        for _ in 0..5 {
            if let Some((snap, pt, norm)) = atlas.target_on_body(body, origin, tau) {
                lead_pt = pt;
                lead_norm = norm;
                target_vel = snap.velocity();
                let eff_chord = lead_pt - (origin + velocity * (0.5 * tau));
                let d = eff_chord.length().max(1.0);
                tau = (2.0 * (d / accel).sqrt()).clamp(1.5, 7_200.0);
            }
        }

        let chord = lead_pt - origin;
        let dist = chord.length();
        let corridor = ((dist as f32) * 0.035).clamp(6.0, 180.0);
        if dist < 1.0 {
            return Some(TransferArc::with_profile(
                [origin.as_vec3(); ARC_NODES],
                lead_norm.normalize_or_zero(),
                corridor,
                CourseProfile::Brachistochrone {
                    flip_point: origin.as_vec3(),
                    total_secs: 0.0,
                    burn_accel: accel as f32,
                },
            ));
        }

        let mut burn_dir = ((lead_pt + target_vel * (0.15 * tau)) - (origin + velocity * (0.35 * tau))).normalize_or_zero();
        if burn_dir.length_squared() < 0.25 {
            burn_dir = chord / dist;
        }
        let mut d_nodes = self.simulate_torch(eph, origin, velocity, burn_dir, tau);
        for _ in 0..5 {
            let miss = lead_pt - d_nodes[ARC_NODES - 1];
            if miss.length_squared() <= 4.0 {
                break;
            }
            let adjusted = burn_dir * dist + miss * 0.75;
            if adjusted.length_squared() > 1e-4 {
                burn_dir = adjusted.normalize();
            }
            d_nodes = self.simulate_torch(eph, origin, velocity, burn_dir, tau);
        }

        let residual = lead_pt - d_nodes[ARC_NODES - 1];
        let denom = (ARC_NODES - 1) as f64;
        let mut nodes = [Vec3::ZERO; ARC_NODES];
        for (i, slot) in nodes.iter_mut().enumerate() {
            let s = (i as f64) / denom;
            let blend = s * s * (3.0 - 2.0 * s);
            *slot = (d_nodes[i] + residual * blend).as_vec3();
        }
        nodes[0] = origin.as_vec3();
        nodes[ARC_NODES - 1] = lead_pt.as_vec3();
        let flip_point = nodes[ARC_NODES / 2];

        Some(TransferArc::with_profile(
            nodes,
            lead_norm.normalize_or_zero(),
            corridor,
            CourseProfile::Brachistochrone {
                flip_point,
                total_secs: tau as f32,
                burn_accel: accel as f32,
            },
        ))
    }
}
