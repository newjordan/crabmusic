// Configuration module
// Handles loading and managing application configuration

// Allow dead code for scaffolding - will be used in CONFIG-001
#![allow(dead_code)]

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};

/// Main application configuration
///
/// Contains all user-configurable settings for CrabMusic.
/// Loaded from YAML files and validated on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Audio capture configuration
    #[serde(default)]
    pub audio: AudioConfig,

    /// DSP processing configuration
    #[serde(default)]
    pub dsp: DspConfig,

    /// Visualization configuration
    #[serde(default)]
    pub visualization: VisualizationConfig,

    /// Rendering configuration
    #[serde(default)]
    pub rendering: RenderingConfig,
}

/// Audio capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate in Hz (e.g., 44100, 48000)
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    /// Number of audio channels (1 = mono, 2 = stereo)
    #[serde(default = "default_channels")]
    pub channels: u16,

    /// Ring buffer capacity in samples
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: usize,

    /// Device name (None = default device)
    #[serde(default)]
    pub device_name: Option<String>,
}

/// DSP processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspConfig {
    /// FFT window size (must be power of 2)
    #[serde(default = "default_fft_size")]
    pub fft_size: usize,

    /// Smoothing factor for audio parameters (0.0-1.0)
    #[serde(default = "default_smoothing")]
    pub smoothing: f32,

    /// Bass frequency range (Hz)
    #[serde(default = "default_bass_range")]
    pub bass_range: (f32, f32),

    /// Mid frequency range (Hz)
    #[serde(default = "default_mid_range")]
    pub mid_range: (f32, f32),

    /// Treble frequency range (Hz)
    #[serde(default = "default_treble_range")]
    pub treble_range: (f32, f32),
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Visualizer type ("sine_wave", "spectrum", "oscilloscope")
    #[serde(default = "default_visualizer_type")]
    pub visualizer_type: String,

    /// Character set type ("basic", "extended", "blocks", "shading", "dots", "lines", "braille")
    #[serde(default = "default_character_set")]
    pub character_set: String,

    /// Sine wave specific configuration
    #[serde(default)]
    pub sine_wave: SineWaveConfig,
}

/// Sine wave visualizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SineWaveConfig {
    /// Amplitude multiplier
    #[serde(default = "default_amplitude")]
    pub amplitude: f32,

    /// Frequency multiplier
    #[serde(default = "default_frequency")]
    pub frequency: f32,

    /// Phase offset
    #[serde(default)]
    pub phase: f32,

    /// Wave thickness in rows
    #[serde(default = "default_thickness")]
    pub thickness: usize,

    /// Smoothing factor (0.0-1.0)
    #[serde(default = "default_wave_smoothing")]
    pub smoothing: f32,
}

/// Rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// Target frames per second
    #[serde(default = "default_fps")]
    pub target_fps: u32,

    /// Minimum terminal width
    #[serde(default = "default_min_width")]
    pub min_width: u16,

    /// Minimum terminal height
    #[serde(default = "default_min_height")]
    pub min_height: u16,
}

// Default value functions
fn default_sample_rate() -> u32 { 44100 }
fn default_channels() -> u16 { 2 }
fn default_buffer_capacity() -> usize { 8192 }
fn default_fft_size() -> usize { 2048 }
fn default_smoothing() -> f32 { 0.1 }
fn default_bass_range() -> (f32, f32) { (20.0, 250.0) }
fn default_mid_range() -> (f32, f32) { (250.0, 4000.0) }
fn default_treble_range() -> (f32, f32) { (4000.0, 20000.0) }
fn default_visualizer_type() -> String { "sine_wave".to_string() }
fn default_character_set() -> String { "blocks".to_string() }
fn default_amplitude() -> f32 { 1.0 }
fn default_frequency() -> f32 { 1.0 }
fn default_thickness() -> usize { 3 }
fn default_wave_smoothing() -> f32 { 0.15 }
fn default_fps() -> u32 { 60 }
fn default_min_width() -> u16 { 40 }
fn default_min_height() -> u16 { 12 }

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: default_sample_rate(),
            channels: default_channels(),
            buffer_capacity: default_buffer_capacity(),
            device_name: None,
        }
    }
}

impl Default for DspConfig {
    fn default() -> Self {
        Self {
            fft_size: default_fft_size(),
            smoothing: default_smoothing(),
            bass_range: default_bass_range(),
            mid_range: default_mid_range(),
            treble_range: default_treble_range(),
        }
    }
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            visualizer_type: default_visualizer_type(),
            character_set: default_character_set(),
            sine_wave: SineWaveConfig::default(),
        }
    }
}

impl Default for SineWaveConfig {
    fn default() -> Self {
        Self {
            amplitude: default_amplitude(),
            frequency: default_frequency(),
            phase: 0.0,
            thickness: default_thickness(),
            smoothing: default_wave_smoothing(),
        }
    }
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            target_fps: default_fps(),
            min_width: default_min_width(),
            min_height: default_min_height(),
        }
    }
}

impl AppConfig {
    /// Load configuration from a YAML file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// Loaded and validated AppConfig
    ///
    /// # Errors
    /// Returns `ConfigError` if file cannot be read or parsed
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        // TODO: Implement in CONFIG-002
        Err(ConfigError::FileNotFound(path.to_string()))
    }

    /// Get default configuration
    ///
    /// # Returns
    /// AppConfig with sensible defaults
    pub fn default_config() -> Self {
        Self {
            audio: AudioConfig::default(),
            dsp: DspConfig::default(),
            visualization: VisualizationConfig::default(),
            rendering: RenderingConfig::default(),
        }
    }

    /// Validate configuration values
    ///
    /// # Errors
    /// Returns `ConfigError` if any values are invalid
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate audio config
        if self.audio.sample_rate < 8000 || self.audio.sample_rate > 192000 {
            return Err(ConfigError::InvalidValue {
                field: "audio.sample_rate".to_string(),
                reason: "must be between 8000 and 192000".to_string(),
            });
        }

        if self.audio.channels < 1 || self.audio.channels > 2 {
            return Err(ConfigError::InvalidValue {
                field: "audio.channels".to_string(),
                reason: "must be 1 or 2".to_string(),
            });
        }

        if self.audio.buffer_capacity < 1024 {
            return Err(ConfigError::InvalidValue {
                field: "audio.buffer_capacity".to_string(),
                reason: "must be at least 1024".to_string(),
            });
        }

        // Validate DSP config
        if !self.dsp.fft_size.is_power_of_two() {
            return Err(ConfigError::InvalidValue {
                field: "dsp.fft_size".to_string(),
                reason: "must be a power of 2".to_string(),
            });
        }

        if self.dsp.fft_size < 256 || self.dsp.fft_size > 16384 {
            return Err(ConfigError::InvalidValue {
                field: "dsp.fft_size".to_string(),
                reason: "must be between 256 and 16384".to_string(),
            });
        }

        if !(0.0..=1.0).contains(&self.dsp.smoothing) {
            return Err(ConfigError::InvalidValue {
                field: "dsp.smoothing".to_string(),
                reason: "must be between 0.0 and 1.0".to_string(),
            });
        }

        // Validate visualization config
        let valid_visualizers = ["sine_wave", "spectrum", "oscilloscope"];
        if !valid_visualizers.contains(&self.visualization.visualizer_type.as_str()) {
            return Err(ConfigError::InvalidValue {
                field: "visualization.visualizer_type".to_string(),
                reason: format!("must be one of: {:?}", valid_visualizers),
            });
        }

        let valid_charsets = ["basic", "extended", "blocks", "shading", "dots", "lines", "braille"];
        if !valid_charsets.contains(&self.visualization.character_set.as_str()) {
            return Err(ConfigError::InvalidValue {
                field: "visualization.character_set".to_string(),
                reason: format!("must be one of: {:?}", valid_charsets),
            });
        }

        if !(0.0..=1.0).contains(&self.visualization.sine_wave.smoothing) {
            return Err(ConfigError::InvalidValue {
                field: "visualization.sine_wave.smoothing".to_string(),
                reason: "must be between 0.0 and 1.0".to_string(),
            });
        }

        if self.visualization.sine_wave.thickness < 1 || self.visualization.sine_wave.thickness > 10 {
            return Err(ConfigError::InvalidValue {
                field: "visualization.sine_wave.thickness".to_string(),
                reason: "must be between 1 and 10".to_string(),
            });
        }

        // Validate rendering config
        if self.rendering.target_fps < 1 || self.rendering.target_fps > 120 {
            return Err(ConfigError::InvalidValue {
                field: "rendering.target_fps".to_string(),
                reason: "must be between 1 and 120".to_string(),
            });
        }

        if self.rendering.min_width < 20 {
            return Err(ConfigError::InvalidValue {
                field: "rendering.min_width".to_string(),
                reason: "must be at least 20".to_string(),
            });
        }

        if self.rendering.min_height < 10 {
            return Err(ConfigError::InvalidValue {
                field: "rendering.min_height".to_string(),
                reason: "must be at least 10".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Configuration manager
///
/// Manages configuration loading, validation, and hot-reload watching.
pub struct ConfigManager {
    // TODO: Define fields in CONFIG-001
    // Will include:
    // - current_config: AppConfig
    // - config_path: PathBuf
    // - watcher: Option<FileWatcher> (for hot-reload)
    current_config: AppConfig,
}

impl ConfigManager {
    /// Create a new configuration manager
    ///
    /// # Arguments
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    /// A new ConfigManager instance with loaded configuration
    ///
    /// # Errors
    /// Returns `ConfigError` if configuration cannot be loaded
    pub fn new(_config_path: &str) -> Result<Self, ConfigError> {
        // TODO: Implement in CONFIG-001
        Ok(Self {
            current_config: AppConfig::default(),
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &AppConfig {
        // TODO: Implement in CONFIG-001
        &self.current_config
    }

    /// Enable hot-reload watching
    ///
    /// Watches the configuration file for changes and reloads automatically.
    ///
    /// # Errors
    /// Returns `ConfigError` if file watching cannot be set up
    #[cfg(feature = "hot-reload")]
    pub fn enable_hot_reload(&mut self) -> Result<(), ConfigError> {
        // TODO: Implement in CONFIG-003
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_audio_config() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 2);
        assert_eq!(config.buffer_capacity, 8192);
        assert!(config.device_name.is_none());
    }

    #[test]
    fn test_default_dsp_config() {
        let config = DspConfig::default();
        assert_eq!(config.fft_size, 2048);
        assert_eq!(config.smoothing, 0.1);
        assert_eq!(config.bass_range, (20.0, 250.0));
        assert_eq!(config.mid_range, (250.0, 4000.0));
        assert_eq!(config.treble_range, (4000.0, 20000.0));
    }

    #[test]
    fn test_default_visualization_config() {
        let config = VisualizationConfig::default();
        assert_eq!(config.visualizer_type, "sine_wave");
        assert_eq!(config.character_set, "blocks");
    }

    #[test]
    fn test_default_sine_wave_config() {
        let config = SineWaveConfig::default();
        assert_eq!(config.amplitude, 1.0);
        assert_eq!(config.frequency, 1.0);
        assert_eq!(config.phase, 0.0);
        assert_eq!(config.thickness, 3);
        assert_eq!(config.smoothing, 0.15);
    }

    #[test]
    fn test_default_rendering_config() {
        let config = RenderingConfig::default();
        assert_eq!(config.target_fps, 60);
        assert_eq!(config.min_width, 40);
        assert_eq!(config.min_height, 12);
    }

    #[test]
    fn test_validate_invalid_sample_rate() {
        let mut config = AppConfig::default();
        config.audio.sample_rate = 1000; // Too low
        assert!(config.validate().is_err());

        config.audio.sample_rate = 200000; // Too high
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_channels() {
        let mut config = AppConfig::default();
        config.audio.channels = 0;
        assert!(config.validate().is_err());

        config.audio.channels = 3;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_buffer_capacity() {
        let mut config = AppConfig::default();
        config.audio.buffer_capacity = 512; // Too small
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_fft_size() {
        let mut config = AppConfig::default();
        config.dsp.fft_size = 1000; // Not power of 2
        assert!(config.validate().is_err());

        config.dsp.fft_size = 128; // Too small
        assert!(config.validate().is_err());

        config.dsp.fft_size = 32768; // Too large
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_smoothing() {
        let mut config = AppConfig::default();
        config.dsp.smoothing = -0.1;
        assert!(config.validate().is_err());

        config.dsp.smoothing = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_visualizer_type() {
        let mut config = AppConfig::default();
        config.visualization.visualizer_type = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_character_set() {
        let mut config = AppConfig::default();
        config.visualization.character_set = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_thickness() {
        let mut config = AppConfig::default();
        config.visualization.sine_wave.thickness = 0;
        assert!(config.validate().is_err());

        config.visualization.sine_wave.thickness = 11;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_fps() {
        let mut config = AppConfig::default();
        config.rendering.target_fps = 0;
        assert!(config.validate().is_err());

        config.rendering.target_fps = 121;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_min_dimensions() {
        let mut config = AppConfig::default();
        config.rendering.min_width = 10;
        assert!(config.validate().is_err());

        config.rendering.min_height = 5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new("test.yaml");
        assert!(manager.is_ok());
    }

    #[test]
    fn test_config_manager_get_config() {
        let manager = ConfigManager::new("test.yaml").unwrap();
        let config = manager.config();
        assert!(config.validate().is_ok());
    }
}
