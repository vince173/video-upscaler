//! Configuration for video processing

/// Quality/Speed preset for video processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityPreset {
    /// Ultra fast: Lower quality, maximum speed (bicubic + ultrafast preset + CRF 23)
    UltraFast,
    /// Balanced: Good quality, decent speed (bicubic + fast preset + CRF 20)
    Balanced,
    /// High Quality: Best quality, slower (lanczos + medium preset + CRF 18)
    HighQuality,
}

/// Hardware encoder type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareEncoder {
    /// Software encoding (libx264)
    None,
    /// AMD AMF (Advanced Media Framework) - for AMD GPUs
    AMD,
    /// NVIDIA NVENC - for NVIDIA GPUs
    NVIDIA,
    /// Intel Quick Sync Video - for Intel GPUs
    Intel,
}

/// Configuration for the video upscaler
#[derive(Debug, Clone)]
pub struct Config {
    /// Quality preset (UltraFast/Balanced/HighQuality)
    pub quality_preset: QualityPreset,
    /// Hardware encoder to use (None/AMD/NVIDIA/Intel)
    pub hardware_encoder: HardwareEncoder,
    /// Upscale factor (2, 4, etc.)
    pub scale: u32,
    /// Preview mode: process only first N seconds (0 = full video)
    pub preview_duration_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            quality_preset: QualityPreset::UltraFast,
            hardware_encoder: HardwareEncoder::None,
            scale: 4,
            preview_duration_secs: 0,
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the quality preset
    pub fn with_quality_preset(mut self, preset: QualityPreset) -> Self {
        self.quality_preset = preset;
        self
    }

    /// Set the hardware encoder
    pub fn with_hardware_encoder(mut self, encoder: HardwareEncoder) -> Self {
        self.hardware_encoder = encoder;
        self
    }

    /// Set the scale factor
    pub fn with_scale(mut self, scale: u32) -> Self {
        self.scale = scale;
        self
    }

    /// Set preview duration (0 = full video, 60 = 1 minute preview)
    pub fn with_preview_duration(mut self, secs: u64) -> Self {
        self.preview_duration_secs = secs;
        self
    }
}
