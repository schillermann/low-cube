use glam::Vec3;
use world::{CosmicFrame, VesselBerth};
use crate::audience::CHAT_EARSHOT_METERS;

pub const HELMET_TOUCH_METERS: f32 = 1.85;
pub const SUIT_RADIO_METERS: f32 = 32.0;

/// Physical acoustic & radio state of a speaking or listening companion (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChatLocus {
    frame: CosmicFrame,
    berth: VesselBerth,
    position: Vec3,
    radio_enabled: bool,
}

impl ChatLocus {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        frame: CosmicFrame,
        berth: VesselBerth,
        position: Vec3,
        radio_enabled: bool,
    ) -> Self {
        Self { frame, berth, position, radio_enabled }
    }

    fn within_meters(self, other: Self, range: f32) -> bool {
        self.frame == other.frame && (self.position - other.position).length_squared() <= range * range
    }
}

/// Pure-OOP contract determining whether speech propagates from `speaker` to `listener`.
pub trait ChatMedium {
    fn carries(&self, speaker: ChatLocus, listener: ChatLocus) -> bool;
}

/// Origin `ChatMedium` conducting solid-borne voice through touching helmet visors (`<= 1.85 m`) even in vacuum under radio silence.
#[derive(Clone, Copy, Debug)]
pub struct HelmetContactMedium {
    reach_meters: f32,
}

impl HelmetContactMedium {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(reach_meters: f32) -> Self {
        Self { reach_meters }
    }

    pub const fn visor_touch() -> Self {
        Self::new(HELMET_TOUCH_METERS)
    }
}

impl ChatMedium for HelmetContactMedium {
    fn carries(&self, speaker: ChatLocus, listener: ChatLocus) -> bool {
        speaker.within_meters(listener, self.reach_meters)
    }
}

/// `ChatMedium` decorator carrying voice through pressurized spacecraft decks (`VesselBerth::Aboard`) at any speed.
#[derive(Clone, Copy, Debug)]
pub struct CabinIntercomMedium<M> {
    origin: M,
}

impl<M> CabinIntercomMedium<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: M) -> Self {
        Self { origin }
    }
}

impl<M: ChatMedium> ChatMedium for CabinIntercomMedium<M> {
    fn carries(&self, speaker: ChatLocus, listener: ChatLocus) -> bool {
        (speaker.berth.aboard_any() && speaker.berth == listener.berth)
            || self.origin.carries(speaker, listener)
    }
}

/// `ChatMedium` decorator carrying open acoustic waves (`<= 96 m`) strictly on atmosphere-bearing worlds (`0 m` in vacuum).
#[derive(Clone, Copy, Debug)]
pub struct AtmosphericVoiceMedium<M> {
    origin: M,
    earshot_meters: f32,
}

impl<M> AtmosphericVoiceMedium<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: M, earshot_meters: f32) -> Self {
        Self { origin, earshot_meters }
    }
}

impl<M: ChatMedium> ChatMedium for AtmosphericVoiceMedium<M> {
    fn carries(&self, speaker: ChatLocus, listener: ChatLocus) -> bool {
        (speaker.frame.atmospheric_sound() && speaker.within_meters(listener, self.earshot_meters))
            || self.origin.carries(speaker, listener)
    }
}

/// `ChatMedium` decorator carrying short-range EVA suit VHF (`<= 32 m`) unless silenced via `~` prefix.
#[derive(Clone, Copy, Debug)]
pub struct SuitTransceiverMedium<M> {
    origin: M,
    radio_meters: f32,
}

impl<M> SuitTransceiverMedium<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: M, radio_meters: f32) -> Self {
        Self { origin, radio_meters }
    }
}

impl<M: ChatMedium> ChatMedium for SuitTransceiverMedium<M> {
    fn carries(&self, speaker: ChatLocus, listener: ChatLocus) -> bool {
        (speaker.radio_enabled
            && listener.radio_enabled
            && speaker.within_meters(listener, self.radio_meters))
            || self.origin.carries(speaker, listener)
    }
}

impl SuitTransceiverMedium<AtmosphericVoiceMedium<CabinIntercomMedium<HelmetContactMedium>>> {
    /// Assembles the full Hard-Sci-Fi acoustic/radio medium chain (`SuitTransceiver -> AtmosphericVoice -> CabinIntercom -> HelmetContact`).
    pub const fn cosmos() -> Self {
        Self::new(
            AtmosphericVoiceMedium::new(
                CabinIntercomMedium::new(HelmetContactMedium::visor_touch()),
                CHAT_EARSHOT_METERS,
            ),
            SUIT_RADIO_METERS,
        )
    }
}
