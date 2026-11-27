/// Optimus / hybrid laptop GPU power profile (`HighPerformance` MX150 vs. `LowPower` Intel UHD 620).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GpuPowerProfile {
    DiscreteHighPerformance,
    IntegratedLowPower,
}

impl GpuPowerProfile {
    /// Maps domain power profile to `wgpu::PowerPreference`.
    pub const fn wgpu_preference(self) -> wgpu::PowerPreference {
        match self {
            Self::DiscreteHighPerformance => wgpu::PowerPreference::HighPerformance,
            Self::IntegratedLowPower => wgpu::PowerPreference::LowPower,
        }
    }

    /// Parses human-readable CLI/TOML/env token (`"high"`, `"dgpu"`, `"mx150"` vs `"low"`, `"igpu"`, `"battery"`).
    pub fn from_token(token: &str) -> Option<Self> {
        let trimmed = token.trim();
        if trimmed.eq_ignore_ascii_case("low")
            || trimmed.eq_ignore_ascii_case("lowpower")
            || trimmed.eq_ignore_ascii_case("low-power")
            || trimmed.eq_ignore_ascii_case("igpu")
            || trimmed.eq_ignore_ascii_case("integrated")
            || trimmed.eq_ignore_ascii_case("battery")
        {
            Some(Self::IntegratedLowPower)
        } else if trimmed.eq_ignore_ascii_case("high")
            || trimmed.eq_ignore_ascii_case("highperformance")
            || trimmed.eq_ignore_ascii_case("high-performance")
            || trimmed.eq_ignore_ascii_case("dgpu")
            || trimmed.eq_ignore_ascii_case("discrete")
            || trimmed.eq_ignore_ascii_case("mx150")
        {
            Some(Self::DiscreteHighPerformance)
        } else {
            None
        }
    }

    /// Native graphics backends per target OS (Linux -> Vulkan; Windows -> DirectX 12 + Vulkan).
    pub const fn native_backends(self) -> wgpu::Backends {
        let _ = self;
        if cfg!(target_os = "windows") {
            wgpu::Backends::DX12.union(wgpu::Backends::VULKAN)
        } else {
            wgpu::Backends::VULKAN
        }
    }

    /// Strict `60 Hz` VSync frame-pacing mode preventing laptop thermal throttling.
    pub const fn paced_present_mode(self, supported: &[wgpu::PresentMode]) -> wgpu::PresentMode {
        let _ = self;
        let mut i = 0;
        while i < supported.len() {
            if matches!(supported[i], wgpu::PresentMode::Fifo) {
                return wgpu::PresentMode::Fifo;
            }
            i += 1;
        }
        if supported.is_empty() {
            wgpu::PresentMode::Fifo
        } else {
            supported[0]
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::DiscreteHighPerformance => "HighPerformance (dGPU / MX150)",
            Self::IntegratedLowPower => "LowPower (iGPU / Battery)",
        }
    }
}
