use super::frequency::SpatialFrequency;
use glam::Vec3;

const GRADIENTS: [[f32; 3]; 12] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
];

/// Deterministic 3D gradient lattice coherent noise object.
pub struct CoherentNoise {
    seed: u64,
    frequency: f32,
}

impl CoherentNoise {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(seed: u64, frequency: f32) -> Self {
        Self { seed, frequency }
    }

    fn hash_coord(&self, x: i32, y: i32, z: i32) -> usize {
        let mut h = self.seed.wrapping_add(x as u64).wrapping_mul(0x517cc1b727220a95);
        h = h.wrapping_add(y as u64).wrapping_mul(0x6c62272e07bb0142);
        h = h.wrapping_add(z as u64).wrapping_mul(0x9e3779b97f4a7c15);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        (h as usize) % 12
    }

    fn quintic(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn grad_dot(&self, x: i32, y: i32, z: i32, dx: f32, dy: f32, dz: f32) -> f32 {
        let idx = self.hash_coord(x, y, z);
        let g = GRADIENTS[idx];
        g[0] * dx + g[1] * dy + g[2] * dz
    }
}

impl SpatialFrequency for CoherentNoise {
    fn sample(&self, point: Vec3) -> f32 {
        let p = point * self.frequency;
        let x0 = p.x.floor() as i32;
        let y0 = p.y.floor() as i32;
        let z0 = p.z.floor() as i32;

        let dx0 = p.x - p.x.floor();
        let dy0 = p.y - p.y.floor();
        let dz0 = p.z - p.z.floor();

        let dx1 = dx0 - 1.0;
        let dy1 = dy0 - 1.0;
        let dz1 = dz0 - 1.0;

        let u = self.quintic(dx0);
        let v = self.quintic(dy0);
        let w = self.quintic(dz0);

        let c000 = self.grad_dot(x0, y0, z0, dx0, dy0, dz0);
        let c100 = self.grad_dot(x0 + 1, y0, z0, dx1, dy0, dz0);
        let c010 = self.grad_dot(x0, y0 + 1, z0, dx0, dy1, dz0);
        let c110 = self.grad_dot(x0 + 1, y0 + 1, z0, dx1, dy1, dz0);

        let c001 = self.grad_dot(x0, y0, z0 + 1, dx0, dy0, dz1);
        let c101 = self.grad_dot(x0 + 1, y0, z0 + 1, dx1, dy0, dz1);
        let c011 = self.grad_dot(x0, y0 + 1, z0 + 1, dx0, dy1, dz1);
        let c111 = self.grad_dot(x0 + 1, y0 + 1, z0 + 1, dx1, dy1, dz1);

        let nx00 = c000 + u * (c100 - c000);
        let nx10 = c010 + u * (c110 - c010);
        let nx01 = c001 + u * (c101 - c001);
        let nx11 = c011 + u * (c111 - c011);

        let nxy0 = nx00 + v * (nx10 - nx00);
        let nxy1 = nx01 + v * (nx11 - nx01);

        nxy0 + w * (nxy1 - nxy0)
    }
}
