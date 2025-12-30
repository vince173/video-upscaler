//! Configuration for video processing

/// Processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingMode {
    /// Fast mode: FFmpeg scaling + sharpening (real-time speed)
    Fast,
}

/// Quality/Speed preset for Fast mode
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
    /// Processing mode (Fast)
    pub mode: ProcessingMode,
    /// Quality preset for Fast mode (UltraFast/Balanced/HighQuality)
    pub quality_preset: QualityPreset,
    /// Hardware encoder to use (None/AMD/NVIDIA/Intel)
    pub hardware_encoder: HardwareEncoder,
    /// Upscale factor (2, 4, etc.)
    pub scale: u32,
    /// Output video quality (bitrate in bps)
    pub bitrate: u64,
    /// Preview mode: process only first N seconds (0 = full video)
    pub preview_duration_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: ProcessingMode::Fast,
            quality_preset: QualityPreset::UltraFast,
            hardware_encoder: HardwareEncoder::None,
            scale: 4,
            bitrate: 5_000_000,
            preview_duration_secs: 0,
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the processing mode
    pub fn with_mode(mut self, mode: ProcessingMode) -> Self {
        self.mode = mode;
        self
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

    /// Set the output bitrate
    pub fn with_bitrate(mut self, bitrate: u64) -> Self {
        self.bitrate = bitrate;
        self
    }

    /// Set preview duration (0 = full video, 60 = 1 minute preview)
    pub fn with_preview_duration(mut self, secs: u64) -> Self {
        self.preview_duration_secs = secs;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.scale != 2 && self.scale != 4 {
            return Err("Scale must be 2 or 4".to_string());
        }
        if self.bitrate == 0 {
            return Err("Bitrate must be greater than 0".to_string());
        }
        Ok(())
    }
}
