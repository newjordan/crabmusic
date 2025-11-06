// Terminal rendering module
use crate::braille::{BrailleGrid, Color};
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor::{Hide, Show, MoveTo},
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

/// Terminal renderer with panic-safe cleanup
pub struct TerminalRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    _cleanup: TerminalCleanup,
}

/// RAII guard for terminal cleanup
struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            Show
        );
    }
}

impl TerminalRenderer {
    /// Create a new terminal renderer
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            _cleanup: TerminalCleanup,
        })
    }

    /// Get terminal size (width, height)
    pub fn size(&self) -> Result<(u16, u16)> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }

    /// Clear the terminal
    pub fn clear(&mut self) -> Result<()> {
        self.terminal.clear()?;
        Ok(())
    }

    /// Render a BrailleGrid to the terminal
    pub fn render_braille(&mut self, grid: &BrailleGrid) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();

            // Build lines from braille grid
            let mut lines = Vec::new();
            for y in 0..grid.height().min(area.height as usize) {
                let mut spans = Vec::new();
                for x in 0..grid.width().min(area.width as usize) {
                    let ch = grid.get_char(x, y);
                    let color = grid.get_color(x, y);

                    let style = if let Some(c) = color {
                        Style::default().fg(ratatui::style::Color::Rgb(c.r, c.g, c.b))
                    } else {
                        Style::default()
                    };

                    spans.push(Span::styled(ch.to_string(), style));
                }
                lines.push(Line::from(spans));
            }

            let paragraph = Paragraph::new(lines);
            frame.render_widget(paragraph, area);
        })?;

        Ok(())
    }

    /// Render text lines to the terminal
    pub fn render_text(&mut self, text: &str) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            let paragraph = Paragraph::new(text);
            frame.render_widget(paragraph, area);
        })?;
        Ok(())
    }

    /// Render BrailleGrid with text below it
    pub fn render_braille_with_text(&mut self, grid: &BrailleGrid, text: &str) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();

            // Split area: top for braille, bottom for text
            let braille_height = grid.height().min((area.height as usize).saturating_sub(5)) as u16;
            let text_height = area.height.saturating_sub(braille_height);

            // Render braille in top area
            let braille_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: braille_height,
            };

            let mut braille_lines = Vec::new();
            for y in 0..grid.height().min(braille_height as usize) {
                let mut spans = Vec::new();
                for x in 0..grid.width().min(area.width as usize) {
                    let ch = grid.get_char(x, y);
                    let color = grid.get_color(x, y);

                    let style = if let Some(c) = color {
                        Style::default().fg(ratatui::style::Color::Rgb(c.r, c.g, c.b))
                    } else {
                        Style::default()
                    };

                    spans.push(Span::styled(ch.to_string(), style));
                }
                braille_lines.push(Line::from(spans));
            }

            let braille_paragraph = Paragraph::new(braille_lines);
            frame.render_widget(braille_paragraph, braille_area);

            // Render text in bottom area
            let text_area = Rect {
                x: area.x,
                y: area.y + braille_height,
                width: area.width,
                height: text_height,
            };

            let text_paragraph = Paragraph::new(text);
            frame.render_widget(text_paragraph, text_area);
        })?;

        Ok(())
    }
}

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create terminal renderer")
    }
}
