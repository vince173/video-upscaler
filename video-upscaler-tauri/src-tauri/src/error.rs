//! Error types for the video upscaler

use thiserror::Error;

/// Main error type for the video upscaler application
#[derive(Error, Debug)]
pub enum UpscalerError {
    #[error("Video decoding failed: {0}")]
    DecodingError(String),

    #[error("Video encoding failed: {0}")]
    EncodingError(String),

    #[error("AI inference failed: {0}")]
    InferenceError(String),

    #[error("Model loading failed: {0}")]
    ModelLoadError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Processing cancelled")]
    Cancelled,
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, UpscalerError>;

impl From<image::ImageError> for UpscalerError {
    fn from(err: image::ImageError) -> Self {
        UpscalerError::ImageError(err.to_string())
    }
}
