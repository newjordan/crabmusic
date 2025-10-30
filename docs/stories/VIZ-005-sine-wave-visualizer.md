# [VIZ-005] Sine Wave Visualizer (MVP)

**Epic**: Visualization Engine
**Priority**: P0 (Blocking - MVP Core)
**Estimated Effort**: 2-3 days
**Status**: Not Started

---

## Description

Implement the MVP sine wave visualizer that reacts to audio parameters. This is the **proof-of-concept for the entire visualization engine** - it validates the grid-based character coverage system works before building more complex visualizers.

**Agent Instructions**: Create a SineWaveVisualizer that:
- Generates mathematical sine wave shape
- Maps audio parameters to wave properties (amplitude, frequency, thickness)
- Calculates character coverage for each grid cell
- Produces visually smooth, satisfying audio-reactive visuals

---

## Acceptance Criteria

- [ ] SineWaveVisualizer struct implements Visualizer trait
- [ ] Sine wave equation correctly generates wave shape
- [ ] Audio parameters drive wave properties:
  - `overall_amplitude` → wave amplitude (vertical height)
  - `bass` → wave thickness
  - `mid` → wave frequency/speed
- [ ] Character coverage algorithm determines cell fill percentage
- [ ] Coverage maps to character selection (0-25%: ' ', 25-50%: '░', 50-75%: '▒', 75-100%: '▓')
- [ ] Wave renders smoothly across terminal width
- [ ] Visual response feels natural (not jittery or laggy)
- [ ] Configuration allows adjusting sensitivity per parameter
- [ ] Unit tests validate sine math and coverage calculation
- [ ] Visual test: sine wave looks smooth with static parameters

---

## Technical Approach

### SineWaveVisualizer Structure

Reference: **docs/architecture.md - Visualization Engine Component**

```rust
pub struct SineWaveVisualizer {
    phase: f32,          // Current wave phase for animation
    amplitude: f32,      // Current amplitude (smoothed)
    frequency: f32,      // Current frequency (smoothed)
    thickness: f32,      // Current line thickness (smoothed)
    config: SineWaveConfig,
}

#[derive(Debug, Clone)]
pub struct SineWaveConfig {
    pub amplitude_sensitivity: f32,
    pub frequency_sensitivity: f32,
    pub thickness_sensitivity: f32,
    pub base_frequency: f32,      // Base wave cycles across screen
    pub smoothing_factor: f32,     // 0.0-1.0, higher = smoother
}
```

### Visualizer Trait Implementation

```rust
impl Visualizer for SineWaveVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Apply smoothing to prevent jitter
        let smoothing = self.config.smoothing_factor;

        self.amplitude = lerp(
            self.amplitude,
            params.overall_amplitude * self.config.amplitude_sensitivity,
            smoothing
        );

        self.frequency = lerp(
            self.frequency,
            self.config.base_frequency + params.mid * self.config.frequency_sensitivity,
            smoothing
        );

        self.thickness = lerp(
            self.thickness,
            1.0 + params.bass * self.config.thickness_sensitivity,
            smoothing
        );

        // Advance phase for animation
        self.phase += 0.1; // TODO: Make configurable
        if self.phase > 2.0 * std::f32::consts::PI {
            self.phase -= 2.0 * std::f32::consts::PI;
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        for y in 0..grid.height {
            for x in 0..grid.width {
                let coverage = self.calculate_coverage(x, y, grid.width, grid.height);
                let character = select_character_for_coverage(coverage);
                grid.set_cell(x, y, character);
            }
        }
    }
}
```

### Sine Wave Math

```rust
impl SineWaveVisualizer {
    fn calculate_coverage(&self, x: u16, y: u16, width: u16, height: u16) -> f32 {
        // Normalize coordinates to 0.0-1.0
        let norm_x = x as f32 / width as f32;
        let norm_y = y as f32 / height as f32;

        // Calculate sine wave center position at this x coordinate
        let wave_cycles = self.frequency;
        let wave_x = norm_x * wave_cycles * 2.0 * std::f32::consts::PI + self.phase;
        let wave_center_y = 0.5 + self.amplitude * wave_x.sin() * 0.4; // 0.4 keeps it on screen

        // Calculate distance from this cell to wave center
        let distance = (norm_y - wave_center_y).abs();

        // Convert distance to coverage based on thickness
        let half_thickness = self.thickness / height as f32 / 2.0;

        if distance < half_thickness {
            // Full coverage
            1.0
        } else if distance < half_thickness * 2.0 {
            // Partial coverage (anti-aliasing)
            1.0 - ((distance - half_thickness) / half_thickness)
        } else {
            // No coverage
            0.0
        }
    }
}
```

### Character Selection

```rust
fn select_character_for_coverage(coverage: f32) -> char {
    match coverage {
        c if c < 0.25 => ' ',
        c if c < 0.50 => '░',
        c if c < 0.75 => '▒',
        _ => '▓',
    }
}
```

### Linear Interpolation Helper

```rust
fn lerp(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}
```

---

## Dependencies

- **Depends on**:
  - VIZ-001 (GridBuffer exists)
  - VIZ-003 (Character coverage algorithm designed)
  - VIZ-004 (Visualizer trait defined)
  - DSP-002 (AudioParameters available)
- **Blocks**: PIPELINE-001 (need visualizer for integration)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Visualization Engine Component"
- **Brainstorming**: docs/brainstorming-session-results.md - Sine Wave as MVP
- **Tech Stack**: Pure Rust mathematical shape generation

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_wave_center_calculation() {
        let viz = SineWaveVisualizer::new(SineWaveConfig::default());

        // At phase=0, x=0, wave should be at center (0.5)
        let coverage_center = viz.calculate_coverage(40, 12, 80, 24);
        let coverage_off = viz.calculate_coverage(40, 0, 80, 24);

        assert!(coverage_center > coverage_off);
    }

    #[test]
    fn test_coverage_respects_thickness() {
        let mut viz = SineWaveVisualizer::new(SineWaveConfig::default());

        viz.thickness = 1.0;
        let coverage_thin = viz.calculate_coverage(40, 12, 80, 24);

        viz.thickness = 5.0;
        let coverage_thick = viz.calculate_coverage(40, 14, 80, 24);

        // Thicker wave should have more coverage at farther distances
        assert!(coverage_thick > coverage_thin);
    }

    #[test]
    fn test_character_selection_thresholds() {
        assert_eq!(select_character_for_coverage(0.0), ' ');
        assert_eq!(select_character_for_coverage(0.3), '░');
        assert_eq!(select_character_for_coverage(0.6), '▒');
        assert_eq!(select_character_for_coverage(0.9), '▓');
    }

    #[test]
    fn test_smoothing_prevents_jitter() {
        let mut viz = SineWaveVisualizer::new(SineWaveConfig {
            smoothing_factor: 0.1,
            ..Default::default()
        });

        let params = AudioParameters {
            overall_amplitude: 1.0,
            ..Default::default()
        };

        viz.update(&params);
        let amp1 = viz.amplitude;

        viz.update(&params);
        let amp2 = viz.amplitude;

        // Should move toward target but not instantly
        assert!(amp2 > amp1);
        assert!(amp2 < 1.0); // Not instant
    }
}
```

### Visual Tests

Create test harness that renders static sine wave:
```rust
// tests/visual_test.rs
#[test]
fn test_static_sine_wave_renders() {
    let mut viz = SineWaveVisualizer::new(SineWaveConfig::default());
    let mut grid = GridBuffer::new(80, 24);

    let params = AudioParameters {
        overall_amplitude: 0.5,
        bass: 0.5,
        mid: 0.5,
        ..Default::default()
    };

    viz.update(&params);
    viz.render(&mut grid);

    // Print to console for visual inspection
    for y in 0..grid.height {
        for x in 0..grid.width {
            print!("{}", grid.get_cell(x, y));
        }
        println!();
    }

    // Validate: center row should have more filled characters
    // (This is a weak test - mainly for visual inspection)
}
```

---

## Notes for AI Agent

**This is the MVP proof-of-concept - nail this first!**

**Visual Quality Priorities**:
1. **Smoothness**: Wave should look continuous, not blocky
2. **Responsiveness**: Visual should react to audio within 1 frame
3. **Stability**: No jitter when audio is constant
4. **Satisfaction**: Watching the wave should feel good!

**Smoothing is Critical**:
- Without smoothing, visuals will be jittery (raw audio params fluctuate)
- Smoothing factor 0.1-0.3 usually feels good
- Too much smoothing = laggy response
- Too little smoothing = nervous jitter
- Make it configurable so users can tune!

**Character Coverage Algorithm**:
- The "anti-aliasing" zone (half_thickness to 2*half_thickness) creates visual smoothness
- Without it, wave looks stepped/jagged
- Experiment with the falloff curve for best visual quality

**Configuration Tuning**:
Default config suggestions:
```rust
impl Default for SineWaveConfig {
    fn default() -> Self {
        Self {
            amplitude_sensitivity: 0.4,  // Moderate amplitude reaction
            frequency_sensitivity: 2.0,   // Noticeable frequency changes
            thickness_sensitivity: 3.0,   // Visible bass reaction
            base_frequency: 2.0,          // 2 sine wave cycles across screen
            smoothing_factor: 0.2,        // Balanced smoothness
        }
    }
}
```

**Testing Strategy**:
1. Unit test the math (coverage calculation)
2. Visual test with static parameters (does it look good?)
3. Integration test with synthetic audio (does it react correctly?)
4. Manual test with real music (does it feel good?)

**Success Indicator**: You can play music, run the app, and watching the sine wave is satisfying and smooth. The visual clearly reacts to bass, mids, and overall volume.

**Time Estimate**: 2-3 days including tuning to get visual quality right
