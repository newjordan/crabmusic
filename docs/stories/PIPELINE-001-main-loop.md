# [PIPELINE-001] Main Application Loop

**Epic**: Pipeline Integration
**Priority**: P0 (Blocking)
**Estimated Effort**: 2-3 days
**Status**: Not Started

---

## Description

Implement the main application loop that orchestrates all components into a functioning real-time visualizer. This is the heart of the application that ties audio capture, DSP, visualization, and rendering together.

**Agent Instructions**: Create the main.rs application that:
- Initializes all components in correct order
- Runs the rendering loop at target FPS (60)
- Coordinates audio thread and rendering thread
- Handles graceful startup and shutdown
- Provides clean error messages to users

---

## Acceptance Criteria

- [ ] main.rs entry point with CLI argument parsing (using clap)
- [ ] Component initialization sequence:
  1. Configuration loading
  2. Audio capture initialization
  3. DSP processor setup
  4. Visualizer creation
  5. Terminal renderer initialization
- [ ] Main rendering loop runs at target FPS (60)
- [ ] Audio samples flow: Capture → Ring Buffer → DSP → Visualizer → Renderer
- [ ] FPS counter/limiter implemented to maintain target rate
- [ ] Graceful shutdown on Ctrl+C (cleanup terminal state)
- [ ] Startup errors provide actionable user messages
- [ ] Application runs continuously without crashes
- [ ] Performance: Sustained 60 FPS with <10% CPU usage
- [ ] Manual test: Run with real music, visualizer reacts smoothly

---

## Technical Approach

### Main Application Structure

Reference: **docs/architecture.md - Core Workflows**

```rust
// src/main.rs
use clap::Parser;
use anyhow::Result;
use tracing::{info, error};

#[derive(Parser, Debug)]
#[command(name = "crabmusic")]
#[command(about = "ASCII music visualizer", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/default.yaml")]
    config: String,

    /// Target FPS
    #[arg(short, long, default_value_t = 60)]
    fps: u32,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    setup_logging(args.debug)?;

    info!("Starting crabmusic...");

    // Initialize components
    let config = load_config(&args.config)?;
    let app = Application::new(config, args.fps)?;

    // Setup Ctrl+C handler
    setup_shutdown_handler()?;

    // Run main loop
    app.run()?;

    info!("Shutdown complete");
    Ok(())
}
```

### Application Structure

```rust
pub struct Application {
    audio_device: Box<dyn AudioCaptureDevice>,
    dsp_processor: DspProcessor,
    visualizer: Box<dyn Visualizer>,
    renderer: TerminalRenderer,
    ring_buffer: Arc<RingBuffer>,
    target_fps: u32,
    config: AppConfig,
}

impl Application {
    pub fn new(config: AppConfig, target_fps: u32) -> Result<Self> {
        info!("Initializing components...");

        // Create ring buffer for audio pipeline
        let ring_buffer = Arc::new(RingBuffer::new(8192));

        // Initialize audio capture
        let audio_device = CpalAudioDevice::new(ring_buffer.clone())
            .map_err(|e| anyhow!("Failed to initialize audio: {}", e))?;

        // Initialize DSP processor
        let dsp_processor = DspProcessor::new(
            audio_device.sample_rate(),
            config.dsp.window_size
        );

        // Initialize visualizer based on config
        let visualizer: Box<dyn Visualizer> = match config.visualization.mode {
            VisualizerMode::SineWave => {
                Box::new(SineWaveVisualizer::new(config.visualization.sine_wave_config))
            },
            // Future: other visualizer types
        };

        // Initialize terminal renderer
        let renderer = TerminalRenderer::init()?;

        info!("All components initialized successfully");

        Ok(Self {
            audio_device,
            dsp_processor,
            visualizer,
            renderer,
            ring_buffer,
            target_fps,
            config,
        })
    }
}
```

### Main Loop Implementation

```rust
impl Application {
    pub fn run(mut self) -> Result<()> {
        info!("Starting main loop at {} FPS", self.target_fps);

        // Start audio capture
        self.audio_device.start_capture()?;

        // Calculate frame time
        let frame_duration = Duration::from_secs_f32(1.0 / self.target_fps as f32);
        let mut last_frame = Instant::now();

        // Performance tracking
        let mut frame_count = 0;
        let mut fps_timer = Instant::now();

        loop {
            let frame_start = Instant::now();

            // Check for shutdown signal
            if should_shutdown() {
                break;
            }

            // 1. Read audio samples from ring buffer
            if let Some(audio_buffer) = self.ring_buffer.read() {
                // 2. Process audio → extract parameters
                let audio_params = self.dsp_processor.process_buffer(&audio_buffer);

                // 3. Update visualizer with audio parameters
                self.visualizer.update(&audio_params);
            }

            // 4. Render visualization to grid
            let (width, height) = self.renderer.get_dimensions();
            let mut grid = GridBuffer::new(width, height);
            self.visualizer.render(&mut grid);

            // 5. Update terminal display
            self.renderer.render(&grid)?;

            // Frame timing
            frame_count += 1;
            let frame_elapsed = frame_start.elapsed();

            // FPS tracking (log every second)
            if fps_timer.elapsed() >= Duration::from_secs(1) {
                let actual_fps = frame_count;
                if self.config.debug {
                    info!("FPS: {} (target: {}), frame time: {:?}",
                          actual_fps, self.target_fps, frame_elapsed);
                }
                frame_count = 0;
                fps_timer = Instant::now();
            }

            // Sleep to maintain target FPS
            if let Some(sleep_time) = frame_duration.checked_sub(frame_elapsed) {
                std::thread::sleep(sleep_time);
            }

            last_frame = Instant::now();
        }

        // Cleanup
        info!("Shutting down...");
        self.audio_device.stop_capture()?;
        self.renderer.cleanup()?;

        Ok(())
    }
}
```

### Shutdown Handler

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn setup_shutdown_handler() -> Result<()> {
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        SHUTDOWN.store(true, Ordering::SeqCst);
    })?;
    Ok(())
}

fn should_shutdown() -> bool {
    SHUTDOWN.load(Ordering::SeqCst)
}
```

### Logging Setup

```rust
fn setup_logging(debug: bool) -> Result<()> {
    use tracing_subscriber::EnvFilter;

    let level = if debug { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("crabmusic={}", level)))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    Ok(())
}
```

---

## Dependencies

- **Depends on**:
  - AUDIO-002 (audio capture working)
  - AUDIO-003 (ring buffer implemented)
  - DSP-001 (FFT processor working)
  - DSP-002 (audio parameters extracted)
  - VIZ-005 (sine wave visualizer working)
  - RENDER-002 (ratatui rendering working)
  - CONFIG-002 (config loading working)
- **Blocks**: TEST-005 (need complete pipeline to test)

---

## Architecture References

- **Core Workflows**: docs/architecture.md - "Primary Rendering Loop Workflow" sequence diagram
- **Components**: All component sections for initialization order
- **Error Handling**: docs/architecture.md - "Error Handling Strategy"

---

## Testing Requirements

### Integration Tests

```rust
// tests/pipeline_integration_test.rs
#[test]
fn test_pipeline_with_synthetic_audio() {
    // Create synthetic audio source
    let audio_buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);

    // Initialize pipeline components
    let ring_buffer = Arc::new(RingBuffer::new(8192));
    ring_buffer.write(audio_buffer.samples.clone());

    let mut dsp = DspProcessor::new(44100, 2048);
    let mut viz = SineWaveVisualizer::new(SineWaveConfig::default());
    let mut grid = GridBuffer::new(80, 24);

    // Process through pipeline
    let audio_buf = ring_buffer.read().unwrap();
    let params = dsp.process_buffer(&audio_buf);
    viz.update(&params);
    viz.render(&mut grid);

    // Validate: Grid should have non-empty cells
    let filled_cells = grid.count_non_empty();
    assert!(filled_cells > 0, "Visualization produced no output");
}
```

### Performance Tests

```rust
#[test]
fn test_frame_time_under_16ms() {
    // Setup pipeline
    let app = Application::new(AppConfig::default(), 60).unwrap();

    // Time 100 frames
    let start = Instant::now();
    for _ in 0..100 {
        // Simulate frame processing
        // (extract from app.run() loop)
    }
    let elapsed = start.elapsed();

    let avg_frame_time = elapsed / 100;
    assert!(avg_frame_time < Duration::from_millis(16),
            "Average frame time too high: {:?}", avg_frame_time);
}
```

### Manual Testing

**Test Procedure**:
1. Run `cargo run` with default config
2. Play music (any audio) on system
3. Observe:
   - ✅ Sine wave appears immediately
   - ✅ Wave reacts to music (amplitude, thickness, frequency)
   - ✅ Rendering is smooth (no flicker, no lag)
   - ✅ CPU usage reasonable (<15%)
   - ✅ No audio dropouts or glitches
4. Test Ctrl+C shutdown - terminal should restore cleanly

---

## Notes for AI Agent

**Initialization Order Matters**:
1. Config first (everything else depends on it)
2. Ring buffer (audio and DSP need it)
3. Audio capture (starts producing samples)
4. DSP processor (consumes samples)
5. Visualizer (consumes parameters)
6. Terminal renderer last (displays result)

**Frame Timing Strategy**:
- Calculate desired frame duration: `1.0 / target_fps`
- Time each frame: `Instant::now()` at start
- Sleep for remainder: `frame_duration - elapsed`
- If frame takes too long, skip sleep (don't accumulate lag)

**Performance Monitoring**:
- Log actual FPS every second in debug mode
- Log frame time if it exceeds target (warning sign)
- Consider adding `--benchmark` flag for detailed timing of each stage

**Error Handling Philosophy**:
- Initialization errors: Exit immediately with clear message
- Runtime errors: Log warning, continue if possible
- Audio errors: Critical - show error and exit
- Rendering errors: Try to recover, worst case exit cleanly

**Graceful Shutdown Critical**:
- Must call `renderer.cleanup()` to restore terminal
- Must call `audio_device.stop_capture()` to release audio resources
- Use Ctrl+C handler to set shutdown flag, then break from loop
- Don't `std::process::exit()` in loop - let cleanup run

**Ring Buffer Behavior**:
- If no audio available, still render (use last known parameters)
- Don't block waiting for audio (maintain FPS even if silent)
- Ring buffer should be large enough for ~200ms audio (avoid underruns)

**Success Indicator**:
Run app, play music, watch smooth audio-reactive sine wave at 60 FPS.
Ctrl+C exits cleanly with terminal restored.
No panics, no audio glitches, feels responsive and satisfying.

**This is the moment everything comes together - the MVP is alive!** 🎉
