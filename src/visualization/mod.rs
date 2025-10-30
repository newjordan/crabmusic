// Visualization module
// Transforms audio parameters into visual grid representations

#![allow(dead_code)]

pub mod character_sets;
mod sine_wave;

pub use character_sets::{CharacterSet, CharacterSetType, get_character_set, get_all_character_sets};
pub use sine_wave::{SineWaveConfig, SineWaveVisualizer};

use crate::dsp::AudioParameters;

/// Trait for audio visualizers
///
/// Implementations generate visual representations from audio parameters.
/// Each visualizer produces a GridBuffer that can be rendered to the terminal.
///
/// # Design Philosophy
///
/// The trait is intentionally minimal to support diverse visualization approaches:
/// - `update()` allows visualizers to maintain internal state (phase, smoothing, etc.)
/// - `render()` is separate from update to support frame-independent rendering
/// - `name()` enables runtime identification and configuration
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::{Visualizer, GridBuffer};
/// use crabmusic::dsp::AudioParameters;
///
/// struct SimpleVisualizer;
///
/// impl Visualizer for SimpleVisualizer {
///     fn update(&mut self, params: &AudioParameters) {
///         // Update internal state based on audio
///     }
///
///     fn render(&self, grid: &mut GridBuffer) {
///         // Render visualization to grid
///         grid.set_cell(0, 0, '█');
///     }
///
///     fn name(&self) -> &str {
///         "Simple"
///     }
/// }
/// ```
pub trait Visualizer {
    /// Update visualizer state from audio parameters
    ///
    /// Called once per audio frame to update internal state (e.g., smoothed values,
    /// animation phase, beat detection state). This method should be fast (<1ms)
    /// to maintain real-time performance.
    ///
    /// # Arguments
    /// * `params` - Audio parameters extracted from DSP processing
    ///
    /// # Implementation Notes
    /// - Apply smoothing to prevent jitter
    /// - Update animation state (phase, position, etc.)
    /// - Store parameters for use in render()
    /// - Keep computation minimal - heavy work goes in render()
    fn update(&mut self, params: &AudioParameters);

    /// Render visualization to grid buffer
    ///
    /// Called once per frame to generate the visual representation. This method
    /// should fill the grid buffer with characters based on the current state.
    ///
    /// # Arguments
    /// * `grid` - Grid buffer to render into
    ///
    /// # Implementation Notes
    /// - Clear grid first if needed (or render over existing content)
    /// - Use grid.width() and grid.height() for dimensions
    /// - Use select_character_for_coverage() for smooth visuals
    /// - Target: Complete in <16ms for 60 FPS (preferably <5ms)
    fn render(&self, grid: &mut GridBuffer);

    /// Get the name of this visualizer
    ///
    /// Used for identification in logs, configuration, and UI.
    ///
    /// # Returns
    /// Human-readable name of the visualizer
    ///
    /// # Examples
    /// - "Sine Wave"
    /// - "Spectrum Analyzer"
    /// - "Oscilloscope"
    fn name(&self) -> &str;
}

/// Select a character based on coverage percentage
///
/// Maps coverage values to block element characters for smooth visual transitions.
///
/// # Arguments
/// * `coverage` - Coverage percentage (0.0-1.0)
///
/// # Returns
/// Character representing the coverage level
///
/// # Coverage Thresholds
/// - 0.00-0.25: ' ' (empty)
/// - 0.25-0.50: '░' (light shade)
/// - 0.50-0.75: '▒' (medium shade)
/// - 0.75-1.00: '▓' (dark shade)
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::select_character_for_coverage;
///
/// assert_eq!(select_character_for_coverage(0.0), ' ');
/// assert_eq!(select_character_for_coverage(0.3), '░');
/// assert_eq!(select_character_for_coverage(0.6), '▒');
/// assert_eq!(select_character_for_coverage(0.9), '▓');
/// ```
#[inline]
pub fn select_character_for_coverage(coverage: f32) -> char {
    match coverage {
        c if c < 0.25 => ' ',
        c if c < 0.50 => '░',
        c if c < 0.75 => '▒',
        _ => '▓',
    }
}

/// Linear interpolation between two values
///
/// Smoothly transitions from current value toward target value.
///
/// # Arguments
/// * `current` - Current value
/// * `target` - Target value
/// * `factor` - Interpolation factor (0.0-1.0)
///   - 0.0 = no change (stay at current)
///   - 1.0 = instant change (jump to target)
///   - 0.1 = slow smooth transition
///
/// # Returns
/// Interpolated value between current and target
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::lerp;
///
/// // Smooth transition
/// let current = 0.0;
/// let target = 1.0;
/// let result = lerp(current, target, 0.1);
/// assert_eq!(result, 0.1); // Moved 10% toward target
///
/// // Instant transition
/// let result = lerp(current, target, 1.0);
/// assert_eq!(result, 1.0); // Jumped to target
/// ```
#[inline]
pub fn lerp(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}

/// Grid buffer for character-based visualization
///
/// Represents a 2D grid of characters that will be rendered to the terminal.
/// Each cell contains a character and optional styling information.
///
/// Supports differential rendering by tracking which cells have changed since
/// the last render, allowing for optimized updates.
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
    /// Dirty flags for changed cells (for differential rendering)
    dirty: Vec<bool>,
    /// Whether the entire grid needs to be redrawn
    full_redraw: bool,
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
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::GridBuffer;
    ///
    /// let grid = GridBuffer::new(80, 24);
    /// assert_eq!(grid.width(), 80);
    /// assert_eq!(grid.height(), 24);
    /// ```
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let cells = vec![GridCell::empty(); size];
        let dirty = vec![false; size];
        Self {
            width,
            height,
            cells,
            dirty,
            full_redraw: true, // First render needs full redraw
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
        assert!(
            x < self.width,
            "x coordinate {} out of bounds (width: {})",
            x,
            self.width
        );
        assert!(
            y < self.height,
            "y coordinate {} out of bounds (height: {})",
            y,
            self.height
        );
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
        assert!(
            x < self.width,
            "x coordinate {} out of bounds (width: {})",
            x,
            self.width
        );
        assert!(
            y < self.height,
            "y coordinate {} out of bounds (height: {})",
            y,
            self.height
        );
        let index = y * self.width + x;
        let new_cell = GridCell::new(character);

        // Only mark as dirty if the cell actually changed
        if self.cells[index] != new_cell {
            self.cells[index] = new_cell;
            self.dirty[index] = true;
        }
    }

    /// Clear the grid buffer (fill with spaces)
    pub fn clear(&mut self) {
        let empty = GridCell::empty();
        for (i, cell) in self.cells.iter_mut().enumerate() {
            if *cell != empty {
                *cell = empty;
                self.dirty[i] = true;
            }
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

    /// Check if a cell is dirty (changed since last render)
    ///
    /// # Arguments
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    ///
    /// # Returns
    /// True if the cell has changed since the last render
    pub fn is_dirty(&self, x: usize, y: usize) -> bool {
        if self.full_redraw {
            return true;
        }
        if x >= self.width || y >= self.height {
            return false;
        }
        self.dirty[y * self.width + x]
    }

    /// Mark all cells as clean (called after rendering)
    pub fn mark_clean(&mut self) {
        self.dirty.fill(false);
        self.full_redraw = false;
    }

    /// Force a full redraw on the next render
    pub fn mark_full_redraw(&mut self) {
        self.full_redraw = true;
    }

    /// Check if a full redraw is needed
    pub fn needs_full_redraw(&self) -> bool {
        self.full_redraw
    }

    /// Get the number of dirty cells
    ///
    /// # Returns
    /// Count of cells that have changed since last render
    pub fn dirty_count(&self) -> usize {
        if self.full_redraw {
            return self.cells.len();
        }
        self.dirty.iter().filter(|&&d| d).count()
    }
}

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
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::GridCell;
    ///
    /// let cell = GridCell::new('█');
    /// assert_eq!(cell.character, '█');
    /// ```
    pub fn new(character: char) -> Self {
        Self { character }
    }

    /// Create an empty grid cell (space character)
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::GridCell;
    ///
    /// let cell = GridCell::empty();
    /// assert_eq!(cell.character, ' ');
    /// ```
    pub fn empty() -> Self {
        Self { character: ' ' }
    }
}

impl Default for GridCell {
    fn default() -> Self {
        Self::empty()
    }
}

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

    // Coverage algorithm tests
    #[test]
    fn test_character_selection_empty() {
        assert_eq!(select_character_for_coverage(0.0), ' ');
        assert_eq!(select_character_for_coverage(0.1), ' ');
        assert_eq!(select_character_for_coverage(0.24), ' ');
    }

    #[test]
    fn test_character_selection_light() {
        assert_eq!(select_character_for_coverage(0.25), '░');
        assert_eq!(select_character_for_coverage(0.3), '░');
        assert_eq!(select_character_for_coverage(0.49), '░');
    }

    #[test]
    fn test_character_selection_medium() {
        assert_eq!(select_character_for_coverage(0.50), '▒');
        assert_eq!(select_character_for_coverage(0.6), '▒');
        assert_eq!(select_character_for_coverage(0.74), '▒');
    }

    #[test]
    fn test_character_selection_dark() {
        assert_eq!(select_character_for_coverage(0.75), '▓');
        assert_eq!(select_character_for_coverage(0.9), '▓');
        assert_eq!(select_character_for_coverage(1.0), '▓');
    }

    #[test]
    fn test_character_selection_boundary_values() {
        // Test exact boundary values
        assert_eq!(select_character_for_coverage(0.25), '░');
        assert_eq!(select_character_for_coverage(0.50), '▒');
        assert_eq!(select_character_for_coverage(0.75), '▓');
    }

    #[test]
    fn test_lerp_no_change() {
        let result = lerp(5.0, 10.0, 0.0);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_lerp_instant_change() {
        let result = lerp(5.0, 10.0, 1.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_lerp_partial_change() {
        let result = lerp(0.0, 1.0, 0.5);
        assert_eq!(result, 0.5);

        let result = lerp(0.0, 1.0, 0.1);
        assert!((result - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_lerp_negative_values() {
        let result = lerp(-10.0, 10.0, 0.5);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_lerp_smoothing() {
        // Simulate smoothing over multiple frames
        let mut current = 0.0;
        let target = 1.0;
        let smoothing = 0.1;

        // After 10 frames, should be close to target but not there yet
        for _ in 0..10 {
            current = lerp(current, target, smoothing);
        }

        assert!(current > 0.5); // Made significant progress
        assert!(current < 1.0); // But not instant
    }

    // Visualizer trait tests
    /// Mock visualizer for testing
    struct MockVisualizer {
        update_count: usize,
        last_params: Option<AudioParameters>,
    }

    impl MockVisualizer {
        fn new() -> Self {
            Self {
                update_count: 0,
                last_params: None,
            }
        }
    }

    impl Visualizer for MockVisualizer {
        fn update(&mut self, params: &AudioParameters) {
            self.update_count += 1;
            self.last_params = Some(params.clone());
        }

        fn render(&self, grid: &mut GridBuffer) {
            // Simple test pattern: fill first cell
            if grid.width() > 0 && grid.height() > 0 {
                grid.set_cell(0, 0, '█');
            }
        }

        fn name(&self) -> &str {
            "Mock"
        }
    }

    #[test]
    fn test_visualizer_trait_update() {
        let mut viz = MockVisualizer::new();
        let params = AudioParameters {
            bass: 0.5,
            mid: 0.3,
            treble: 0.2,
            amplitude: 0.4,
            beat: false,
        };

        viz.update(&params);

        assert_eq!(viz.update_count, 1);
        assert!(viz.last_params.is_some());
        let last = viz.last_params.unwrap();
        assert_eq!(last.bass, 0.5);
        assert_eq!(last.mid, 0.3);
    }

    #[test]
    fn test_visualizer_trait_render() {
        let viz = MockVisualizer::new();
        let mut grid = GridBuffer::new(10, 10);

        viz.render(&mut grid);

        assert_eq!(grid.get_cell(0, 0).character, '█');
    }

    #[test]
    fn test_visualizer_trait_name() {
        let viz = MockVisualizer::new();
        assert_eq!(viz.name(), "Mock");
    }

    #[test]
    fn test_visualizer_trait_multiple_updates() {
        let mut viz = MockVisualizer::new();
        let params = AudioParameters::default();

        for _ in 0..10 {
            viz.update(&params);
        }

        assert_eq!(viz.update_count, 10);
    }

    // Differential rendering tests
    #[test]
    fn test_grid_buffer_initial_full_redraw() {
        let grid = GridBuffer::new(10, 10);
        assert!(grid.needs_full_redraw());
        assert_eq!(grid.dirty_count(), 100); // All cells dirty on first render
    }

    #[test]
    fn test_grid_buffer_mark_clean() {
        let mut grid = GridBuffer::new(10, 10);
        grid.mark_clean();

        assert!(!grid.needs_full_redraw());
        assert_eq!(grid.dirty_count(), 0);
    }

    #[test]
    fn test_grid_buffer_dirty_tracking() {
        let mut grid = GridBuffer::new(10, 10);
        grid.mark_clean();

        // Set a cell - should mark it dirty
        grid.set_cell(5, 5, '█');
        assert!(grid.is_dirty(5, 5));
        assert_eq!(grid.dirty_count(), 1);

        // Set another cell
        grid.set_cell(3, 3, '▓');
        assert!(grid.is_dirty(3, 3));
        assert_eq!(grid.dirty_count(), 2);

        // Clean cells should not be dirty
        assert!(!grid.is_dirty(0, 0));
    }

    #[test]
    fn test_grid_buffer_no_change_no_dirty() {
        let mut grid = GridBuffer::new(10, 10);
        grid.mark_clean();

        // Set a cell
        grid.set_cell(5, 5, '█');
        assert_eq!(grid.dirty_count(), 1);

        grid.mark_clean();
        assert_eq!(grid.dirty_count(), 0);

        // Set the same character again - should not mark as dirty
        grid.set_cell(5, 5, '█');
        assert_eq!(grid.dirty_count(), 0);
        assert!(!grid.is_dirty(5, 5));
    }

    #[test]
    fn test_grid_buffer_clear_marks_dirty() {
        let mut grid = GridBuffer::new(10, 10);

        // Fill with characters
        for y in 0..10 {
            for x in 0..10 {
                grid.set_cell(x, y, '█');
            }
        }

        grid.mark_clean();
        assert_eq!(grid.dirty_count(), 0);

        // Clear should mark all non-empty cells as dirty
        grid.clear();
        assert_eq!(grid.dirty_count(), 100);
    }

    #[test]
    fn test_grid_buffer_force_full_redraw() {
        let mut grid = GridBuffer::new(10, 10);
        grid.mark_clean();

        assert!(!grid.needs_full_redraw());

        grid.mark_full_redraw();
        assert!(grid.needs_full_redraw());
        assert_eq!(grid.dirty_count(), 100);
    }

    #[test]
    fn test_grid_buffer_is_dirty_bounds() {
        let mut grid = GridBuffer::new(10, 10);
        grid.mark_clean(); // Clear full_redraw flag

        // Out of bounds should return false
        assert!(!grid.is_dirty(10, 10));
        assert!(!grid.is_dirty(100, 100));
    }
}
