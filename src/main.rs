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
mod error;
mod rendering;
mod visualization;

use audio::{AudioCaptureDevice, AudioOutputDevice, AudioRingBuffer, CpalAudioDevice};
use dsp::DspProcessor;
use rendering::TerminalRenderer;
use visualization::{
    character_sets::{get_all_character_sets, get_character_set, CharacterSet, CharacterSetType},
    GridBuffer, SineWaveConfig, SineWaveVisualizer, Visualizer,
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

    /// Audio device name (overrides config file)
    #[arg(long, value_name = "NAME")]
    device: Option<String>,

    /// List available audio devices and exit
    #[arg(long)]
    list_devices: bool,

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

    /// Show version information
    #[arg(long)]
    version_info: bool,
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

    if let Some(device) = args.device {
        config.audio.device_name = Some(device.clone());
        tracing::info!("Overriding audio device from CLI: {}", device);
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

    // Create and run application
    let app = Application::new_with_config(config, args.no_audio_output)?;

    if args.test {
        app.run_test_mode()?;
    } else {
        app.run()?;
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

/// Application state
struct Application {
    audio_device: CpalAudioDevice,
    audio_output: Option<AudioOutputDevice>,
    dsp_processor: DspProcessor,
    visualizer: SineWaveVisualizer,
    renderer: TerminalRenderer,
    #[allow(dead_code)] // Held for lifetime management
    ring_buffer: Arc<AudioRingBuffer>,
    target_fps: u32,
    current_charset: CharacterSet,
    charset_index: usize,
    microphone_enabled: bool,
}

impl Application {
    /// Create a new application instance with configuration
    fn new_with_config(config: config::AppConfig, no_audio_output: bool) -> Result<Self> {
        tracing::info!("Initializing components with configuration...");
        tracing::debug!("Configuration: sample_rate={}, fft_size={}, fps={}",
            config.audio.sample_rate, config.dsp.fft_size, config.rendering.target_fps);

        // Create ring buffer for audio pipeline
        // Use a reasonable default of 10 buffers if buffer_size is not available
        let ring_buffer = Arc::new(AudioRingBuffer::new(10));
        tracing::debug!("Ring buffer created with capacity: 10");

        // Initialize audio capture with retry logic
        tracing::debug!("Initializing audio capture device...");
        let audio_device = Self::init_audio_capture_with_retry(ring_buffer.clone())?;
        tracing::info!("Audio capture device initialized successfully");

        // Initialize audio output (optional, with graceful degradation)
        let audio_output = if no_audio_output {
            tracing::info!("Audio output disabled by CLI flag");
            None
        } else {
            tracing::debug!("Initializing audio output device...");
            match AudioOutputDevice::new() {
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

        // Initialize DSP processor
        tracing::debug!("Initializing DSP processor...");
        let dsp_processor = DspProcessor::new(
            config.audio.sample_rate,
            config.dsp.fft_size,
        )
        .context("Failed to initialize DSP processor")?;
        tracing::info!("DSP processor initialized: sample_rate={}, fft_size={}",
            config.audio.sample_rate, config.dsp.fft_size);

        // Initialize visualizer
        tracing::debug!("Initializing visualizer...");
        let viz_config = SineWaveConfig {
            amplitude_sensitivity: config.visualization.sine_wave.amplitude,
            ..Default::default()
        };
        let visualizer = SineWaveVisualizer::new(viz_config);
        tracing::info!("Visualizer initialized: type=sine_wave, sensitivity={}",
            config.visualization.sine_wave.amplitude);

        // Initialize terminal renderer
        tracing::debug!("Initializing terminal renderer...");
        let renderer = TerminalRenderer::new().context("Failed to initialize terminal renderer")?;
        let (width, height) = renderer.dimensions();
        tracing::info!("Terminal renderer initialized: {}x{}", width, height);

        // Determine initial character set
        let charset_type = match config.visualization.character_set.as_str() {
            "basic" => CharacterSetType::Basic,
            "extended" => CharacterSetType::Extended,
            "blocks" => CharacterSetType::Blocks,
            "shading" => CharacterSetType::Shading,
            "dots" => CharacterSetType::Dots,
            "lines" => CharacterSetType::Lines,
            "braille" => CharacterSetType::Braille,
            _ => CharacterSetType::Blocks,
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
        };

        tracing::info!("All components initialized successfully");

        Ok(Self {
            audio_device,
            audio_output,
            dsp_processor,
            visualizer,
            renderer,
            ring_buffer,
            target_fps: config.rendering.target_fps,
            current_charset,
            charset_index,
            microphone_enabled: true, // Start with microphone enabled
        })
    }

    /// Create a new application instance (legacy method for backward compatibility)
    #[allow(dead_code)]
    fn new(target_fps: u32, sensitivity: f32) -> Result<Self> {
        let mut config = config::AppConfig::default();
        config.rendering.target_fps = target_fps;
        config.visualization.sine_wave.amplitude = sensitivity;
        Self::new_with_config(config, false)
    }

    /// Initialize audio capture with retry logic
    ///
    /// Attempts to initialize audio capture with exponential backoff retry strategy.
    /// Retries 3 times with delays of 100ms, 500ms, and 1000ms.
    fn init_audio_capture_with_retry(ring_buffer: Arc<AudioRingBuffer>) -> Result<CpalAudioDevice> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAYS_MS: [u64; 3] = [100, 500, 1000];

        for attempt in 0..MAX_RETRIES {
            match CpalAudioDevice::new(ring_buffer.clone()) {
                Ok(device) => {
                    if attempt > 0 {
                        tracing::info!("Audio capture initialized successfully after {} retries", attempt);
                    }
                    return Ok(device);
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
                        tracing::error!("Failed to initialize audio capture after {} attempts: {}", MAX_RETRIES, e);
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
    fn next_charset(&mut self) {
        let charsets = get_all_character_sets();
        self.charset_index = (self.charset_index + 1) % charsets.len();
        self.current_charset = charsets[self.charset_index].clone();
        tracing::info!(
            "Switched to character set: {}",
            self.current_charset.name
        );
    }

    /// Toggle microphone on/off
    fn toggle_microphone(&mut self) {
        self.microphone_enabled = !self.microphone_enabled;
        let status = if self.microphone_enabled { "ON" } else { "OFF" };
        tracing::info!("Microphone toggled: {}", status);
    }

    /// Apply character set mapping to the grid
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
                grid.set_cell(x, y, new_char);
            }
        }
    }

    /// Add UI overlay with character set name and controls
    fn add_ui_overlay(&self, grid: &mut GridBuffer) {
        let charset_name = &self.current_charset.name;
        let mic_status = if self.microphone_enabled { "MIC:ON" } else { "MIC:OFF" };
        let info_text = format!(" {} | {} | Press 'C' to change charset | 'M' to toggle mic | 'Q' to quit ",
            charset_name, mic_status);

        // Draw info bar at the top
        let start_x = (grid.width().saturating_sub(info_text.len())) / 2;
        for (i, ch) in info_text.chars().enumerate() {
            let x = start_x + i;
            if x < grid.width() {
                grid.set_cell(x, 0, ch);
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

            // Check for keyboard input (Ctrl+C or 'q' to quit, 'c' to change charset, 'm' to toggle mic)
            if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            tracing::info!("Quit key pressed");
                            break;
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            self.next_charset();
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            self.toggle_microphone();
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
            if let Some(audio_buffer) = self.audio_device.read_samples() {
                // Only process audio if microphone is enabled
                if self.microphone_enabled {
                    // 1a. Pass audio through to output (so you can hear it) if enabled
                    if let Some(ref audio_output) = self.audio_output {
                        audio_output.write_samples(&audio_buffer);
                    }

                    // 2. Process audio → extract parameters
                    let audio_params = self.dsp_processor.process(&audio_buffer);

                    // Debug: Log audio parameters occasionally
                    if frame_count % 60 == 0 {
                        tracing::debug!(
                            "Audio params - amp: {:.3}, bass: {:.3}, mid: {:.3}, treble: {:.3}",
                            audio_params.amplitude,
                            audio_params.bass,
                            audio_params.mid,
                            audio_params.treble
                        );
                    }

                    // 3. Update visualizer with audio parameters
                    self.visualizer.update(&audio_params);
                }
            }

            // 4. Render visualization to grid
            let (width, height) = self.renderer.dimensions();
            let mut grid = GridBuffer::new(width as usize, height as usize);
            self.visualizer.render(&mut grid);

            // 5. Apply character set mapping to grid
            self.apply_charset_to_grid(&mut grid);

            // 6. Add UI overlay (character set name and controls)
            self.add_ui_overlay(&mut grid);

            // 7. Update terminal display
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

                // Log performance metrics
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
                } else {
                    tracing::debug!(
                        "Performance: FPS={} (target={}), avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                        actual_fps,
                        self.target_fps,
                        avg_frame_time.as_secs_f32() * 1000.0,
                        min_frame_time.as_secs_f32() * 1000.0,
                        max_frame_time.as_secs_f32() * 1000.0
                    );
                }

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
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(fmt::time::uptime())  // Show time since start
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

    // List input devices
    println!("Input devices:");
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

        if is_default {
            println!("  {}. {} (default)", i + 1, name);
        } else {
            println!("  {}. {}", i + 1, name);
        }
    }

    println!();

    // List output devices
    println!("Output devices:");
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
            println!("  {}. {} (default)", i + 1, name);
        } else {
            println!("  {}. {}", i + 1, name);
        }
    }

    Ok(())
}
