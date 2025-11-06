// Command executor module

use anyhow::{Context, Result};
use std::process::{Command, Output, Stdio};

/// Result of command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CommandResult {
    /// Create from std::process::Output
    fn from_output(output: Output) -> Self {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        Self {
            stdout,
            stderr,
            exit_code,
            success,
        }
    }

    /// Get combined output (stdout + stderr)
    pub fn combined_output(&self) -> String {
        let mut result = String::new();
        if !self.stdout.is_empty() {
            result.push_str(&self.stdout);
        }
        if !self.stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&self.stderr);
        }
        result
    }
}

/// Command executor for running CLI tools
pub struct CommandExecutor {
    program: String,
    args: Vec<String>,
    cwd: Option<String>,
}

impl CommandExecutor {
    /// Create a new command executor
    ///
    /// # Arguments
    /// * `program` - The program to execute (e.g., "git", "cargo")
    /// * `args` - Arguments to pass to the program
    ///
    /// # Example
    /// ```
    /// use crabcrust::executor::CommandExecutor;
    ///
    /// let executor = CommandExecutor::new("git", &["status"]);
    /// ```
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            cwd: None,
        }
    }

    /// Set working directory for command execution
    pub fn with_cwd(mut self, cwd: &str) -> Self {
        self.cwd = Some(cwd.to_string());
        self
    }

    /// Execute the command and return the result
    pub fn run(&self) -> Result<CommandResult> {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args);

        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }

        // Capture stdout and stderr
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .with_context(|| format!("Failed to execute command: {} {:?}", self.program, self.args))?;

        Ok(CommandResult::from_output(output))
    }

    /// Execute the command asynchronously
    pub async fn run_async(&self) -> Result<CommandResult> {
        let program = self.program.clone();
        let args = self.args.clone();
        let cwd = self.cwd.clone();

        tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new(&program);
            cmd.args(&args);

            if let Some(cwd) = cwd {
                cmd.current_dir(cwd);
            }

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let output = cmd
                .output()
                .with_context(|| format!("Failed to execute command: {} {:?}", program, args))?;

            Ok(CommandResult::from_output(output))
        })
        .await
        .context("Failed to join async task")?
    }

    /// Get the command as a string for display
    pub fn command_string(&self) -> String {
        format!("{} {}", self.program, self.args.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor_creation() {
        let executor = CommandExecutor::new("echo", &["hello"]);
        assert_eq!(executor.program, "echo");
        assert_eq!(executor.args, vec!["hello"]);
    }

    #[test]
    fn test_command_execution() {
        let executor = CommandExecutor::new("echo", &["hello", "world"]);
        let result = executor.run().expect("Failed to run echo");

        assert!(result.success);
        assert!(result.stdout.contains("hello world"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_command_string() {
        let executor = CommandExecutor::new("git", &["commit", "-m", "test"]);
        assert_eq!(executor.command_string(), "git commit -m test");
    }

    #[test]
    fn test_combined_output() {
        let result = CommandResult {
            stdout: "output".to_string(),
            stderr: "error".to_string(),
            exit_code: 0,
            success: true,
        };

        let combined = result.combined_output();
        assert!(combined.contains("output"));
        assert!(combined.contains("error"));
    }
}
