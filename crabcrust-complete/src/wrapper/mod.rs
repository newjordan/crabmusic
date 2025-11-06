// Wrapper module for integrating animations with CLI commands

pub mod git;

use crate::animation::{AnimationPlayer, SaveAnimation, SpinnerAnimation};
use crate::executor::{CommandExecutor, CommandResult};
use anyhow::Result;
use std::time::Duration;

/// Wrapper for CLI commands with animations
pub struct CliWrapper {
    player: AnimationPlayer,
}

impl CliWrapper {
    /// Create a new CLI wrapper
    pub fn new() -> Result<Self> {
        Ok(Self {
            player: AnimationPlayer::new()?,
        })
    }

    /// Run a command with default animations based on success/failure
    pub fn run_with_default_animations(
        &mut self,
        executor: CommandExecutor,
    ) -> Result<CommandResult> {
        // Show loading animation while command runs
        self.player.play_for(SpinnerAnimation::new(), Duration::from_millis(500))?;

        // Execute command
        let result = executor.run()?;

        // Show success or error animation
        if result.success {
            self.player.play(SaveAnimation::default())?;
        } else {
            self.player.play_for(SpinnerAnimation::new(), Duration::from_millis(500))?;
        }

        // Display output
        self.player
            .renderer_mut()
            .render_text(&result.combined_output())?;

        std::thread::sleep(Duration::from_millis(500));

        Ok(result)
    }
}

impl Default for CliWrapper {
    fn default() -> Self {
        Self::new().expect("Failed to create CLI wrapper")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_wrapper_creation() {
        let wrapper = CliWrapper::new();
        assert!(wrapper.is_ok());
    }
}
