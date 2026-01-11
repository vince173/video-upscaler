//! Constants for the video upscaler application
//!
//! Centralizes magic values and configuration constants for better maintainability.

/// Default scale factor for sample generation
pub const DEFAULT_SAMPLE_SCALE: u32 = 4;

/// Default duration for sample generation (in seconds)
pub const DEFAULT_SAMPLE_DURATION_SECS: u64 = 10;

/// Maximum number of retry attempts for file deletion operations
pub const FILE_DELETE_RETRY_COUNT: u32 = 10;

/// Delay between file deletion retry attempts (in milliseconds)
pub const FILE_DELETE_RETRY_DELAY_MS: u64 = 500;

/// Grace period for FFmpeg cancellation (in milliseconds)
pub const CANCELLATION_GRACE_PERIOD_MS: u64 = 500;

/// Quality preset string identifiers
pub mod quality {
    pub const ULTRA_FAST: &str = "ultra_fast";
    pub const BALANCED: &str = "balanced";
    pub const HIGH_QUALITY: &str = "high_quality";
}

/// Hardware encoder string identifiers
pub mod encoder {
    pub const AMD: &str = "amd";
    pub const NVIDIA: &str = "nvidia";
    pub const INTEL: &str = "intel";
}
