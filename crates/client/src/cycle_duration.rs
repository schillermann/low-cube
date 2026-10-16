/// Diurnal cycle duration configured in minutes.
pub struct CycleDuration {
    seconds: f32,
}

impl CycleDuration {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(seconds: f32) -> Self {
        Self { seconds }
    }

    /// Secondary constructor initializing directly from a minute count.
    pub fn from_minutes(minutes: f32) -> Self {
        Self::new(minutes * 60.0)
    }

    /// Secondary constructor parsing a minute value from text.
    pub fn parse(text: &str) -> Option<Self> {
        let minutes = text.trim().parse::<f32>().ok()?;
        Some(Self::from_minutes(minutes))
    }

    /// Returns the resolved cycle duration in seconds.
    pub fn seconds(&self) -> f32 {
        self.seconds.max(0.0)
    }
}
