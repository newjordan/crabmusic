# CrabCrust 🦀✨

**Add arcade-style animations to your CLI tools!**

Transform boring command-line interfaces into engaging, arcade-like experiences with stunning Braille-based terminal animations. Every git commit becomes a save animation, every push launches a rocket, and every command feels like a celebration!

## ✨ Features

- **High-Resolution Braille Graphics**: Uses Unicode Braille characters (⣿) for 8× terminal resolution (2×4 dots per cell)
- **Procedural Animations**: Hand-crafted animations including spinners, rockets, save disks, and more
- **Git Integration**: Themed animations for git commands (commit, push, pull, etc.)
- **Zero Dependencies on Video Files**: All animations are procedurally generated
- **Terminal-Native**: Works in any modern terminal with Unicode support
- **Fast & Lightweight**: Written in Rust for blazing-fast performance

## 🎮 Demo

```bash
# Test all animations
crabcrust demo all

# Test individual animations
crabcrust demo spinner
crabcrust demo rocket
crabcrust demo save
```

## 🚀 Installation

### From Source

```bash
git clone https://github.com/yourusername/crabcrust.git
cd crabcrust
cargo install --path .
```

### Using Cargo

```bash
cargo install crabcrust
```

## 📖 Usage

### Git Wrapper

The most common use case is wrapping git commands:

```bash
# Use crabcrust to run git commands with animations
crabcrust git commit -m "Add new feature"  # Shows save animation
crabcrust git push                          # Shows rocket launch animation
crabcrust git pull                          # Shows loading spinner
crabcrust git status                        # Shows quick spinner
```

### Shell Alias

For the ultimate experience, add these to your `.bashrc` or `.zshrc`:

```bash
alias git="crabcrust git"
```

Now every git command automatically gets animated!

```bash
git commit -m "This will show a floppy disk save animation! 💾"
git push  # 🚀 Rocket launch!
```

## 🎨 Animations

### Spinner Animation
A smooth rotating circle with a trailing effect - perfect for loading states.

**Used for**: Generic commands, status checks, pull operations

### Rocket Animation
A rocket ship launching upward with flame effects and stars - celebrating your code going live!

**Used for**: `git push`

**Duration**: 2 seconds

**Features**:
- Procedurally generated stars
- Animated flame with flickering effect
- Smooth easing animation

### Save Animation
A floppy disk icon with progress bar and checkmark - the classic save icon!

**Used for**: `git commit`

**Duration**: 1.5 seconds

**Phases**:
1. Disk appears
2. Progress bar fills
3. Checkmark appears
4. Success state

## 🏗️ Architecture

CrabCrust is built on a modular architecture extracted from the [crabmusic](https://github.com/yourusername/crabmusic) project:

```
crabcrust/
├── braille/          # High-res Braille grid rendering
├── rendering/        # Terminal management (Ratatui + Crossterm)
├── animation/        # Animation engine & procedural animations
│   ├── spinner.rs    # Rotating spinner
│   ├── rocket.rs     # Rocket launch
│   └── save.rs       # Floppy disk save
├── executor/         # Command execution & output capture
└── wrapper/          # CLI wrappers (git, cargo, etc.)
```

### Key Components

#### BrailleGrid
High-resolution terminal graphics using Unicode Braille patterns:
- Each terminal cell = 2×4 dots (8 possible dots)
- 256 unique patterns per cell (U+2800 to U+28FF)
- Full RGB color support per cell
- Bresenham's line algorithm for smooth curves
- Circle drawing with midpoint algorithm

#### Animation Trait
Simple trait for creating custom animations:
```rust
pub trait Animation {
    fn update(&mut self, delta_time: Duration) -> bool;
    fn render(&self, grid: &mut BrailleGrid);
    fn name(&self) -> &str;
}
```

#### Command Executor
Spawns subprocesses, captures output, and preserves exit codes:
```rust
let executor = CommandExecutor::new("git", &["status"]);
let result = executor.run()?;
assert_eq!(result.exit_code, 0);
```

## 🔧 Creating Custom Animations

Want to create your own animation? It's easy!

```rust
use crabcrust::{Animation, BrailleGrid, Color};
use std::time::Duration;

struct MyAnimation {
    elapsed: Duration,
}

impl Animation for MyAnimation {
    fn update(&mut self, delta_time: Duration) -> bool {
        self.elapsed += delta_time;
        self.elapsed < Duration::from_secs(2)  // Run for 2 seconds
    }

    fn render(&self, grid: &mut BrailleGrid) {
        let center_x = grid.dot_width() / 2;
        let center_y = grid.dot_height() / 2;

        // Draw something cool!
        grid.draw_circle(center_x, center_y, 20, Color::CYAN);
    }

    fn name(&self) -> &str {
        "MyAnimation"
    }
}
```

## 🎯 Roadmap

- [ ] Video file playback support (FFmpeg integration)
- [ ] More animations (download, merge, error states)
- [ ] Cargo wrapper (`crabcrust cargo build`)
- [ ] NPM wrapper (`crabcrust npm install`)
- [ ] Configuration file for custom mappings
- [ ] Plugin system for community animations
- [ ] Animation library/marketplace
- [ ] Audio support (terminal beeps synced to animations)

## 🤝 Contributing

Contributions welcome! Here's how you can help:

1. **Create new animations**: Add more procedural animations in `src/animation/`
2. **Add CLI wrappers**: Support more tools (cargo, npm, docker, etc.)
3. **Improve rendering**: Optimize BrailleGrid performance
4. **Fix bugs**: Check the issues page
5. **Add tests**: Expand test coverage

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Credits

CrabCrust is built on technology extracted from [crabmusic](https://github.com/yourusername/crabmusic), a real-time terminal-based audio visualizer.

The BrailleGrid rendering system was originally developed for audio visualization and has been adapted for general-purpose terminal animations.

## 🎪 Philosophy

Command-line tools don't have to be boring! We spend hours every day in the terminal - why not make it delightful?

CrabCrust believes that:
- **Feedback should be engaging**: Visual feedback makes commands more satisfying
- **CLI can be beautiful**: Terminal graphics can be stunning with the right techniques
- **Celebration matters**: Every `git push` is an achievement worth celebrating

Made with 🦀 and ✨ by the Rust community.

---

**Star this repo if you love making terminals fun!** ⭐
