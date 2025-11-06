// Example: Creating a custom animation with CrabCrust

use crabcrust::{Animation, AnimationPlayer, BrailleGrid, Color};
use std::time::Duration;

/// A simple pulsing circle animation
struct PulsingCircle {
    elapsed: Duration,
    duration: Duration,
}

impl PulsingCircle {
    fn new(duration: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration,
        }
    }
}

impl Animation for PulsingCircle {
    fn update(&mut self, delta_time: Duration) -> bool {
        self.elapsed += delta_time;
        self.elapsed < self.duration
    }

    fn render(&self, grid: &mut BrailleGrid) {
        let center_x = grid.dot_width() / 2;
        let center_y = grid.dot_height() / 2;

        // Calculate pulsing radius
        let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let pulse = (progress * std::f32::consts::PI * 4.0).sin().abs();
        let radius = (10.0 + pulse * 15.0) as usize;

        // Draw pulsing circle
        grid.draw_circle(
            center_x,
            center_y,
            radius,
            Color::new(
                (255.0 * pulse) as u8,
                (100.0 + 155.0 * pulse) as u8,
                255,
            ),
        );

        // Draw center dot
        grid.set_dot_with_color(center_x, center_y, Color::WHITE);
    }

    fn name(&self) -> &str {
        "PulsingCircle"
    }

    fn duration(&self) -> Option<Duration> {
        Some(self.duration)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Custom Animation Example");
    println!("Creating a pulsing circle animation...\n");

    let mut player = AnimationPlayer::new()?;
    let animation = PulsingCircle::new(Duration::from_secs(3));

    player.play(animation)?;

    println!("\n✨ Animation complete!");

    Ok(())
}
