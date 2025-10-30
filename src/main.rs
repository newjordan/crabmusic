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
    #[arg(short, long, default_value = "config/default.yaml")]
    config: String,

    /// Target FPS
    #[arg(short, long, default_value_t = 60)]
    fps: u32,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Test mode: render test pattern instead of audio visualization
    #[arg(short, long)]
    test: bool,

    /// Amplitude sensitivity multiplier (default: 10.0)
    #[arg(long, default_value_t = 10.0)]
    sensitivity: f32,
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging
    init_logging(args.verbose)?;

    tracing::info!("CrabMusic v{} starting...", env!("CARGO_PKG_VERSION"));

    // Setup Ctrl+C handler
    setup_shutdown_handler()?;

    // Create and run application
    let app = Application::new(args.fps, args.sensitivity)?;

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
    audio_output: AudioOutputDevice,
    dsp_processor: DspProcessor,
    visualizer: SineWaveVisualizer,
    renderer: TerminalRenderer,
    #[allow(dead_code)] // Held for lifetime management
    ring_buffer: Arc<AudioRingBuffer>,
    target_fps: u32,
    current_charset: CharacterSet,
    charset_index: usize,
}

impl Application {
    /// Create a new application instance
    fn new(target_fps: u32, sensitivity: f32) -> Result<Self> {
        tracing::info!("Initializing components...");
        tracing::info!("Amplitude sensitivity: {}", sensitivity);

        // Create ring buffer for audio pipeline
        let ring_buffer = Arc::new(AudioRingBuffer::new(10));

        // Initialize audio capture
        let audio_device = CpalAudioDevice::new(ring_buffer.clone())
            .context("Failed to initialize audio capture")?;

        // Initialize audio output
        let audio_output =
            AudioOutputDevice::new().context("Failed to initialize audio output")?;

        // Initialize DSP processor
        let dsp_processor =
            DspProcessor::new(44100, 2048).context("Failed to initialize DSP processor")?;

        // Initialize visualizer with custom sensitivity
        let config = SineWaveConfig {
            amplitude_sensitivity: sensitivity,
            ..Default::default()
        };
        let visualizer = SineWaveVisualizer::new(config);

        // Initialize terminal renderer
        let renderer = TerminalRenderer::new().context("Failed to initialize terminal renderer")?;

        tracing::info!("All components initialized successfully");

        Ok(Self {
            audio_device,
            audio_output,
            dsp_processor,
            visualizer,
            renderer,
            ring_buffer,
            target_fps,
            current_charset: get_character_set(CharacterSetType::Blocks),
            charset_index: 2, // Start with blocks (index 2)
        })
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
        let info_text = format!(" {} | Press 'C' to change | 'Q' to quit ", charset_name);

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

        // Start audio output (playback)
        self.audio_output
            .start_playback()
            .context("Failed to start audio playback")?;

        // Calculate frame time
        let frame_duration = Duration::from_secs_f32(1.0 / self.target_fps as f32);

        // Performance tracking
        let mut frame_count = 0;
        let mut fps_timer = Instant::now();

        loop {
            let frame_start = Instant::now();

            // Check for shutdown signal
            if SHUTDOWN.load(Ordering::Relaxed) {
                tracing::info!("Shutdown signal received");
                break;
            }

            // Check for keyboard input (Ctrl+C or 'q' to quit, 'c' to change charset)
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
                        _ => {}
                    }
                }
            }

            // 1. Read audio samples from ring buffer
            if let Some(audio_buffer) = self.audio_device.read_samples() {
                // 1a. Pass audio through to output (so you can hear it)
                self.audio_output.write_samples(&audio_buffer);

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

            // FPS tracking (log every second)
            if fps_timer.elapsed() >= Duration::from_secs(1) {
                let actual_fps = frame_count;
                tracing::debug!(
                    "FPS: {} (target: {}), frame time: {:?}",
                    actual_fps,
                    self.target_fps,
                    frame_elapsed
                );
                frame_count = 0;
                fps_timer = Instant::now();
            }

            // Sleep to maintain target FPS
            if let Some(sleep_time) = frame_duration.checked_sub(frame_elapsed) {
                std::thread::sleep(sleep_time);
            }
        }

        // Stop audio capture
        self.audio_device
            .stop_capture()
            .context("Failed to stop audio capture")?;

        // Stop audio output
        self.audio_output
            .stop_playback()
            .context("Failed to stop audio playback")?;

        // Cleanup terminal
        self.renderer
            .cleanup()
            .context("Failed to cleanup terminal")?;

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
fn init_logging(verbose: bool) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = if verbose {
        EnvFilter::new("crabmusic=debug,info")
    } else {
        EnvFilter::new("crabmusic=info")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
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
