# VIZ-012: Braille Conversion & Terminal Display

**Epic**: Epic 10: Braille-Based 3D Model Viewer
**Priority**: P3 (Experimental)
**Estimated Effort**: 2-3 days
**Status**: Draft

---

## Story

**As a** user of the terminal visualizer,
**I want** the ray tracer's intensity buffer converted to Braille characters and displayed in classic **green-on-black terminal aesthetic**,
**so that** I can see a 3D wireframe sphere with that iconic retro vector display look (think Matrix, oscilloscopes, old CRT terminals).

---

## Description

Implement the final piece of the ray tracing pipeline: converting the 2D intensity buffer into Braille characters and displaying them in the terminal with the **classic green-on-black aesthetic**.

**Visual Goal**: **Green wireframe sphere on black background** - the quintessential retro terminal look.

Key components:
- **Braille Conversion**: Map intensity values (0.0-1.0) to Braille character density
- **Green Terminal Colors**: Use ANSI codes for green text on black background
- **Brightness Levels**: Multiple shades of green for depth (bright green for foreground, darker for background edges)
- **Terminal Display**: Print colored Braille output to terminal
- **Example Program**: Demonstration program that renders and displays a wireframe sphere
- **Integration**: Connect ray tracer output to existing terminal rendering infrastructure (if applicable)

The deliverable is a working example program that shows a beautifully rendered 3D wireframe sphere in green Braille characters on black background.

---

## Acceptance Criteria

### Braille Conversion
- [ ] `intensity_to_braille()` function converts 2D intensity buffer to Braille characters
- [ ] Braille characters are chosen based on intensity (darker = sparse, brighter = dense)
- [ ] Character mapping provides smooth visual gradient (at least 8-10 levels)
- [ ] Wireframe mode produces clean, high-contrast grid lines

### Green-on-Black Aesthetic
- [ ] ANSI color codes implemented for green text (`\x1b[32m` or variants)
- [ ] Multiple green brightness levels supported (e.g., bright green, green, dark green)
- [ ] Background is black (terminal default)
- [ ] Intensity levels map to green brightness (1.0 = bright green, 0.0 = black)
- [ ] Optional: Support for classic phosphor green color (RGB: 0, 255, 0 or similar)

### Demo Program
- [ ] Example program `examples/ray_tracer_demo.rs` created
- [ ] Demo renders sphere in wireframe mode by default
- [ ] Demo prints green Braille output to terminal
- [ ] Terminal output visually shows recognizable wireframe sphere
- [ ] Terminal output has classic green-on-black retro aesthetic
- [ ] Demo runs without errors on typical terminal sizes (80×24 minimum)
- [ ] Performance acceptable for interactive use (<100ms render time)
- [ ] Optional: Command-line flag to switch between wireframe and solid modes

### Testing & Quality
- [ ] Documentation in README or example comments explains usage
- [ ] Integration test verifies Braille conversion produces expected character density
- [ ] Visual output matches retro vector display aesthetic
- [ ] All code formatted with `rustfmt` and passes `clippy`

---

## Technical Approach

### Module Structure

Add Braille conversion utility:

**Option A**: Add to ray_tracer module
```
src/visualization/ray_tracer/
└── braille.rs      # 🆕 Braille conversion utility
```

**Option B**: Add to rendering module (if shared with other visualizers)
```
src/rendering/
└── braille.rs      # 🆕 Braille conversion utility
```

**Recommendation**: Option A for MVP (ray tracer-specific), migrate to Option B if other visualizers need Braille conversion.

### Braille Character Mapping

Braille Unicode range: U+2800 to U+28FF (256 patterns)

For intensity mapping, use subset with increasing density:

```rust
/// Map intensity (0.0 to 1.0) to Braille character
pub fn intensity_to_braille_char(intensity: f32) -> char {
    let index = (intensity * 9.0) as usize;
    match index {
        0 => ' ',        // Empty
        1 => '⠁',       // Sparse
        2 => '⠃',
        3 => '⠇',
        4 => '⠏',
        5 => '⠟',
        6 => '⠿',
        7 => '⡿',
        8 => '⣿',
        _ => '⣿',       // Full (brightest)
    }
}
```

**Alternative approach**: Use actual Braille dot density calculation (more complex but more accurate)

### Green Color Mapping

Map intensity to green ANSI color codes for the classic retro aesthetic:

```rust
/// ANSI color codes for green brightness levels
const ANSI_RESET: &str = "\x1b[0m";
const ANSI_BLACK: &str = "\x1b[30m";          // Background elements
const ANSI_DARK_GREEN: &str = "\x1b[32m";     // Dim lines
const ANSI_GREEN: &str = "\x1b[92m";          // Normal green
const ANSI_BRIGHT_GREEN: &str = "\x1b[1;92m"; // Bright foreground

/// Map intensity to green color code
pub fn intensity_to_green_ansi(intensity: f32) -> &'static str {
    match intensity {
        i if i < 0.01 => ANSI_BLACK,          // Background (invisible)
        i if i < 0.33 => ANSI_DARK_GREEN,     // Dark grid lines
        i if i < 0.66 => ANSI_GREEN,          // Medium brightness
        _ => ANSI_BRIGHT_GREEN,               // Bright foreground
    }
}

/// Wrap character in green ANSI color based on intensity
pub fn colorize_char(c: char, intensity: f32) -> String {
    let color = intensity_to_green_ansi(intensity);
    format!("{}{}{}", color, c, ANSI_RESET)
}
```

### Buffer to String Conversion (with Green Colors)

```rust
/// Convert 2D intensity buffer to green Braille string (multi-line)
pub fn intensity_buffer_to_green_braille(buffer: &[Vec<f32>]) -> String {
    buffer
        .iter()
        .map(|row| {
            row.iter()
                .map(|&intensity| {
                    let char = intensity_to_braille_char(intensity);
                    colorize_char(char, intensity)
                })
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Monochrome version (no colors) for compatibility
pub fn intensity_buffer_to_braille(buffer: &[Vec<f32>]) -> String {
    buffer
        .iter()
        .map(|row| {
            row.iter()
                .map(|&intensity| intensity_to_braille_char(intensity))
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n")
}
```

### Example Program (Green Wireframe Aesthetic)

Create `examples/ray_tracer_demo.rs`:

```rust
use crabmusic::visualization::ray_tracer::*;

fn main() {
    // Get terminal size
    let (width, height) = (80, 24);

    // Create scene with sphere
    let scene = Scene::new_with_sphere_and_light();

    // Create camera
    let camera = Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        4.0,
        3.0,
    );

    // Clear terminal and set black background
    print!("\x1B[2J\x1B[1;1H");    // ANSI clear screen
    print!("\x1B[40m");             // Black background

    // Render in WIREFRAME mode (default) to intensity buffer
    println!("\x1B[92mRendering 3D wireframe sphere...\x1B[0m");
    let buffer = render(&scene, &camera, width, height, RenderMode::Wireframe);

    // Convert to GREEN Braille characters
    let braille_output = intensity_buffer_to_green_braille(&buffer);

    // Display the green wireframe sphere
    println!("{}", braille_output);

    // Info text in green
    println!("\n\x1B[92m3D Wireframe Sphere - Classic Terminal Aesthetic\x1B[0m");
    println!("\x1B[32mPress Ctrl+C to exit\x1B[0m");

    // Keep running or add simple animation (future)
    std::thread::sleep(std::time::Duration::from_secs(10));
}
```

**Key features**:
- ✅ Wireframe mode as default
- ✅ Green ANSI colors (`\x1B[92m` = bright green, `\x1B[32m` = green)
- ✅ Black background (`\x1B[40m`)
- ✅ Classic retro terminal aesthetic

### Integration with Existing Infrastructure

Check if existing code has Braille utilities:
- `src/rendering/mod.rs` - Terminal rendering module
- `src/visualization/character_sets.rs` - Character palette definitions

If found, reuse or integrate. If not, create new module.

---

## Dependencies

**Depends on**:
- VIZ-009 (Ray Tracing Primitives)
- VIZ-010 (Camera & Rendering)
- VIZ-011 (Lighting & Shading)

**Blocks**:
- None (final story in epic)

---

## Architecture References

- **Source Tree**: `docs/architecture/source-tree.md`
  - Rendering module: Lines 234-247
  - Example programs: Line 79 (see night_night.rs, quantum_donut.rs)

- **Coding Standards**: `docs/architecture/coding-standards.md`
  - Documentation standards: Lines 81-102

- **Tech Stack**: `docs/architecture/tech-stack.md`
  - Terminal backend (crossterm): Line 26
  - Rust version 1.75+: Line 11

---

## Testing Requirements

### Unit Tests

**braille.rs tests**:

```rust
#[test]
fn test_braille_char_mapping_min() {
    let char_result = intensity_to_braille_char(0.0);
    assert_eq!(char_result, ' ');
}

#[test]
fn test_braille_char_mapping_max() {
    let char_result = intensity_to_braille_char(1.0);
    assert_eq!(char_result, '⣿');
}

#[test]
fn test_braille_char_mapping_mid() {
    let char_result = intensity_to_braille_char(0.5);
    // Should be somewhere in middle of density range
    assert!(char_result != ' ');
    assert!(char_result != '⣿');
}

#[test]
fn test_buffer_to_braille_dimensions() {
    let buffer = vec![
        vec![0.0, 0.5, 1.0],
        vec![0.0, 0.5, 1.0],
    ];
    let output = intensity_buffer_to_braille(&buffer);

    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].chars().count(), 3);
}

#[test]
fn test_buffer_to_braille_gradient() {
    let buffer = vec![
        vec![0.0, 0.3, 0.6, 1.0],
    ];
    let output = intensity_buffer_to_braille(&buffer);

    let chars: Vec<char> = output.chars().collect();
    // Verify increasing density
    assert_eq!(chars[0], ' ');
    assert_ne!(chars[1], ' ');
    assert_ne!(chars[3], ' ');
}
```

### Integration Test

Create `tests/braille_integration_test.rs`:

```rust
#[test]
fn test_end_to_end_sphere_rendering() {
    use crabmusic::visualization::ray_tracer::*;

    // Render small scene
    let scene = Scene::new_with_sphere_and_light();
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let buffer = render(&scene, &camera, 20, 15);

    // Convert to Braille
    let braille = intensity_buffer_to_braille(&buffer);

    // Verify output structure
    let lines: Vec<&str> = braille.lines().collect();
    assert_eq!(lines.len(), 15);

    // Verify some non-empty characters exist (sphere rendered)
    let total_chars: String = lines.join("");
    let non_space_chars = total_chars.chars().filter(|&c| c != ' ').count();
    assert!(non_space_chars > 10, "Should have substantial sphere rendering");
}
```

### Manual Testing

Run the example and verify visual output:

```bash
cargo run --example ray_tracer_demo
```

**Expected output**:
- Clear sphere shape visible
- Bright side facing light source
- Gradual darkening toward shadow side
- Recognizable as 3D object (not flat circle)

---

## Notes for Dev Agent

### Implementation Order

1. **Braille character mapping** (braille.rs):
   - Start with simple intensity-to-char mapping
   - Unit test edge cases (0.0, 1.0, mid values)

2. **Buffer conversion** (braille.rs):
   - Convert 2D buffer to multi-line string
   - Test dimensions and format

3. **Example program** (examples/ray_tracer_demo.rs):
   - Integrate all ray tracer components
   - Add terminal clearing and display
   - Test with various terminal sizes

4. **Integration test**:
   - Verify end-to-end pipeline
   - Check output has expected structure

5. **Documentation**:
   - Add comments to example program
   - Update README with usage instructions

### Character Selection Strategy

**Simple approach** (recommended for MVP):
- Use predefined list of ~10 Braille chars ordered by visual density
- Map intensity ranges to these chars

**Advanced approach** (future enhancement):
- Calculate actual Braille dot count
- Use full 256-char range dynamically
- Dithering algorithms (Floyd-Steinberg) for better gradients

### Terminal Compatibility

Test on multiple terminals:
- **Good**: iTerm2, Alacritty, Kitty, Windows Terminal
- **OK**: gnome-terminal, konsole
- **Poor**: Older terminals with weak Unicode support

Add note in README about recommended terminals.

### Performance Validation

Target: <100ms for 80×24 rendering

Profile if needed:
```bash
cargo build --release
time cargo run --release --example ray_tracer_demo
```

Expected breakdown:
- Ray tracing: 50-80ms
- Braille conversion: <5ms
- Terminal output: <5ms

### Visual Enhancements (Optional)

If time permits:
- Add ambient lighting term (minimum brightness)
- Add multiple spheres at different depths
- Add simple camera controls (arrow keys to rotate)
- Add animation loop (rotating light source)

These are POST-MVP enhancements, not required for story completion.

### Common Pitfalls

1. **Unicode rendering**: Ensure terminal supports UTF-8
2. **Aspect ratio**: Braille chars are ~2:1 aspect, may need to adjust camera viewport
3. **Character spacing**: Some terminals add extra spacing between Braille chars
4. **Screen clearing**: Use ANSI codes for cross-platform compatibility

### Code Quality

Run before committing:
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test braille_integration_test
cargo run --example ray_tracer_demo
```

---

## Completion Checklist

- [ ] Braille character mapping implemented
- [ ] Green ANSI color functions implemented
- [ ] Buffer to green Braille string conversion implemented
- [ ] Unit tests for Braille conversion pass
- [ ] Example program created (wireframe mode, green colors)
- [ ] Example renders recognizable wireframe sphere
- [ ] Visual output has green-on-black aesthetic
- [ ] Integration test passes
- [ ] Performance <100ms for 80×24
- [ ] Documentation comments complete
- [ ] README updated with usage instructions and screenshot/description
- [ ] Code passes rustfmt and clippy
- [ ] Manual testing on at least 2 terminals successful (verify green colors)

---

## Success Criteria for Demo

When running `cargo run --example ray_tracer_demo`, user should see:

```
Rendering 3D wireframe sphere...

         ⣿       ⣿       ⣿
      ⣿                     ⣿
    ⣿                         ⣿
   ⣿                           ⣿
  ⣿                             ⣿
  ⣿                             ⣿
 ⣿                               ⣿
 ⣿                               ⣿
 ⣿                               ⣿
 ⣿                               ⣿
  ⣿                             ⣿
  ⣿                             ⣿
   ⣿                           ⣿
    ⣿                         ⣿
      ⣿                     ⣿
         ⣿       ⣿       ⣿

3D Wireframe Sphere - Classic Terminal Aesthetic
Press Ctrl+C to exit
```

**Visual Characteristics**:
- ✅ **Green text on black background** (classic terminal look)
- ✅ **Wireframe grid lines** visible as latitude/longitude curves
- ✅ **High contrast** - grid lines bright, empty space black
- ✅ **Recognizable sphere shape** with depth perception
- ✅ **Retro aesthetic** reminiscent of vector displays and oscilloscopes

**Note**:
- Actual output will be in **bright green color** (rendered with ANSI codes)
- Grid density and pattern will vary based on meridian/parallel settings
- Characters shown above represent wireframe structure (actual Braille patterns may differ)

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-11-05 | 1.0 | Initial story creation - split from VIZ-009-MVP | Sarah (PO) |
| 2025-11-05 | 1.1 | Updated to emphasize green-on-black wireframe aesthetic | Sarah (PO) |
