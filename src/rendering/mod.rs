// Terminal rendering module
// Handles terminal initialization and rendering of grid buffers

#![allow(dead_code)]

use crate::error::RenderError;
use crate::visualization::braille::BrailleGrid;
use crate::visualization::{Color, GridBuffer, GridCell};
use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color as CrosstermColor, Print, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetSize,
    },
};

use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::io::{self, Stdout, Write};

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
    previous_cells: Vec<GridCell>,
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
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));

        // if width < 40 || height < 12 {
        //     return Err(RenderError::TerminalTooSmall {
        //         min_width: 40,
        //         min_height: 12,
        //     });
        // }

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
            previous_cells: Vec::new(),
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
                let term_width = area.width as usize;
                let term_height = area.height as usize;

                // Convert GridBuffer to Ratatui Lines with color support
                // Ensure we fill the entire terminal to prevent artifacts
                let lines: Vec<Line> = (0..term_height)
                    .map(|y| {
                        let mut spans = Vec::new();
                        let mut current_color: Option<ratatui::style::Color> = None;
                        let mut current_text = String::new();

                        for x in 0..term_width {
                            // Get cell from grid, or use space if out of bounds
                            let (ch, cell_color) = if y < grid.height() && x < grid.width() {
                                let cell = grid.get_cell(x, y);
                                (
                                    cell.character,
                                    cell.foreground_color.map(|c| c.to_ratatui_color()),
                                )
                            } else {
                                (' ', None)
                            };

                            // If color changed, flush current span and start new one
                            if cell_color != current_color {
                                if !current_text.is_empty() {
                                    let span = if let Some(color) = current_color {
                                        Span::styled(
                                            current_text.clone(),
                                            Style::default().fg(color),
                                        )
                                    } else {
                                        Span::raw(current_text.clone())
                                    };
                                    spans.push(span);
                                    current_text.clear();
                                }
                                current_color = cell_color;
                            }

                            current_text.push(ch);
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

    /// Render a grid buffer using a lower-overhead direct terminal path.
    ///
    /// This avoids rebuilding Ratatui widgets every frame and is intended for
    /// high-frequency video/webcam rendering where raw throughput matters more
    /// than layout composition.
    pub fn render_fast(&mut self, grid: &mut GridBuffer) -> Result<(), RenderError> {
        let area = self.terminal.size().unwrap_or(Rect::new(0, 0, 80, 24));
        let size = (area.width, area.height);
        let term_width = area.width as usize;
        let term_height = area.height as usize;
        let display_cell_count = term_width * term_height;
        let full_redraw = grid.needs_full_redraw()
            || size != self.last_size
            || self.previous_cells.len() != display_cell_count;

        if self.previous_cells.len() != display_cell_count {
            self.previous_cells = vec![GridCell::empty(); display_cell_count];
        }

        let backend = self.terminal.backend_mut();
        if full_redraw {
            queue!(backend, MoveTo(0, 0), Clear(ClearType::All))
                .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
        }

        for y in 0..term_height {
            if !full_redraw && !row_is_dirty(grid, y, term_width) {
                continue;
            }

            if full_redraw {
                queue!(backend, MoveTo(0, y as u16))
                    .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
                write_grid_row(backend, grid, y, term_width)
                    .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
            } else {
                let previous_row = &self.previous_cells[y * term_width..(y + 1) * term_width];
                write_changed_row_segments(backend, grid, y, term_width, previous_row)
                    .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
            }

            copy_display_row_into(&mut self.previous_cells, grid, y, term_width);
        }

        queue!(backend, ResetColor).map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
        backend
            .flush()
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;

        self.last_size = size;
        grid.mark_clean();
        Ok(())
    }

    /// Render a BrailleGrid to the terminal
    pub fn render_braille(&mut self, grid: &BrailleGrid) -> Result<(), RenderError> {
        self.terminal
            .draw(|frame| {
                let area = frame.size();
                let term_width = area.width as usize;
                let term_height = area.height as usize;

                // Ensure we fill the entire terminal to prevent artifacts
                let lines: Vec<Line> = (0..term_height)
                    .map(|y| {
                        let mut spans = Vec::new();
                        let mut current_color: Option<ratatui::style::Color> = None;
                        let mut current_text = String::new();

                        for x in 0..term_width {
                            // Get cell from grid, or use space if out of bounds
                            let (ch, color) = if y < grid.height() && x < grid.width() {
                                (
                                    grid.get_char(x, y),
                                    grid.get_color(x, y).map(|c| c.to_ratatui_color()),
                                )
                            } else {
                                (' ', None)
                            };

                            if color != current_color {
                                if !current_text.is_empty() {
                                    let span = if let Some(color) = current_color {
                                        Span::styled(
                                            current_text.clone(),
                                            Style::default().fg(color),
                                        )
                                    } else {
                                        Span::raw(current_text.clone())
                                    };
                                    spans.push(span);
                                    current_text.clear();
                                }
                                current_color = color;
                            }
                            current_text.push(ch);
                        }

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

                let paragraph = Paragraph::new(lines);
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

    /// Request a specific terminal size in character cells (best-effort)
    /// Note: The terminal/OS may clamp to supported limits.
    pub fn set_size(&mut self, width: u16, height: u16) -> Result<(), RenderError> {
        let mut stdout = io::stdout();
        execute!(stdout, SetSize(width, height))
            .map_err(|e| RenderError::RenderingFailed(e.to_string()))?;
        // Update cached size to current actual size
        self.last_size = self.dimensions();
        Ok(())
    }

    /// Try to maximize canvas by requesting large sizes (best-effort growth)
    /// Returns when a larger size is achieved or after attempts are exhausted.
    pub fn maximize_canvas(&mut self) -> Result<(), RenderError> {
        let before = self.dimensions();
        // Try progressively smaller large sizes; terminal will clamp as needed
        let candidates: &[(u16, u16)] = &[
            (400, 200),
            (360, 180),
            (320, 160),
            (300, 150),
            (280, 140),
            (260, 130),
            (240, 120),
            (220, 110),
            (200, 100),
        ];
        for &(w, h) in candidates {
            let _ = self.set_size(w, h);
            let after = self.dimensions();
            if after.0 > before.0 || after.1 > before.1 {
                return Ok(());
            }
        }
        Ok(())
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

fn row_is_dirty(grid: &GridBuffer, y: usize, term_width: usize) -> bool {
    if y >= grid.height() {
        return false;
    }
    if grid.is_row_dirty(y) {
        return true;
    }

    (0..grid.width().min(term_width)).any(|x| grid.is_dirty(x, y))
}

fn write_grid_row<W: Write>(
    out: &mut W,
    grid: &GridBuffer,
    y: usize,
    term_width: usize,
) -> io::Result<()> {
    let mut current_color: Option<Color> = None;
    let mut current_text = String::with_capacity(term_width);

    for x in 0..term_width {
        let cell = display_cell(grid, x, y);

        if cell.foreground_color != current_color {
            flush_text_run(out, &mut current_text, current_color)?;
            current_color = cell.foreground_color;
        }

        current_text.push(cell.character);
    }

    flush_text_run(out, &mut current_text, current_color)?;
    if current_color.is_some() {
        queue!(out, ResetColor)?;
    }

    Ok(())
}

fn write_grid_row_segment<W: Write>(
    out: &mut W,
    grid: &GridBuffer,
    y: usize,
    start_x: usize,
    end_x: usize,
) -> io::Result<()> {
    let mut current_color: Option<Color> = None;
    let mut current_text = String::with_capacity(end_x.saturating_sub(start_x));

    for x in start_x..end_x {
        let cell = display_cell(grid, x, y);
        if cell.foreground_color != current_color {
            flush_text_run(out, &mut current_text, current_color)?;
            current_color = cell.foreground_color;
        }
        current_text.push(cell.character);
    }

    flush_text_run(out, &mut current_text, current_color)?;
    if current_color.is_some() {
        queue!(out, ResetColor)?;
    }

    Ok(())
}

fn write_changed_row_segments<W: Write>(
    out: &mut W,
    grid: &GridBuffer,
    y: usize,
    term_width: usize,
    previous_row: &[GridCell],
) -> io::Result<()> {
    let mut x = 0;
    while x < term_width {
        let current = display_cell(grid, x, y);
        if current == previous_row[x] {
            x += 1;
            continue;
        }

        let start = x;
        x += 1;
        while x < term_width && display_cell(grid, x, y) != previous_row[x] {
            x += 1;
        }

        queue!(out, MoveTo(start as u16, y as u16))?;
        write_grid_row_segment(out, grid, y, start, x)?;
    }

    Ok(())
}

fn copy_display_row_into(
    previous_cells: &mut [GridCell],
    grid: &GridBuffer,
    y: usize,
    term_width: usize,
) {
    let row_start = y * term_width;
    let row = &mut previous_cells[row_start..row_start + term_width];
    for (x, cell) in row.iter_mut().enumerate() {
        *cell = display_cell(grid, x, y);
    }
}

fn display_cell(grid: &GridBuffer, x: usize, y: usize) -> GridCell {
    if y < grid.height() && x < grid.width() {
        *grid.get_cell(x, y)
    } else {
        GridCell::empty()
    }
}

fn flush_text_run<W: Write>(
    out: &mut W,
    current_text: &mut String,
    color: Option<Color>,
) -> io::Result<()> {
    if current_text.is_empty() {
        if let Some(color) = color {
            queue!(out, SetForegroundColor(to_crossterm_color(color)))?;
        }
        return Ok(());
    }

    match color {
        Some(color) => queue!(
            out,
            SetForegroundColor(to_crossterm_color(color)),
            Print(std::mem::take(current_text))
        )?,
        None => queue!(out, ResetColor, Print(std::mem::take(current_text)))?,
    }

    Ok(())
}

fn to_crossterm_color(color: Color) -> CrosstermColor {
    CrosstermColor::Rgb {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_dirty_detects_changed_cells_only() {
        let mut grid = GridBuffer::new(4, 2);
        grid.mark_clean();
        assert!(!row_is_dirty(&grid, 0, 4));

        grid.set_cell(1, 0, 'X');
        assert!(row_is_dirty(&grid, 0, 4));
        assert!(!row_is_dirty(&grid, 1, 4));
    }

    #[test]
    fn write_grid_row_inserts_color_sequences_when_color_changes() {
        let mut grid = GridBuffer::new(3, 1);
        grid.set_cell(0, 0, 'A');
        grid.set_cell_with_color(1, 0, 'B', Color::new(255, 0, 0));
        grid.set_cell(2, 0, 'C');

        let mut out = Vec::new();
        write_grid_row(&mut out, &grid, 0, 3).expect("row write should succeed");

        let rendered = String::from_utf8(out).expect("row output should be utf8");
        assert!(rendered.contains("A"));
        assert!(rendered.contains("B"));
        assert!(rendered.contains("C"));
        assert!(rendered.contains("\u{1b}[38;2;255;0;0m"));
    }

    #[test]
    fn write_changed_row_segments_skips_unchanged_prefix_and_suffix() {
        let mut grid = GridBuffer::new(5, 1);
        grid.set_cell(0, 0, 'A');
        grid.set_cell(1, 0, 'B');
        grid.set_cell(2, 0, 'X');
        grid.set_cell(3, 0, 'D');
        grid.set_cell(4, 0, 'E');

        let previous_row = [
            GridCell::new('A'),
            GridCell::new('B'),
            GridCell::new('C'),
            GridCell::new('D'),
            GridCell::new('E'),
        ];
        let mut out = Vec::new();
        write_changed_row_segments(&mut out, &grid, 0, 5, &previous_row)
            .expect("changed row write should succeed");

        let rendered = String::from_utf8(out).expect("row output should be utf8");
        assert!(rendered.contains("X"));
        assert!(!rendered.contains("AB"));
        assert!(!rendered.contains("DE"));
    }

    #[test]
    fn copy_display_row_into_captures_rendered_cells() {
        let mut grid = GridBuffer::new(3, 1);
        grid.set_cell(0, 0, 'A');
        grid.set_cell_with_color(1, 0, 'B', Color::new(0, 255, 0));

        let mut previous = vec![GridCell::empty(); 3];
        copy_display_row_into(&mut previous, &grid, 0, 3);

        assert_eq!(previous[0], GridCell::new('A'));
        assert_eq!(
            previous[1],
            GridCell::with_color('B', Color::new(0, 255, 0))
        );
        assert_eq!(previous[2], GridCell::empty());
    }

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
