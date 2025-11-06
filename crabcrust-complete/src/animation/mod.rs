// Animation module for procedural terminal animations

mod spinner;
mod rocket;
mod save;

pub use spinner::SpinnerAnimation;
pub use rocket::RocketAnimation;
pub use save::SaveAnimation;

use crate::braille::BrailleGrid;
use crate::rendering::TerminalRenderer;
use anyhow::Result;
use std::time::{Duration, Instant};

/// Trait for animations
pub trait Animation {
    /// Update animation state
    /// Returns true if animation should continue, false if done
    fn update(&mut self, delta_time: Duration) -> bool;

    /// Render animation to braille grid
    fn render(&self, grid: &mut BrailleGrid);

    /// Get animation name
    fn name(&self) -> &str;

    /// Get total duration (if finite)
    fn duration(&self) -> Option<Duration> {
        None
    }
}

/// Animation player for running animations
pub struct AnimationPlayer {
    renderer: TerminalRenderer,
}

impl AnimationPlayer {
    /// Create a new animation player
    pub fn new() -> Result<Self> {
        Ok(Self {
            renderer: TerminalRenderer::new()?,
        })
    }

    /// Play an animation to completion
    pub fn play<A: Animation>(&mut self, mut animation: A) -> Result<()> {
        let (width, height) = self.renderer.size()?;
        let mut grid = BrailleGrid::new(width as usize, height as usize);

        let mut last_frame = Instant::now();
        let target_fps = 60;
        let frame_duration = Duration::from_millis(1000 / target_fps);

        loop {
            let now = Instant::now();
            let delta = now.duration_since(last_frame);

            // Update animation
            let should_continue = animation.update(delta);

            // Render
            grid.clear();
            animation.render(&mut grid);
            self.renderer.render_braille(&grid)?;

            // Check if done
            if !should_continue {
                break;
            }

            // Frame rate limiting
            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }

            last_frame = now;
        }

        Ok(())
    }

    /// Play animation for a specific duration
    pub fn play_for<A: Animation>(
        &mut self,
        mut animation: A,
        duration: Duration,
    ) -> Result<()> {
        let (width, height) = self.renderer.size()?;
        let mut grid = BrailleGrid::new(width as usize, height as usize);

        let start = Instant::now();
        let mut last_frame = start;
        let target_fps = 60;
        let frame_duration = Duration::from_millis(1000 / target_fps);

        while start.elapsed() < duration {
            let now = Instant::now();
            let delta = now.duration_since(last_frame);

            // Update animation
            animation.update(delta);

            // Render
            grid.clear();
            animation.render(&mut grid);
            self.renderer.render_braille(&grid)?;

            // Frame rate limiting
            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }

            last_frame = now;
        }

        Ok(())
    }

    /// Get access to the terminal renderer
    pub fn renderer_mut(&mut self) -> &mut TerminalRenderer {
        &mut self.renderer
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create animation player")
    }
}
