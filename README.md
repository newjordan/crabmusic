# 🦀 CrabMusic

**Real-time ASCII music visualizer for your terminal**

CrabMusic captures audio and renders beautiful ASCII visualizations directly in your terminal. Visualize your music, games, or any system audio in real-time!

## ✨ Features

- 🎵 **Real-time audio capture** - Microphone input or system audio (Windows WASAPI loopback)
- 🎨 **Multiple character sets** - 7 different ASCII/Unicode styles (blocks, shading, dots, lines, braille, etc.)
- ⚡ **High performance** - Written in Rust for smooth 60 FPS rendering
- 🔧 **Highly configurable** - YAML-based configuration with hot-reload support
- 🎛️ **Flexible audio routing** - Choose input/output devices independently
- 🖥️ **Cross-platform** - Works on Linux, macOS, and Windows
- 🎧 **Audio passthrough** - Hear your audio while visualizing it

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/newjordan/crabmusic.git
cd crabmusic

# Build the project
cargo build --release

# Run with default settings (microphone input)
cargo run --release
```

### Windows: System Audio Capture (Recommended!)

On Windows, you can capture system audio directly without any virtual cables:

```bash
# Capture whatever is playing through your speakers
cargo run --release -- --loopback
```

Play some music and watch it visualize! 🎵

### Usage Examples

```bash
# List available audio devices
cargo run --release -- --list-devices

# Use specific input device (microphone, line-in, etc.)
cargo run --release -- --device "Microphone"

# Use specific output device for playback
cargo run --release -- --output-device "Speakers"

# Adjust sensitivity (if audio is too quiet/loud)
cargo run --release -- --sensitivity 2.0

# Use different character set
cargo run --release -- --charset shading

# Combine options (Windows system audio + specific output)
cargo run --release -- --loopback --output-device "Headphones"

# Enable verbose logging for debugging
cargo run --release -- --verbose

# Show all available options
cargo run --release -- --help
```

## ⌨️ Keyboard Controls

- **`Q`** or **`ESC`** - Quit the application
- **`C`** - Cycle through character sets (basic → extended → blocks → shading → dots → lines → braille)
- **`O`** - Cycle through color schemes (monochrome → rainbow → heat map → blue-purple → green-yellow → cyan-magenta)
- **`V`** - Cycle through visualizer modes (sine wave → spectrum analyzer → oscilloscope)
- **`M`** - Toggle microphone input on/off
- **`+`** or **`=`** - Increase sensitivity by 10%
- **`-`** or **`_`** - Decrease sensitivity by 10%
- **`1-9`** - Set sensitivity preset (1=0.5x, 2=1.0x, 3=1.5x, ..., 9=4.5x)

## 📝 Configuration

CrabMusic uses YAML configuration files. The default configuration is loaded from `config.default.yaml`.

Key configuration options:
- Audio sample rate and buffer sizes
- DSP processing parameters (FFT size, smoothing, frequency ranges)
- Visualization settings (amplitude scale, frequency scale, wave count)
- Character set selection
- Target FPS

See `config.default.yaml` for all available options and detailed comments.

## 🛠️ Development

### Prerequisites

- Rust 1.75 or later
- System audio libraries:
  - **Linux**: ALSA development files (`libasound2-dev` on Debian/Ubuntu)
  - **macOS**: CoreAudio (included with Xcode)
  - **Windows**: WASAPI (included with Windows)

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
│   ├── main.rs              # Application entry point
│   ├── audio/               # Audio capture and playback
│   │   ├── cpal_device.rs   # CPAL-based audio capture
│   │   ├── wasapi_loopback.rs  # Windows WASAPI loopback (system audio)
│   │   ├── output_device.rs # Audio output/passthrough
│   │   └── ring_buffer.rs   # Lock-free ring buffer
│   ├── dsp/                 # DSP processing (FFT, frequency analysis)
│   ├── visualization/       # Visualization engine
│   │   ├── sine_wave.rs     # Sine wave visualizer
│   │   └── character_sets.rs # ASCII/Unicode character sets
│   ├── rendering/           # Terminal rendering with differential updates
│   └── config/              # Configuration management
├── tests/                   # Integration tests
├── benches/                 # Performance benchmarks
└── config.default.yaml      # Default configuration
```

## 🏗️ Architecture

CrabMusic uses a pipeline architecture with lock-free ring buffers for thread communication:

```
┌─────────────────┐
│  Audio Capture  │ (CPAL or WASAPI)
│   (Thread 1)    │
└────────┬────────┘
         │ Ring Buffer
         ▼
┌─────────────────┐
│ DSP Processing  │ (FFT, Frequency Analysis)
│   (Main Loop)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Visualization  │ (Sine Wave, Character Mapping)
│   (Main Loop)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Rendering    │ (Differential Terminal Updates)
│   (Main Loop)   │
└─────────────────┘
```

**Key Design Decisions:**
- **Lock-free ring buffer** for audio data transfer between threads
- **Trait-based audio devices** for polymorphic capture (CPAL vs WASAPI)
- **Differential rendering** to minimize terminal I/O
- **Hot-reload configuration** for live parameter tuning

## 🤝 Contributing

Contributions are welcome! Areas for improvement:
- Additional visualizer modes (spectrum analyzer, oscilloscope, waveform)
- Color support and themes
- Beat detection and rhythm analysis
- More character sets and visual effects
- Performance optimizations

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

Built with these excellent Rust crates:
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio I/O
- [wasapi](https://github.com/HEnquist/wasapi-rs) - Windows WASAPI bindings
- [rustfft](https://github.com/ejmahler/RustFFT) - Fast Fourier Transform
- [spectrum-analyzer](https://github.com/phip1611/spectrum-analyzer) - Frequency analysis
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation

---

## 📊 Current Status

**Version**: 0.1.0 - MVP Complete! ✅

**Implemented Features:**
- ✅ Real-time audio capture (microphone + Windows system audio)
- ✅ Windows WASAPI loopback (no virtual cables needed!)
- ✅ FFT-based frequency analysis with configurable parameters
- ✅ Audio-reactive sine wave visualization
- ✅ 7 character sets (basic, extended, blocks, shading, dots, lines, braille)
- ✅ 60 FPS terminal rendering with differential updates
- ✅ Device selection for input and output
- ✅ Audio passthrough (hear while visualizing)
- ✅ Adjustable sensitivity and DSP parameters
- ✅ YAML configuration with hot-reload
- ✅ Comprehensive test suite (124 tests)

**Planned Features:**
- 🎨 Color support and themes
- 📊 Spectrum analyzer mode
- 🌊 Oscilloscope mode
- 🎵 Beat detection and rhythm analysis
- 🎛️ Real-time sensitivity controls (keyboard shortcuts)
- 📈 Peak detection and visualization
- 🎨 More visual effects and character sets

