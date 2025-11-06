// Spinning Braille dot animation

use super::Animation;
use crate::braille::{BrailleGrid, Color};
use std::time::Duration;

/// Spinning animation using Braille dots
pub struct SpinnerAnimation {
    angle: f32,
    elapsed: Duration,
    color: Color,
}

impl SpinnerAnimation {
    pub fn new() -> Self {
        Self {
            angle: 0.0,
            elapsed: Duration::ZERO,
            color: Color::CYAN,
        }
    }

    pub fn with_color(color: Color) -> Self {
        Self {
            angle: 0.0,
            elapsed: Duration::ZERO,
            color,
        }
    }
}

impl Default for SpinnerAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation for SpinnerAnimation {
    fn update(&mut self, delta_time: Duration) -> bool {
        self.elapsed += delta_time;
        self.angle += delta_time.as_secs_f32() * 4.0; // Rotate speed
        true // Infinite animation
    }

    fn render(&self, grid: &mut BrailleGrid) {
        let center_x = grid.dot_width() / 2;
        let center_y = grid.dot_height() / 2;

        // Draw spinning circle with trail
        let radius = 20.min(grid.dot_width().min(grid.dot_height()) / 4);

        for i in 0..8 {
            let trail_angle = self.angle - (i as f32 * 0.3);
            let x = center_x as f32 + trail_angle.cos() * radius as f32;
            let y = center_y as f32 + trail_angle.sin() * radius as f32;

            if x >= 0.0 && y >= 0.0 && (x as usize) < grid.dot_width() && (y as usize) < grid.dot_height() {
                // Fade color based on trail position
                let fade = 1.0 - (i as f32 / 8.0) * 0.7;
                let color = Color::new(
                    (self.color.r as f32 * fade) as u8,
                    (self.color.g as f32 * fade) as u8,
                    (self.color.b as f32 * fade) as u8,
                );

                grid.set_dot_with_color(x as usize, y as usize, color);
            }
        }

        // Draw center dot
        if center_x < grid.dot_width() && center_y < grid.dot_height() {
            grid.set_dot_with_color(center_x, center_y, Color::WHITE);
        }
    }

    fn name(&self) -> &str {
        "Spinner"
    }
}
