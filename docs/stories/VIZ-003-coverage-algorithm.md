# [VIZ-003] Character Coverage Algorithm

**Epic**: Visualization Engine
**Priority**: P0 (Blocking)
**Estimated Effort**: 1-2 days
**Status**: Not Started

---

## Description

Implement the character coverage algorithm that determines what percentage of a grid cell is covered by a shape. This is the **secret sauce** of the rendering system - it enables smooth, anti-aliased ASCII visualizations.

**Agent Instructions**: Create a coverage calculation system that:
- Calculates coverage percentage (0.0-1.0) for any grid cell
- Maps coverage to character selection with smooth transitions
- Includes anti-aliasing for smooth visuals
- Is fast enough for 60 FPS rendering (target: <100ns per cell)

---

## Acceptance Criteria

- [ ] `select_character_for_coverage(coverage: f32) -> char` function implemented
- [ ] Coverage thresholds: 0-25% = ' ', 25-50% = '░', 50-75% = '▒', 75-100% = '▓'
- [ ] `lerp(current, target, factor)` helper for smooth interpolation
- [ ] Character selection is deterministic and consistent
- [ ] Performance: Character selection completes in <100ns
- [ ] Unit tests validate all coverage thresholds
- [ ] Unit tests validate lerp behavior
- [ ] Documentation explains coverage calculation approach

---

## Technical Approach

### Character Selection Function

Reference: **docs/architecture.md - Visualization Engine Component**

The coverage algorithm maps a continuous coverage value (0.0-1.0) to discrete characters that represent different fill levels.

```rust
/// Select a character based on coverage percentage
///
/// Maps coverage values to block element characters for smooth visual transitions.
///
/// # Arguments
/// * `coverage` - Coverage percentage (0.0-1.0)
///
/// # Returns
/// Character representing the coverage level
///
/// # Coverage Thresholds
/// - 0.00-0.25: ' ' (empty)
/// - 0.25-0.50: '░' (light shade)
/// - 0.50-0.75: '▒' (medium shade)
/// - 0.75-1.00: '▓' (dark shade)
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::select_character_for_coverage;
///
/// assert_eq!(select_character_for_coverage(0.0), ' ');
/// assert_eq!(select_character_for_coverage(0.3), '░');
/// assert_eq!(select_character_for_coverage(0.6), '▒');
/// assert_eq!(select_character_for_coverage(0.9), '▓');
/// ```
#[inline]
pub fn select_character_for_coverage(coverage: f32) -> char {
    match coverage {
        c if c < 0.25 => ' ',
        c if c < 0.50 => '░',
        c if c < 0.75 => '▒',
        _ => '▓',
    }
}
```

### Linear Interpolation Helper

Used by visualizers to smooth parameter changes and prevent jitter.

```rust
/// Linear interpolation between two values
///
/// Smoothly transitions from current value toward target value.
///
/// # Arguments
/// * `current` - Current value
/// * `target` - Target value
/// * `factor` - Interpolation factor (0.0-1.0)
///   - 0.0 = no change (stay at current)
///   - 1.0 = instant change (jump to target)
///   - 0.1 = slow smooth transition
///
/// # Returns
/// Interpolated value between current and target
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::lerp;
///
/// // Smooth transition
/// let current = 0.0;
/// let target = 1.0;
/// let result = lerp(current, target, 0.1);
/// assert_eq!(result, 0.1); // Moved 10% toward target
///
/// // Instant transition
/// let result = lerp(current, target, 1.0);
/// assert_eq!(result, 1.0); // Jumped to target
/// ```
#[inline]
pub fn lerp(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}
```

### Coverage Calculation (Visualizer-Specific)

**Note**: The actual coverage calculation is visualizer-specific and will be implemented in VIZ-005 (Sine Wave Visualizer). This story only provides the character selection and interpolation utilities.

Example coverage calculation (for reference, implemented in VIZ-005):
```rust
// This is NOT part of VIZ-003 - just showing how it will be used
impl SineWaveVisualizer {
    fn calculate_coverage(&self, x: usize, y: usize, width: usize, height: usize) -> f32 {
        // Normalize coordinates to 0.0-1.0
        let norm_x = x as f32 / width as f32;
        let norm_y = y as f32 / height as f32;

        // Calculate sine wave center position at this x coordinate
        let wave_x = norm_x * self.frequency * 2.0 * std::f32::consts::PI + self.phase;
        let wave_center_y = 0.5 + self.amplitude * wave_x.sin() * 0.4;

        // Calculate distance from this cell to wave center
        let distance = (norm_y - wave_center_y).abs();

        // Convert distance to coverage based on thickness (anti-aliasing)
        let half_thickness = self.thickness / height as f32 / 2.0;

        if distance < half_thickness {
            1.0 // Full coverage
        } else if distance < half_thickness * 2.0 {
            // Partial coverage (anti-aliasing)
            1.0 - ((distance - half_thickness) / half_thickness)
        } else {
            0.0 // No coverage
        }
    }
}
```

---

## Dependencies

- **Depends on**: VIZ-001 (GridBuffer exists)
- **Blocks**: VIZ-005 (Sine Wave Visualizer needs coverage functions)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Visualization Engine Component"
- **Coding Standards**: docs/architecture/coding-standards.md - Performance section

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_selection_empty() {
        assert_eq!(select_character_for_coverage(0.0), ' ');
        assert_eq!(select_character_for_coverage(0.1), ' ');
        assert_eq!(select_character_for_coverage(0.24), ' ');
    }

    #[test]
    fn test_character_selection_light() {
        assert_eq!(select_character_for_coverage(0.25), '░');
        assert_eq!(select_character_for_coverage(0.3), '░');
        assert_eq!(select_character_for_coverage(0.49), '░');
    }

    #[test]
    fn test_character_selection_medium() {
        assert_eq!(select_character_for_coverage(0.50), '▒');
        assert_eq!(select_character_for_coverage(0.6), '▒');
        assert_eq!(select_character_for_coverage(0.74), '▒');
    }

    #[test]
    fn test_character_selection_dark() {
        assert_eq!(select_character_for_coverage(0.75), '▓');
        assert_eq!(select_character_for_coverage(0.9), '▓');
        assert_eq!(select_character_for_coverage(1.0), '▓');
    }

    #[test]
    fn test_character_selection_boundary_values() {
        // Test exact boundary values
        assert_eq!(select_character_for_coverage(0.25), '░');
        assert_eq!(select_character_for_coverage(0.50), '▒');
        assert_eq!(select_character_for_coverage(0.75), '▓');
    }

    #[test]
    fn test_lerp_no_change() {
        let result = lerp(5.0, 10.0, 0.0);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_lerp_instant_change() {
        let result = lerp(5.0, 10.0, 1.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_lerp_partial_change() {
        let result = lerp(0.0, 1.0, 0.5);
        assert_eq!(result, 0.5);

        let result = lerp(0.0, 1.0, 0.1);
        assert!((result - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_lerp_negative_values() {
        let result = lerp(-10.0, 10.0, 0.5);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_lerp_smoothing() {
        // Simulate smoothing over multiple frames
        let mut current = 0.0;
        let target = 1.0;
        let smoothing = 0.1;

        // After 10 frames, should be close to target but not there yet
        for _ in 0..10 {
            current = lerp(current, target, smoothing);
        }

        assert!(current > 0.5); // Made significant progress
        assert!(current < 1.0); // But not instant
    }
}
```

---

## Performance Requirements

- **Character selection**: <100ns per call (inline hint should achieve this)
- **Lerp**: <50ns per call (simple arithmetic)
- **No allocations**: Both functions are pure computation

---

## Notes for AI Agent

**Implementation Location**:
- Add functions to `src/visualization/mod.rs`
- Make them public so visualizers can use them
- Add `#[inline]` hint for performance

**Character Set**:
- Using Unicode block elements: ' ', '░', '▒', '▓'
- These provide 4 levels of fill (0%, 25%, 50%, 75%)
- Future: Could add more granular characters or color support

**Anti-Aliasing**:
- The smooth transitions between characters create anti-aliasing effect
- Visualizers calculate fractional coverage (0.0-1.0)
- Character selection discretizes to 4 levels
- This creates smooth visual appearance despite discrete characters

**Success Indicator**: Unit tests pass, functions are fast and simple

