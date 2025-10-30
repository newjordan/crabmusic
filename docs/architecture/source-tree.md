# CrabMusic Source Tree Structure

**Source**: Extracted from docs/architecture.md
**Purpose**: Definitive directory structure for the project
**Audience**: Dev agents, developers organizing code

---

## Complete Directory Structure

```
crabmusic/
├── Cargo.toml                       # Project dependencies and metadata
├── Cargo.lock                       # Dependency lockfile
├── README.md                        # User-facing project documentation
├── LICENSE                          # License file (TBD)
│
├── .github/
│   └── workflows/
│       └── ci.yml                   # GitHub Actions CI/CD pipeline
│
├── .gitignore                       # Git ignore rules
├── rustfmt.toml                     # Rust formatting configuration (optional)
├── clippy.toml                      # Clippy linting configuration (optional)
│
├── config/                          # Configuration files
│   ├── default.yaml                 # Default configuration
│   └── examples/                    # Example configs for different styles
│       ├── minimal.yaml             # Minimal config example
│       ├── bass_heavy.yaml          # Bass-focused visualization
│       └── spectrum.yaml            # Spectrum analyzer settings
│
├── docs/                            # Documentation
│   ├── architecture.md              # Complete architecture document
│   ├── architecture/                # Sharded architecture sections
│   │   ├── tech-stack.md           # Technology stack reference
│   │   ├── coding-standards.md     # Coding standards (loaded by dev agent)
│   │   ├── source-tree.md          # This file
│   │   ├── components-overview.md  # Component descriptions
│   │   ├── data-models.md          # Data structure definitions
│   │   └── error-handling.md       # Error handling patterns
│   ├── brainstorming-session-results.md  # Project vision and goals
│   ├── implementation-plan.md      # Epic and story breakdown
│   ├── stories/                    # Implementation story files
│   │   ├── README.md               # Story directory guide
│   │   ├── FOUND-001-project-setup.md
│   │   ├── AUDIO-002-cpal-implementation.md
│   │   └── ...                     # Additional story files
│   └── development.md              # Development guide (TBD)
│
├── src/
│   ├── main.rs                     # Application entry point, CLI setup, main loop
│   │
│   ├── audio/                      # Audio capture module
│   │   ├── mod.rs                  # Module definitions and traits
│   │   ├── capture.rs              # AudioCaptureDevice trait definition
│   │   ├── buffer.rs               # AudioBuffer struct
│   │   └── platform/               # Platform-specific implementations
│   │       ├── mod.rs
│   │       └── cpal_device.rs      # CPAL-based cross-platform implementation
│   │
│   ├── dsp/                        # DSP processing module
│   │   ├── mod.rs                  # DSP module interface
│   │   ├── processor.rs            # DspProcessor main implementation
│   │   ├── fft.rs                  # FFT processing logic
│   │   ├── frequency_bands.rs      # Frequency band extraction
│   │   ├── beat_detection.rs       # Beat onset detection
│   │   └── parameters.rs           # AudioParameters struct
│   │
│   ├── visualization/              # Visualization engine module
│   │   ├── mod.rs                  # Visualizer trait and engine
│   │   ├── grid.rs                 # GridBuffer and GridCell structs
│   │   ├── character_sets.rs       # Character set definitions
│   │   ├── coverage.rs             # Character coverage calculation
│   │   └── visualizers/            # Individual visualizer implementations
│   │       ├── mod.rs
│   │       ├── sine_wave.rs        # MVP sine wave visualizer
│   │       ├── spectrum.rs         # Spectrum analyzer (future)
│   │       └── oscilloscope.rs     # Oscilloscope modes (future)
│   │
│   ├── rendering/                  # Terminal rendering module
│   │   ├── mod.rs                  # TerminalRenderer interface
│   │   ├── renderer.rs             # Main renderer implementation
│   │   └── colors.rs               # Color management (future)
│   │
│   ├── config/                     # Configuration module
│   │   ├── mod.rs                  # ConfigManager and AppConfig
│   │   ├── loader.rs               # Config file loading
│   │   ├── validator.rs            # Config validation
│   │   └── watcher.rs              # Hot-reload file watching
│   │
│   └── error.rs                    # Error types and handling
│
├── tests/                          # Integration tests
│   ├── audio_capture_test.rs       # Test audio capture with mock data
│   ├── dsp_processor_test.rs       # Test FFT and parameter extraction
│   ├── visualizer_test.rs          # Test visualization rendering
│   └── pipeline_integration_test.rs # End-to-end pipeline testing
│
└── benches/                        # Performance benchmarks
    ├── fft_benchmark.rs            # FFT performance benchmarks
    └── render_benchmark.rs         # Rendering performance benchmarks
```

---

## Directory Responsibilities

### Root Level

**`Cargo.toml`**: Project manifest
- Dependencies with pinned versions
- Build profiles (release optimization)
- Package metadata

**`Cargo.lock`**: Dependency lockfile
- Committed to version control for reproducibility

**`README.md`**: User-facing documentation
- Installation instructions
- Quick start guide
- Usage examples

---

### `config/` - Configuration Files

**Purpose**: User-facing YAML configuration files

**`default.yaml`**: Shipped with application
- Sensible defaults for all settings
- Works out-of-box for most users

**`examples/`**: Example configurations
- `minimal.yaml` - Bare minimum settings
- `bass_heavy.yaml` - Bass-focused mappings
- `spectrum.yaml` - Spectrum analyzer preset
- Users can copy and modify these

---

### `docs/` - Documentation

**Purpose**: All project documentation

**`architecture.md`**: Complete architecture (master reference)

**`architecture/`**: Sharded sections for dev agents
- `tech-stack.md` - Always loaded by dev agent
- `coding-standards.md` - Always loaded by dev agent
- `source-tree.md` - Always loaded by dev agent (this file)
- Other sections loaded as needed

**`stories/`**: Implementation story files
- Individual story `.md` files
- See `docs/stories/README.md` for structure

---

### `src/` - Application Source Code

#### `src/main.rs` - Application Entry Point

**Responsibilities**:
- CLI argument parsing with `clap`
- Component initialization
- Main rendering loop
- Error handling and logging setup
- Graceful shutdown

**Size**: ~300-400 lines (keep focused)

---

#### `src/audio/` - Audio Capture Module

**Purpose**: All audio input responsibilities

**Module Structure**:
- `mod.rs` - Public API, re-exports
- `capture.rs` - `AudioCaptureDevice` trait
- `buffer.rs` - `AudioBuffer` struct
- `platform/cpal_device.rs` - CPAL implementation

**Key Types**:
- `trait AudioCaptureDevice`
- `struct AudioBuffer`
- `struct CpalAudioDevice`

**Platform abstraction**: Ready for platform-specific optimizations if needed

---

#### `src/dsp/` - DSP Processing Module

**Purpose**: Audio analysis and parameter extraction

**Module Structure**:
- `mod.rs` - Public API
- `processor.rs` - Main `DspProcessor`
- `fft.rs` - FFT computation
- `frequency_bands.rs` - Band isolation
- `beat_detection.rs` - Beat detection (post-MVP)
- `parameters.rs` - `AudioParameters` struct

**Key Types**:
- `struct DspProcessor`
- `struct AudioParameters`

**Independence**: No dependencies on other modules except `audio::AudioBuffer`

---

#### `src/visualization/` - Visualization Engine

**Purpose**: Audio-to-visual transformation

**Module Structure**:
- `mod.rs` - `Visualizer` trait, public API
- `grid.rs` - `GridBuffer`, `GridCell`
- `character_sets.rs` - Character palette definitions
- `coverage.rs` - Coverage calculation algorithms
- `visualizers/` - Individual visualizer implementations

**Key Types**:
- `trait Visualizer`
- `struct GridBuffer`
- `struct SineWaveVisualizer` (MVP)
- Future: `SpectrumVisualizer`, `OscilloscopeVisualizer`

**Extensibility**: New visualizers just implement `Visualizer` trait

---

#### `src/rendering/` - Terminal Rendering

**Purpose**: Terminal display management

**Module Structure**:
- `mod.rs` - Public API
- `renderer.rs` - `TerminalRenderer`
- `colors.rs` - Color management (future)

**Key Types**:
- `struct TerminalRenderer`

**Terminal abstraction**: Uses ratatui/crossterm, isolated from visualization logic

---

#### `src/config/` - Configuration System

**Purpose**: Settings management

**Module Structure**:
- `mod.rs` - `ConfigManager`, `AppConfig`
- `loader.rs` - YAML loading with serde
- `validator.rs` - Configuration validation
- `watcher.rs` - Hot-reload file watching

**Key Types**:
- `struct AppConfig`
- `struct ConfigManager`

**Features**: Validation, defaults, hot-reload

---

#### `src/error.rs` - Error Types

**Purpose**: Centralized error definitions

**Structure**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrabMusicError {
    #[error("Audio error: {0}")]
    AudioCapture(#[from] AudioError),

    #[error("DSP error: {0}")]
    DspProcessing(#[from] DspError),

    #[error("Rendering error: {0}")]
    Rendering(#[from] RenderError),

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

### `tests/` - Integration Tests

**Purpose**: Multi-component integration testing

**Test Files**:
- `audio_capture_test.rs` - Audio capture with synthetic data
- `dsp_processor_test.rs` - FFT and parameter extraction
- `visualizer_test.rs` - Visualization rendering
- `pipeline_integration_test.rs` - Full pipeline end-to-end

**Note**: Unit tests live in `#[cfg(test)]` modules within source files

---

### `benches/` - Performance Benchmarks

**Purpose**: Performance regression tracking with Criterion

**Benchmark Files**:
- `fft_benchmark.rs` - FFT processing performance
- `render_benchmark.rs` - Rendering performance

**Usage**:
```bash
cargo bench
```

**CI Integration**: Benchmarks run on main branch, results archived

---

## File Naming Conventions

**Rust source files**: `snake_case.rs`
- `audio_buffer.rs`, `dsp_processor.rs`

**Test files**: `*_test.rs` in `tests/` directory
- `audio_capture_test.rs`

**Benchmark files**: `*_benchmark.rs` in `benches/` directory
- `fft_benchmark.rs`

**Documentation**: `kebab-case.md`
- `coding-standards.md`, `implementation-plan.md`

**Configuration**: `snake_case.yaml`
- `default.yaml`, `bass_heavy.yaml`

---

## Module Organization Principles

### Clear Boundaries
Each module has a single, well-defined responsibility:
- `audio` - capture only
- `dsp` - processing only
- `visualization` - visual generation only
- `rendering` - terminal display only

### Minimal Dependencies
Dependency graph:
```
main.rs
  ├── audio → (no internal deps)
  ├── dsp → audio::AudioBuffer
  ├── visualization → dsp::AudioParameters
  ├── rendering → visualization::GridBuffer
  └── config → (no internal deps)
```

### Testability
Each module can be tested independently:
- `audio` - Mock audio device
- `dsp` - Synthetic audio buffers
- `visualization` - Synthetic audio parameters
- `rendering` - Mock terminal backend

---

## Growth Strategy

### Adding New Visualizers
1. Create `src/visualization/visualizers/new_viz.rs`
2. Implement `Visualizer` trait
3. Register in `src/visualization/mod.rs`
4. Add to config enum `VisualizerMode`

### Adding New Audio Backends
1. Create `src/audio/platform/new_backend.rs`
2. Implement `AudioCaptureDevice` trait
3. Conditionally compile based on target platform

### Adding New Configuration Sections
1. Add struct to `src/config/mod.rs`
2. Update `AppConfig` with new field
3. Update `config/default.yaml`
4. Add validation in `src/config/validator.rs`

---

**See Also**:
- Full architecture: `docs/architecture.md`
- Tech stack: `docs/architecture/tech-stack.md`
- Coding standards: `docs/architecture/coding-standards.md`
- Implementation plan: `docs/implementation-plan.md`
