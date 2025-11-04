// Terminal rendering module
// Handles terminal initialization and rendering of grid buffers

#![allow(dead_code)]

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
    widgets::Paragraph,
    Terminal,
};
use std::io::{self, Stdout};

/// Zoom mode for rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZoomMode {
    /// Normal rendering (1:1 mapping)
    Normal,
    /// 2× vertical resolution using half-block characters (▀▄)
    Zoom2x,
    /// 4× resolution using quarter-block characters
    Zoom4x,
}

/// Terminal renderer
///
/// Manages terminal state and renders GridBuffer to the terminal display.
/// Uses ratatui and crossterm for cross-platform terminal manipulation.
///
/// Supports automatic resize detection and handling.
/// Supports zoom modes for higher effective resolution.
///
/// # Examples
///
/// ```no_run
/// use crabmusic::rendering::TerminalRenderer;
/// use crabmusic::visualization::GridBuffer;
///
/// let mut renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
/// let mut grid = GridBuffer::new(80, 24);
/// renderer.render(&grid).expect("Failed to render");
/// renderer.cleanup().expect("Failed to cleanup terminal");
/// ```
pub struct TerminalRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    last_size: (u16, u16),
    zoom_mode: ZoomMode,
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
    /// Returns `RenderError::TerminalTooSmall` if terminal is smaller than minimum size (80x24)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crabmusic::rendering::TerminalRenderer;
    ///
    /// let renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
    /// ```
    pub fn new() -> Result<Self, RenderError> {
        let mut stdout = io::stdout();

        // Check terminal size (minimum 40x12 for basic functionality)
        let (width, height) =
            crossterm::terminal::size().map_err(|_| RenderError::InitializationFailed)?;

        if width < 40 || height < 12 {
            return Err(RenderError::TerminalTooSmall {
                min_width: 40,
                min_height: 12,
            });
        }

        // Enter raw mode
        enable_raw_mode().map_err(|_| RenderError::InitializationFailed)?;

        // Enter alternate screen
        execute!(stdout, EnterAlternateScreen).map_err(|_| RenderError::InitializationFailed)?;

        // Set up panic handler to restore terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            original_hook(panic_info);
        }));

        // Create Ratatui terminal
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|_| RenderError::InitializationFailed)?;

        Ok(Self {
            terminal,
            last_size: (width, height),
            zoom_mode: ZoomMode::Normal,
        })
    }

    /// Render a grid buffer to the terminal
    ///
    /// Uses Ratatui's Frame API to efficiently render the grid.
    /// Ratatui handles differential rendering automatically.
    ///
    /// # Arguments
    /// * `grid` - The grid buffer to render
    ///
    /// # Errors
    /// Returns `RenderError::RenderingFailed` if rendering fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crabmusic::rendering::TerminalRenderer;
    /// use crabmusic::visualization::GridBuffer;
    ///
    /// let mut renderer = TerminalRenderer::new().expect("Failed to initialize");
    /// let grid = GridBuffer::new(80, 24);
    /// renderer.render(&grid).expect("Failed to render");
    /// ```
    pub fn render(&mut self, grid: &GridBuffer) -> Result<(), RenderError> {
        self.terminal
            .draw(|frame| {
                let area = frame.size();

                // Convert GridBuffer to Ratatui Lines with color support
                let lines: Vec<Line> = (0..grid.height())
                    .map(|y| {
                        let mut spans = Vec::new();
                        let mut current_color: Option<ratatui::style::Color> = None;
                        let mut current_text = String::new();

                        for x in 0..grid.width() {
                            let cell = grid.get_cell(x, y);
                            let cell_color = cell.foreground_color.map(|c| c.to_ratatui_color());

                            // If color changed, flush current span and start new one
                            if cell_color != current_color {
                                if !current_text.is_empty() {
                                    let span = if let Some(color) = current_color {
                                        Span::styled(current_text.clone(), Style::default().fg(color))
                                    } else {
                                        Span::raw(current_text.clone())
                                    };
                                    spans.push(span);
                                    current_text.clear();
                                }
                                current_color = cell_color;
                            }

                            current_text.push(cell.character);
                        }

                        // Flush remaining text
                        if !current_text.is_empty() {
                            let span = if let Some(color) = current_color {
                                Span::styled(current_text, Style::default().fg(color))
                            } else {
                                Span::raw(current_text)
                            };
                            spans.push(span);
                        }

                        Line::from(spans)
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

    /// Clean up and restore terminal state
    ///
    /// Should be called before the application exits to restore the terminal
    /// to its original state.
    ///
    /// # Errors
    /// Returns `RenderError::RenderingFailed` if cleanup fails
    pub fn cleanup(&mut self) -> Result<(), RenderError> {
        Self::restore_terminal()
    }

    /// Get the current terminal dimensions
    ///
    /// # Returns
    /// A tuple of (width, height) in characters
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crabmusic::rendering::TerminalRenderer;
    ///
    /// let renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
    /// let (width, height) = renderer.dimensions();
    /// assert!(width >= 80);
    /// assert!(height >= 24);
    /// ```
    pub fn dimensions(&self) -> (u16, u16) {
        let size = self.terminal.size().unwrap_or(Rect::new(0, 0, 80, 24));
        (size.width, size.height)
    }

    /// Check if the terminal has been resized since last check
    ///
    /// # Returns
    /// True if the terminal size has changed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crabmusic::rendering::TerminalRenderer;
    ///
    /// let mut renderer = TerminalRenderer::new().expect("Failed to initialize");
    /// if renderer.check_resize() {
    ///     println!("Terminal was resized!");
    /// }
    /// ```
    pub fn check_resize(&mut self) -> bool {
        let current_size = self.dimensions();
        if current_size != self.last_size {
            self.last_size = current_size;
            true
        } else {
            false
        }
    }

    /// Get the last known terminal size
    ///
    /// # Returns
    /// A tuple of (width, height) in characters
    pub fn last_size(&self) -> (u16, u16) {
        self.last_size
    }

    /// Restore terminal to original state (static for panic handler)
    fn restore_terminal() -> Result<(), RenderError> {
        let mut stdout = io::stdout();

        // Leave alternate screen
        execute!(stdout, LeaveAlternateScreen)
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;

        // Disable raw mode
        disable_raw_mode().map_err(|e| RenderError::RenderingFailed(e.to_string()))?;

        Ok(())
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        // Ensure terminal is cleaned up even if cleanup() wasn't called
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires actual terminal - run with `cargo test -- --ignored`
    fn test_terminal_renderer_creation() {
        let renderer = TerminalRenderer::new();
        assert!(renderer.is_ok(), "Failed to create terminal renderer");
    }

    #[test]
    #[ignore] // Requires actual terminal - run with `cargo test -- --ignored`
    fn test_terminal_dimensions() {
        let renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
        let (width, height) = renderer.dimensions();
        assert!(width >= 40, "Terminal width should be at least 40");
        assert!(height >= 12, "Terminal height should be at least 12");
    }

    #[test]
    #[ignore] // Requires actual terminal - run with `cargo test -- --ignored`
    fn test_terminal_resize_detection() {
        let mut renderer = TerminalRenderer::new().expect("Failed to initialize terminal");

        // First check should return false (no resize yet)
        let resized = renderer.check_resize();
        assert!(!resized, "Should not detect resize on first check");

        // Get initial size
        let initial_size = renderer.last_size();
        assert!(initial_size.0 >= 40);
        assert!(initial_size.1 >= 12);
    }

    #[test]
    #[ignore] // Requires actual terminal - run with `cargo test -- --ignored`
    fn test_terminal_cleanup() {
        let mut renderer = TerminalRenderer::new().expect("Failed to initialize terminal");
        let result = renderer.cleanup();
        assert!(result.is_ok(), "Cleanup should succeed");
    }

    #[test]
    #[ignore] // Requires actual terminal - run with `cargo test -- --ignored`
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
        assert!(result.is_ok(), "Render should succeed");
    }

    #[test]
    #[ignore] // Requires actual terminal - run with `cargo test -- --ignored`
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
