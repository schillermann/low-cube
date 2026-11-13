use glam::{Quat, Vec3};

/// Encapsulated 3D stance and orientation of a companion avatar on the World-Cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresenceKinematics {
    position: Vec3,
    surface_rot: Quat,
    view_rot: Quat,
    eye_and_leap: (f32, f32),
}

impl PresenceKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(position: Vec3, surface_rot: Quat, view_rot: Quat, eye_and_leap: (f32, f32)) -> Self {
        Self { position, surface_rot, view_rot, eye_and_leap }
    }

    pub fn transfer<T>(self, f: impl FnOnce(Vec3, Quat, Quat, f32, f32) -> T) -> T {
        f(self.position, self.surface_rot, self.view_rot, self.eye_and_leap.0, self.eye_and_leap.1)
    }
}

/// Complete network-replicated presence of a companion player without exposing mutable fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompanionPresence {
    slot: u32,
    kinematics: PresenceKinematics,
    rhythm: (f32, f32, f32, f32),
    attire: ([u8; 9], [u8; 4]),
}

impl CompanionPresence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        slot: u32,
        kinematics: PresenceKinematics,
        rhythm: (f32, f32, f32, f32),
        attire: ([u8; 9], [u8; 4]),
    ) -> Self {
        Self { slot, kinematics, rhythm, attire }
    }

    pub const fn same_slot(&self, slot: u32) -> bool {
        self.slot == slot
    }

    pub const fn with_slot(self, slot: u32) -> Self {
        Self::new(slot, self.kinematics, self.rhythm, self.attire)
    }

    pub fn within_reach(&self, eye: Vec3, max_dist: f32) -> bool {
        self.kinematics.transfer(|pos, _, _, _, _| (eye - pos).length() <= max_dist)
    }

    pub fn reconciled(self, slot: u32, incoming: Self, world: &dyn crate::PlanetaryWorld) -> Self {
        incoming.kinematics.transfer(|pos, _, _, _, _| {
            let clamped = world.confine_beneath_crust(pos);
            if clamped.is_finite() && (clamped - pos).length_squared() <= 36.0 {
                incoming.with_slot(slot)
            } else {
                self.with_slot(slot)
            }
        })
    }

    pub fn transfer<T>(
        self,
        f: impl FnOnce(u32, PresenceKinematics, (f32, f32, f32, f32), [u8; 9], [u8; 4]) -> T,
    ) -> T {
        f(self.slot, self.kinematics, self.rhythm, self.attire.0, self.attire.1)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.slot.to_le_bytes());
        self.kinematics.transfer(|pos, s_rot, v_rot, eye_h, leap| {
            for f in [
                pos.x, pos.y, pos.z,
                s_rot.x, s_rot.y, s_rot.z, s_rot.w,
                v_rot.x, v_rot.y, v_rot.z, v_rot.w,
                eye_h, leap,
                self.rhythm.0, self.rhythm.1, self.rhythm.2, self.rhythm.3,
            ] {
                out.extend_from_slice(&f.to_le_bytes());
            }
        });
        out.extend_from_slice(&self.attire.0);
        out.extend_from_slice(&self.attire.1);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        const SPAN: usize = 4 + 17 * 4 + 9 + 4;
        if bytes.len() < SPAN {
            return None;
        }
        let slot = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let mut floats = [0.0f32; 17];
        for (i, dst) in floats.iter_mut().enumerate() {
            let off = 4 + i * 4;
            *dst = f32::from_le_bytes(bytes[off..off + 4].try_into().ok()?);
        }
        let mut attire_main = [0u8; 9];
        attire_main.copy_from_slice(&bytes[72..81]);
        let mut attire_reg = [0u8; 4];
        attire_reg.copy_from_slice(&bytes[81..85]);
        let kin = PresenceKinematics::new(
            Vec3::new(floats[0], floats[1], floats[2]),
            Quat::from_xyzw(floats[3], floats[4], floats[5], floats[6]),
            Quat::from_xyzw(floats[7], floats[8], floats[9], floats[10]),
            (floats[11], floats[12]),
        );
        Some((Self::new(slot, kin, (floats[13], floats[14], floats[15], floats[16]), (attire_main, attire_reg)), SPAN))
    }
}
