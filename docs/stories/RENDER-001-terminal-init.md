# [RENDER-001] Terminal Initialization and Cleanup

**Epic**: Terminal Rendering
**Priority**: P0 (Blocking)
**Estimated Effort**: 1-2 days
**Status**: Not Started

---

## Description

Implement terminal initialization and cleanup using crossterm. This story establishes the foundation for terminal rendering by setting up raw mode, alternate screen, and proper cleanup handlers.

**Agent Instructions**: Implement terminal initialization that:
- Enters raw mode for immediate input handling
- Switches to alternate screen (preserves user's terminal session)
- Captures terminal dimensions
- Provides cleanup that restores terminal state
- Handles panics gracefully to ensure terminal is always restored

---

## Acceptance Criteria

- [ ] TerminalRenderer struct holds crossterm terminal state
- [ ] `new()` initializes terminal in raw mode and alternate screen
- [ ] `cleanup()` restores terminal to original state
- [ ] `dimensions()` returns current terminal size
- [ ] Drop trait ensures cleanup even if not explicitly called
- [ ] Panic handler ensures terminal is restored on panic
- [ ] Minimum terminal size validation (80x24)
- [ ] Unit tests validate initialization and cleanup
- [ ] Integration test verifies terminal state changes

---

## Technical Approach

### TerminalRenderer Structure

Reference: **docs/architecture.md - Terminal Rendering Component**

```rust
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};

pub struct TerminalRenderer {
    stdout: Stdout,
    original_hook: Option<Box<dyn Fn(&std::panic::PanicInfo<'_>) + 'static + Sync + Send>>,
}

impl TerminalRenderer {
    /// Initialize a new terminal renderer
    ///
    /// Sets up the terminal in raw mode and alternate screen.
    ///
    /// # Returns
    /// A new TerminalRenderer instance
    ///
    /// # Errors
    /// Returns `RenderError::InitializationFailed` if terminal setup fails
    /// Returns `RenderError::TerminalTooSmall` if terminal is smaller than minimum size
    pub fn new() -> Result<Self, RenderError> {
        let mut stdout = io::stdout();

        // Check terminal size
        let (width, height) = crossterm::terminal::size()
            .map_err(|_| RenderError::InitializationFailed)?;

        if width < 80 || height < 24 {
            return Err(RenderError::TerminalTooSmall {
                min_width: 80,
                min_height: 24,
            });
        }

        // Enter raw mode
        enable_raw_mode().map_err(|_| RenderError::InitializationFailed)?;

        // Enter alternate screen
        execute!(stdout, EnterAlternateScreen)
            .map_err(|_| RenderError::InitializationFailed)?;

        // Set up panic handler to restore terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            original_hook(panic_info);
        }));

        Ok(Self {
            stdout,
            original_hook: Some(original_hook),
        })
    }

    /// Clean up and restore terminal state
    ///
    /// Should be called before the application exits to restore the terminal
    /// to its original state.
    pub fn cleanup(&mut self) -> Result<(), RenderError> {
        Self::restore_terminal()
    }

    /// Get the current terminal dimensions
    ///
    /// # Returns
    /// A tuple of (width, height) in characters
    pub fn dimensions(&self) -> (u16, u16) {
        crossterm::terminal::size().unwrap_or((80, 24))
    }

    /// Restore terminal to original state (static for panic handler)
    fn restore_terminal() -> Result<(), RenderError> {
        let mut stdout = io::stdout();

        // Leave alternate screen
        execute!(stdout, LeaveAlternateScreen)
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;

        // Disable raw mode
        disable_raw_mode()
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;

        Ok(())
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        // Ensure terminal is cleaned up even if cleanup() wasn't called
        let _ = self.cleanup();
    }
}
```

### Panic Handler Setup

The panic handler ensures that even if the application crashes, the terminal is restored to its original state. This prevents leaving the user's terminal in a broken state.

**Key Design Decisions**:
- Use `std::panic::take_hook()` to preserve existing panic handler
- Call `restore_terminal()` before invoking original hook
- Make `restore_terminal()` static so it can be called from panic handler
- Ignore errors in panic handler (best effort cleanup)

### Terminal Size Validation

Minimum terminal size of 80x24 is enforced to ensure visualizations render correctly. This is a standard terminal size that should be available on all platforms.

---

## Dependencies

- **Depends on**: None (foundation story)
- **Blocks**: RENDER-002 (Ratatui Integration)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Terminal Rendering Component"
- **Source Tree**: docs/architecture/source-tree.md - rendering module
- **Tech Stack**: docs/architecture/tech-stack.md - crossterm 0.27

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_dimensions() {
        // Note: This test requires a terminal to be available
        // May need to be an integration test
        let renderer = TerminalRenderer::new();
        if let Ok(r) = renderer {
            let (width, height) = r.dimensions();
            assert!(width > 0);
            assert!(height > 0);
        }
    }

    #[test]
    fn test_terminal_size_validation() {
        // This test is difficult to unit test as it requires
        // mocking terminal size. Consider integration test instead.
    }
}
```

### Integration Tests

```rust
// tests/terminal_integration_test.rs
#[test]
fn test_terminal_initialization_and_cleanup() {
    use crabmusic::rendering::TerminalRenderer;

    // Initialize terminal
    let mut renderer = TerminalRenderer::new().expect("Failed to initialize terminal");

    // Verify dimensions
    let (width, height) = renderer.dimensions();
    assert!(width >= 80);
    assert!(height >= 24);

    // Cleanup
    renderer.cleanup().expect("Failed to cleanup terminal");
}

#[test]
fn test_terminal_drop_cleanup() {
    use crabmusic::rendering::TerminalRenderer;

    // Initialize terminal
    let renderer = TerminalRenderer::new().expect("Failed to initialize terminal");

    // Drop should trigger cleanup
    drop(renderer);

    // Terminal should be restored (manual verification)
}
```

---

## Implementation Notes

### Cross-Platform Considerations

- **Windows**: crossterm handles Windows Console API automatically
- **Linux/macOS**: crossterm uses ANSI escape sequences
- **Terminal Compatibility**: Works with most modern terminals (Windows Terminal, iTerm2, GNOME Terminal, etc.)

### Error Handling

- Initialization errors should be propagated to main() for clean error messages
- Cleanup errors should be logged but not panic (best effort)
- Panic handler should never panic itself

### Performance

- Terminal initialization is a one-time cost at startup
- Dimension queries are fast (cached by crossterm)
- Cleanup is fast (<1ms)

---

## Notes for AI Agent

**Why Alternate Screen?**
- Preserves user's terminal session (scrollback, previous output)
- Provides clean slate for visualization
- Standard practice for full-screen terminal applications

**Why Raw Mode?**
- Disables line buffering (immediate input)
- Disables echo (input not shown)
- Enables future interactive features (pause, quit, mode switching)

**Panic Handler Importance**:
- Without it, panics leave terminal in broken state (no echo, raw mode stuck)
- User would need to run `reset` command to fix terminal
- Professional applications MUST handle this

**Testing Challenges**:
- Terminal tests require actual terminal access
- CI/CD may not have interactive terminal
- Consider marking tests as `#[ignore]` or integration tests
- Manual testing is important for terminal applications

**Success Indicator**: Terminal enters alternate screen, application can query dimensions, cleanup restores terminal properly

