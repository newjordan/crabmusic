//! Phosphor Glow Effect
//!
//! Simulates the temporal persistence of CRT monitor phosphors. Bright pixels
//! fade slowly over time, creating trailing effects like old CRT monitors.

use crate::dsp::AudioParameters;
use crate::effects::Effect;
use crate::visualization::{Color, GridBuffer};

/// Phosphor glow effect that simulates CRT phosphor persistence
///
/// This effect creates temporal persistence by blending the current frame
/// with a decayed version of the previous frame. This creates trailing effects
/// where bright elements leave glowing trails as they move.
///
/// # Examples
///
/// ```
/// use crabmusic::effects::phosphor::PhosphorGlowEffect;
///
/// // Create phosphor with 0.3 decay rate and 0.7 intensity
/// let mut effect = PhosphorGlowEffect::new(0.3, 0.7);
/// effect.set_intensity(0.8);
/// ```
#[derive(Debug, Clone)]
pub struct PhosphorGlowEffect {
    /// Whether the effect is enabled
    enabled: bool,
    /// Effect intensity (0.0-1.0, strength of persistence)
    intensity: f32,
    /// Decay rate (0.0-1.0, how fast glow fades - higher = faster fade)
    decay_rate: f32,
    /// Previous frame buffer for temporal persistence
    previous_frame: Vec<Option<Color>>,
}

impl PhosphorGlowEffect {
    /// Create a new phosphor glow effect
    ///
    /// # Arguments
    /// * `decay_rate` - How fast the glow fades (0.0-1.0, higher = faster fade)
    /// * `intensity` - Strength of persistence effect (0.0-1.0)
    ///
    /// # Returns
    /// A new PhosphorGlowEffect with enabled state (true)
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::phosphor::PhosphorGlowEffect;
    ///
    /// let effect = PhosphorGlowEffect::new(0.3, 0.7);
    /// ```
    pub fn new(decay_rate: f32, intensity: f32) -> Self {
        let decay_rate = decay_rate.clamp(0.0, 1.0);
        let intensity = intensity.clamp(0.0, 1.0);

        Self {
            enabled: true,
            intensity,
            decay_rate,
            previous_frame: Vec::new(),
        }
    }

    /// Get the decay rate
    pub fn decay_rate(&self) -> f32 {
        self.decay_rate
    }

    /// Set the decay rate
    ///
    /// # Arguments
    /// * `decay_rate` - Decay rate (clamped to 0.0-1.0)
    pub fn set_decay_rate(&mut self, decay_rate: f32) {
        self.decay_rate = decay_rate.clamp(0.0, 1.0);
    }

    /// Resize internal buffer to match grid dimensions
    fn resize_buffer(&mut self, width: usize, height: usize) {
        let size = width * height;
        if self.previous_frame.len() != size {
            self.previous_frame.resize(size, None);
        }
    }

    /// Blend two colors with a given mix factor
    ///
    /// # Arguments
    /// * `color1` - First color
    /// * `color2` - Second color
    /// * `mix` - Mix factor (0.0 = all color1, 1.0 = all color2)
    ///
    /// # Returns
    /// Blended color
    fn blend_colors(color1: Color, color2: Color, mix: f32) -> Color {
        let mix = mix.clamp(0.0, 1.0);
        let inv_mix = 1.0 - mix;

        Color::new(
            (color1.r as f32 * inv_mix + color2.r as f32 * mix) as u8,
            (color1.g as f32 * inv_mix + color2.g as f32 * mix) as u8,
            (color1.b as f32 * inv_mix + color2.b as f32 * mix) as u8,
        )
    }

    /// Apply decay to a color
    ///
    /// # Arguments
    /// * `color` - Color to decay
    /// * `decay_rate` - Decay rate (0.0-1.0)
    ///
    /// # Returns
    /// Decayed color
    fn decay_color(color: Color, decay_rate: f32) -> Color {
        let fade = 1.0 - decay_rate;

        Color::new(
            (color.r as f32 * fade) as u8,
            (color.g as f32 * fade) as u8,
            (color.b as f32 * fade) as u8,
        )
    }
}

impl Effect for PhosphorGlowEffect {
    /// Apply the phosphor glow effect to the grid buffer
    ///
    /// This method blends the current frame with a decayed version of the
    /// previous frame to create temporal persistence (trailing effects).
    ///
    /// Algorithm:
    /// 1. For each cell, get current color and previous frame color
    /// 2. Decay the previous frame color
    /// 3. Blend current and decayed previous based on intensity
    /// 4. Store current frame for next iteration
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled {
            return;
        }

        let width = grid.width();
        let height = grid.height();

        // Resize buffer if needed
        self.resize_buffer(width, height);

        // Create temporary buffer for new frame
        let mut new_frame = vec![None; width * height];

        // Process each cell
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let cell = grid.get_cell(x, y);
                let current_color = cell.foreground_color;

                // Store current color for next frame
                new_frame[idx] = current_color;

                // Blend with previous frame if we have both colors
                if let Some(current) = current_color {
                    if let Some(previous) = self.previous_frame[idx] {
                        // Decay the previous frame color
                        let decayed = Self::decay_color(previous, self.decay_rate);

                        // Blend current with decayed previous based on intensity
                        // Higher intensity = more persistence (more of decayed color)
                        let blended = Self::blend_colors(current, decayed, self.intensity);

                        // Update grid with blended color
                        let cell_mut = grid.get_cell_mut(x, y);
                        cell_mut.foreground_color = Some(blended);
                    }
                } else if let Some(previous) = self.previous_frame[idx] {
                    // No current color, but we have previous - show decayed previous
                    let decayed = Self::decay_color(previous, self.decay_rate);

                    // Only show if decay hasn't made it too dark
                    if decayed.r > 0 || decayed.g > 0 || decayed.b > 0 {
                        let cell_mut = grid.get_cell_mut(x, y);
                        cell_mut.foreground_color = Some(decayed);
                        // Keep the character if there was one, otherwise use space
                        if cell_mut.character == ' ' {
                            cell_mut.character = '░'; // Light shade for ghost trail
                        }
                    }
                }
            }
        }

        // Update previous frame buffer
        self.previous_frame = new_frame;
    }

    fn name(&self) -> &str {
        "Phosphor"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        // Clear previous frame when disabling to avoid ghosting when re-enabled
        if !enabled {
            self.previous_frame.clear();
        }
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
    fn test_phosphor_new() {
        let effect = PhosphorGlowEffect::new(0.3, 0.7);
        assert_eq!(effect.decay_rate(), 0.3);
        assert_eq!(effect.intensity(), 0.7);
        assert!(effect.is_enabled());
    }

    #[test]
    fn test_phosphor_decay_rate_clamping() {
        let effect = PhosphorGlowEffect::new(1.5, 0.5);
        assert_eq!(effect.decay_rate(), 1.0);

        let effect = PhosphorGlowEffect::new(-0.5, 0.5);
        assert_eq!(effect.decay_rate(), 0.0);
    }

    #[test]
    fn test_phosphor_set_decay_rate() {
        let mut effect = PhosphorGlowEffect::new(0.3, 0.7);
        effect.set_decay_rate(0.5);
        assert_eq!(effect.decay_rate(), 0.5);
    }

    #[test]
    fn test_decay_color() {
        let color = Color::new(100, 150, 200);
        let decayed = PhosphorGlowEffect::decay_color(color, 0.5);

        // With 0.5 decay rate, colors should be multiplied by 0.5
        assert_eq!(decayed.r, 50);
        assert_eq!(decayed.g, 75);
        assert_eq!(decayed.b, 100);
    }

    #[test]
    fn test_blend_colors() {
        let color1 = Color::new(0, 0, 0);
        let color2 = Color::new(100, 100, 100);

        // 0.0 mix = all color1
        let blended = PhosphorGlowEffect::blend_colors(color1, color2, 0.0);
        assert_eq!(blended.r, 0);

        // 1.0 mix = all color2
        let blended = PhosphorGlowEffect::blend_colors(color1, color2, 1.0);
        assert_eq!(blended.r, 100);

        // 0.5 mix = halfway
        let blended = PhosphorGlowEffect::blend_colors(color1, color2, 0.5);
        assert_eq!(blended.r, 50);
    }

    #[test]
    fn test_phosphor_clear_on_disable() {
        let mut effect = PhosphorGlowEffect::new(0.3, 0.7);
        effect.previous_frame = vec![Some(Color::new(255, 255, 255)); 100];

        effect.set_enabled(false);
        assert_eq!(effect.previous_frame.len(), 0);
    }
}
