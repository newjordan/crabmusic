// Braille character rendering for high-resolution terminal graphics
//
// Braille characters provide 2×4 dot patterns per terminal cell,
// giving us 4× vertical resolution for smooth curves!
//
// Dot positions in a Braille character:
//   1 4
//   2 5
//   3 6
//   7 8
//
// Unicode range: U+2800 to U+28FF (256 patterns)

use super::Color;

/// Braille dot positions
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrailleDot {
    Dot1 = 0b00000001,
    Dot2 = 0b00000010,
    Dot3 = 0b00000100,
    Dot4 = 0b00001000,
    Dot5 = 0b00010000,
    Dot6 = 0b00100000,
    Dot7 = 0b01000000,
    Dot8 = 0b10000000,
}

/// Convert dot pattern to Braille Unicode character
///
/// # Arguments
/// * `dots` - Bit pattern where each bit represents a dot (1 = filled)
///
/// # Returns
/// Unicode Braille character
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::braille::dots_to_char;
///
/// // Empty pattern
/// assert_eq!(dots_to_char(0b00000000), '⠀');
///
/// // All dots filled
/// assert_eq!(dots_to_char(0b11111111), '⣿');
///
/// // Top-left and bottom-right
/// assert_eq!(dots_to_char(0b10000001), '⡁');
/// ```
#[inline]
pub fn dots_to_char(dots: u8) -> char {
    // Braille patterns start at U+2800
    char::from_u32(0x2800 + dots as u32).unwrap_or('⠀')
}

/// High-resolution grid using Braille characters
///
/// Each terminal cell contains a 2×4 dot pattern, giving us
/// 8 dots per character position.
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::BrailleGrid;
///
/// let mut grid = BrailleGrid::new(40, 20);
/// // Each cell is 2 dots wide, 4 dots tall
/// // So we have 80×80 dot resolution!
/// ```
pub struct BrailleGrid {
    /// Width in terminal cells
    width: usize,
    /// Height in terminal cells
    height: usize,
    /// Dot patterns for each cell
    patterns: Vec<u8>,
    /// Optional colors for each cell
    colors: Vec<Option<Color>>,
}

impl BrailleGrid {
    /// Create a new Braille grid
    ///
    /// # Arguments
    /// * `width` - Width in terminal cells (each cell is 2 dots wide)
    /// * `height` - Height in terminal cells (each cell is 4 dots tall)
    ///
    /// # Returns
    /// A new BrailleGrid with all dots cleared
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            patterns: vec![0; size],
            colors: vec![None; size],
        }
    }

    /// Get width in terminal cells
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height in terminal cells
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get width in dots (2× terminal width)
    pub fn dot_width(&self) -> usize {
        self.width * 2
    }

    /// Get height in dots (4× terminal height)
    pub fn dot_height(&self) -> usize {
        self.height * 4
    }

    /// Clear all dots
    pub fn clear(&mut self) {
        self.patterns.fill(0);
        self.colors.fill(None);
    }

    /// Set a single dot at the specified position
    ///
    /// # Arguments
    /// * `dot_x` - X position in dots (0 to width*2-1)
    /// * `dot_y` - Y position in dots (0 to height*4-1)
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::BrailleGrid;
    ///
    /// let mut grid = BrailleGrid::new(40, 20);
    /// grid.set_dot(0, 0);  // Top-left dot
    /// grid.set_dot(1, 0);  // Top-right dot of first cell
    /// ```
    pub fn set_dot(&mut self, dot_x: usize, dot_y: usize) {
        if dot_x >= self.dot_width() || dot_y >= self.dot_height() {
            return;
        }

        // Convert dot coordinates to cell coordinates
        let cell_x = dot_x / 2;
        let cell_y = dot_y / 4;
        let cell_index = cell_y * self.width + cell_x;

        // Determine which dot within the cell (0-7)
        let local_x = dot_x % 2;
        let local_y = dot_y % 4;

        // Map to Braille dot position
        let dot_bit = match (local_x, local_y) {
            (0, 0) => BrailleDot::Dot1 as u8,
            (0, 1) => BrailleDot::Dot2 as u8,
            (0, 2) => BrailleDot::Dot3 as u8,
            (0, 3) => BrailleDot::Dot7 as u8,
            (1, 0) => BrailleDot::Dot4 as u8,
            (1, 1) => BrailleDot::Dot5 as u8,
            (1, 2) => BrailleDot::Dot6 as u8,
            (1, 3) => BrailleDot::Dot8 as u8,
            _ => unreachable!(),
        };

        self.patterns[cell_index] |= dot_bit;
    }

    /// Set a dot with color
    pub fn set_dot_with_color(&mut self, dot_x: usize, dot_y: usize, color: Color) {
        self.set_dot(dot_x, dot_y);

        if dot_x >= self.dot_width() || dot_y >= self.dot_height() {
            return;
        }

        let cell_x = dot_x / 2;
        let cell_y = dot_y / 4;
        let cell_index = cell_y * self.width + cell_x;

        self.colors[cell_index] = Some(color);
    }

    /// Draw a line between two points using Bresenham's algorithm
    ///
    /// # Arguments
    /// * `x0, y0` - Start point in dots
    /// * `x1, y1` - End point in dots
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::BrailleGrid;
    ///
    /// let mut grid = BrailleGrid::new(40, 20);
    /// // Draw diagonal line
    /// grid.draw_line(0, 0, 79, 79);
    /// ```
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = (y1 as i32 - y0 as i32).abs();
        let sx = if x0 < x1 { 1i32 } else { -1i32 };
        let sy = if y0 < y1 { 1i32 } else { -1i32 };
        let mut err = dx - dy;

        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            self.set_dot(x as usize, y as usize);

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a line with color
    pub fn draw_line_with_color(
        &mut self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: Color,
    ) {
        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = (y1 as i32 - y0 as i32).abs();
        let sx = if x0 < x1 { 1i32 } else { -1i32 };
        let sy = if y0 < y1 { 1i32 } else { -1i32 };
        let mut err = dx - dy;

        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            self.set_dot_with_color(x as usize, y as usize, color);

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Get the Braille character at a cell position
    ///
    /// # Arguments
    /// * `cell_x` - X position in cells
    /// * `cell_y` - Y position in cells
    ///
    /// # Returns
    /// Braille character representing the dot pattern
    pub fn get_char(&self, cell_x: usize, cell_y: usize) -> char {
        if cell_x >= self.width || cell_y >= self.height {
            return '⠀';
        }

        let index = cell_y * self.width + cell_x;
        dots_to_char(self.patterns[index])
    }

    /// Get the color at a cell position
    pub fn get_color(&self, cell_x: usize, cell_y: usize) -> Option<Color> {
        if cell_x >= self.width || cell_y >= self.height {
            return None;
        }

        let index = cell_y * self.width + cell_x;
        self.colors[index]
    }

    /// Check if a cell has any dots set
    pub fn is_empty(&self, cell_x: usize, cell_y: usize) -> bool {
        if cell_x >= self.width || cell_y >= self.height {
            return true;
        }

        let index = cell_y * self.width + cell_x;
        self.patterns[index] == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dots_to_char() {
        // Empty pattern
        assert_eq!(dots_to_char(0b00000000), '⠀');

        // All dots
        assert_eq!(dots_to_char(0b11111111), '⣿');

        // Single dots
        assert_eq!(dots_to_char(0b00000001), '⠁'); // Dot 1
        assert_eq!(dots_to_char(0b00001000), '⠈'); // Dot 4
    }

    #[test]
    fn test_braille_grid_creation() {
        let grid = BrailleGrid::new(40, 20);
        assert_eq!(grid.width(), 40);
        assert_eq!(grid.height(), 20);
        assert_eq!(grid.dot_width(), 80);
        assert_eq!(grid.dot_height(), 80);
    }

    #[test]
    fn test_set_dot() {
        let mut grid = BrailleGrid::new(10, 10);

        // Set top-left dot of first cell
        grid.set_dot(0, 0);
        assert_eq!(grid.get_char(0, 0), '⠁');

        // Set top-right dot of first cell
        grid.set_dot(1, 0);
        assert_eq!(grid.get_char(0, 0), '⠉');
    }

    #[test]
    fn test_clear() {
        let mut grid = BrailleGrid::new(10, 10);

        grid.set_dot(0, 0);
        grid.set_dot(5, 5);

        grid.clear();

        assert_eq!(grid.get_char(0, 0), '⠀');
        assert!(grid.is_empty(0, 0));
    }

    #[test]
    fn test_draw_line_horizontal() {
        let mut grid = BrailleGrid::new(10, 10);

        // Draw horizontal line
        grid.draw_line(0, 0, 5, 0);

        // Should have dots along the line
        for x in 0..=2 {
            assert!(!grid.is_empty(x, 0));
        }
    }

    #[test]
    fn test_draw_line_vertical() {
        let mut grid = BrailleGrid::new(10, 10);

        // Draw vertical line
        grid.draw_line(0, 0, 0, 7);

        // Should have dots along the line
        assert!(!grid.is_empty(0, 0));
        assert!(!grid.is_empty(0, 1));
    }

    #[test]
    fn test_draw_line_diagonal() {
        let mut grid = BrailleGrid::new(10, 10);

        // Draw diagonal line
        grid.draw_line(0, 0, 10, 10);

        // Should have dots along diagonal
        for i in 0..5 {
            assert!(!grid.is_empty(i, i));
        }
    }

    #[test]
    fn test_set_dot_with_color() {
        let mut grid = BrailleGrid::new(10, 10);
        let color = Color::new(255, 0, 0);

        grid.set_dot_with_color(0, 0, color);

        assert_eq!(grid.get_char(0, 0), '⠁');
        assert_eq!(grid.get_color(0, 0), Some(color));
    }

    #[test]
    fn test_dot_positions() {
        let mut grid = BrailleGrid::new(10, 10);

        // Test all 8 dot positions in first cell
        grid.clear();
        grid.set_dot(0, 0); // Dot 1
        assert_eq!(grid.patterns[0], 0b00000001);

        grid.clear();
        grid.set_dot(0, 1); // Dot 2
        assert_eq!(grid.patterns[0], 0b00000010);

        grid.clear();
        grid.set_dot(0, 2); // Dot 3
        assert_eq!(grid.patterns[0], 0b00000100);

        grid.clear();
        grid.set_dot(0, 3); // Dot 7
        assert_eq!(grid.patterns[0], 0b01000000);

        grid.clear();
        grid.set_dot(1, 0); // Dot 4
        assert_eq!(grid.patterns[0], 0b00001000);

        grid.clear();
        grid.set_dot(1, 1); // Dot 5
        assert_eq!(grid.patterns[0], 0b00010000);

        grid.clear();
        grid.set_dot(1, 2); // Dot 6
        assert_eq!(grid.patterns[0], 0b00100000);

        grid.clear();
        grid.set_dot(1, 3); // Dot 8
        assert_eq!(grid.patterns[0], 0b10000000);
    }

    #[test]
    fn test_bounds_checking() {
        let mut grid = BrailleGrid::new(10, 10);

        // Should not panic, just ignore
        grid.set_dot(1000, 1000);

        assert_eq!(grid.get_char(1000, 1000), '⠀');
        assert!(grid.is_empty(1000, 1000));
    }
}
