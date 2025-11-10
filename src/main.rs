// CrabMusic - Real-time ASCII music visualizer
// Main application entry point

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Module declarations
mod audio;
mod config;
mod dsp;
mod effects;
mod error;
mod img;
mod rendering;
mod video;
mod visualization;

#[cfg(windows)]
use audio::WasapiLoopbackDevice;
use audio::{AudioCaptureDevice, AudioOutputDevice, AudioRingBuffer, CpalAudioDevice};
use dsp::DspProcessor;
use effects::EffectPipeline;
use rendering::TerminalRenderer;
use visualization::{
    character_sets::{get_all_character_sets, get_character_set, CharacterSet, CharacterSetType},
    color_schemes::{ColorScheme, ColorSchemeType},
    FlowerOfLifeConfig, FlowerOfLifeVisualizer, GridBuffer, HistoryTVChannelVisualizer,
    ImageChannelVisualizer, MandalaConfig, MandalaVisualizer, ObjViewerVisualizer,
    NightNightVisualizer, OscilloscopeConfig, OscilloscopeVisualizer, Raycaster3DVisualizer,
    ScrollDirection, SineWaveConfig, SineWaveVisualizer, SpectrogramVisualizer, SpectrumConfig,
    SpectrumMapping, SpectrumVisualizer, TriggerSlope, VideoChannelVisualizer, Visualizer,
    WaveformMode, WaveformTunnelVisualizer,
};

/// Global shutdown flag
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// CrabMusic - Real-time ASCII music visualizer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Target FPS (overrides config file)
    #[arg(short, long, value_name = "FPS")]
    fps: Option<u32>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug logging (more verbose than -v)
    #[arg(short, long)]
    debug: bool,

    /// Test mode: render test pattern instead of audio visualization
    #[arg(short, long)]
    test: bool,

    /// Amplitude sensitivity multiplier (overrides config file)
    #[arg(long, value_name = "FLOAT")]
    sensitivity: Option<f32>,

    /// Input audio device name (overrides config file)
    #[arg(long, value_name = "NAME")]
    device: Option<String>,

    /// Output audio device name for playback (overrides config file)
    #[arg(long, value_name = "NAME")]
    output_device: Option<String>,

    /// List available audio devices and exit
    #[arg(long)]
    list_devices: bool,

    /// Use Windows WASAPI loopback to capture system audio (Windows only, no virtual cable needed)
    #[cfg(windows)]
    #[arg(long)]
    loopback: bool,

    /// Character set to use (basic, extended, blocks, shading, dots, lines, braille)
    #[arg(long, value_name = "SET")]
    charset: Option<String>,

    /// Sample rate in Hz (overrides config file)
    #[arg(long, value_name = "HZ")]
    sample_rate: Option<u32>,

    /// FFT size (must be power of 2: 512, 1024, 2048, 4096, 8192)
    #[arg(long, value_name = "SIZE")]
    fft_size: Option<usize>,

    /// Enable hot-reload of configuration file
    #[arg(long)]
    hot_reload: bool,

    /// Disable audio output (visualization only, no playback)
    #[arg(long)]
    no_audio_output: bool,

    /// Show frequency labels on spectrum analyzer for debugging/calibration
    #[arg(long)]
    show_labels: bool,

    /// Show version information
    #[arg(long)]
    version_info: bool,

    /// Play a video file in the terminal using Braille rendering (feature-gated)
    #[arg(long, value_name = "FILE")]
    video: Option<String>,

    /// Display an image file as Braille art (requires feature: image)
    #[arg(long, value_name = "FILE")]
    image: Option<String>,

    /// Start image drag-and-drop mode (paste file paths to render)
    #[arg(long)]
    image_drop: bool,

    /// Start morph between two images (provide both)
    #[arg(long, value_name = "FILE")]
    morph_a: Option<String>,

    /// Second image for morph (B)
    #[arg(long, value_name = "FILE")]
    morph_b: Option<String>,

    /// Morph duration in milliseconds for A→B leg
    #[arg(long, value_name = "MS")]
    morph_duration: Option<u64>,
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Handle version info
    if args.version_info {
        print_version_info();
        return Ok(());
    }

    // Handle list devices
    if args.list_devices {
        list_audio_devices()?;
        return Ok(());
    }

    // Initialize logging
    init_logging(args.verbose, args.debug)?;

    tracing::info!("CrabMusic v{} starting...", env!("CARGO_PKG_VERSION"));

    // Video mode takes over if requested
    if let Some(path) = args.video.as_deref() {
        tracing::info!("Starting video mode for file: {}", path);
        return video::run_video_playback(path);
    }
    // Image morph mode takes over if both morph_a and morph_b are provided
    if let (Some(ref a), Some(ref b)) = (args.morph_a.as_ref(), args.morph_b.as_ref()) {
        tracing::info!("Starting image morph mode: {} <-> {}", a, b);
        return img::render_image(a.as_str(), Some(b.as_str()), args.morph_duration);
    }

    // Image modes take over if requested
    if let Some(path) = args.image.as_deref() {
        tracing::info!("Starting image mode for file: {}", path);
        return img::render_image(path, None, None);
    }
    if args.image_drop {
        tracing::info!("Starting image drag-and-drop mode");
        return img::drop_loop();
    }

    // Load configuration
    let config_path = args.config.as_deref().unwrap_or("config.yaml");
    let mut config = config::AppConfig::load_or_default(config_path)?;

    // Apply CLI overrides
    if let Some(fps) = args.fps {
        config.rendering.target_fps = fps;
        tracing::info!("Overriding FPS from CLI: {}", fps);
    }

    if let Some(sensitivity) = args.sensitivity {
        config.visualization.sine_wave.amplitude = sensitivity;
        tracing::info!("Overriding sensitivity from CLI: {}", sensitivity);
    }

    if let Some(ref device) = args.device {
        config.audio.device_name = Some(device.clone());
        tracing::info!("Overriding input audio device from CLI: {}", device);
    }

    if let Some(output_device) = args.output_device {
        config.audio.output_device_name = Some(output_device.clone());
        tracing::info!("Overriding output audio device from CLI: {}", output_device);
    }

    if let Some(sample_rate) = args.sample_rate {
        config.audio.sample_rate = sample_rate;
        tracing::info!("Overriding sample rate from CLI: {}", sample_rate);
    }

    if let Some(fft_size) = args.fft_size {
        config.dsp.fft_size = fft_size;
        tracing::info!("Overriding FFT size from CLI: {}", fft_size);
    }

    if let Some(charset) = args.charset.as_ref() {
        config.visualization.character_set = charset.clone();
        tracing::info!("Overriding character set from CLI: {}", charset);
    }

    // Validate configuration
    config.validate().context("Invalid configuration")?;

    // Setup Ctrl+C handler
    setup_shutdown_handler()?;

    // Determine if we should use loopback (persisted via config; CLI overrides)
    #[cfg(windows)]
    let use_loopback = if args.loopback {
        true
    } else if args.device.is_some() {
        false
    } else {
        config.audio.use_loopback
    };
    #[cfg(not(windows))]
    let use_loopback = false;

    // Persist the resolved loopback preference for next runs
    // If config file doesn't exist yet, save current preference to create it
    if !std::path::Path::new(config_path).exists() {
        let mut cfg_to_save = config.clone();
        cfg_to_save.audio.use_loopback = use_loopback;
        if let Err(e) = cfg_to_save.save(config_path) {
            tracing::warn!("Failed to save initial config (creating file): {}", e);
        } else {
            tracing::info!("Created {} with loopback preference", config_path);
        }
    }

    if config.audio.use_loopback != use_loopback {
        config.audio.use_loopback = use_loopback;
        if let Err(e) = config.save(config_path) {
            tracing::warn!("Failed to save config (loopback preference): {}", e);
        } else {
            tracing::info!("Saved loopback preference to {}", config_path);
        }
    }

    // Create and run application
    let app =
        Application::new_with_config(config, args.no_audio_output, use_loopback, args.show_labels)?;

    // Friendly tip: if loopback is off, mic starts OFF too until you press 'M'
    if !use_loopback {
        tracing::warn!("No system loopback. MIC starts OFF. Press 'M' to enable mic input, or run with --loopback to capture system audio.");
    }

    if args.test {
        app.run_test_mode()?;
    } else {
        app.run()?;
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

/// Visualizer mode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisualizerMode {
    SineWave,
    Spectrum,
    Oscilloscope,
    XYOscilloscope,
    Spectrogram,
    WaveformTunnel,
    FlowerOfLife,
    Mandala,
    Raycaster3D,
    ObjViewer,
    NightNight,
    Image,
    Video,
    HistoryTV,
}

impl VisualizerMode {
    /// Get the next visualizer mode in the cycle
    fn next(&self) -> Self {
        match self {
            VisualizerMode::SineWave => VisualizerMode::Spectrum,
            VisualizerMode::Spectrum => VisualizerMode::Oscilloscope,
            VisualizerMode::Oscilloscope => VisualizerMode::XYOscilloscope,
            VisualizerMode::XYOscilloscope => VisualizerMode::Spectrogram,
            VisualizerMode::Spectrogram => VisualizerMode::WaveformTunnel,
            VisualizerMode::WaveformTunnel => VisualizerMode::FlowerOfLife,
            VisualizerMode::FlowerOfLife => VisualizerMode::Mandala,
            VisualizerMode::Mandala => VisualizerMode::Raycaster3D,
            VisualizerMode::Raycaster3D => VisualizerMode::ObjViewer,
            VisualizerMode::ObjViewer => VisualizerMode::NightNight,
            VisualizerMode::NightNight => VisualizerMode::Image,
            VisualizerMode::Image => VisualizerMode::Video,
            VisualizerMode::Video => VisualizerMode::HistoryTV,
            VisualizerMode::HistoryTV => VisualizerMode::SineWave,
        }
    }

    /// Get the previous visualizer mode in the cycle
    fn prev(&self) -> Self {
        match self {
            VisualizerMode::SineWave => VisualizerMode::HistoryTV,
            VisualizerMode::Spectrum => VisualizerMode::SineWave,
            VisualizerMode::Oscilloscope => VisualizerMode::Spectrum,
            VisualizerMode::XYOscilloscope => VisualizerMode::Oscilloscope,
            VisualizerMode::Spectrogram => VisualizerMode::XYOscilloscope,
            VisualizerMode::WaveformTunnel => VisualizerMode::Spectrogram,
            VisualizerMode::FlowerOfLife => VisualizerMode::WaveformTunnel,
            VisualizerMode::Mandala => VisualizerMode::FlowerOfLife,
            VisualizerMode::Raycaster3D => VisualizerMode::Mandala,
            VisualizerMode::ObjViewer => VisualizerMode::Raycaster3D,
            VisualizerMode::NightNight => VisualizerMode::ObjViewer,
            VisualizerMode::Image => VisualizerMode::NightNight,
            VisualizerMode::Video => VisualizerMode::Image,
            VisualizerMode::HistoryTV => VisualizerMode::Video,
        }
    }

    /// Get the name of the visualizer mode
    fn name(&self) -> &str {
        match self {
            VisualizerMode::SineWave => "Sine Wave",
            VisualizerMode::Spectrum => "Spectrum Analyzer",
            VisualizerMode::Oscilloscope => "Oscilloscope",
            VisualizerMode::XYOscilloscope => "XY Oscilloscope (Lissajous)",
            VisualizerMode::Spectrogram => "Spectrogram",
            VisualizerMode::WaveformTunnel => "Waveform Tunnel",
            VisualizerMode::FlowerOfLife => "Flower of Life",
            VisualizerMode::Mandala => "Mandala",
            VisualizerMode::Raycaster3D => "Raycaster 3D",
            VisualizerMode::ObjViewer => "OBJ Viewer",
            VisualizerMode::NightNight => "Night Night",
            VisualizerMode::Image => "Image Viewer",
            VisualizerMode::Video => "Video Player",
            VisualizerMode::HistoryTV => "History TV Channel",
        }
    }

    /// Get zero-based index of the mode (for channel number)
    fn index(&self) -> usize {
        match self {
            VisualizerMode::SineWave => 0,
            VisualizerMode::Spectrum => 1,
            VisualizerMode::Oscilloscope => 2,
            VisualizerMode::XYOscilloscope => 3,
            VisualizerMode::Spectrogram => 4,
            VisualizerMode::WaveformTunnel => 5,
            VisualizerMode::FlowerOfLife => 6,
            VisualizerMode::Mandala => 7,
            VisualizerMode::Raycaster3D => 8,
            VisualizerMode::ObjViewer => 9,
            VisualizerMode::NightNight => 10,
            VisualizerMode::Image => 11,
            VisualizerMode::Video => 12,
            VisualizerMode::HistoryTV => 13,
        }
    }

    /// Total number of channels
    fn count() -> usize {
        14
    }
}

/// Application state
struct Application {
    audio_device: Box<dyn AudioCaptureDevice>,
    audio_output: Option<AudioOutputDevice>,
    dsp_processor: DspProcessor,
    visualizer: Box<dyn Visualizer>,
    effect_pipeline: EffectPipeline,
    renderer: TerminalRenderer,
    #[allow(dead_code)] // Held for lifetime management
    ring_buffer: Arc<AudioRingBuffer>,
    target_fps: u32,
    sample_rate: u32,
    /// True if we're capturing system audio via WASAPI loopback (Windows)
    use_loopback: bool,
    current_charset: CharacterSet,
    #[allow(dead_code)] // Reserved for future charset cycling feature
    charset_index: usize,
    /// When false and not using loopback, we ignore mic input for processing
    microphone_enabled: bool,
    sensitivity_multiplier: f32,
    show_labels: bool,
    visualizer_mode: VisualizerMode,
    color_scheme: ColorScheme,
    color_scheme_index: usize,
    last_key_press: Instant,
    key_debounce_ms: u64,
    // Oscilloscope configuration state
    osc_show_grid: bool,
    osc_waveform_mode: WaveformMode,
    osc_trigger_slope: TriggerSlope,
    // Spectrum configuration state
    spectrum_peak_hold: bool,
    spectrum_mapping: SpectrumMapping,
    spectrum_range_preset_index: usize,
    // Raycaster 3D configuration state
    ray3d_mode: crate::visualization::ray_tracer::RenderMode,
    ray3d_brightness_boost: f32,
    ray3d_wire_step_rad: f32,
    ray3d_wire_tol_rad: f32,
    ray3d_rotation_speed_y: f32,
    ray3d_auto_rotate: bool,
    // Model Viewer configuration state
    model_viewer_index: usize,
    model_viewer_auto_rotate: bool,

    // Effect control state
    selected_effect_for_intensity: Option<String>, // Track which effect to adjust intensity for
    // UI overlays
    show_channel_number: bool,
    // File input prompt state (Image/Video channels)
    file_prompt_active: bool,
    file_prompt_buffer: String,
    file_prompt_error: Option<String>,
    // Suppress char-by-char events immediately after a paste to avoid duplication
    paste_suppress_deadline: Option<Instant>,
}

impl Application {
    /// Create a new application instance with configuration
    fn new_with_config(
        config: config::AppConfig,
        no_audio_output: bool,
        use_loopback: bool,
        show_labels: bool,
    ) -> Result<Self> {
        tracing::info!("Initializing components with configuration...");
        tracing::debug!(
            "Configuration: sample_rate={}, fft_size={}, fps={}",
            config.audio.sample_rate,
            config.dsp.fft_size,
            config.rendering.target_fps
        );

        // Create ring buffer for audio pipeline
        // REDUCED from 10 to 4 for lower latency - just enough to prevent dropouts
        // With 512-sample chunks @ 44.1kHz, this is ~46ms of buffering (4 * 11.6ms)
        let ring_buffer = Arc::new(AudioRingBuffer::new(4));
        tracing::debug!("Ring buffer created with capacity: 4 (low-latency mode)");

        // Initialize audio capture with retry logic
        tracing::debug!("Initializing audio capture device...");
        let audio_device = Self::init_audio_capture_with_retry(
            ring_buffer.clone(),
            config.audio.device_name.clone(),
            use_loopback,
        )?;
        tracing::info!("Audio capture device initialized successfully");

        // Get the actual sample rate from the audio device
        let actual_sample_rate = audio_device.get_config().sample_rate;
        if actual_sample_rate != config.audio.sample_rate {
            tracing::warn!(
                "Audio device sample rate ({} Hz) differs from config ({} Hz). Using device sample rate.",
                actual_sample_rate,
                config.audio.sample_rate
            );
        }

        // Initialize audio output (optional, with graceful degradation)
        let audio_output = if no_audio_output {
            tracing::info!("Audio output disabled by CLI flag");
            None
        } else {
            tracing::debug!("Initializing audio output device...");
            match AudioOutputDevice::new_with_device(config.audio.output_device_name.clone()) {
                Ok(output) => {
                    tracing::info!("Audio output device initialized successfully");
                    Some(output)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to initialize audio output: {}. Continuing without audio playback.",
                        e
                    );
                    None
                }
            }
        };

        // Initialize DSP processor with actual sample rate from audio device
        tracing::debug!("Initializing DSP processor...");
        let dsp_processor = DspProcessor::new(actual_sample_rate, config.dsp.fft_size)
            .context("Failed to initialize DSP processor")?;
        tracing::info!(
            "DSP processor initialized: sample_rate={}, fft_size={}",
            actual_sample_rate,
            config.dsp.fft_size
        );

        // Determine initial character set
        let charset_type = match config.visualization.character_set.as_str() {
            "basic" => CharacterSetType::Basic,
            "extended" => CharacterSetType::Extended,
            "blocks" => CharacterSetType::Blocks,
            "shading" => CharacterSetType::Shading,
            "dots" => CharacterSetType::Dots,
            "lines" => CharacterSetType::Lines,
            "braille" => CharacterSetType::Braille,
            "smooth64" | "smooth_64" => CharacterSetType::Smooth64,
            "smooth128" | "smooth_128" => CharacterSetType::Smooth128,
            "smooth256" | "smooth_256" => CharacterSetType::Smooth256,
            _ => CharacterSetType::Smooth64, // Default to smooth gradients!
        };
        let current_charset = get_character_set(charset_type);
        let charset_index = match charset_type {
            CharacterSetType::Basic => 0,
            CharacterSetType::Extended => 1,
            CharacterSetType::Blocks => 2,
            CharacterSetType::Shading => 3,
            CharacterSetType::Dots => 4,
            CharacterSetType::Lines => 5,
            CharacterSetType::Braille => 6,
            CharacterSetType::Smooth64 => 7,
            CharacterSetType::Smooth128 => 8,
            CharacterSetType::Smooth256 => 9,
        };

        // Initialize visualizer (start with sine wave)
        tracing::debug!("Initializing visualizer...");
        let viz_config = SineWaveConfig {
            amplitude_sensitivity: config.visualization.sine_wave.amplitude,
            ..Default::default()
        };
        let visualizer: Box<dyn Visualizer> = Box::new(SineWaveVisualizer::new(
            viz_config.clone(),
            current_charset.clone(),
        ));
        let visualizer_mode = VisualizerMode::SineWave;
        tracing::info!(
            "Visualizer initialized: type=sine_wave, sensitivity={}",
            config.visualization.sine_wave.amplitude
        );

        // Initialize terminal renderer
        tracing::debug!("Initializing terminal renderer...");
        let renderer = TerminalRenderer::new().context("Failed to initialize terminal renderer")?;
        let (width, height) = renderer.dimensions();
        tracing::info!("Terminal renderer initialized: {}x{}", width, height);
        tracing::info!("All components initialized successfully");

        // Initialize color scheme (start with monochrome)
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let color_scheme_index = 0;

        // Initialize effect pipeline with effects
        let mut effect_pipeline = EffectPipeline::new();
        // Add bloom effect (glow for bright elements with Gaussian blur)
        effect_pipeline.add_effect(Box::new(effects::bloom::BloomEffect::new(0.7, 2)));
        // Add scanline effect (CRT-style horizontal lines)
        effect_pipeline.add_effect(Box::new(effects::scanline::ScanlineEffect::new(2)));
        // Add phosphor glow effect (temporal persistence for CRT-style trails)
        effect_pipeline.add_effect(Box::new(effects::phosphor::PhosphorGlowEffect::new(
            0.3, 0.7,
        )));
        // Add grid overlay effect for testing (optional)
        // effect_pipeline.add_effect(Box::new(effects::grid_overlay::GridOverlayEffect::new(10)));
        effect_pipeline.set_enabled(false); // Start with effects disabled
        tracing::debug!(
            "Effect pipeline initialized with Bloom + Scanline + Phosphor effects (disabled)"
        );

        Ok(Self {
            audio_device,
            audio_output,
            dsp_processor,
            visualizer,
            effect_pipeline,
            renderer,
            ring_buffer,
            target_fps: config.rendering.target_fps,
            sample_rate: actual_sample_rate,
            use_loopback,
            current_charset,
            charset_index,
            microphone_enabled: false, // Start with microphone disabled by default
            sensitivity_multiplier: 1.0, // Start at 100% sensitivity
            show_labels,
            visualizer_mode,
            color_scheme,
            color_scheme_index,
            last_key_press: Instant::now(),
            key_debounce_ms: 200, // 200ms debounce = max 5 key presses per second
            // Oscilloscope defaults
            osc_show_grid: true,
            osc_waveform_mode: WaveformMode::LineAndFill,
            osc_trigger_slope: TriggerSlope::Positive,
            // Spectrum defaults
            spectrum_peak_hold: true, // Start with peaks enabled
            spectrum_mapping: SpectrumMapping::NoteBars,
            spectrum_range_preset_index: 1, // Default to A1–A5
            // Raycaster 3D defaults
            ray3d_mode: crate::visualization::ray_tracer::RenderMode::Wireframe {
                step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
                tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            },
            ray3d_rotation_speed_y: 0.6,
            ray3d_auto_rotate: true,

            ray3d_brightness_boost: 0.0,
            ray3d_wire_step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
            ray3d_wire_tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            // Model Viewer defaults
            model_viewer_index: 0, // Start with first model (cube)
            model_viewer_auto_rotate: true,
            // Effect control defaults
            selected_effect_for_intensity: None, // No effect selected initially
            // UI overlays defaults
            show_channel_number: true,
            // File prompt defaults
            file_prompt_active: false,
            file_prompt_buffer: String::new(),
            file_prompt_error: None,
            paste_suppress_deadline: None,
        })
    }

    /// Create a new application instance (legacy method for backward compatibility)
    #[allow(dead_code)]
    fn new(target_fps: u32, sensitivity: f32) -> Result<Self> {
        let mut config = config::AppConfig::default();
        config.rendering.target_fps = target_fps;
        config.visualization.sine_wave.amplitude = sensitivity;
        Self::new_with_config(config, false, false, false)
    }

    /// Initialize audio capture with retry logic
    ///
    /// Attempts to initialize audio capture with exponential backoff retry strategy.
    /// Retries 3 times with delays of 100ms, 500ms, and 1000ms.
    fn init_audio_capture_with_retry(
        ring_buffer: Arc<AudioRingBuffer>,
        device_name: Option<String>,
        use_loopback: bool,
    ) -> Result<Box<dyn AudioCaptureDevice>> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAYS_MS: [u64; 3] = [100, 500, 1000];

        // Use WASAPI loopback on Windows if requested
        #[cfg(windows)]
        if use_loopback {
            tracing::info!("Using Windows WASAPI loopback for system audio capture");
            match WasapiLoopbackDevice::new(ring_buffer.clone()) {
                Ok(device) => {
                    tracing::info!("WASAPI loopback device initialized successfully");
                    return Ok(Box::new(device));
                }
                Err(e) => {
                    tracing::error!("Failed to initialize WASAPI loopback: {}", e);
                    tracing::info!("Falling back to standard input device (microphone) via CPAL");
                    // Continue to CPAL fallback below
                }
            }
        }

        // Fall back to CPAL device (microphone or specified device)
        for attempt in 0..MAX_RETRIES {
            match CpalAudioDevice::new_with_device(ring_buffer.clone(), device_name.clone()) {
                Ok(device) => {
                    if attempt > 0 {
                        tracing::info!(
                            "Audio capture initialized successfully after {} retries",
                            attempt
                        );
                    }
                    return Ok(Box::new(device));
                }
                Err(e) => {
                    if attempt < MAX_RETRIES - 1 {
                        let delay = RETRY_DELAYS_MS[attempt as usize];
                        tracing::warn!(
                            "Failed to initialize audio capture (attempt {}/{}): {}. Retrying in {}ms...",
                            attempt + 1,
                            MAX_RETRIES,
                            e,
                            delay
                        );
                        std::thread::sleep(Duration::from_millis(delay));
                    } else {
                        tracing::error!(
                            "Failed to initialize audio capture after {} attempts: {}",
                            MAX_RETRIES,
                            e
                        );
                        return Err(e).context(format!(
                            "Failed to initialize audio capture after {} attempts. \
                             Please ensure:\n\
                             - An audio input device is connected and enabled\n\
                             - Your audio system is running (PulseAudio/PipeWire on Linux)\n\
                             - You have permission to access audio devices (check 'audio' group on Linux)",
                            MAX_RETRIES
                        ));
                    }
                }
            }
        }

        unreachable!()
    }

    /// Cycle to the next character set
    #[allow(dead_code)] // Reserved for future charset cycling feature
    fn next_charset(&mut self) {
        let charsets = get_all_character_sets();
        self.charset_index = (self.charset_index + 1) % charsets.len();
        self.current_charset = charsets[self.charset_index].clone();
        tracing::info!("Switched to character set: {}", self.current_charset.name);
    }

    /// Cycle to the next color scheme
    fn next_color_scheme(&mut self) {
        let schemes = ColorSchemeType::all();
        self.color_scheme_index = (self.color_scheme_index + 1) % schemes.len();
        let scheme_type = schemes[self.color_scheme_index];
        self.color_scheme = ColorScheme::new(scheme_type);
        // Recreate visualizer to apply new color scheme
        self.recreate_visualizer();
        tracing::info!("Switched to color scheme: {}", scheme_type.name());
    }

    /// Toggle microphone on/off
    fn toggle_microphone(&mut self) {
        self.microphone_enabled = !self.microphone_enabled;
        let status = if self.microphone_enabled { "ON" } else { "OFF" };
        tracing::info!("Microphone toggled: {}", status);
    }

    /// Toggle effects pipeline on/off (master toggle)
    fn toggle_effects(&mut self) {
        let new_state = !self.effect_pipeline.is_enabled();
        self.effect_pipeline.set_enabled(new_state);
        let status = if new_state { "ON" } else { "OFF" };
        self.selected_effect_for_intensity = None; // Master toggle affects all effects
        tracing::info!("Effects toggled: {}", status);
    }

    /// Toggle a specific effect by name
    fn toggle_effect(&mut self, effect_name: &str) {
        if let Some(effect) = self.effect_pipeline.get_effect_mut(effect_name) {
            let new_state = !effect.is_enabled();
            effect.set_enabled(new_state);
            let status = if new_state { "ON" } else { "OFF" };
            self.selected_effect_for_intensity = Some(effect_name.to_string());
            tracing::info!("{} effect toggled: {}", effect_name, status);
        }
    }

    /// Increase intensity of selected effect (or all if none selected)
    fn increase_effect_intensity(&mut self) {
        if let Some(ref effect_name) = self.selected_effect_for_intensity {
            // Adjust specific effect
            if let Some(effect) = self.effect_pipeline.get_effect_mut(effect_name) {
                let old_intensity = effect.intensity();
                let new_intensity = (old_intensity + 0.1).min(1.0);
                effect.set_intensity(new_intensity);
                tracing::info!(
                    "{} intensity: {:.1}% → {:.1}%",
                    effect_name,
                    old_intensity * 100.0,
                    new_intensity * 100.0
                );
            }
        } else {
            // Adjust all effects - collect names as owned Strings to avoid borrow issues
            let effect_names: Vec<String> = self
                .effect_pipeline
                .effect_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            for effect_name in effect_names {
                if let Some(effect) = self.effect_pipeline.get_effect_mut(&effect_name) {
                    let new_intensity = (effect.intensity() + 0.1).min(1.0);
                    effect.set_intensity(new_intensity);
                }
            }
            tracing::info!("All effects intensity increased by 10%");
        }
    }

    /// Decrease intensity of selected effect (or all if none selected)
    fn decrease_effect_intensity(&mut self) {
        if let Some(ref effect_name) = self.selected_effect_for_intensity {
            // Adjust specific effect
            if let Some(effect) = self.effect_pipeline.get_effect_mut(effect_name) {
                let old_intensity = effect.intensity();
                let new_intensity = (old_intensity - 0.1).max(0.0);
                effect.set_intensity(new_intensity);
                tracing::info!(
                    "{} intensity: {:.1}% → {:.1}%",
                    effect_name,
                    old_intensity * 100.0,
                    new_intensity * 100.0
                );
            }
        } else {
            // Adjust all effects - collect names as owned Strings to avoid borrow issues
            let effect_names: Vec<String> = self
                .effect_pipeline
                .effect_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            for effect_name in effect_names {
                if let Some(effect) = self.effect_pipeline.get_effect_mut(&effect_name) {
                    let new_intensity = (effect.intensity() - 0.1).max(0.0);
                    effect.set_intensity(new_intensity);
                }
            }
            tracing::info!("All effects intensity decreased by 10%");
        }
    }

    /// Increase sensitivity by 10%
    fn increase_sensitivity(&mut self) {
        self.sensitivity_multiplier = (self.sensitivity_multiplier + 0.1).min(5.0);
        self.recreate_visualizer();
        tracing::info!(
            "Sensitivity increased to {:.1}x",
            self.sensitivity_multiplier
        );
    }

    /// Decrease sensitivity by 10%
    fn decrease_sensitivity(&mut self) {
        self.sensitivity_multiplier = (self.sensitivity_multiplier - 0.1).max(0.1);
        self.recreate_visualizer();
        tracing::info!(
            "Sensitivity decreased to {:.1}x",
            self.sensitivity_multiplier
        );
    }

    /// Set sensitivity to a preset value (1-9 = 0.5x to 4.5x)
    fn set_sensitivity_preset(&mut self, preset: u8) {
        if (1..=9).contains(&preset) {
            self.sensitivity_multiplier = 0.5 * preset as f32;
            self.recreate_visualizer();
            tracing::info!(
                "Sensitivity preset {} set to {:.1}x",
                preset,
                self.sensitivity_multiplier
            );
        }
    }

    /// Switch to the next visualizer mode
    fn next_visualizer_mode(&mut self) {
        self.visualizer_mode = self.visualizer_mode.next();
        self.recreate_visualizer();
        tracing::info!("Switched to visualizer: {}", self.visualizer_mode.name());
    }

    /// Switch to the previous visualizer mode
    fn prev_visualizer_mode(&mut self) {
        self.visualizer_mode = self.visualizer_mode.prev();
        self.recreate_visualizer();
        tracing::info!("Switched to visualizer: {}", self.visualizer_mode.name());
    }

    /// Recreate visualizer with current mode and sensitivity
    fn recreate_visualizer(&mut self) {
        self.visualizer = match self.visualizer_mode {
            VisualizerMode::SineWave => {
                let mut config = SineWaveConfig::default();
                config.amplitude_sensitivity *= self.sensitivity_multiplier;
                config.frequency_sensitivity *= self.sensitivity_multiplier;
                config.thickness_sensitivity *= self.sensitivity_multiplier;
                let mut viz = SineWaveVisualizer::new(config, self.current_charset.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Spectrum => {
                let mut config = SpectrumConfig::default();
                config.amplitude_sensitivity *= self.sensitivity_multiplier;
                config.show_labels = self.show_labels;
                config.peak_hold_enabled = self.spectrum_peak_hold;
                config.mapping = self.spectrum_mapping;
                if matches!(self.spectrum_mapping, SpectrumMapping::NoteBars) {
                    let (_label, min, max) = match self.spectrum_range_preset_index % 3 {
                        0 => ("A2-A5", 110.0, 880.0),
                        1 => ("A1-A5", 55.0, 880.0),
                        _ => ("A1-A6", 55.0, 1760.0),
                    };
                    config.freq_min = min;
                    config.freq_max = max;
                }
                let mut viz =
                    SpectrumVisualizer::new(config, self.sample_rate, self.current_charset.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Oscilloscope => {
                let mut config = OscilloscopeConfig::default();
                config.amplitude_sensitivity *= self.sensitivity_multiplier;
                // Apply oscilloscope-specific settings
                config.show_grid = self.osc_show_grid;
                config.waveform_mode = self.osc_waveform_mode;
                config.trigger_slope = self.osc_trigger_slope;
                let mut viz = OscilloscopeVisualizer::new(config);
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::XYOscilloscope => {
                let mut config = visualization::XYOscilloscopeConfig::default();
                config.sensitivity *= self.sensitivity_multiplier;
                let mut viz = visualization::XYOscilloscopeVisualizer::new(config);
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Spectrogram => {
                let viz = SpectrogramVisualizer::new(
                    self.color_scheme.clone(),
                    ScrollDirection::Up, // Default to scrolling up
                );
                Box::new(viz)
            }
            VisualizerMode::WaveformTunnel => {
                let viz = WaveformTunnelVisualizer::new(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::FlowerOfLife => {
                let mut viz = FlowerOfLifeVisualizer::new(FlowerOfLifeConfig::default());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Mandala => {
                let mut viz = MandalaVisualizer::new(MandalaConfig::default());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Raycaster3D => {
                let mode = match self.ray3d_mode {
                    crate::visualization::ray_tracer::RenderMode::Wireframe { .. } => {
                        crate::visualization::ray_tracer::RenderMode::Wireframe {
                            step_rad: self.ray3d_wire_step_rad,
                            tol_rad: self.ray3d_wire_tol_rad,
                        }
                    }
                    crate::visualization::ray_tracer::RenderMode::Solid => {
                        crate::visualization::ray_tracer::RenderMode::Solid
                    }
                };
                let mut viz = Raycaster3DVisualizer::new_with(mode, self.ray3d_brightness_boost);
                viz.set_rotation_speed_y(self.ray3d_rotation_speed_y);
                viz.set_auto_rotate(self.ray3d_auto_rotate);
                Box::new(viz)
            }
            VisualizerMode::ObjViewer => {
                let mut viz = ObjViewerVisualizer::new_with_model_index(self.model_viewer_index);
                viz.set_auto_rotate(self.model_viewer_auto_rotate);
                Box::new(viz)
            }
            VisualizerMode::NightNight => {
                let viz = NightNightVisualizer::new(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Image => {
                let mut viz = ImageChannelVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Video => {
                let mut viz = VideoChannelVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::HistoryTV => {
                let mut viz = HistoryTVChannelVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
        };
    }

    /// Apply character set mapping and colors to the grid
    #[allow(dead_code)] // Reserved for future charset mapping feature
    fn apply_charset_to_grid(&self, grid: &mut GridBuffer) {
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let cell = grid.get_cell(x, y);
                // Map intensity (0.0 = space, 1.0 = filled) based on character
                let intensity = match cell.character {
                    ' ' => 0.0,
                    '.' => 0.1,
                    ':' => 0.2,
                    '-' => 0.3,
                    '=' => 0.4,
                    '+' => 0.5,
                    '*' => 0.6,
                    '#' => 0.7,
                    '%' => 0.8,
                    '@' => 0.9,
                    '█' => 1.0,
                    '▓' => 0.75,
                    '▒' => 0.5,
                    '░' => 0.25,
                    _ => 0.5, // Default for unknown characters
                };
                let new_char = self.current_charset.get_char(intensity);

                // Apply color based on intensity
                if let Some(color) = self.color_scheme.get_color(intensity) {
                    grid.set_cell_with_color(x, y, new_char, color);
                } else {
                    grid.set_cell(x, y, new_char);
                }
            }
        }
    }

    /// Try to load a file into the current channel (Image/Video)
    fn try_load_current_channel_path(&mut self, path: &str) -> Result<(), String> {
        match self.visualizer_mode {
            VisualizerMode::Image => {
                if let Some(v) = (&mut *self.visualizer as &mut dyn std::any::Any)
                    .downcast_mut::<crate::visualization::ImageChannelVisualizer>()
                {
                    v.try_load(path)
                } else {
                    Err("internal visualizer type mismatch (Image)".into())
                }
            }
            VisualizerMode::Video => {
                if let Some(v) = (&mut *self.visualizer as &mut dyn std::any::Any)
                    .downcast_mut::<crate::visualization::VideoChannelVisualizer>()
                {
                    v.try_load(path)
                } else {
                    Err("internal visualizer type mismatch (Video)".into())
                }
            }
            _ => Err("File input is only available in Image/Video channels".into()),
        }
    }

    /// Add UI overlay with renderer info and controls
    fn add_ui_overlay(&self, grid: &mut GridBuffer) {
        let visualizer_name = self.visualizer.name();
        let scheme_type = self.color_scheme.scheme_type();
        let color_scheme_name = scheme_type.name();
        let mic_status = if self.microphone_enabled {
            "MIC:ON"
        } else {
            "MIC:OFF"
        };

        // Build effect status string with individual effect states
        let mut fx_parts = Vec::new();
        if self.effect_pipeline.is_enabled() {
            fx_parts.push("FX:ON".to_string());
        } else {
            fx_parts.push("FX:OFF".to_string());
        }

        // Show individual effect states and intensities
        for effect_name in self.effect_pipeline.effect_names() {
            if let Some(effect) = self.effect_pipeline.get_effect(effect_name) {
                let short_name = match effect_name {
                    "Bloom" => "B",
                    "Scanline" => "S",
                    "Phosphor" => "P",
                    _ => &effect_name[0..1],
                };
                let intensity_pct = (effect.intensity() * 100.0) as u8;
                if effect.is_enabled() {
                    fx_parts.push(format!("{}:{}%", short_name, intensity_pct));
                } else {
                    fx_parts.push(format!("{}:off", short_name));
                }
            }
        }
        let fx_status = fx_parts.join(" ");

        // Optional channel prefix (e.g., "CH 3/11: ")
        let channel_prefix = if self.show_channel_number {
            format!(
                "CH {}/{}: ",
                self.visualizer_mode.index() + 1,
                VisualizerMode::count()
            )
        } else {
            String::new()
        };

        let info_text = if self.visualizer_mode == VisualizerMode::Oscilloscope {
            format!(
                " {}{} | {} | {} | {} | ←/→ V:chan I:num O:color E:fx B:bloom S:scan H:phosphor []:intensity G:grid F:fill T:trigger M:mic Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        } else if self.visualizer_mode == VisualizerMode::Spectrum {
            let map_name = match self.spectrum_mapping {
                SpectrumMapping::NoteBars => "NOTES",
                SpectrumMapping::LogBars => "LOG",
            };
            if matches!(self.spectrum_mapping, SpectrumMapping::NoteBars) {
                let (range_label, _min, _max) = match self.spectrum_range_preset_index % 3 {
                    0 => ("A2-A5", 110.0, 880.0),
                    1 => ("A1-A5", 55.0, 880.0),
                    _ => ("A1-A6", 55.0, 1760.0),
                };
                format!(
                    " {}{} | {} | {} | {} | ←/→ V:chan I:num O:color E:fx B:bloom S:scan H:phosphor []:intensity P:peaks L:labels N:map({}) R:range({}) M:mic +/-:sens Q:quit ",
                    channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status, map_name, range_label
                )
            } else {
                format!(
                    " {}{} | {} | {} | {} | ←/→ V:chan I:num O:color E:fx B:bloom S:scan H:phosphor []:intensity P:peaks L:labels N:map({}) M:mic +/-:sens Q:quit ",
                    channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status, map_name
                )
            }
        } else if self.visualizer_mode == VisualizerMode::Raycaster3D {
            let step_deg = self.ray3d_wire_step_rad.to_degrees();
            let tol = self.ray3d_wire_tol_rad;
            let mode_name = match self.ray3d_mode {
                crate::visualization::ray_tracer::RenderMode::Wireframe { .. } => "WF",
                crate::visualization::ray_tracer::RenderMode::Solid => "SOL",
            };
            let auto_label = if self.ray3d_auto_rotate { "ON" } else { "OFF" };
            let rot_speed = self.ray3d_rotation_speed_y;
            format!(
                " {}{}({}) | {} | {} | {} | W:mode G/H:step({:.0}°) T/Y:thick({:.3}) J/K:rot({:.1}) R:auto({}) Up/Down:bright ←/→ V:chan I:num O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, mode_name, color_scheme_name, mic_status, fx_status, step_deg, tol, rot_speed, auto_label
            )
        } else if self.visualizer_mode == VisualizerMode::ObjViewer {
            let (model_name, line_px, dot_px) = if let Some(viz) = (&*self.visualizer as &dyn std::any::Any)
                .downcast_ref::<crate::visualization::ObjViewerVisualizer>()
            {
                let (lp, dp) = viz.wire_px().unwrap_or((1, 2));
                (viz.model_name(), lp, dp)
            } else {
                ("Unknown", 1, 2)
            };
            let auto_label = if self.model_viewer_auto_rotate { "ON" } else { "OFF" };
            format!(
                " {}{} | {} | {} | {} | Model: {} | W:mode A/D:yaw J/K:pitch ,/.:roll G/H:line({}px) T/Y:dot({}px) Z/X:zoom F:focus R:auto({}) Up/Down:switch ←/→ V:chan I:num O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status, model_name, line_px, dot_px, auto_label
            )
        } else {
            format!(
                " {}{} | {} | {} | {} | ←/→ V:chan I:num O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        };

        // Draw info bar at the top
        let start_x = (grid.width().saturating_sub(info_text.len())) / 2;
        for (i, ch) in info_text.chars().enumerate() {
            let x = start_x + i;
            if x < grid.width() {
                grid.set_cell(x, 0, ch);
            }
        }

        // Secondary hint: Image/Video temporarily disabled
        if matches!(
            self.visualizer_mode,
            VisualizerMode::Image | VisualizerMode::Video
        ) {
            let y = 1usize;
            let hint = "White noise mode: image/video temporarily disabled";
            for (i, ch) in hint.chars().enumerate() {
                if i < grid.width() {
                    grid.set_cell(i, y, ch);
                } else {
                    break;
                }
            }
        }
    }

    /// Run the main application loop
    fn run(mut self) -> Result<()> {
        tracing::info!("Starting main loop at {} FPS", self.target_fps);

        // Start audio capture
        self.audio_device
            .start_capture()
            .context("Failed to start audio capture")?;

        // Start audio output (playback) if enabled
        if let Some(audio_output) = self.audio_output.as_mut() {
            audio_output
                .start_playback()
                .context("Failed to start audio playback")?;
        }

        // Calculate frame time
        let frame_duration = Duration::from_secs_f32(1.0 / self.target_fps as f32);

        // Performance tracking
        let mut frame_count = 0;
        let mut fps_timer = Instant::now();
        let mut total_frame_time = Duration::ZERO;
        let mut max_frame_time = Duration::ZERO;
        let mut min_frame_time = Duration::from_secs(1);

        loop {
            let frame_start = Instant::now();

            // Check for shutdown signal
            if SHUTDOWN.load(Ordering::Relaxed) {
                tracing::info!("Shutdown signal received");
                break;
            }

            // Check for keyboard/paste input
            if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    match ev {
                        Event::Paste(_s) => {
                            // Image/Video inline path input temporarily disabled; ignore paste
                        }
                        Event::Key(KeyEvent { code, .. }) => {
                            // When file prompt is active, handle editing without debounce
                            if self.file_prompt_active {
                                match code {
                                    KeyCode::Esc => {
                                        self.file_prompt_active = false;
                                        self.file_prompt_buffer.clear();
                                        self.file_prompt_error = None;
                                        self.paste_suppress_deadline = None;
                                    }
                                    KeyCode::Enter => {
                                        let candidate_owned = self.file_prompt_buffer.clone();
                                        let path =
                                            candidate_owned.trim().trim_matches('"').to_string();
                                        if path.is_empty() {
                                            self.file_prompt_error = Some("Empty path".to_string());
                                        } else {
                                            match self.try_load_current_channel_path(&path) {
                                                Ok(_) => {
                                                    self.file_prompt_active = false;
                                                    self.file_prompt_error = None;
                                                    self.paste_suppress_deadline = None;
                                                }
                                                Err(err) => {
                                                    self.file_prompt_error = Some(err);
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        self.file_prompt_buffer.pop();
                                    }
                                    KeyCode::Char(c) => {
                                        // Avoid duplicating paste content from terminals that also emit Char events
                                        if let Some(deadline) = self.paste_suppress_deadline {
                                            if Instant::now() <= deadline { /* skip */
                                            } else {
                                                self.file_prompt_buffer.push(c);
                                            }
                                        } else {
                                            self.file_prompt_buffer.push(c);
                                        }
                                    }
                                    _ => {}
                                }
                                // Skip normal key handling when in prompt
                            } else {
                                // Normal key handling with debounce
                                let now = Instant::now();
                                let time_since_last_press = now.duration_since(self.last_key_press);
                                let is_quit_key = matches!(
                                    code,
                                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc
                                );
                                if is_quit_key
                                    || time_since_last_press.as_millis()
                                        >= self.key_debounce_ms as u128
                                {
                                    if !is_quit_key {
                                        self.last_key_press = now;
                                    }
                                    match code {
                                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                            tracing::info!("Quit key pressed");
                                            break;
                                        }
                                        // Enter: no-op (image/video inline path input disabled temporarily)
                                        KeyCode::Enter => {}
                                        KeyCode::Char('o') | KeyCode::Char('O') => {
                                            self.next_color_scheme();
                                        }
                                        KeyCode::Char('e') | KeyCode::Char('E') => {
                                            self.toggle_effects();
                                        }
                                        KeyCode::Char('b') | KeyCode::Char('B') => {
                                            self.toggle_effect("Bloom");
                                        }
                                        KeyCode::Char('s') | KeyCode::Char('S') => {
                                            self.toggle_effect("Scanline");
                                        }
                                        KeyCode::Char('h') | KeyCode::Char('H') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Increase wireframe grid step (sparser)
                                                let step_prev = self.ray3d_wire_step_rad;
                                                self.ray3d_wire_step_rad = (self.ray3d_wire_step_rad + (2.0_f32.to_radians()))
                                                    .min(45.0_f32.to_radians());
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe step: {:.1}° (was {:.1}°)",
                                                    self.ray3d_wire_step_rad.to_degrees(),
                                                    step_prev.to_degrees()
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (step_prev, _tol) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_step = (step_prev + 2.0_f32.to_radians()).min(45.0_f32.to_radians());
                                                    viz.set_wire_step_rad(new_step);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe step: {:.1}° (was {:.1}°)",
                                                        new_step.to_degrees(),
                                                        step_prev.to_degrees()
                                                    );
                                                }
                                            } else {
                                                self.toggle_effect("Phosphor");
                                            }
                                        }
                                        KeyCode::Char('[') | KeyCode::Char('{') => {
                                            self.decrease_effect_intensity();
                                        }
                                        KeyCode::Char(']') | KeyCode::Char('}') => {
                                            self.increase_effect_intensity();
                                        }
                                        KeyCode::Char('m') | KeyCode::Char('M') => {
                                            self.toggle_microphone();
                                        }
                                        KeyCode::Right => {
                                            if self.visualizer_mode == VisualizerMode::HistoryTV {
                                                // Next era in History TV mode
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::HistoryTVChannelVisualizer>()
                                                {
                                                    viz.next_era();
                                                    tracing::info!("History TV: Next era");
                                                }
                                            } else {
                                                self.next_visualizer_mode();
                                            }
                                        }
                                        KeyCode::Left => {
                                            if self.visualizer_mode == VisualizerMode::HistoryTV {
                                                // Previous era in History TV mode
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::HistoryTVChannelVisualizer>()
                                                {
                                                    viz.previous_era();
                                                    tracing::info!("History TV: Previous era");
                                                }
                                            } else {
                                                self.prev_visualizer_mode();
                                            }
                                        }
                                        KeyCode::PageUp => {
                                            if self.visualizer_mode == VisualizerMode::HistoryTV {
                                                // Switch to next universe in History TV mode
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::HistoryTVChannelVisualizer>()
                                                {
                                                    viz.next_universe();
                                                    tracing::info!("Multi-verse TV: Next universe");
                                                }
                                            }
                                        }
                                        KeyCode::PageDown => {
                                            if self.visualizer_mode == VisualizerMode::HistoryTV {
                                                // Switch to previous universe in History TV mode
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::HistoryTVChannelVisualizer>()
                                                {
                                                    viz.previous_universe();
                                                    tracing::info!("Multi-verse TV: Previous universe");
                                                }
                                            }
                                        }
                                        KeyCode::Char('v') | KeyCode::Char('V') => {
                                            self.next_visualizer_mode();
                                        }
                                        KeyCode::Char('i') | KeyCode::Char('I') => {
                                            self.show_channel_number = !self.show_channel_number;
                                        }
                                        KeyCode::Char('+') | KeyCode::Char('=') => {
                                            self.increase_sensitivity();
                                        }
                                        KeyCode::Char('-') | KeyCode::Char('_') => {
                                            self.decrease_sensitivity();
                                        }
                                        KeyCode::Char('1') => self.set_sensitivity_preset(1),
                                        KeyCode::Char('2') => self.set_sensitivity_preset(2),
                                        KeyCode::Char('3') => self.set_sensitivity_preset(3),
                                        KeyCode::Char('4') => self.set_sensitivity_preset(4),
                                        KeyCode::Char('5') => self.set_sensitivity_preset(5),
                                        KeyCode::Char('6') => self.set_sensitivity_preset(6),
                                        KeyCode::Char('7') => self.set_sensitivity_preset(7),
                                        KeyCode::Char('8') => self.set_sensitivity_preset(8),
                                        KeyCode::Char('9') => self.set_sensitivity_preset(9),
                                        // Raycaster 3D specific controls
                                        KeyCode::Char('w') | KeyCode::Char('W') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_mode = match self.ray3d_mode {
                                                    crate::visualization::ray_tracer::RenderMode::Wireframe { .. } => crate::visualization::ray_tracer::RenderMode::Solid,
                                                    crate::visualization::ray_tracer::RenderMode::Solid => crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    },
                                                };
                                                self.recreate_visualizer();
                                                let mode_name = match self.ray3d_mode { crate::visualization::ray_tracer::RenderMode::Wireframe { .. } => "WIREFRAME", crate::visualization::ray_tracer::RenderMode::Solid => "SOLID" };
                                                tracing::info!(
                                                    "Raycaster 3D mode toggled: {}",
                                                    mode_name
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.toggle_render_mode();
                                                    tracing::info!("OBJ Viewer: render mode toggled");
                                                }
                                            }
                                        }
                                        KeyCode::Up => {
                                            if self.visualizer_mode == VisualizerMode::HistoryTV {
                                                // Next video in current era
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::HistoryTVChannelVisualizer>()
                                                {
                                                    viz.next_video();
                                                    tracing::info!("History TV: Next video");
                                                }
                                            } else if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_brightness_boost =
                                                    (self.ray3d_brightness_boost + 0.05).min(0.7);
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D brightness boost: +{:.2}",
                                                    self.ray3d_brightness_boost
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                // Next model
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.next_model();
                                                    tracing::info!("OBJ Viewer: {}", viz.model_name());
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            if self.visualizer_mode == VisualizerMode::HistoryTV {
                                                // Previous video in current era
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::HistoryTVChannelVisualizer>()
                                                {
                                                    viz.previous_video();
                                                    tracing::info!("History TV: Previous video");
                                                }
                                            } else if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_brightness_boost =
                                                    (self.ray3d_brightness_boost - 0.05).max(-0.3);
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D brightness boost: +{:.2}",
                                                    self.ray3d_brightness_boost
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                // Previous model
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.prev_model();
                                                    tracing::info!("OBJ Viewer: {}", viz.model_name());
                                                }
                                            }
                                        }
                                        KeyCode::Char('g') | KeyCode::Char('G') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Decrease wireframe step (denser grid)
                                                let step_prev = self.ray3d_wire_step_rad;
                                                self.ray3d_wire_step_rad = (self.ray3d_wire_step_rad - (2.0_f32.to_radians()))
                                                    .max(2.0_f32.to_radians());
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe step: {:.1} deg (was {:.1} deg)",
                                                    self.ray3d_wire_step_rad.to_degrees(),
                                                    step_prev.to_degrees()
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (step_prev, _tol) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_step = (step_prev - 2.0_f32.to_radians()).max(2.0_f32.to_radians());
                                                    viz.set_wire_step_rad(new_step);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe step: {:.1} deg (was {:.1} deg)",
                                                        new_step.to_degrees(),
                                                        step_prev.to_degrees()
                                                    );
                                                }
                                            } else if self.visualizer_mode == VisualizerMode::Oscilloscope {
                                                self.osc_show_grid = !self.osc_show_grid;
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Toggled oscilloscope grid: {}",
                                                    self.osc_show_grid
                                                );
                                            }
                                        }
                                        KeyCode::Char('f') | KeyCode::Char('F') => {
                                            if self.visualizer_mode == VisualizerMode::Oscilloscope {
                                                self.osc_waveform_mode = match self.osc_waveform_mode {
                                                    WaveformMode::Line => WaveformMode::Filled,
                                                    WaveformMode::Filled => WaveformMode::LineAndFill,
                                                    WaveformMode::LineAndFill => WaveformMode::Line,
                                                };
                                                self.recreate_visualizer();
                                                tracing::info!("Toggled oscilloscope fill mode");
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.focus_fit();
                                                    tracing::info!("OBJ Viewer: focus fit");
                                                }
                                            }
                                        }
                                        KeyCode::Char('l') | KeyCode::Char('L') => {
                                            if self.visualizer_mode == VisualizerMode::Spectrum {
                                                self.show_labels = !self.show_labels;
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Labels toggled: {}",
                                                    if self.show_labels { "ON" } else { "OFF" }
                                                );
                                            }
                                        }
                                        KeyCode::Char('p') | KeyCode::Char('P') => {
                                            if self.visualizer_mode == VisualizerMode::Spectrum {
                                                self.spectrum_peak_hold = !self.spectrum_peak_hold;
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Peak hold toggled: {}",
                                                    if self.spectrum_peak_hold {
                                                        "ON"
                                                    } else {
                                                        "OFF"
                                                    }
                                                );
                                            }
                                        }
                                        KeyCode::Char('n') | KeyCode::Char('N') => {
                                            if self.visualizer_mode == VisualizerMode::Spectrum {
                                                self.spectrum_mapping = match self.spectrum_mapping
                                                {
                                                    SpectrumMapping::NoteBars => {
                                                        SpectrumMapping::LogBars
                                                    }
                                                    SpectrumMapping::LogBars => {
                                                        SpectrumMapping::NoteBars
                                                    }
                                                };
                                                self.recreate_visualizer();
                                                let name = match self.spectrum_mapping {
                                                    SpectrumMapping::NoteBars => "NOTES",
                                                    SpectrumMapping::LogBars => "LOG",
                                                };
                                                tracing::info!(
                                                    "Spectrum mapping toggled: {}",
                                                    name
                                                );
                                            }
                                        }
                                        KeyCode::Char('t') | KeyCode::Char('T') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Decrease tolerance (thinner lines)
                                                let tol_prev = self.ray3d_wire_tol_rad;
                                                self.ray3d_wire_tol_rad =
                                                    (self.ray3d_wire_tol_rad - 0.005).max(0.002);
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                    self.ray3d_wire_tol_rad,
                                                    tol_prev
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (_step, tol_prev) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_tol = (tol_prev - 0.005).max(0.002);
                                                    viz.set_wire_tol_rad(new_tol);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                        new_tol,
                                                        tol_prev
                                                    );
                                                }
                                            } else if self.visualizer_mode == VisualizerMode::Oscilloscope {
                                                self.osc_trigger_slope = match self.osc_trigger_slope {
                                                    TriggerSlope::Positive => TriggerSlope::Negative,
                                                    TriggerSlope::Negative => TriggerSlope::Both,
                                                    TriggerSlope::Both => TriggerSlope::Positive,
                                                };
                                                self.recreate_visualizer();
                                                tracing::info!("Toggled oscilloscope trigger mode");
                                            }
                                        }
                                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Increase tolerance (thicker lines)
                                                let tol_prev = self.ray3d_wire_tol_rad;
                                                self.ray3d_wire_tol_rad = (self.ray3d_wire_tol_rad + 0.005).min(0.15);
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                    self.ray3d_wire_tol_rad,
                                                    tol_prev
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (_step, tol_prev) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_tol = (tol_prev + 0.005).min(0.15);
                                                    viz.set_wire_tol_rad(new_tol);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                        new_tol,
                                                        tol_prev
                                                    );
                                                }
                                            }
                                        }
                                        KeyCode::Char('j') | KeyCode::Char('J') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                let prev = self.ray3d_rotation_speed_y;
                                                self.ray3d_rotation_speed_y =
                                                    (self.ray3d_rotation_speed_y - 0.1).max(0.0);
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::Raycaster3DVisualizer>()
                                                {
                                                    viz.set_rotation_speed_y(self.ray3d_rotation_speed_y);
                                                }
                                                tracing::info!("Raycaster 3D rotation speed: {:.2} rad/s (was {:.2})", self.ray3d_rotation_speed_y, prev);
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.pitch_up(step);
                                                    tracing::info!("OBJ Viewer: pitch up ({:.1}°)", 5.0);
                                                }
                                            }
                                        }
                                        KeyCode::Char('k') | KeyCode::Char('K') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                let prev = self.ray3d_rotation_speed_y;
                                                self.ray3d_rotation_speed_y =
                                                    (self.ray3d_rotation_speed_y + 0.1).min(5.0);
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::Raycaster3DVisualizer>()
                                                {
                                                    viz.set_rotation_speed_y(self.ray3d_rotation_speed_y);
                                                }
                                                tracing::info!("Raycaster 3D rotation speed: {:.2} rad/s (was {:.2})", self.ray3d_rotation_speed_y, prev);
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.pitch_down(step);
                                                    tracing::info!("OBJ Viewer: pitch down ({:.1}°)", 5.0);
                                                }
                                            }
                                        }

                                        KeyCode::Char('r') | KeyCode::Char('R') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_auto_rotate = !self.ray3d_auto_rotate;
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::Raycaster3DVisualizer>()
                                                {
                                                    viz.set_auto_rotate(self.ray3d_auto_rotate);
                                                }
                                                tracing::info!(
                                                    "Raycaster 3D auto-rotate: {}",
                                                    if self.ray3d_auto_rotate {
                                                        "ON"
                                                    } else {
                                                        "OFF"
                                                    }
                                                );
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                self.model_viewer_auto_rotate = !self.model_viewer_auto_rotate;
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.set_auto_rotate(self.model_viewer_auto_rotate);
                                                }
                                                tracing::info!(
                                                    "OBJ Viewer auto-rotate: {}",
                                                    if self.model_viewer_auto_rotate { "ON" } else { "OFF" }
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Spectrum
                                                && matches!(
                                                    self.spectrum_mapping,
                                                    SpectrumMapping::NoteBars
                                                )
                                            {
                                                self.spectrum_range_preset_index =
                                                    (self.spectrum_range_preset_index + 1) % 3;
                                                self.recreate_visualizer();
                                                let (label, _min, _max) =
                                                    match self.spectrum_range_preset_index % 3 {
                                                        0 => ("A2-A5", 110.0, 880.0),
                                                        1 => ("A1-A5", 55.0, 880.0),
                                                        _ => ("A1-A6", 55.0, 1760.0),
                                                    };
                                                tracing::info!(
                                                    "Spectrum note range preset: {}",
                                                    label
                                                );
                                            }
                                        }
                                        KeyCode::Char('a') | KeyCode::Char('A') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.yaw_left(step);
                                                    tracing::info!("OBJ Viewer: yaw left ({:.1}°)", 5.0);
                                                }
                                            }
                                        }
                                        KeyCode::Char('d') | KeyCode::Char('D') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.yaw_right(step);
                                                    tracing::info!("OBJ Viewer: yaw right ({:.1}°)", 5.0);
                                                }
                                            }
                                        }
                                        KeyCode::Char(',') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.roll_ccw(step);
                                                    tracing::info!("OBJ Viewer: roll CCW ({:.1}°)", 5.0);
                                                }
                                            }
                                        }
                                        KeyCode::Char('.') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.roll_cw(step);
                                                    tracing::info!("OBJ Viewer: roll CW ({:.1}°)", 5.0);
                                                }
                                            }
                                        }

                                        KeyCode::Char('z') | KeyCode::Char('Z') => {
                                            if self.visualizer_mode == VisualizerMode::XYOscilloscope {
                                                tracing::info!("XY Oscilloscope zoom control (use +/- for sensitivity)");
                                            } else if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.zoom_in();
                                                    tracing::info!("OBJ Viewer: zoom in");
                                                }
                                            }
                                        }
                                        KeyCode::Char('x') | KeyCode::Char('X') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.zoom_out();
                                                    tracing::info!("OBJ Viewer: zoom out");
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Check if audio capture is still active
            if !self.audio_device.is_capturing() {
                tracing::error!("Audio capture stopped unexpectedly. This may indicate:");
                tracing::error!("  - Audio device was disconnected");
                tracing::error!("  - Audio system crashed or restarted");
                tracing::error!("  - Permission was revoked");
                tracing::error!("Exiting...");
                break;
            }

            // 1. Read audio samples from ring buffer (only if microphone is enabled)
            let audio_params = if let Some(audio_buffer) = self.audio_device.read_samples() {
                // Debug: Log that we're receiving audio (disabled for production)
                // if frame_count % 60 == 0 {
                //     tracing::debug!(
                //         "Received audio buffer: {} samples, {} channels",
                //         audio_buffer.samples.len(),
                //         audio_buffer.channels
                //     );
                // }

                // 1a. If microphone passthrough is enabled, write to output so you can hear it
                if self.microphone_enabled {
                    if let Some(ref audio_output) = self.audio_output {
                        audio_output.write_samples(&audio_buffer);
                    }
                }

                // 2. Process audio only when appropriate source is active
                // - Loopback: always process (system audio) WITHOUT amplitude squelch
                // - Mic: process only when microphone_enabled is true (WITH squelch)
                if self.use_loopback {
                    self.dsp_processor.process(&audio_buffer)
                } else if self.microphone_enabled {
                    let mut audio_params = self.dsp_processor.process(&audio_buffer);
                    const SQUELCH_THRESHOLD: f32 = 0.005; // conservative floor for mic noise
                    if audio_params.amplitude < SQUELCH_THRESHOLD {
                        audio_params = dsp::AudioParameters::default();
                    }
                    audio_params
                } else {
                    // Mic is OFF and we're not in loopback: feed silence so visuals decay to zero
                    dsp::AudioParameters::default()
                }
            } else {
                // No new audio available this frame: feed silence so visuals decay
                // Debug: Log when no audio is available (disabled for production)
                // if frame_count % 300 == 0 {
                //     tracing::debug!("No audio buffer available from ring buffer");
                // }
                dsp::AudioParameters::default()
            };

            // 3. Update visualizer with audio parameters
            self.visualizer.update(&audio_params);

            // 4. Render visualization to grid
            let (width, height) = self.renderer.dimensions();
            let mut grid = GridBuffer::new(width as usize, height as usize);
            self.visualizer.render(&mut grid);

            // 5. Apply post-processing effects
            self.effect_pipeline.apply(&mut grid, &audio_params);

            // 6. All visualizers now use Braille rendering directly!
            // No need to apply character set mapping - Braille gives 8× resolution

            // 7. Add UI overlay (character set name and controls)
            self.add_ui_overlay(&mut grid);

            // 8. Update terminal display
            self.renderer
                .render(&grid)
                .context("Failed to render frame")?;

            // Frame timing
            frame_count += 1;
            let frame_elapsed = frame_start.elapsed();

            // Track performance metrics
            total_frame_time += frame_elapsed;
            max_frame_time = max_frame_time.max(frame_elapsed);
            min_frame_time = min_frame_time.min(frame_elapsed);

            // FPS tracking and diagnostics (log every second)
            if fps_timer.elapsed() >= Duration::from_secs(1) {
                let actual_fps = frame_count;
                let avg_frame_time = total_frame_time / frame_count;
                let target_frame_time = frame_duration;

                // Log performance metrics (only warnings, not regular debug)
                if actual_fps < self.target_fps * 9 / 10 {
                    // Warn if FPS drops below 90% of target
                    tracing::warn!(
                        "Performance: FPS={} (target={}), avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                        actual_fps,
                        self.target_fps,
                        avg_frame_time.as_secs_f32() * 1000.0,
                        min_frame_time.as_secs_f32() * 1000.0,
                        max_frame_time.as_secs_f32() * 1000.0
                    );
                }
                // Disabled regular performance debug logging for production
                // else {
                //     tracing::debug!(
                //         "Performance: FPS={} (target={}), avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                //         actual_fps,
                //         self.target_fps,
                //         avg_frame_time.as_secs_f32() * 1000.0,
                //         min_frame_time.as_secs_f32() * 1000.0,
                //         max_frame_time.as_secs_f32() * 1000.0
                //     );
                // }

                // Warn if frame time exceeds target significantly
                if max_frame_time > target_frame_time * 2 {
                    tracing::warn!(
                        "Frame time spike detected: {:.2}ms (target: {:.2}ms)",
                        max_frame_time.as_secs_f32() * 1000.0,
                        target_frame_time.as_secs_f32() * 1000.0
                    );
                }

                // Reset counters
                frame_count = 0;
                fps_timer = Instant::now();
                total_frame_time = Duration::ZERO;
                max_frame_time = Duration::ZERO;
                min_frame_time = Duration::from_secs(1);
            }

            // Sleep to maintain target FPS
            if let Some(sleep_time) = frame_duration.checked_sub(frame_elapsed) {
                std::thread::sleep(sleep_time);
            } else {
                // Frame took longer than target - log at trace level
                tracing::trace!(
                    "Frame overrun: {:.2}ms (target: {:.2}ms)",
                    frame_elapsed.as_secs_f32() * 1000.0,
                    frame_duration.as_secs_f32() * 1000.0
                );
            }
        }

        tracing::info!("Shutting down application...");

        // Stop audio capture
        tracing::debug!("Stopping audio capture...");
        self.audio_device
            .stop_capture()
            .context("Failed to stop audio capture")?;
        tracing::info!("Audio capture stopped");

        // Stop audio output if enabled
        if let Some(audio_output) = self.audio_output.as_mut() {
            tracing::debug!("Stopping audio output...");
            audio_output
                .stop_playback()
                .context("Failed to stop audio playback")?;
            tracing::info!("Audio output stopped");
        }

        // Cleanup terminal
        tracing::debug!("Cleaning up terminal...");
        self.renderer
            .cleanup()
            .context("Failed to cleanup terminal")?;
        tracing::info!("Terminal cleanup complete");

        Ok(())
    }

    /// Run in test mode with test patterns
    fn run_test_mode(mut self) -> Result<()> {
        tracing::info!("Running in TEST MODE - rendering test patterns");
        tracing::info!("Press 'q', 'Q', or ESC to quit");

        let (width, height) = self.renderer.dimensions();
        let mut grid = GridBuffer::new(width as usize, height as usize);

        // Test pattern 1: Grid lines
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let ch = if x % 10 == 0 || y % 5 == 0 {
                    '+'
                } else if x % 5 == 0 {
                    '|'
                } else if y % 2 == 0 {
                    '-'
                } else {
                    ' '
                };
                grid.set_cell(x, y, ch);
            }
        }

        // Test pattern 2: Sine wave with known parameters
        let mut phase = 0.0_f32;

        loop {
            // Check for quit
            if SHUTDOWN.load(Ordering::Relaxed) {
                break;
            }

            if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            tracing::info!("Quit key pressed");
                            break;
                        }
                        KeyCode::Char('1') => {
                            tracing::info!("Test pattern 1: Grid");
                            // Already set above
                        }
                        KeyCode::Char('2') => {
                            tracing::info!("Test pattern 2: Sine wave");
                            grid.clear();
                            let center_y = grid.height() / 2;
                            for x in 0..grid.width() {
                                let norm_x = x as f32 / grid.width() as f32;
                                let wave_y = center_y as f32
                                    + (norm_x * 4.0 * std::f32::consts::PI + phase).sin()
                                        * (grid.height() as f32 * 0.3);
                                let y = wave_y as usize;
                                if y < grid.height() {
                                    grid.set_cell(x, y, '█');
                                    if y > 0 {
                                        grid.set_cell(x, y - 1, '▓');
                                    }
                                    if y + 1 < grid.height() {
                                        grid.set_cell(x, y + 1, '▓');
                                    }
                                }
                            }
                            phase += 0.1;
                        }
                        KeyCode::Char('3') => {
                            tracing::info!("Test pattern 3: Checkerboard");
                            grid.clear();
                            for y in 0..grid.height() {
                                for x in 0..grid.width() {
                                    let ch = if (x + y) % 2 == 0 { '█' } else { ' ' };
                                    grid.set_cell(x, y, ch);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Render
            self.renderer.render(&grid)?;
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }

        // Cleanup
        self.renderer.cleanup()?;
        Ok(())
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool, debug: bool) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    // Determine log level
    let filter = if debug {
        EnvFilter::new("crabmusic=trace,debug")
    } else if verbose {
        EnvFilter::new("crabmusic=debug,info")
    } else {
        EnvFilter::new("crabmusic=info")
    };

    // Configure logging format
    // Write to stderr instead of stdout to avoid interference with terminal UI
    // and disable ANSI codes when in alternate screen mode
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(fmt::time::uptime())  // Show time since start
        .with_writer(std::io::stderr)  // Write to stderr instead of stdout
        .with_ansi(false)  // Disable ANSI codes to avoid terminal conflicts
        .with_level(true)
        .init();

    Ok(())
}

/// Setup Ctrl+C handler for graceful shutdown
fn setup_shutdown_handler() -> Result<()> {
    ctrlc::set_handler(move || {
        tracing::info!("Received Ctrl+C, shutting down...");
        SHUTDOWN.store(true, Ordering::Relaxed);
    })
    .context("Failed to set Ctrl+C handler")?;

    Ok(())
}

/// Print version information
fn print_version_info() {
    println!("CrabMusic v{}", env!("CARGO_PKG_VERSION"));
    println!("Real-time ASCII music visualizer for terminal");
    println!();
    println!("Build information:");
    println!("  Rust version: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!("  Target: {}", std::env::consts::ARCH);
    println!("  OS: {}", std::env::consts::OS);
    println!();
    println!("Features:");
    println!("  - Real-time audio capture and visualization");
    println!("  - 7 character sets (basic, extended, blocks, shading, dots, lines, braille)");
    println!("  - Audio passthrough (hear what you visualize)");
    println!("  - Hot-reload configuration");
    println!("  - Cross-platform (Linux, macOS, Windows)");
    println!();
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("License: {}", env!("CARGO_PKG_LICENSE"));
}

/// List available audio devices
fn list_audio_devices() -> Result<()> {
    use cpal::traits::{DeviceTrait, HostTrait};

    println!("Available audio devices:");
    println!();

    let host = cpal::default_host();
    println!("Host: {:?}", host.id());
    println!();

    // List input devices
    println!("Input devices (microphones and loopback):");
    let input_devices = host
        .input_devices()
        .context("Failed to enumerate input devices")?;

    for (i, device) in input_devices.enumerate() {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        let is_default = host
            .default_input_device()
            .and_then(|d| d.name().ok())
            .map(|n| n == name)
            .unwrap_or(false);

        let name_lower = name.to_lowercase();
        let is_loopback = name_lower.contains("stereo mix")
            || name_lower.contains("loopback")
            || name_lower.contains("monitor")
            || name_lower.contains("what u hear")
            || name_lower.contains("wave out");

        let mut tags = Vec::new();
        if is_default {
            tags.push("default");
        }
        if is_loopback {
            tags.push("LOOPBACK");
        }

        if tags.is_empty() {
            println!("  {}. {}", i + 1, name);
        } else {
            println!("  {}. {} [{}]", i + 1, name, tags.join(", "));
        }
    }

    println!();

    // List output devices
    println!("Output devices (speakers/headphones):");
    let output_devices = host
        .output_devices()
        .context("Failed to enumerate output devices")?;

    for (i, device) in output_devices.enumerate() {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        let is_default = host
            .default_output_device()
            .and_then(|d| d.name().ok())
            .map(|n| n == name)
            .unwrap_or(false);

        if is_default {
            println!("  {}. {} [default]", i + 1, name);
        } else {
            println!("  {}. {}", i + 1, name);
        }
    }

    println!();
    println!("To use a specific device, use: --device \"device name\"");
    println!("For system audio capture, look for devices marked [LOOPBACK]");

    Ok(())
}
