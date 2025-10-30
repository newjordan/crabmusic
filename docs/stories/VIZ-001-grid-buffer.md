# [VIZ-001] Grid Buffer Implementation

**Epic**: Visualization Engine
**Priority**: P0 (Blocking)
**Estimated Effort**: 0.5-1 day
**Status**: Not Started

---

## Description

Implement the GridBuffer data structure that represents a 2D grid of characters for terminal visualization. This is the canvas that all visualizers draw on before rendering to the terminal.

**Agent Instructions**: Create a GridBuffer that:
- Stores a 2D array of GridCell objects
- Provides methods to get/set cells by (x, y) coordinates
- Supports clearing the grid
- Provides dimension queries
- Is efficient for 60 FPS rendering

---

## Acceptance Criteria

- [ ] GridBuffer struct stores width, height, and cells vector
- [ ] new(width, height) creates buffer filled with empty cells
- [ ] get_cell(x, y) returns GridCell at coordinates
- [ ] set_cell(x, y, cell) updates cell at coordinates
- [ ] clear() fills entire grid with empty cells (spaces)
- [ ] width() and height() return dimensions
- [ ] Bounds checking prevents out-of-range access
- [ ] GridCell struct has character field
- [ ] GridCell::new(char) and GridCell::empty() constructors
- [ ] Unit tests validate all operations
- [ ] Performance: Grid operations complete in <1µs

---

## Technical Approach

### GridBuffer Structure

Reference: **docs/architecture.md - Visualization Engine Component**

```rust
/// Grid buffer for character-based visualization
///
/// Represents a 2D grid of characters that will be rendered to the terminal.
/// Each cell contains a character and optional styling information.
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::GridBuffer;
///
/// let mut grid = GridBuffer::new(80, 24);
/// grid.set_cell(10, 5, '█');
/// assert_eq!(grid.get_cell(10, 5).character, '█');
/// ```
pub struct GridBuffer {
    /// Grid width in characters
    width: usize,
    /// Grid height in characters
    height: usize,
    /// Flat array of cells (row-major order)
    cells: Vec<GridCell>,
}

impl GridBuffer {
    /// Create a new grid buffer with specified dimensions
    ///
    /// # Arguments
    /// * `width` - Grid width in characters
    /// * `height` - Grid height in characters
    ///
    /// # Returns
    /// A new GridBuffer instance filled with empty cells
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![GridCell::empty(); width * height];
        Self {
            width,
            height,
            cells,
        }
    }

    /// Get a cell at the specified coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    ///
    /// # Returns
    /// Reference to the GridCell at (x, y)
    ///
    /// # Panics
    /// Panics if coordinates are out of bounds
    pub fn get_cell(&self, x: usize, y: usize) -> &GridCell {
        assert!(x < self.width, "x coordinate {} out of bounds (width: {})", x, self.width);
        assert!(y < self.height, "y coordinate {} out of bounds (height: {})", y, self.height);
        &self.cells[y * self.width + x]
    }

    /// Set a cell at the specified coordinates
    ///
    /// # Arguments
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    /// * `character` - Character to set
    ///
    /// # Panics
    /// Panics if coordinates are out of bounds
    pub fn set_cell(&mut self, x: usize, y: usize, character: char) {
        assert!(x < self.width, "x coordinate {} out of bounds (width: {})", x, self.width);
        assert!(y < self.height, "y coordinate {} out of bounds (height: {})", y, self.height);
        self.cells[y * self.width + x] = GridCell::new(character);
    }

    /// Clear the grid buffer (fill with spaces)
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = GridCell::empty();
        }
    }

    /// Get the width of the grid
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the grid
    pub fn height(&self) -> usize {
        self.height
    }
}
```

### GridCell Structure

```rust
/// A single cell in the grid buffer
///
/// Contains a character and optional styling information (color, attributes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCell {
    /// The character to display
    pub character: char,
    // Future: Add styling fields
    // - foreground_color: Option<Color>
    // - background_color: Option<Color>
    // - attributes: Attributes (bold, italic, etc.)
}

impl GridCell {
    /// Create a new grid cell with a character
    pub fn new(character: char) -> Self {
        Self { character }
    }

    /// Create an empty grid cell (space character)
    pub fn empty() -> Self {
        Self { character: ' ' }
    }
}

impl Default for GridCell {
    fn default() -> Self {
        Self::empty()
    }
}
```

---

## Dependencies

- **Depends on**: None (foundational component)
- **Blocks**: VIZ-003, VIZ-004, VIZ-005 (all need GridBuffer)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Visualization Engine Component"
- **Source Tree**: docs/architecture/source-tree.md - visualization module

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_buffer_creation() {
        let grid = GridBuffer::new(80, 24);
        assert_eq!(grid.width(), 80);
        assert_eq!(grid.height(), 24);

        // All cells should be empty
        for y in 0..24 {
            for x in 0..80 {
                assert_eq!(grid.get_cell(x, y).character, ' ');
            }
        }
    }

    #[test]
    fn test_set_and_get_cell() {
        let mut grid = GridBuffer::new(10, 10);

        grid.set_cell(5, 5, '█');
        assert_eq!(grid.get_cell(5, 5).character, '█');

        grid.set_cell(0, 0, 'A');
        assert_eq!(grid.get_cell(0, 0).character, 'A');

        grid.set_cell(9, 9, 'Z');
        assert_eq!(grid.get_cell(9, 9).character, 'Z');
    }

    #[test]
    fn test_clear() {
        let mut grid = GridBuffer::new(10, 10);

        // Fill with characters
        for y in 0..10 {
            for x in 0..10 {
                grid.set_cell(x, y, '█');
            }
        }

        // Clear
        grid.clear();

        // All should be empty
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(grid.get_cell(x, y).character, ' ');
            }
        }
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_get_cell_out_of_bounds_x() {
        let grid = GridBuffer::new(10, 10);
        grid.get_cell(10, 5); // x = 10 is out of bounds
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_get_cell_out_of_bounds_y() {
        let grid = GridBuffer::new(10, 10);
        grid.get_cell(5, 10); // y = 10 is out of bounds
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_set_cell_out_of_bounds() {
        let mut grid = GridBuffer::new(10, 10);
        grid.set_cell(10, 10, '█'); // Both out of bounds
    }

    #[test]
    fn test_grid_cell_creation() {
        let cell = GridCell::new('█');
        assert_eq!(cell.character, '█');

        let empty = GridCell::empty();
        assert_eq!(empty.character, ' ');

        let default = GridCell::default();
        assert_eq!(default.character, ' ');
    }

    #[test]
    fn test_grid_cell_equality() {
        let cell1 = GridCell::new('A');
        let cell2 = GridCell::new('A');
        let cell3 = GridCell::new('B');

        assert_eq!(cell1, cell2);
        assert_ne!(cell1, cell3);
    }
}
```

---

## Notes for AI Agent

**Memory Layout**:
- Use flat Vec<GridCell> with row-major order: `index = y * width + x`
- This is cache-friendly for row-by-row rendering
- Avoids Vec<Vec<>> which has poor cache locality

**Performance Considerations**:
- Pre-allocate cells vector in new()
- Use simple indexing (no HashMap or complex structures)
- clear() should be fast (simple loop over cells)
- Target: All operations <1µs for 80x24 grid

**Future Extensions** (not in this story):
- Color support (foreground/background)
- Text attributes (bold, italic, underline)
- Differential rendering (track changed cells)

**Success Indicator**: Unit tests pass, grid operations are fast and simple

