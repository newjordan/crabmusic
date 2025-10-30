# 🦀 CrabMusic

**Real-time ASCII music visualizer for your terminal**

CrabMusic captures system audio output and renders beautiful, audio-reactive ASCII visualizations directly in your terminal. No music player integration needed - it visualizes whatever audio is playing on your system.

## Features

- 🎵 **Real-time audio capture** - Taps system audio output automatically
- 🎨 **ASCII visualization** - Beautiful character-based graphics in your terminal
- ⚡ **High performance** - Written in Rust for smooth 60 FPS rendering
- 🔧 **Configurable** - YAML-based configuration with hot-reload support
- 🖥️ **Cross-platform** - Works on Linux, macOS, and Windows

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/newjordan/crabmusic.git
cd crabmusic

# Build the project
cargo build --release

# Run
cargo run --release
```

### Usage

```bash
# Run with default settings
cargo run --release

# Run with custom sensitivity (if audio is too quiet/loud)
cargo run --release -- --sensitivity 20

# Run in test mode (see test patterns)
cargo run --release -- --test

# Run with verbose logging
cargo run --release -- --verbose

# Show help
cargo run --release -- --help
```

**Controls:**
- Press `q`, `Q`, or `ESC` to quit
- In test mode: Press `1`, `2`, or `3` to switch test patterns

## Configuration

CrabMusic uses YAML configuration files. See `config/default.yaml` for all available options.

Example configurations are provided in `config/examples/`:
- `minimal.yaml` - Bare minimum settings
- `bass_heavy.yaml` - Bass-focused visualization
- `spectrum.yaml` - Spectrum analyzer mode

## Development

### Prerequisites

- Rust 1.75 or later
- System audio libraries (ALSA/PulseAudio on Linux, CoreAudio on macOS, WASAPI on Windows)

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code quality
cargo clippy
cargo fmt
```

### Project Structure

```
crabmusic/
├── src/
│   ├── main.rs           # Application entry point
│   ├── audio/            # Audio capture module
│   ├── dsp/              # DSP processing
│   ├── visualization/    # Visualization engine
│   ├── rendering/        # Terminal rendering
│   ├── config/           # Configuration management
│   └── error.rs          # Error types
├── tests/                # Integration tests
├── benches/              # Performance benchmarks
└── config/               # Configuration files
```

## Architecture

CrabMusic uses a pipeline architecture:

```
Audio Capture → DSP Processing → Visualization → Terminal Rendering
```

See `docs/architecture.md` for detailed architecture documentation.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with:
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio I/O
- [rustfft](https://github.com/ejmahler/RustFFT) - Fast Fourier Transform
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation

---

**Status**: ✅ MVP Complete! Sine wave visualizer working with real-time audio capture.

**Current Features:**
- ✅ Real-time audio capture from microphone
- ✅ FFT-based frequency analysis
- ✅ Audio-reactive sine wave visualization
- ✅ 60 FPS terminal rendering
- ✅ Adjustable sensitivity
- ✅ Test mode for debugging

**Coming Soon:**
- 🎨 Color support
- 📊 Spectrum analyzer mode
- 🎵 Beat detection
- 🎛️ More visualizer modes

