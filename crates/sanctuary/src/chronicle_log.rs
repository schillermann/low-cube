use world::WorldFact;

/// Ordered log of durable `WorldFact`s enabling loss-tolerant UDP streaming and late-joiner sync.
pub struct ChronicleLog {
    facts: Vec<WorldFact>,
}

impl ChronicleLog {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(facts: Vec<WorldFact>) -> Self {
        Self { facts }
    }

    /// Secondary constructor for an empty authoritative chronicle.
    pub const fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn total(&self) -> u32 {
        self.facts.len() as u32
    }

    pub fn record(&mut self, fact: WorldFact) {
        if fact.durable() {
            self.facts.push(fact);
        }
    }

    /// Collects an MTU-safe batch of durable facts starting at `ack_seq` (`<= 24` facts, `<= 1_100` bytes).
    pub fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)> {
        let start = ack_seq as usize;
        if start >= self.facts.len() {
            return None;
        }
        let limit = (start + 24).min(self.facts.len());
        let mut scratch = Vec::with_capacity(128);
        let mut used_bytes = 0usize;
        let mut batch = Vec::with_capacity(limit - start);
        for fact in &self.facts[start..limit] {
            scratch.clear();
            fact.inscribe(&mut scratch);
            if !batch.is_empty() && used_bytes + scratch.len() > 1_100 {
                break;
            }
            used_bytes += scratch.len();
            batch.push(fact.clone());
        }
        Some((ack_seq, batch))
    }
}
