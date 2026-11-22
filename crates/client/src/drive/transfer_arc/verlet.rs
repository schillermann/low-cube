use glam::{DVec3, Vec3};
use world::{BodyId, CelestialAtlas, SolarEphemeris};
use super::{ARC_NODES, CourseProfile, TransferArc};

/// Multi-body 64-bit `Velocity-Verlet` Keplerian & gravity-assist slingshot trajectory solver.
#[derive(Clone, Copy, Debug)]
pub struct SymplecticOrbit {
    ephemeris: SolarEphemeris,
    substeps: usize,
}

impl SymplecticOrbit {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ephemeris: SolarEphemeris, substeps: usize) -> Self {
        Self { ephemeris, substeps }
    }

    /// Integrates an unpowered multi-body trajectory through moving planetary fields (`1/r²` ellipses & slingshots).
    pub fn propagate_dvec(&self, origin: DVec3, initial_vel: DVec3, duration_secs: f64) -> ([DVec3; ARC_NODES], DVec3) {
        let mut out = [origin; ARC_NODES];
        let sub = self.substeps.max(1);
        let total_steps = (ARC_NODES - 1) * sub;
        let dt = (duration_secs / (total_steps as f64)).max(1e-3);
        let (mut p, mut v, mut t) = (origin, initial_vel, 0.0_f64);
        let mut a = self.ephemeris.gravity_accel_at(p, t);

        for seg in 1..ARC_NODES {
            for _ in 0..sub {
                let next_p = p + v * dt + a * (0.5 * dt * dt);
                let next_t = t + dt;
                let next_a = self.ephemeris.gravity_accel_at(next_p, next_t);
                v += (a + next_a) * (0.5 * dt);
                p = next_p;
                t = next_t;
                a = next_a;
            }
            out[seg] = p;
        }
        (out, v)
    }

    pub fn coasting_arc(&self, origin: DVec3, velocity: DVec3, horizon_secs: f64) -> [Vec3; ARC_NODES] {
        let (pts, _) = self.propagate_dvec(origin, velocity, horizon_secs.max(1.0));
        std::array::from_fn(|i| pts[i].as_vec3())
    }

    fn estimate_transfer_time(&self, origin: DVec3, velocity: DVec3, target: DVec3) -> f64 {
        let dist = (target - origin).length().max(1.0);
        let snaps = self.ephemeris.snapshot_all(0.0);
        let sun = snaps[0];
        let r_sun = (origin - sun.center()).length().max(100.0);
        let v_kepler = (sun.gm() / r_sun).sqrt() * 0.45;
        let v_ref = velocity.length().max(v_kepler).clamp(32.0, 24_000.0);
        (dist / v_ref).clamp(3.0, 14_400.0)
    }

    /// Solves a curved multi-body ballistic transfer to a fixed target coordinate `target`.
    pub fn point_intercept(&self, origin: DVec3, velocity: DVec3, target: DVec3, target_normal: Vec3) -> TransferArc {
        let tau = self.estimate_transfer_time(origin, velocity, target);
        self.shoot_to(origin, velocity, target, target_normal, tau)
    }

    /// Solves a curved Keplerian / slingshot transfer to the future orbital position $t + \tau_{\text{arrival}}$ of `body`.
    pub fn intercept_body(
        &self,
        origin: DVec3,
        velocity: DVec3,
        body: BodyId,
        atlas: CelestialAtlas,
    ) -> Option<TransferArc> {
        let (_, target0, norm0) = atlas.target_on_body(body, origin, 0.0)?;
        let mut tau = self.estimate_transfer_time(origin, velocity, target0);
        let mut lead_pt = target0;
        let mut lead_norm = norm0;
        for _ in 0..4 {
            if let Some((snap, pt, norm)) = atlas.target_on_body(body, origin, tau) {
                lead_pt = pt;
                lead_norm = norm;
                let rel_speed = (velocity - snap.velocity()).length().max(38.0);
                let next_tau = ((lead_pt - origin).length() / rel_speed).clamp(3.0, 18_000.0);
                tau = 0.5 * tau + 0.5 * next_tau;
            }
        }
        Some(self.shoot_to(origin, velocity, lead_pt, lead_norm, tau))
    }

    fn shoot_to(&self, origin: DVec3, velocity: DVec3, target: DVec3, target_normal: Vec3, duration: f64) -> TransferArc {
        let chord = target - origin;
        let dist = chord.length();
        let corridor = ((dist as f32) * 0.04).clamp(4.0, 120.0);
        if dist < 0.5 {
            return TransferArc::with_profile(
                [origin.as_vec3(); ARC_NODES],
                target_normal.normalize_or_zero(),
                corridor,
                CourseProfile::Ballistic { arrival_secs: 0.0 },
            );
        }
        let g0 = self.ephemeris.gravity_accel_at(origin, 0.0);
        let mut launch_v = chord / duration - g0 * (0.35 * duration);
        if velocity.length_squared() > 16.0 && velocity.dot(chord) > 0.0 {
            launch_v = launch_v * 0.80 + velocity * 0.20;
        }
        let (mut d_nodes, _) = self.propagate_dvec(origin, launch_v, duration);
        for _ in 0..8 {
            let miss = target - d_nodes[ARC_NODES - 1];
            if miss.length_squared() <= 1.0 {
                break;
            }
            launch_v += (miss / duration) * 0.78;
            (d_nodes, _) = self.propagate_dvec(origin, launch_v, duration);
        }
        let residual = target - d_nodes[ARC_NODES - 1];
        let denom = (ARC_NODES - 1) as f64;
        let mut nodes = [Vec3::ZERO; ARC_NODES];
        for (i, slot) in nodes.iter_mut().enumerate() {
            let s = (i as f64) / denom;
            let blend = s * s * (3.0 - 2.0 * s);
            *slot = (d_nodes[i] + residual * blend).as_vec3();
        }
        nodes[0] = origin.as_vec3();
        nodes[ARC_NODES - 1] = target.as_vec3();
        TransferArc::with_profile(
            nodes,
            target_normal.normalize_or_zero(),
            corridor,
            CourseProfile::Ballistic { arrival_secs: duration as f32 },
        )
    }
}
