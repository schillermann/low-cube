use world::PlanetaryWorld;
use crate::controls::KeyMotion;
use super::Avatar;

/// Declarative player hand gesture (short tap click or sustained press) for atomic behavioral specs.
#[derive(Clone, Copy, Debug)]
pub enum HandGesture {
    RightTap,
    RightHold(u32),
}

impl HandGesture {
    /// Sends this hand gesture to `avatar` against `world` ("Tell, Don't Ask").
    pub fn perform(self, avatar: &mut dyn Avatar, world: &mut dyn PlanetaryWorld) -> bool {
        match self {
            Self::RightTap => {
                let immediate = avatar.use_right_hand(world);
                let released = avatar.release_hand(world);
                immediate || released
            }
            Self::RightHold(frames) => {
                let mut changed = avatar.use_right_hand(world);
                let idle = KeyMotion::idle();
                for _ in 0..frames {
                    avatar.step_motion(1.0 / 60.0, &idle, world);
                    if avatar.step_digging(world) {
                        changed = true;
                    }
                }
                let _ = avatar.release_hand(world);
                changed
            }
        }
    }
}
