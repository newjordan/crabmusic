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

    /// Input device name (None = default input device)
    #[serde(default)]
    pub device_name: Option<String>,

    /// Output device name (None = default output device)
    #[serde(default)]
    pub output_device_name: Option<String>,
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
            output_device_name: None,
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
        use std::fs;
        use std::path::Path;

        // Check if file exists
        if !Path::new(path).exists() {
            return Err(ConfigError::FileNotFound(path.to_string()));
        }

        // Read file contents
        let contents = fs::read_to_string(path).map_err(|e| {
            ConfigError::InvalidFormat(format!("Failed to read file {}: {}", path, e))
        })?;

        // Parse YAML
        let config: Self = serde_yaml::from_str(&contents).map_err(|e| {
            ConfigError::InvalidFormat(format!("Failed to parse YAML: {}", e))
        })?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a YAML file, or return default if file doesn't exist
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// Loaded and validated AppConfig, or default config if file doesn't exist
    ///
    /// # Errors
    /// Returns `ConfigError` if file exists but cannot be parsed or is invalid
    pub fn load_or_default(path: &str) -> Result<Self, ConfigError> {
        use std::path::Path;

        if !Path::new(path).exists() {
            tracing::info!("Config file {} not found, using defaults", path);
            return Ok(Self::default());
        }

        Self::load(path)
    }

    /// Save configuration to a YAML file
    ///
    /// # Arguments
    /// * `path` - Path to save the configuration file
    ///
    /// # Errors
    /// Returns `ConfigError` if file cannot be written
    pub fn save(&self, path: &str) -> Result<(), ConfigError> {
        use std::fs;

        // Validate before saving
        self.validate()?;

        // Serialize to YAML
        let yaml = serde_yaml::to_string(self).map_err(|e| {
            ConfigError::InvalidFormat(format!("Failed to serialize to YAML: {}", e))
        })?;

        // Write to file
        fs::write(path, yaml).map_err(|e| {
            ConfigError::InvalidFormat(format!("Failed to write file {}: {}", path, e))
        })?;

        Ok(())
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
    current_config: std::sync::Arc<std::sync::RwLock<AppConfig>>,
    config_path: std::path::PathBuf,
    #[allow(dead_code)]
    watcher: Option<Box<dyn std::any::Any + Send>>,
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
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let config = AppConfig::load_or_default(config_path)?;
        Ok(Self {
            current_config: std::sync::Arc::new(std::sync::RwLock::new(config)),
            config_path: std::path::PathBuf::from(config_path),
            watcher: None,
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> AppConfig {
        self.current_config.read().unwrap().clone()
    }

    /// Reload configuration from file
    ///
    /// # Errors
    /// Returns `ConfigError` if configuration cannot be reloaded
    pub fn reload(&mut self) -> Result<(), ConfigError> {
        let new_config = AppConfig::load(self.config_path.to_str().unwrap())?;
        *self.current_config.write().unwrap() = new_config;
        tracing::info!("Configuration reloaded from {:?}", self.config_path);
        Ok(())
    }

    /// Enable hot-reload watching
    ///
    /// Watches the configuration file for changes and reloads automatically.
    ///
    /// # Errors
    /// Returns `ConfigError` if file watching cannot be set up
    pub fn enable_hot_reload(&mut self) -> Result<(), ConfigError> {
        use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
        use std::time::Duration;

        let config_arc = self.current_config.clone();
        let config_path = self.config_path.clone();

        // Create watcher
        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Only reload on modify events
                    if matches!(event.kind, EventKind::Modify(_)) {
                        tracing::debug!("Config file modified, reloading...");
                        match AppConfig::load(config_path.to_str().unwrap()) {
                            Ok(new_config) => {
                                *config_arc.write().unwrap() = new_config;
                                tracing::info!("Configuration hot-reloaded successfully");
                            }
                            Err(e) => {
                                tracing::error!("Failed to reload configuration: {}", e);
                            }
                        }
                    }
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_secs(1)),
        )
        .map_err(|e| {
            ConfigError::InvalidFormat(format!("Failed to create file watcher: {}", e))
        })?;

        // Watch the config file
        watcher
            .watch(&self.config_path, RecursiveMode::NonRecursive)
            .map_err(|e| {
                ConfigError::InvalidFormat(format!("Failed to watch config file: {}", e))
            })?;

        tracing::info!("Hot-reload enabled for {:?}", self.config_path);

        // Store watcher to keep it alive
        self.watcher = Some(Box::new(watcher));

        Ok(())
    }

    /// Check if hot-reload is enabled
    pub fn is_hot_reload_enabled(&self) -> bool {
        self.watcher.is_some()
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

    #[test]
    fn test_load_valid_yaml() {
        use std::fs;
        use std::io::Write;

        let temp_file = "test_config_valid.yaml";
        let yaml_content = r#"
audio:
  sample_rate: 48000
  channels: 2
  buffer_size: 2048
  buffer_capacity: 8192
  device_name: "default"

dsp:
  fft_size: 2048
  hop_size: 512
  window_type: "hann"
  frequency_range:
    min: 20.0
    max: 20000.0

visualization:
  sine_wave:
    amplitude_scale: 1.0
    frequency_scale: 1.0
    phase_offset: 0.0
    smoothing_factor: 0.8
    character_set: "blocks"

rendering:
  target_fps: 60
  enable_differential: true
  enable_double_buffer: true
"#;

        // Write test file
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        // Load and validate
        let config = AppConfig::load(temp_file);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.audio.sample_rate, 48000);
        assert_eq!(config.audio.channels, 2);
        assert_eq!(config.dsp.fft_size, 2048);
        assert_eq!(config.rendering.target_fps, 60);

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_load_invalid_yaml() {
        use std::fs;
        use std::io::Write;

        let temp_file = "test_config_invalid.yaml";
        let yaml_content = "invalid: yaml: content: [[[";

        // Write test file
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        // Try to load
        let config = AppConfig::load(temp_file);
        assert!(config.is_err());

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let config = AppConfig::load("nonexistent_file.yaml");
        assert!(config.is_err());
        match config {
            Err(ConfigError::FileNotFound(_)) => (),
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_load_or_default_nonexistent() {
        let config = AppConfig::load_or_default("nonexistent_file.yaml");
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.audio.sample_rate, 44100); // Default value
    }

    #[test]
    fn test_load_or_default_existing() {
        use std::fs;
        use std::io::Write;

        let temp_file = "test_config_or_default.yaml";
        let yaml_content = r#"
audio:
  sample_rate: 44100
  channels: 1
  buffer_size: 1024
  buffer_capacity: 4096
  device_name: "test"

dsp:
  fft_size: 1024
  hop_size: 256
  window_type: "hamming"
  frequency_range:
    min: 20.0
    max: 20000.0

visualization:
  sine_wave:
    amplitude_scale: 1.0
    frequency_scale: 1.0
    phase_offset: 0.0
    smoothing_factor: 0.8
    character_set: "basic"

rendering:
  target_fps: 30
  enable_differential: false
  enable_double_buffer: false
"#;

        // Write test file
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        // Load
        let config = AppConfig::load_or_default(temp_file);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.audio.sample_rate, 44100); // Custom value
        assert_eq!(config.rendering.target_fps, 30); // Custom value

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_save_config() {
        use std::fs;

        let temp_file = "test_config_save.yaml";
        let config = AppConfig::default();

        // Save
        let result = config.save(temp_file);
        assert!(result.is_ok());

        // Load back
        let loaded = AppConfig::load(temp_file);
        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.audio.sample_rate, config.audio.sample_rate);
        assert_eq!(loaded.dsp.fft_size, config.dsp.fft_size);

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_save_invalid_config() {
        let mut config = AppConfig::default();
        config.audio.sample_rate = 1000; // Invalid

        let result = config.save("test_invalid_save.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_manager_new() {
        use std::fs;
        use std::io::Write;

        let temp_file = "test_manager_new.yaml";
        let yaml_content = r#"
audio:
  sample_rate: 48000
  channels: 2
  buffer_size: 2048
  buffer_capacity: 8192
  device_name: "default"

dsp:
  fft_size: 2048
  hop_size: 512
  window_type: "hann"
  frequency_range:
    min: 20.0
    max: 20000.0

visualization:
  sine_wave:
    amplitude_scale: 1.0
    frequency_scale: 1.0
    phase_offset: 0.0
    smoothing_factor: 0.8
    character_set: "blocks"

rendering:
  target_fps: 60
  enable_differential: true
  enable_double_buffer: true
"#;

        // Write test file
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        // Create manager
        let manager = ConfigManager::new(temp_file);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        let config = manager.config();
        assert_eq!(config.audio.sample_rate, 48000);

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_config_manager_reload() {
        use std::fs;
        use std::io::Write;

        let temp_file = "test_manager_reload.yaml";
        let yaml_content_1 = r#"
audio:
  sample_rate: 44100
  channels: 2
  buffer_size: 2048
  buffer_capacity: 8192
  device_name: "default"

dsp:
  fft_size: 2048
  hop_size: 512
  window_type: "hann"
  frequency_range:
    min: 20.0
    max: 20000.0

visualization:
  sine_wave:
    amplitude_scale: 1.0
    frequency_scale: 1.0
    phase_offset: 0.0
    smoothing_factor: 0.8
    character_set: "blocks"

rendering:
  target_fps: 60
  enable_differential: true
  enable_double_buffer: true
"#;

        // Write initial file
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(yaml_content_1.as_bytes()).unwrap();

        // Create manager
        let mut manager = ConfigManager::new(temp_file).unwrap();
        assert_eq!(manager.config().audio.sample_rate, 44100);

        // Modify file
        let yaml_content_2 = r#"
audio:
  sample_rate: 48000
  channels: 2
  buffer_size: 2048
  buffer_capacity: 8192
  device_name: "default"

dsp:
  fft_size: 2048
  hop_size: 512
  window_type: "hann"
  frequency_range:
    min: 20.0
    max: 20000.0

visualization:
  sine_wave:
    amplitude_scale: 1.0
    frequency_scale: 1.0
    phase_offset: 0.0
    smoothing_factor: 0.8
    character_set: "blocks"

rendering:
  target_fps: 60
  enable_differential: true
  enable_double_buffer: true
"#;
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(yaml_content_2.as_bytes()).unwrap();

        // Reload
        let result = manager.reload();
        assert!(result.is_ok());
        assert_eq!(manager.config().audio.sample_rate, 48000);

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_config_manager_hot_reload_enabled() {
        use std::fs;
        use std::io::Write;

        let temp_file = "test_manager_hot_reload.yaml";
        let yaml_content = r#"
audio:
  sample_rate: 44100
  channels: 2
  buffer_size: 2048
  buffer_capacity: 8192
  device_name: "default"

dsp:
  fft_size: 2048
  hop_size: 512
  window_type: "hann"
  frequency_range:
    min: 20.0
    max: 20000.0

visualization:
  sine_wave:
    amplitude_scale: 1.0
    frequency_scale: 1.0
    phase_offset: 0.0
    smoothing_factor: 0.8
    character_set: "blocks"

rendering:
  target_fps: 60
  enable_differential: true
  enable_double_buffer: true
"#;

        // Write test file
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        // Create manager
        let mut manager = ConfigManager::new(temp_file).unwrap();
        assert!(!manager.is_hot_reload_enabled());

        // Enable hot-reload
        let result = manager.enable_hot_reload();
        assert!(result.is_ok());
        assert!(manager.is_hot_reload_enabled());

        // Cleanup
        fs::remove_file(temp_file).ok();
    }
}
