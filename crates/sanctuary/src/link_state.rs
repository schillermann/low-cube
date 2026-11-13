use world::CompanionPresence;
use crate::seal::{HandshakeIdentity, SessionSeal};
use crate::ward::ReturnPath;

/// Polymorphic session establishment phase without nullable cipher fields.
pub enum LinkPhase {
    Knocking {
        keys: HandshakeIdentity,
        retry_clock: f32,
    },
    Sealed {
        companion_id: u32,
        seal: SessionSeal,
        path: ReturnPath,
    },
}

/// Synchronized chronicle cursor, companion list, and outbound avatar presence.
pub struct LinkSync {
    pub(crate) ack_seq: u32,
    pub(crate) pulse_clock: f32,
    pub(crate) companions: Vec<CompanionPresence>,
    pub(crate) local_presence: CompanionPresence,
}

impl LinkSync {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        ack_seq: u32,
        pulse_clock: f32,
        companions: Vec<CompanionPresence>,
        local_presence: CompanionPresence,
    ) -> Self {
        Self { ack_seq, pulse_clock, companions, local_presence }
    }
}
