# [RENDER-002] Ratatui Integration for Grid Rendering

**Epic**: Terminal Rendering
**Priority**: P0 (Blocking)
**Estimated Effort**: 1-2 days
**Status**: Not Started

---

## Description

Integrate Ratatui to render GridBuffer to the terminal. This story completes the rendering pipeline by implementing the actual drawing of characters to the screen using Ratatui's efficient rendering system.

**Agent Instructions**: Implement grid rendering that:
- Creates a Ratatui Terminal with crossterm backend
- Renders GridBuffer to terminal using Ratatui's drawing API
- Leverages Ratatui's built-in double-buffering and differential rendering
- Achieves target 60 FPS performance
- Handles terminal resize gracefully

---

## Acceptance Criteria

- [ ] TerminalRenderer holds a Ratatui Terminal instance
- [ ] `render()` method draws GridBuffer to terminal
- [ ] Uses Ratatui's Frame API for efficient rendering
- [ ] Leverages Ratatui's differential rendering (only updates changed cells)
- [ ] Renders at 60 FPS with minimal CPU usage
- [ ] Handles terminal resize events
- [ ] Unit tests validate rendering logic
- [ ] Integration test verifies visual output

---

## Technical Approach

### TerminalRenderer with Ratatui

Reference: **docs/architecture.md - Terminal Rendering Component**

```rust
use crate::error::RenderError;
use crate::visualization::GridBuffer;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};

pub struct TerminalRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalRenderer {
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

        // Set up panic handler
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            original_hook(panic_info);
        }));

        // Create Ratatui terminal
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|_| RenderError::InitializationFailed)?;

        Ok(Self { terminal })
    }

    /// Render a grid buffer to the terminal
    ///
    /// Uses Ratatui's Frame API to efficiently render the grid.
    /// Ratatui handles differential rendering automatically.
    pub fn render(&mut self, grid: &GridBuffer) -> Result<(), RenderError> {
        self.terminal
            .draw(|frame| {
                let area = frame.size();

                // Convert GridBuffer to Ratatui Lines
                let lines: Vec<Line> = (0..grid.height())
                    .map(|y| {
                        let chars: String = (0..grid.width())
                            .map(|x| grid.get_cell(x, y).character)
                            .collect();
                        Line::from(chars)
                    })
                    .collect();

                // Create paragraph widget
                let paragraph = Paragraph::new(lines);

                // Render to frame
                frame.render_widget(paragraph, area);
            })
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<(), RenderError> {
        Self::restore_terminal()
    }

    pub fn dimensions(&self) -> (u16, u16) {
        let size = self.terminal.size().unwrap_or(Rect::new(0, 0, 80, 24));
        (size.width, size.height)
    }

    fn restore_terminal() -> Result<(), RenderError> {
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen)
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
        disable_raw_mode()
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
        Ok(())
    }
}
```

### Rendering Strategy

**Ratatui's Built-in Optimizations**:
- **Double-buffering**: Ratatui maintains two buffers (current and previous)
- **Differential rendering**: Only cells that changed are updated
- **Efficient cursor positioning**: Minimizes terminal commands

**Our Approach**:
1. Convert GridBuffer to Ratatui Lines (one per row)
2. Use Paragraph widget for simple text rendering
3. Let Ratatui handle all optimization automatically
4. Target: <16ms per frame (60 FPS)

### Performance Considerations

- GridBuffer → Lines conversion is O(width * height) but fast (just char copying)
- Ratatui's differential rendering minimizes actual terminal writes
- For 80x24 grid: ~1920 characters, trivial to process
- Bottleneck is terminal I/O, not our code

---

## Dependencies

- **Depends on**:
  - RENDER-001 (Terminal initialization)
  - VIZ-001 (GridBuffer exists)
- **Blocks**: PIPELINE-001 (Main loop needs rendering)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Terminal Rendering Component"
- **Source Tree**: docs/architecture/source-tree.md - rendering module
- **Tech Stack**: docs/architecture/tech-stack.md - ratatui 0.26, crossterm 0.27

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::GridBuffer;

    #[test]
    #[ignore] // Requires actual terminal
    fn test_render_grid_buffer() {
        let mut renderer = TerminalRenderer::new().expect("Failed to initialize");
        let mut grid = GridBuffer::new(80, 24);

        // Fill grid with test pattern
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                grid.set_cell(x, y, if (x + y) % 2 == 0 { '█' } else { ' ' });
            }
        }

        // Render should succeed
        let result = renderer.render(&grid);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires actual terminal
    fn test_render_performance() {
        use std::time::Instant;

        let mut renderer = TerminalRenderer::new().expect("Failed to initialize");
        let grid = GridBuffer::new(80, 24);

        // Measure render time
        let start = Instant::now();
        for _ in 0..60 {
            renderer.render(&grid).expect("Render failed");
        }
        let elapsed = start.elapsed();

        // Should render 60 frames in < 1 second
        assert!(elapsed.as_secs() < 1, "Rendering too slow: {:?}", elapsed);
    }
}
```

### Integration Tests

```rust
// tests/rendering_integration_test.rs
#[test]
#[ignore] // Requires actual terminal
fn test_full_rendering_pipeline() {
    use crabmusic::rendering::TerminalRenderer;
    use crabmusic::visualization::GridBuffer;

    // Initialize renderer
    let mut renderer = TerminalRenderer::new().expect("Failed to initialize");

    // Create test grid
    let mut grid = GridBuffer::new(80, 24);
    grid.set_cell(40, 12, '█'); // Center cell

    // Render
    renderer.render(&grid).expect("Render failed");

    // Cleanup
    renderer.cleanup().expect("Cleanup failed");
}
```

---

## Implementation Notes

### Why Paragraph Widget?

- Simplest widget for rendering text
- No borders or styling needed for MVP
- Efficient for our use case
- Easy to upgrade to custom widget later if needed

### Future Enhancements (Post-MVP)

- **Color support**: Add foreground/background colors to GridCell
- **Custom widget**: Optimize for our specific use case
- **Partial updates**: Only re-render changed regions
- **Terminal resize handling**: Recreate GridBuffer on resize

### Ratatui Best Practices

- Always use `terminal.draw()` for rendering (handles buffering)
- Don't call `terminal.flush()` manually (draw() does it)
- Use widgets instead of raw buffer manipulation
- Let Ratatui handle cursor positioning

---

## Notes for AI Agent

**Ratatui Architecture**:
- `Terminal` manages the backend and buffers
- `Frame` is passed to draw closure for rendering
- `Widgets` implement the `Widget` trait
- `Buffer` is the internal representation (we don't touch it directly)

**Why This Approach Works**:
- GridBuffer is already optimized for terminal rendering
- Ratatui's Paragraph widget is perfect for our needs
- Differential rendering happens automatically
- No need to track changes ourselves

**Testing Challenges**:
- Rendering tests require actual terminal
- Mark tests as `#[ignore]` for CI/CD
- Manual testing is important
- Consider visual regression tests (screenshots)

**Success Indicator**: GridBuffer renders to terminal, 60 FPS achieved, visual output matches expectations

