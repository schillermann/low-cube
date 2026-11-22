use glam::Vec3;
use world::CelestialAtlas;

/// Right-handed 3D coordinate mapping with honest two-stage auto-scale (`TRANSFER` vs `APPROACH` zoom).
#[derive(Clone, Copy, Debug)]
pub struct OrreryFrame {
    bases: ([Vec3; 3], [Vec3; 3]),
    anchors: (Vec3, Vec3),
    scale_and_radii: (f32, f32, f32, bool),
}

impl OrreryFrame {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        bases: ([Vec3; 3], [Vec3; 3]),
        anchors: (Vec3, Vec3),
        scale_and_radii: (f32, f32, f32, bool),
    ) -> Self {
        Self { bases, anchors, scale_and_radii }
    }

    #[cfg(test)]
    pub fn from_atlas(atlas: CelestialAtlas) -> Self {
        Self::for_ship(atlas, Vec3::ZERO)
    }

    /// Builds a 1:1 true-scale Transfer frame in deep space, switching cleanly to 1:1 Approach Zoom near the Moon.
    pub fn for_ship(atlas: CelestialAtlas, ship_pos: Vec3) -> Self {
        atlas.transfer(|home_half, moon_center, moon_half| {
            let dist = moon_center.length().max(48.0);
            let world_u = if moon_center.length_squared() > 1.0 { moon_center / dist } else { Vec3::Y };
            let up_ref = if world_u.y.abs() < 0.90 { Vec3::Y } else { Vec3::Z };
            let world_w = (up_ref - world_u * world_u.dot(up_ref)).normalize_or_zero();
            let world_v = world_w.cross(world_u).normalize_or_zero();

            let earth_holo = Vec3::new(-0.42, -0.06, 0.12);
            let moon_holo = Vec3::new(0.44, 0.18, -0.14);
            let span = (moon_holo - earth_holo).length();
            let holo_u = (moon_holo - earth_holo) / span;
            let holo_w = (Vec3::Y - holo_u * holo_u.y).normalize_or_zero();
            let holo_v = holo_w.cross(holo_u).normalize_or_zero();

            let approach_gate = moon_half * 18.0;
            let in_approach = moon_half > 0.0
                && dist > moon_half * 32.0
                && (ship_pos - moon_center).abs().max_element() <= approach_gate;
            if in_approach {
                let vis_moon = 0.092_f32;
                let scale = vis_moon / moon_half.max(1.0);
                return Self::new(
                    ([world_u, world_v, world_w], [holo_u, holo_v, holo_w]),
                    (Vec3::new(0.0, -0.04, 0.0), moon_center),
                    (scale, 0.0, vis_moon, true),
                );
            }

            let scale = span / dist;
            let vis_earth = (home_half * scale).clamp(0.004, 0.34);
            let vis_moon = if moon_half > 0.0 { (moon_half * scale).clamp(0.002, 0.24) } else { 0.0 };
            Self::new(
                ([world_u, world_v, world_w], [holo_u, holo_v, holo_w]),
                (earth_holo, moon_center),
                (scale, vis_earth, vis_moon, false),
            )
        })
    }

    pub const fn earth_holo(&self) -> Vec3 {
        self.anchors.0
    }

    pub const fn moon_world(&self) -> Vec3 {
        self.anchors.1
    }

    pub const fn visual_radii(&self) -> (f32, f32) {
        (self.scale_and_radii.1, self.scale_and_radii.2)
    }

    pub const fn approach_zoom(&self) -> bool {
        self.scale_and_radii.3
    }

    pub fn rotate_dir(&self, d: Vec3) -> Vec3 {
        let ([wu, wv, ww], [hu, hv, hw]) = self.bases;
        hu * d.dot(wu) + hv * d.dot(wv) + hw * d.dot(ww)
    }

    pub fn map_point(&self, world_pt: Vec3) -> Vec3 {
        let (anchor_holo, moon_world) = self.anchors;
        let (scale, _, _, approach) = self.scale_and_radii;
        if approach {
            return anchor_holo + self.rotate_dir(world_pt - moon_world) * scale;
        }
        anchor_holo + self.rotate_dir(world_pt) * scale
    }
}
