// Git-specific wrapper with custom animations

use super::CliWrapper;
use crate::animation::{RocketAnimation, SaveAnimation, SpinnerAnimation};
use crate::executor::{CommandExecutor, CommandResult};
use anyhow::Result;
use std::time::Duration;

/// Git command wrapper with themed animations
pub struct GitWrapper {
    wrapper: CliWrapper,
}

impl GitWrapper {
    /// Create a new Git wrapper
    pub fn new() -> Result<Self> {
        Ok(Self {
            wrapper: CliWrapper::new()?,
        })
    }

    /// Execute a git command with appropriate animation
    pub fn run(&mut self, args: &[&str]) -> Result<CommandResult> {
        let executor = CommandExecutor::new("git", args);

        // Determine animation based on git subcommand
        let subcommand = args.first().copied().unwrap_or("");

        match subcommand {
            "commit" => self.run_commit(executor),
            "push" => self.run_push(executor),
            "pull" => self.run_pull(executor),
            _ => self.wrapper.run_with_default_animations(executor),
        }
    }

    /// Run git commit with save animation
    fn run_commit(&mut self, executor: CommandExecutor) -> Result<CommandResult> {
        use crate::animation::AnimationPlayer;

        let mut player = AnimationPlayer::new()?;

        // Show loading animation
        player.play_for(SpinnerAnimation::new(), Duration::from_millis(500))?;

        // Execute command
        let result = executor.run()?;

        // Show success or error animation
        if result.success {
            player.play(SaveAnimation::default())?;
        }

        // Display output
        player.renderer_mut().render_text(&result.combined_output())?;
        std::thread::sleep(Duration::from_millis(500));

        Ok(result)
    }

    /// Run git push with rocket animation
    fn run_push(&mut self, executor: CommandExecutor) -> Result<CommandResult> {
        use crate::animation::AnimationPlayer;

        let mut player = AnimationPlayer::new()?;

        // Show loading animation
        player.play_for(SpinnerAnimation::new(), Duration::from_millis(500))?;

        // Execute command
        let result = executor.run()?;

        // Show success or error animation
        if result.success {
            player.play(RocketAnimation::new(Duration::from_secs(2)))?;
        }

        // Display output
        player.renderer_mut().render_text(&result.combined_output())?;
        std::thread::sleep(Duration::from_millis(500));

        Ok(result)
    }

    /// Run git pull with download animation
    fn run_pull(&mut self, executor: CommandExecutor) -> Result<CommandResult> {
        // For now use spinner, later we can add a download animation
        self.wrapper.run_with_default_animations(executor)
    }

    /// Execute git command directly (for convenience)
    pub fn commit(&mut self, message: &str) -> Result<CommandResult> {
        self.run(&["commit", "-m", message])
    }

    pub fn push(&mut self) -> Result<CommandResult> {
        self.run(&["push"])
    }

    pub fn pull(&mut self) -> Result<CommandResult> {
        self.run(&["pull"])
    }

    pub fn status(&mut self) -> Result<CommandResult> {
        self.run(&["status"])
    }
}

impl Default for GitWrapper {
    fn default() -> Self {
        Self::new().expect("Failed to create Git wrapper")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_wrapper_creation() {
        let wrapper = GitWrapper::new();
        assert!(wrapper.is_ok());
    }
}
