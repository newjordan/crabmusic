//! Post-processing visual effects framework
//!
//! This module provides a composable effect pipeline for transforming visualizer output.
//! Effects can be layered, enabled/disabled at runtime, and configured with parameters.
//!
//! # Architecture
//!
//! - **Effect trait**: Interface for all effects (scanlines, bloom, CRT, etc.)
//! - **EffectPipeline**: Manages ordered list of effects and applies them sequentially
//!
//! # Examples
//!
//! ```
//! use crabmusic::effects::{EffectPipeline, passthrough::PassthroughEffect};
//! use crabmusic::visualization::GridBuffer;
//! use crabmusic::dsp::AudioParameters;
//!
//! let mut pipeline = EffectPipeline::new();
//! pipeline.add_effect(Box::new(PassthroughEffect::new()));
//!
//! let mut grid = GridBuffer::new(80, 24);
//! let params = AudioParameters::default();
//! pipeline.apply(&mut grid, &params);
//! ```

use crate::dsp::AudioParameters;
use crate::visualization::GridBuffer;

pub mod passthrough;
pub mod grid_overlay;

/// Trait for post-processing visual effects
///
/// Effects transform a GridBuffer in-place, optionally using audio parameters
/// for audio-reactive behavior (e.g., beat-synced effects, amplitude-based intensity).
///
/// # Design Philosophy
///
/// - **In-place transformation**: Effects modify the grid directly (no copying)
/// - **Audio-reactive**: Effects can respond to audio parameters
/// - **Runtime configuration**: All effects support enable/disable and intensity
/// - **Zero-cost when disabled**: Disabled effects should return immediately
///
/// # Examples
///
/// ```
/// use crabmusic::effects::Effect;
/// use crabmusic::visualization::GridBuffer;
/// use crabmusic::dsp::AudioParameters;
///
/// struct MyEffect {
///     enabled: bool,
///     intensity: f32,
/// }
///
/// impl Effect for MyEffect {
///     fn apply(&mut self, grid: &mut GridBuffer, params: &AudioParameters) {
///         if !self.enabled { return; }
///         // Transform grid here...
///     }
///
///     fn name(&self) -> &str { "MyEffect" }
///     fn is_enabled(&self) -> bool { self.enabled }
///     fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
///     fn intensity(&self) -> f32 { self.intensity }
///     fn set_intensity(&mut self, intensity: f32) { self.intensity = intensity.clamp(0.0, 1.0); }
/// }
/// ```
pub trait Effect {
    /// Apply the effect to a grid buffer
    ///
    /// This method transforms the grid in-place. It receives audio parameters
    /// for audio-reactive effects (e.g., beat flash, amplitude modulation).
    ///
    /// # Arguments
    /// * `grid` - Grid buffer to transform (modified in-place)
    /// * `params` - Current audio parameters (for audio-reactive effects)
    ///
    /// # Performance
    /// Should complete in <2ms for 60 FPS real-time performance.
    /// Disabled effects should return immediately (zero overhead).
    fn apply(&mut self, grid: &mut GridBuffer, params: &AudioParameters);

    /// Get effect name for debugging/UI
    ///
    /// Used for identifying effects in the pipeline and displaying in UI.
    fn name(&self) -> &str;

    /// Check if effect is currently enabled
    ///
    /// Disabled effects should skip processing in `apply()`.
    fn is_enabled(&self) -> bool;

    /// Enable or disable the effect
    ///
    /// # Arguments
    /// * `enabled` - true to enable, false to disable
    fn set_enabled(&mut self, enabled: bool);

    /// Get effect intensity (0.0-1.0)
    ///
    /// Intensity controls the strength of the effect.
    /// 0.0 = no effect, 1.0 = full effect.
    fn intensity(&self) -> f32;

    /// Set effect intensity (0.0-1.0)
    ///
    /// # Arguments
    /// * `intensity` - Effect strength (will be clamped to 0.0-1.0)
    fn set_intensity(&mut self, intensity: f32);
}

/// Pipeline for composing multiple effects
///
/// Applies effects in order, each effect seeing the output of the previous effect.
/// The pipeline itself can be enabled/disabled, and individual effects can be
/// toggled independently.
///
/// # Examples
///
/// ```
/// use crabmusic::effects::{EffectPipeline, passthrough::PassthroughEffect, grid_overlay::GridOverlayEffect};
/// use crabmusic::visualization::GridBuffer;
/// use crabmusic::dsp::AudioParameters;
///
/// let mut pipeline = EffectPipeline::new();
/// pipeline.add_effect(Box::new(PassthroughEffect::new()));
/// pipeline.add_effect(Box::new(GridOverlayEffect::new(10)));
///
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// // Apply all effects in sequence
/// pipeline.apply(&mut grid, &params);
///
/// // Disable entire pipeline
/// pipeline.set_enabled(false);
/// ```
pub struct EffectPipeline {
    /// Ordered list of effects
    effects: Vec<Box<dyn Effect>>,
    /// Master enable/disable for entire pipeline
    enabled: bool,
}

impl EffectPipeline {
    /// Create a new empty effect pipeline
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::EffectPipeline;
    ///
    /// let pipeline = EffectPipeline::new();
    /// assert!(pipeline.is_enabled());
    /// assert_eq!(pipeline.effect_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            enabled: true,
        }
    }

    /// Add an effect to the end of the pipeline
    ///
    /// Effects are applied in the order they are added.
    ///
    /// # Arguments
    /// * `effect` - Boxed effect to add
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::{EffectPipeline, passthrough::PassthroughEffect};
    ///
    /// let mut pipeline = EffectPipeline::new();
    /// pipeline.add_effect(Box::new(PassthroughEffect::new()));
    /// assert_eq!(pipeline.effect_count(), 1);
    /// ```
    pub fn add_effect(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }

    /// Remove an effect by name
    ///
    /// Returns the removed effect if found, None otherwise.
    ///
    /// # Arguments
    /// * `name` - Name of the effect to remove
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::{EffectPipeline, passthrough::PassthroughEffect};
    ///
    /// let mut pipeline = EffectPipeline::new();
    /// pipeline.add_effect(Box::new(PassthroughEffect::new()));
    ///
    /// let removed = pipeline.remove_effect("Passthrough");
    /// assert!(removed.is_some());
    /// assert_eq!(pipeline.effect_count(), 0);
    /// ```
    pub fn remove_effect(&mut self, name: &str) -> Option<Box<dyn Effect>> {
        let index = self.effects.iter().position(|e| e.name() == name)?;
        Some(self.effects.remove(index))
    }

    /// Apply all enabled effects in sequence
    ///
    /// If the pipeline is disabled, this is a no-op.
    /// Each effect sees the output of the previous effect.
    ///
    /// # Arguments
    /// * `grid` - Grid buffer to transform
    /// * `params` - Current audio parameters
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::{EffectPipeline, passthrough::PassthroughEffect};
    /// use crabmusic::visualization::GridBuffer;
    /// use crabmusic::dsp::AudioParameters;
    ///
    /// let mut pipeline = EffectPipeline::new();
    /// pipeline.add_effect(Box::new(PassthroughEffect::new()));
    ///
    /// let mut grid = GridBuffer::new(80, 24);
    /// let params = AudioParameters::default();
    /// pipeline.apply(&mut grid, &params);
    /// ```
    pub fn apply(&mut self, grid: &mut GridBuffer, params: &AudioParameters) {
        if !self.enabled {
            return;
        }

        for effect in &mut self.effects {
            effect.apply(grid, params);
        }
    }

    /// Enable or disable the entire pipeline
    ///
    /// When disabled, `apply()` becomes a no-op.
    ///
    /// # Arguments
    /// * `enabled` - true to enable, false to disable
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the pipeline is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get mutable reference to an effect by name
    ///
    /// Useful for adjusting effect parameters at runtime.
    ///
    /// # Arguments
    /// * `name` - Name of the effect to find
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::{EffectPipeline, passthrough::PassthroughEffect};
    ///
    /// let mut pipeline = EffectPipeline::new();
    /// pipeline.add_effect(Box::new(PassthroughEffect::new()));
    ///
    /// if let Some(effect) = pipeline.get_effect_mut("Passthrough") {
    ///     effect.set_intensity(0.5);
    /// }
    /// ```
    pub fn get_effect_mut(&mut self, name: &str) -> Option<&mut Box<dyn Effect>> {
        self.effects.iter_mut().find(|e| e.name() == name)
    }

    /// Get the number of effects in the pipeline
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    /// Get names of all effects in the pipeline
    ///
    /// Returns effects in the order they are applied.
    pub fn effect_names(&self) -> Vec<&str> {
        self.effects.iter().map(|e| e.name()).collect()
    }
}

impl Default for EffectPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::passthrough::PassthroughEffect;

    #[test]
    fn test_pipeline_new() {
        let pipeline = EffectPipeline::new();
        assert!(pipeline.is_enabled());
        assert_eq!(pipeline.effect_count(), 0);
    }

    #[test]
    fn test_pipeline_add_effect() {
        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(PassthroughEffect::new()));
        assert_eq!(pipeline.effect_count(), 1);
        assert_eq!(pipeline.effect_names(), vec!["Passthrough"]);
    }

    #[test]
    fn test_pipeline_remove_effect() {
        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(PassthroughEffect::new()));

        let removed = pipeline.remove_effect("Passthrough");
        assert!(removed.is_some());
        assert_eq!(pipeline.effect_count(), 0);

        let not_found = pipeline.remove_effect("NonExistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_pipeline_enable_disable() {
        let mut pipeline = EffectPipeline::new();
        assert!(pipeline.is_enabled());

        pipeline.set_enabled(false);
        assert!(!pipeline.is_enabled());

        pipeline.set_enabled(true);
        assert!(pipeline.is_enabled());
    }

    #[test]
    fn test_pipeline_get_effect_mut() {
        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(PassthroughEffect::new()));

        let effect = pipeline.get_effect_mut("Passthrough");
        assert!(effect.is_some());
        effect.unwrap().set_intensity(0.5);

        let effect = pipeline.get_effect_mut("Passthrough");
        assert_eq!(effect.unwrap().intensity(), 0.5);
    }

    #[test]
    fn test_pipeline_apply_empty() {
        use crate::dsp::AudioParameters;
        use crate::visualization::GridBuffer;

        let mut pipeline = EffectPipeline::new();
        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Fill grid with test pattern
        for y in 0..10 {
            for x in 0..10 {
                grid.set_cell(x, y, 'X');
            }
        }

        // Apply empty pipeline - should not modify grid
        pipeline.apply(&mut grid, &params);

        // Verify grid is unchanged
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(grid.get_cell(x, y).character, 'X');
            }
        }
    }

    #[test]
    fn test_pipeline_apply_disabled() {
        use crate::dsp::AudioParameters;
        use crate::visualization::GridBuffer;

        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(PassthroughEffect::new()));
        pipeline.set_enabled(false);

        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Fill grid with test pattern
        for y in 0..10 {
            for x in 0..10 {
                grid.set_cell(x, y, 'X');
            }
        }

        // Apply disabled pipeline - should not modify grid
        pipeline.apply(&mut grid, &params);

        // Verify grid is unchanged
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(grid.get_cell(x, y).character, 'X');
            }
        }
    }

    #[test]
    fn test_pipeline_apply_passthrough() {
        use crate::dsp::AudioParameters;
        use crate::visualization::GridBuffer;

        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(PassthroughEffect::new()));

        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Fill grid with test pattern
        for y in 0..10 {
            for x in 0..10 {
                grid.set_cell(x, y, 'X');
            }
        }

        // Apply passthrough effect - should not modify grid
        pipeline.apply(&mut grid, &params);

        // Verify grid is unchanged (passthrough does nothing)
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(grid.get_cell(x, y).character, 'X');
            }
        }
    }

    #[test]
    fn test_pipeline_apply_grid_overlay() {
        use crate::dsp::AudioParameters;
        use crate::visualization::GridBuffer;
        use crate::effects::grid_overlay::GridOverlayEffect;

        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(GridOverlayEffect::new(5)));

        let mut grid = GridBuffer::new(20, 20);
        let params = AudioParameters::default();

        // Apply grid overlay effect
        pipeline.apply(&mut grid, &params);

        // Verify grid has overlay characters at expected positions
        // Grid overlay should draw lines at multiples of spacing (5)
        assert_eq!(grid.get_cell(0, 0).character, '┼'); // Intersection
        assert_eq!(grid.get_cell(5, 0).character, '┼'); // Intersection
        assert_eq!(grid.get_cell(0, 5).character, '┼'); // Intersection
        assert_eq!(grid.get_cell(5, 5).character, '┼'); // Intersection
    }

    #[test]
    fn test_pipeline_performance_empty() {
        use crate::dsp::AudioParameters;
        use crate::visualization::GridBuffer;
        use std::time::Instant;

        let mut pipeline = EffectPipeline::new();
        let mut grid = GridBuffer::new(200, 100); // Typical terminal size
        let params = AudioParameters::default();

        // Warm up
        for _ in 0..10 {
            pipeline.apply(&mut grid, &params);
        }

        // Benchmark: empty pipeline should be <0.1ms (100 microseconds)
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            pipeline.apply(&mut grid, &params);
        }
        let elapsed = start.elapsed();
        let avg_micros = elapsed.as_micros() / iterations;

        println!("Empty pipeline: {} µs/frame (target: <100 µs)", avg_micros);
        assert!(avg_micros < 100, "Empty pipeline too slow: {} µs (target: <100 µs)", avg_micros);
    }

    #[test]
    fn test_pipeline_performance_passthrough() {
        use crate::dsp::AudioParameters;
        use crate::visualization::GridBuffer;
        use std::time::Instant;

        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(PassthroughEffect::new()));
        let mut grid = GridBuffer::new(200, 100); // Typical terminal size
        let params = AudioParameters::default();

        // Warm up
        for _ in 0..10 {
            pipeline.apply(&mut grid, &params);
        }

        // Benchmark: passthrough should be <0.5ms (500 microseconds)
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            pipeline.apply(&mut grid, &params);
        }
        let elapsed = start.elapsed();
        let avg_micros = elapsed.as_micros() / iterations;

        println!("Passthrough effect: {} µs/frame (target: <500 µs)", avg_micros);
        assert!(avg_micros < 500, "Passthrough too slow: {} µs (target: <500 µs)", avg_micros);
    }

    #[test]
    fn test_pipeline_performance_grid_overlay() {
        use crate::dsp::AudioParameters;
        use crate::visualization::GridBuffer;
        use crate::effects::grid_overlay::GridOverlayEffect;
        use std::time::Instant;

        let mut pipeline = EffectPipeline::new();
        pipeline.add_effect(Box::new(GridOverlayEffect::new(10)));
        let mut grid = GridBuffer::new(200, 100); // Typical terminal size
        let params = AudioParameters::default();

        // Warm up
        for _ in 0..10 {
            pipeline.apply(&mut grid, &params);
        }

        // Benchmark: grid overlay should be <2ms (2000 microseconds)
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            pipeline.apply(&mut grid, &params);
        }
        let elapsed = start.elapsed();
        let avg_micros = elapsed.as_micros() / iterations;

        println!("Grid overlay effect: {} µs/frame (target: <2000 µs)", avg_micros);
        assert!(avg_micros < 2000, "Grid overlay too slow: {} µs (target: <2000 µs)", avg_micros);
    }
}

