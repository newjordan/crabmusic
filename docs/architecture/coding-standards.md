# CrabMusic Coding Standards

**Source**: Extracted from docs/architecture.md
**Purpose**: Mandatory coding standards for human and AI developers
**Audience**: Dev agents, developers writing code

---

## Core Standards

**Language & Runtime**: Rust 1.75+ (stable channel only)

**Style & Linting**:
- Enforce with `rustfmt` (use default style, no custom config)
- Enforce with `clippy::all` + `clippy::pedantic`
- Run in CI - builds fail if checks don't pass
- Use `#[allow(clippy::...)]` sparingly with justification comment

**Test Organization**:
- **Unit tests**: `#[cfg(test)] mod tests { ... }` at bottom of each file
- **Integration tests**: `tests/` directory
- **Benchmarks**: `benches/` directory using `criterion`

---

## Critical Rules

### ❌ NEVER DO THIS

**1. No panics in library code**
- All fallible operations MUST return `Result`
- Only `main.rs` may panic/unwrap after startup validation
- Use `.expect("reason")` instead of `.unwrap()` to document why panic is impossible

**2. No unwrap() without justification**
```rust
// ❌ Bad
let value = option.unwrap();

// ✅ Good - with justification
let value = option.expect("Config validated at startup, this field is required");

// ✅ Better - handle error
let value = option.ok_or(ConfigError::MissingField)?;
```

**3. Audio thread must never block**
- No locks, no allocations, no I/O in audio callback
- Use lock-free ring buffer for communication
- Pre-allocate all buffers at startup

**4. No allocations in hot paths**
- DSP processing: Pre-allocate FFT buffers
- Visualization: Pre-allocate grid buffer
- Rendering: Reuse buffer for differential updates

**5. FFT window size must be power of 2**
- Validate in config loading, not at runtime
- Use `assert!(size.is_power_of_two())` in debug builds

**6. Grid dimensions must match terminal size**
- Validate on resize, prevent buffer overruns
- Handle terminal resize gracefully

**7. All configuration fields must have defaults**
- Use `#[serde(default)]` for optional fields
- Provide sensible defaults in `impl Default`

**8. Floating point comparisons use epsilon**
```rust
// ❌ Bad
if value == 0.0 { }

// ✅ Good
const EPSILON: f32 = 1e-6;
if value.abs() < EPSILON { }
```

---

## API Documentation

**All public APIs must have doc comments**:
```rust
/// Captures audio from system output device
///
/// # Errors
///
/// Returns `AudioError::DeviceNotAvailable` if no audio device found
///
/// # Examples
///
/// ```
/// let device = CpalAudioDevice::new(ring_buffer)?;
/// device.start_capture()?;
/// ```
pub fn start_capture(&mut self) -> Result<(), AudioError> {
    // ...
}
```

**Run `cargo doc` to verify documentation quality**

---

## Naming Conventions

Follow Rust standard naming conventions (no deviations):

| Element | Convention | Example |
|---------|-----------|---------|
| Types | PascalCase | `AudioBuffer`, `DspProcessor` |
| Traits | PascalCase | `AudioCaptureDevice`, `Visualizer` |
| Functions | snake_case | `process_buffer()`, `start_capture()` |
| Variables | snake_case | `sample_rate`, `audio_params` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_SAMPLE_RATE`, `MAX_BUFFER_SIZE` |
| Modules | snake_case | `audio`, `dsp`, `visualization` |

---

## Error Handling

**Error types**:
```rust
// Application code (main.rs)
use anyhow::{Result, Context};

fn main() -> Result<()> {
    let config = load_config("config.yaml")
        .context("Failed to load configuration")?;
    Ok(())
}

// Library code (modules)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Audio device not available")]
    DeviceNotAvailable,

    #[error("Permission denied accessing audio device")]
    PermissionDenied,

    #[error("CPAL error: {0}")]
    CpalError(#[from] cpal::Error),
}
```

**Error propagation**:
- Use `?` operator for ergonomic propagation
- Add context with `.context()` for user-facing errors
- Log errors before propagating if needed for debugging

---

## Rust-Specific Guidelines

### Ownership & Borrowing

**Prefer borrowing over cloning**:
```rust
// ✅ Good - borrow
fn process_buffer(&mut self, buffer: &AudioBuffer) -> Vec<f32> { }

// ❌ Bad - unnecessary clone
fn process_buffer(&mut self, buffer: AudioBuffer) -> Vec<f32> { }
```

**Use `Cow<T>` for potentially borrowed/owned data**:
```rust
use std::borrow::Cow;

struct Config {
    pub name: Cow<'static, str>,
}
```

### Concurrency

**Thread communication**:
- Audio thread → Main thread: Lock-free ring buffer
- Config watcher → Main thread: `tokio::sync::mpsc` channel
- Never use `Mutex` in audio callback

**Example**:
```rust
use ringbuf::RingBuffer;
use std::sync::Arc;

let ring_buffer = Arc::new(RingBuffer::new(8192));
let producer = ring_buffer.clone();
let consumer = ring_buffer.clone();

// Audio thread writes
producer.write(samples);

// Main thread reads
if let Some(buffer) = consumer.read() {
    // Process
}
```

### Performance

**Pre-allocate buffers**:
```rust
// ✅ Good - pre-allocated
pub struct DspProcessor {
    scratch_buffer: Vec<Complex<f32>>, // Pre-allocated at init
}

impl DspProcessor {
    pub fn new(window_size: usize) -> Self {
        Self {
            scratch_buffer: vec![Complex::new(0.0, 0.0); window_size],
        }
    }

    pub fn process_buffer(&mut self, buffer: &AudioBuffer) -> Vec<f32> {
        // Reuse scratch_buffer - no allocation
        self.scratch_buffer[..].fill(Complex::new(0.0, 0.0));
        // ...
    }
}
```

**Use inline hints on hot paths**:
```rust
#[inline]
fn select_character_for_coverage(coverage: f32) -> char {
    match coverage {
        c if c < 0.25 => ' ',
        c if c < 0.50 => '░',
        c if c < 0.75 => '▒',
        _ => '▓',
    }
}
```

**Profile before optimizing**:
```bash
# CPU profiling with flamegraph
cargo install flamegraph
sudo cargo flamegraph --bin crabmusic

# Memory profiling
RUSTFLAGS="-Z print-type-sizes" cargo +nightly build --release
```

### Safety

**Unsafe code requires justification**:
```rust
// Only use unsafe if absolutely necessary
// Justify why it's safe with comments

/// SAFETY: Buffer is guaranteed to be properly aligned and initialized
/// because we pre-allocated it with the correct size in `new()`.
unsafe {
    // Unsafe operation here
}
```

**Expected unsafe usage**: Minimal, possibly in platform-specific audio code if needed

---

## Testing Standards

**Test naming**:
```rust
#[test]
fn test_fft_detects_440hz_sine_wave() { }

#[test]
fn test_stereo_to_mono_conversion() { }

#[test]
fn test_character_coverage_full() { }
```

**Test structure (AAA pattern)**:
```rust
#[test]
fn test_example() {
    // Arrange
    let buffer = AudioBuffer::new(vec![0.0; 1024], 44100, 1);
    let mut processor = DspProcessor::new(44100, 1024);

    // Act
    let result = processor.process_buffer(&buffer);

    // Assert
    assert_eq!(result.len(), 512); // Half of window size
}
```

**Coverage goals**:
- Core logic (DSP, visualization): 80%+ coverage
- Audio/rendering: Best-effort (hardware-dependent)

---

## Code Review Checklist

Before submitting code:

- [ ] `cargo build` compiles without warnings
- [ ] `cargo test` all tests pass
- [ ] `cargo clippy` no warnings
- [ ] `cargo fmt` applied
- [ ] `cargo doc` generates without warnings
- [ ] Added unit tests for new functions
- [ ] Updated integration tests if needed
- [ ] No `unwrap()` without `.expect()` justification
- [ ] No allocations in hot paths
- [ ] Public APIs have doc comments
- [ ] Error types implement `Error` trait
- [ ] Performance-sensitive code benchmarked

---

## CI/CD Requirements

**Pre-commit checks** (run locally):
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps
```

**CI pipeline** (GitHub Actions):
- Runs on Linux, macOS, Windows
- Enforces all pre-commit checks
- Runs benchmarks (informational)
- Checks for vulnerable dependencies with `cargo audit`

---

**See Also**:
- Full architecture: `docs/architecture.md`
- Tech stack: `docs/architecture/tech-stack.md`
- Error handling: `docs/architecture/error-handling.md`
