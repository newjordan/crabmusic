# [FOUND-001] Project Setup and Scaffolding

**Epic**: Project Foundation
**Priority**: P0 (Blocking)
**Estimated Effort**: 0.5-1 day
**Status**: Not Started

---

## Description

Initialize the Rust project with proper structure, dependencies, and basic module organization. This story establishes the foundation that all other work builds upon.

**Agent Instructions**: Create the complete project structure including:
- Cargo.toml with all required dependencies (from tech stack)
- Source tree matching architecture document
- Basic module definitions (no implementation yet)
- Development tooling configuration (rustfmt, clippy)

---

## Acceptance Criteria

- [ ] `cargo new --bin crabmusic` executed successfully
- [ ] Cargo.toml contains all dependencies from architecture tech stack table
- [ ] Source tree matches structure in docs/architecture.md "Source Tree" section
- [ ] All modules have mod.rs files with placeholder traits/structs
- [ ] `cargo build` compiles without errors
- [ ] `cargo test` runs (even if no tests yet)
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt` runs successfully
- [ ] .gitignore configured for Rust projects
- [ ] LICENSE file present (specify license)

---

## Technical Approach

### Cargo.toml Dependencies

Reference: **docs/architecture.md - Tech Stack Table**

Key dependencies to add:
- `cpal = "0.15"`
- `rustfft = "6.1"`
- `spectrum-analyzer = "1.0"`
- `ratatui = "0.26"`
- `crossterm = "0.27"`
- `serde = { version = "1.0", features = ["derive"] }`
- `serde_yaml = "0.9"`
- `clap = { version = "4.5", features = ["derive"] }`
- `anyhow = "1.0"`
- `thiserror = "1.0"`
- `tracing = "0.1"`
- `tracing-subscriber = "0.1"`

Dev dependencies:
- `criterion = "0.5"`

### Module Structure

Create these directories and mod.rs files:
```
src/
├── main.rs
├── audio/
│   └── mod.rs          # AudioCaptureDevice trait, AudioBuffer struct
├── dsp/
│   └── mod.rs          # DspProcessor, AudioParameters
├── visualization/
│   └── mod.rs          # Visualizer trait, GridBuffer
├── rendering/
│   └── mod.rs          # TerminalRenderer
├── config/
│   └── mod.rs          # ConfigManager, AppConfig
└── error.rs            # CrabMusicError enum
```

### Placeholder Trait Examples

In `src/audio/mod.rs`:
```rust
/// Captures audio from system output
pub trait AudioCaptureDevice {
    // TODO: Define interface in AUDIO-001
}

/// Audio buffer containing captured samples
pub struct AudioBuffer {
    // TODO: Define fields
}
```

---

## Dependencies

- **Depends on**: None (first story)
- **Blocks**: All other stories

---

## Architecture References

- **Tech Stack**: docs/architecture.md - "Tech Stack" section
- **Source Tree**: docs/architecture.md - "Source Tree" section
- **Components**: docs/architecture.md - "Components" section

---

## Testing Requirements

### Build Verification
- `cargo build` succeeds
- `cargo build --release` succeeds
- `cargo test` runs (no tests yet, just framework)

### Linting
- `cargo clippy -- -D warnings` passes
- `cargo fmt -- --check` passes

### Documentation
- `cargo doc --no-deps` generates documentation

---

## Notes for AI Agent

**IMPORTANT**: This is pure scaffolding - no implementation logic yet!

**Checklist**:
1. Copy exact dependency versions from architecture tech stack table
2. Create directory structure matching architecture source tree
3. Add placeholder trait definitions with TODO comments
4. Configure rustfmt.toml and clippy.toml if custom rules needed
5. Create empty tests/ and benches/ directories
6. Add basic README.md with build instructions

**Success Indicator**: The project compiles with no errors and all placeholder modules exist

**Time Estimate**: Should take 30-60 minutes to scaffold properly
