# [VIZ-004] Visualizer Trait Definition

**Epic**: Visualization Engine
**Priority**: P0 (Blocking)
**Estimated Effort**: 0.5-1 day
**Status**: Not Started

---

## Description

Define the Visualizer trait that all visualizers must implement. This trait provides a consistent interface for the rendering pipeline to interact with different visualization modes.

**Agent Instructions**: Create a Visualizer trait that:
- Defines the interface for all visualizers
- Supports updating state from audio parameters
- Supports rendering to a grid buffer
- Provides a name for identification
- Is simple and extensible

---

## Acceptance Criteria

- [ ] Visualizer trait defined with three methods:
  - `update(&mut self, params: &AudioParameters)` - Update internal state
  - `render(&self, grid: &mut GridBuffer)` - Render to grid
  - `name(&self) -> &str` - Return visualizer name
- [ ] Trait is public and well-documented
- [ ] Example mock implementation for testing
- [ ] Unit tests validate mock implementation
- [ ] Documentation explains trait design and usage

---

## Technical Approach

### Visualizer Trait Definition

Reference: **docs/architecture.md - Visualization Engine Component**

```rust
use crate::dsp::AudioParameters;

/// Trait for audio visualizers
///
/// Implementations generate visual representations from audio parameters.
/// Each visualizer produces a GridBuffer that can be rendered to the terminal.
///
/// # Design Philosophy
///
/// The trait is intentionally minimal to support diverse visualization approaches:
/// - `update()` allows visualizers to maintain internal state (phase, smoothing, etc.)
/// - `render()` is separate from update to support frame-independent rendering
/// - `name()` enables runtime identification and configuration
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::{Visualizer, GridBuffer};
/// use crabmusic::dsp::AudioParameters;
///
/// struct SimpleVisualizer;
///
/// impl Visualizer for SimpleVisualizer {
///     fn update(&mut self, params: &AudioParameters) {
///         // Update internal state based on audio
///     }
///
///     fn render(&self, grid: &mut GridBuffer) {
///         // Render visualization to grid
///         grid.set_cell(0, 0, '█');
///     }
///
///     fn name(&self) -> &str {
///         "Simple"
///     }
/// }
/// ```
pub trait Visualizer {
    /// Update visualizer state from audio parameters
    ///
    /// Called once per audio frame to update internal state (e.g., smoothed values,
    /// animation phase, beat detection state). This method should be fast (<1ms)
    /// to maintain real-time performance.
    ///
    /// # Arguments
    /// * `params` - Audio parameters extracted from DSP processing
    ///
    /// # Implementation Notes
    /// - Apply smoothing to prevent jitter
    /// - Update animation state (phase, position, etc.)
    /// - Store parameters for use in render()
    /// - Keep computation minimal - heavy work goes in render()
    fn update(&mut self, params: &AudioParameters);

    /// Render visualization to grid buffer
    ///
    /// Called once per frame to generate the visual representation. This method
    /// should fill the grid buffer with characters based on the current state.
    ///
    /// # Arguments
    /// * `grid` - Grid buffer to render into
    ///
    /// # Implementation Notes
    /// - Clear grid first if needed (or render over existing content)
    /// - Use grid.width() and grid.height() for dimensions
    /// - Use select_character_for_coverage() for smooth visuals
    /// - Target: Complete in <16ms for 60 FPS (preferably <5ms)
    fn render(&self, grid: &mut GridBuffer);

    /// Get the name of this visualizer
    ///
    /// Used for identification in logs, configuration, and UI.
    ///
    /// # Returns
    /// Human-readable name of the visualizer
    ///
    /// # Examples
    /// - "Sine Wave"
    /// - "Spectrum Analyzer"
    /// - "Oscilloscope"
    fn name(&self) -> &str;
}
```

### Mock Implementation for Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::AudioParameters;

    /// Mock visualizer for testing
    struct MockVisualizer {
        update_count: usize,
        render_count: usize,
        last_params: Option<AudioParameters>,
    }

    impl MockVisualizer {
        fn new() -> Self {
            Self {
                update_count: 0,
                render_count: 0,
                last_params: None,
            }
        }
    }

    impl Visualizer for MockVisualizer {
        fn update(&mut self, params: &AudioParameters) {
            self.update_count += 1;
            self.last_params = Some(params.clone());
        }

        fn render(&self, grid: &mut GridBuffer) {
            self.render_count += 1;
            // Simple test pattern: fill first cell
            if grid.width() > 0 && grid.height() > 0 {
                grid.set_cell(0, 0, '█');
            }
        }

        fn name(&self) -> &str {
            "Mock"
        }
    }

    #[test]
    fn test_visualizer_trait_update() {
        let mut viz = MockVisualizer::new();
        let params = AudioParameters {
            bass: 0.5,
            mid: 0.3,
            treble: 0.2,
            amplitude: 0.4,
            beat: false,
        };

        viz.update(&params);

        assert_eq!(viz.update_count, 1);
        assert!(viz.last_params.is_some());
        assert_eq!(viz.last_params.unwrap().bass, 0.5);
    }

    #[test]
    fn test_visualizer_trait_render() {
        let viz = MockVisualizer::new();
        let mut grid = GridBuffer::new(10, 10);

        viz.render(&mut grid);

        assert_eq!(grid.get_cell(0, 0).character, '█');
    }

    #[test]
    fn test_visualizer_trait_name() {
        let viz = MockVisualizer::new();
        assert_eq!(viz.name(), "Mock");
    }

    #[test]
    fn test_visualizer_trait_multiple_updates() {
        let mut viz = MockVisualizer::new();
        let params = AudioParameters::default();

        for _ in 0..10 {
            viz.update(&params);
        }

        assert_eq!(viz.update_count, 10);
    }
}
```

---

## Dependencies

- **Depends on**:
  - VIZ-001 (GridBuffer exists)
  - DSP-002 (AudioParameters exists)
- **Blocks**: VIZ-005 (Sine Wave Visualizer implements this trait)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Visualization Engine Component"
- **Source Tree**: docs/architecture/source-tree.md - visualization module

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_trait_update() {
        // Test that update() is called and parameters are received
    }

    #[test]
    fn test_visualizer_trait_render() {
        // Test that render() modifies the grid
    }

    #[test]
    fn test_visualizer_trait_name() {
        // Test that name() returns expected string
    }

    #[test]
    fn test_visualizer_trait_multiple_updates() {
        // Test that multiple update() calls work correctly
    }
}
```

---

## Notes for AI Agent

**Trait Design Philosophy**:
- **Minimal interface**: Only three methods to keep it simple
- **Separation of concerns**: update() for state, render() for output
- **No configuration method**: Configuration handled in constructors (simpler for MVP)
- **Stateful**: Visualizers maintain internal state between frames

**Why separate update() and render()?**
- Allows frame-independent updates (audio rate ≠ render rate)
- Enables future optimizations (skip renders if nothing changed)
- Clearer separation of concerns

**Future Extensions** (not in this story):
- `configure(&mut self, config: &VisualizerConfig)` - Runtime configuration
- `reset(&mut self)` - Reset to initial state
- `supports_color(&self) -> bool` - Color capability query

**Success Indicator**: Trait compiles, mock implementation passes tests, ready for VIZ-005

