# CrabMusic Tech Stack

**Source**: Extracted from docs/architecture.md
**Purpose**: Quick reference for all technology decisions and versions
**Audience**: Dev agents, developers implementing features

---

## Technology Stack Summary

**Language**: Rust 1.75+ (stable channel)
**Architecture**: Single-binary pipeline application
**Platform**: Cross-platform (Linux, macOS, Windows)

---

## Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|---------|-----------|
| **Language** | Rust | 1.75+ | Primary development language | Zero-cost abstractions, no GC pauses (critical for real-time audio), memory safety, excellent async support, strong audio/terminal ecosystem |
| **Audio Capture** | cpal | 0.15+ | Cross-platform audio I/O | Pure Rust, supports ALSA/PulseAudio/JACK/PipeWire (Linux), CoreAudio (macOS), WASAPI (Windows) |
| **FFT Processing** | rustfft | 6.1+ | Fast Fourier Transform | High-performance pure Rust FFT, widely used, well-optimized for audio processing |
| **Audio Analysis** | spectrum-analyzer | 1.0+ | Frequency spectrum extraction | Built on rustfft, provides frequency spectrum for audio with minimal code |
| **Terminal UI** | ratatui | 0.26+ | Terminal user interface framework | Modern fork of tui-rs, active development, widget-based UI, excellent for complex layouts |
| **Terminal Backend** | crossterm | 0.27+ | Low-level terminal manipulation | Cross-platform, default backend for ratatui, handles cursor, colors, input |
| **Configuration** | serde + serde_yaml | 1.0+ / 0.9+ | Settings serialization | De-facto standard for Rust config, human-readable YAML format |
| **Async Runtime** | tokio | 1.35+ (optional) | Async I/O for config hot-reload | Industry standard, enables file watching and async event handling if needed |
| **CLI Arguments** | clap | 4.5+ | Command-line argument parsing | Derive macros for ergonomic CLI, auto-generated help, validation |
| **Error Handling** | anyhow + thiserror | 1.0+ / 1.0+ | Error management | anyhow for applications, thiserror for library-style errors, ergonomic error propagation |
| **Logging** | tracing + tracing-subscriber | 0.1+ | Structured logging and diagnostics | Better than log crate for performance tracing, structured events, production debugging |
| **Build System** | cargo | (bundled with Rust) | Build, test, dependency management | Standard Rust toolchain, excellent dependency resolution |
| **Testing** | cargo test + criterion | (bundled) + 0.5+ | Unit/integration tests + benchmarks | Built-in test framework, criterion for performance benchmarking of DSP code |

---

## Additional Development Tools

| Tool | Purpose |
|------|---------|
| **rustfmt** | Code formatting (enforced in CI) |
| **clippy** | Linting and best practices |
| **cargo-watch** | Auto-rebuild on file changes during development |
| **cargo-criterion** | Benchmark runner for performance testing |

---

## Platform-Specific Considerations

### Linux
- **Primary audio**: CPAL with PipeWire support (modern default)
- **Fallback**: ALSA direct for lower latency on older systems
- **Terminal**: All major emulators supported (alacritty, kitty, gnome-terminal, konsole)

### macOS
- **Audio**: CPAL with CoreAudio backend
- **Terminal**: iTerm2, Terminal.app, alacritty

### Windows
- **Audio**: CPAL with WASAPI backend
- **Terminal**: Windows Terminal, alacritty (note: older terminals may have poor Unicode support)

---

## Cargo.toml Dependencies

**Production Dependencies**:
```toml
[dependencies]
cpal = "0.15"
rustfft = "6.1"
spectrum-analyzer = "1.0"
ratatui = "0.26"
crossterm = "0.27"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.1"
tokio = { version = "1.35", features = ["fs", "time"], optional = true }
```

**Development Dependencies**:
```toml
[dev-dependencies]
criterion = "0.5"
```

**Build Profile (Release)**:
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = "fat"             # Link-time optimization
codegen-units = 1       # Single codegen unit for better optimization
strip = true            # Strip symbols
panic = "abort"         # Smaller binary, faster panic
```

---

## Technology Selection Rationale

### Why Rust?
- **Real-time performance**: Zero-cost abstractions, no GC pauses
- **Memory safety**: Prevents crashes in long-running daemon
- **Rich ecosystem**: RustAudio community active in 2025
- **Excellent terminal support**: Mature Ratatui/Crossterm
- **Project naming**: "crabmusic" 🦀

### Why CPAL?
- Pure Rust, cross-platform
- Abstracts ALSA/PulseAudio/PipeWire/CoreAudio/WASAPI
- Active maintenance, wide usage
- Low-latency capable

### Why RustFFT?
- High-performance, pure Rust
- No C dependencies
- Well-optimized for audio processing
- Comparable to FFTW performance

### Why Ratatui + Crossterm?
- Ratatui: Modern TUI framework (fork of tui-rs)
- Crossterm: Best cross-platform terminal support
- Work seamlessly together (crossterm is ratatui's default backend)
- Active development, good documentation

---

## Version Pinning Policy

**Lock versions for reproducibility**:
- All dependencies pinned to specific minor versions
- Update security patches immediately
- Update minor versions quarterly
- Evaluate major versions before upgrading (breaking changes)

**CI Checks**:
- `cargo audit` - Check for vulnerable dependencies
- `cargo outdated` - Monitor available updates (informational)

---

**See Also**:
- Full architecture: `docs/architecture.md`
- Coding standards: `docs/architecture/coding-standards.md`
- Source tree structure: `docs/architecture/source-tree.md`
