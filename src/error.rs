// CrabMusic error types
// Centralized error definitions using thiserror

// Allow dead code for scaffolding - errors will be used throughout implementation
#![allow(dead_code)]

use thiserror::Error;

/// Main error type for CrabMusic application
#[derive(Error, Debug)]
pub enum CrabMusicError {
    /// Audio capture errors
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    /// DSP processing errors
    #[error("DSP error: {0}")]
    Dsp(#[from] DspError),

    /// Rendering errors
    #[error("Rendering error: {0}")]
    Rendering(#[from] RenderError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// I/O errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Audio capture and processing errors
#[derive(Error, Debug)]
pub enum AudioError {
    /// Audio device not available
    #[error(
        "Audio device not available. Please ensure an audio input device is connected and enabled."
    )]
    DeviceNotAvailable,

    /// Permission denied accessing audio device
    #[error("Permission denied accessing audio device. On Linux, ensure your user is in the 'audio' group.")]
    PermissionDenied,

    /// Invalid audio format
    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),

    /// Buffer overflow
    #[error("Audio buffer overflow. Try increasing buffer size in configuration.")]
    BufferOverflow,

    /// CPAL-specific error
    #[error("CPAL error: {0}")]
    CpalError(String),

    /// Audio configuration error
    #[error("Audio configuration error: {0}")]
    ConfigError(String),

    /// Audio stream error
    #[error("Audio stream error: {0}. The audio device may have been disconnected.")]
    StreamError(String),
}

/// DSP processing errors
#[derive(Error, Debug)]
pub enum DspError {
    /// Invalid FFT window size (must be power of 2)
    #[error("Invalid FFT window size: {0} (must be power of 2)")]
    InvalidWindowSize(usize),

    /// Invalid sample rate
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),

    /// Processing buffer too small
    #[error("Processing buffer too small: expected {expected}, got {actual}")]
    BufferTooSmall { expected: usize, actual: usize },
}

/// Terminal rendering errors
#[derive(Error, Debug)]
pub enum RenderError {
    /// Terminal initialization failed
    #[error("Failed to initialize terminal")]
    InitializationFailed,

    /// Terminal too small for visualization
    #[error("Terminal too small: minimum size is {min_width}x{min_height}")]
    TerminalTooSmall { min_width: u16, min_height: u16 },

    /// Rendering failed
    #[error("Rendering failed: {0}")]
    RenderingFailed(String),
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    /// Missing required field
    #[error("Missing required configuration field: {0}")]
    MissingField(String),

    /// Invalid value
    #[error("Invalid configuration value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },
}

// Type alias for Results using CrabMusicError
pub type Result<T> = std::result::Result<T, CrabMusicError>;
