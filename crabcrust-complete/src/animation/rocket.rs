// Rocket launch animation for git push

use super::Animation;
use crate::braille::{BrailleGrid, Color};
use std::time::Duration;

/// Rocket launching upward animation
pub struct RocketAnimation {
    position_y: f32,
    elapsed: Duration,
    duration: Duration,
    stars: Vec<(usize, usize)>,
}

impl RocketAnimation {
    pub fn new(duration: Duration) -> Self {
        // Generate random stars
        let mut stars = Vec::new();
        for i in 0..50 {
            let x = (i * 17) % 160; // Pseudo-random x
            let y = (i * 31) % 80;  // Pseudo-random y
            stars.push((x, y));
        }

        Self {
            position_y: 0.0,
            elapsed: Duration::ZERO,
            duration,
            stars,
        }
    }
}

impl Default for RocketAnimation {
    fn default() -> Self {
        Self::new(Duration::from_secs(2))
    }
}

impl Animation for RocketAnimation {
    fn update(&mut self, delta_time: Duration) -> bool {
        self.elapsed += delta_time;

        // Calculate position (ease-out)
        let progress = (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        let eased = 1.0 - (1.0 - progress).powi(3); // Cubic ease-out
        self.position_y = eased;

        self.elapsed < self.duration
    }

    fn render(&self, grid: &mut BrailleGrid) {
        let center_x = grid.dot_width() / 2;
        let height = grid.dot_height();

        // Draw stars
        for (x, y) in &self.stars {
            if *x < grid.dot_width() && *y < grid.dot_height() {
                grid.set_dot_with_color(*x, *y, Color::WHITE);
            }
        }

        // Calculate rocket position (moving up)
        let rocket_y = height as f32 * (1.0 - self.position_y);
        let rocket_y = rocket_y as usize;

        if rocket_y < height && rocket_y > 5 {
            // Draw rocket body (simple triangle)
            let size = 8;

            // Nose cone
            for dy in 0..size {
                for dx in 0..=(dy / 2) {
                    let x1 = center_x.saturating_sub(dx);
                    let x2 = (center_x + dx).min(grid.dot_width() - 1);
                    let y = rocket_y.saturating_sub(size - dy);

                    if y < grid.dot_height() {
                        grid.set_dot_with_color(x1, y, Color::RED);
                        grid.set_dot_with_color(x2, y, Color::RED);
                    }
                }
            }

            // Body
            for dy in 0..size {
                for dx in 0..3 {
                    let x1 = center_x.saturating_sub(dx);
                    let x2 = (center_x + dx).min(grid.dot_width() - 1);
                    let y = rocket_y + dy;

                    if y < grid.dot_height() {
                        grid.set_dot_with_color(x1, y, Color::new(255, 255, 255));
                        grid.set_dot_with_color(x2, y, Color::new(200, 200, 200));
                    }
                }
            }

            // Flame (below rocket)
            let flame_size = 6 + ((self.elapsed.as_secs_f32() * 10.0).sin() * 2.0) as usize;
            for dy in 0..flame_size {
                let width = flame_size - dy;
                for dx in 0..=width {
                    let x1 = center_x.saturating_sub(dx);
                    let x2 = (center_x + dx).min(grid.dot_width() - 1);
                    let y = rocket_y + size + dy;

                    if y < grid.dot_height() {
                        let color = if dy < flame_size / 2 {
                            Color::new(255, 255, 0) // Yellow
                        } else {
                            Color::new(255, 100, 0) // Orange
                        };
                        grid.set_dot_with_color(x1, y, color);
                        grid.set_dot_with_color(x2, y, color);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Rocket"
    }

    fn duration(&self) -> Option<Duration> {
        Some(self.duration)
    }
}
