//! Passthrough effect (no-op for testing)
//!
//! This effect does nothing - it passes the grid through unchanged.
//! Useful for testing the effect pipeline interface and measuring overhead.

use super::Effect;
use crate::dsp::AudioParameters;
use crate::visualization::GridBuffer;

/// Passthrough effect that does nothing
///
/// This effect is a no-op that passes the grid through unchanged.
/// It's useful for:
/// - Testing the Effect trait interface
/// - Measuring pipeline overhead
/// - Placeholder during development
///
/// # Examples
///
/// ```
/// use crabmusic::effects::{Effect, passthrough::PassthroughEffect};
/// use crabmusic::visualization::GridBuffer;
/// use crabmusic::dsp::AudioParameters;
///
/// let mut effect = PassthroughEffect::new();
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// effect.apply(&mut grid, &params);
/// // Grid is unchanged
/// ```
#[derive(Debug, Clone)]
pub struct PassthroughEffect {
    enabled: bool,
    intensity: f32,
}

impl PassthroughEffect {
    /// Create a new passthrough effect
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::passthrough::PassthroughEffect;
    ///
    /// let effect = PassthroughEffect::new();
    /// assert!(effect.is_enabled());
    /// assert_eq!(effect.intensity(), 1.0);
    /// ```
    pub fn new() -> Self {
        Self {
            enabled: true,
            intensity: 1.0,
        }
    }
}

impl Default for PassthroughEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for PassthroughEffect {
    fn apply(&mut self, _grid: &mut GridBuffer, _params: &AudioParameters) {
        // Passthrough - do nothing
        // Even when enabled, this effect has zero impact
    }

    fn name(&self) -> &str {
        "Passthrough"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }

    fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough_new() {
        let effect = PassthroughEffect::new();
        assert!(effect.is_enabled());
        assert_eq!(effect.intensity(), 1.0);
        assert_eq!(effect.name(), "Passthrough");
    }

    #[test]
    fn test_passthrough_enable_disable() {
        let mut effect = PassthroughEffect::new();
        assert!(effect.is_enabled());

        effect.set_enabled(false);
        assert!(!effect.is_enabled());

        effect.set_enabled(true);
        assert!(effect.is_enabled());
    }

    #[test]
    fn test_passthrough_intensity() {
        let mut effect = PassthroughEffect::new();
        assert_eq!(effect.intensity(), 1.0);

        effect.set_intensity(0.5);
        assert_eq!(effect.intensity(), 0.5);

        // Test clamping
        effect.set_intensity(1.5);
        assert_eq!(effect.intensity(), 1.0);

        effect.set_intensity(-0.5);
        assert_eq!(effect.intensity(), 0.0);
    }

    #[test]
    fn test_passthrough_apply() {
        let mut effect = PassthroughEffect::new();
        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Set a cell
        grid.set_cell(5, 5, '█');

        // Apply effect (should do nothing)
        effect.apply(&mut grid, &params);

        // Cell should be unchanged
        assert_eq!(grid.get_cell(5, 5).character, '█');
    }
}
