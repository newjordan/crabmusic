// Character sets for ASCII visualization
// Provides different character sets for various visual densities and styles

/// Character set type for visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSetType {
    /// Basic ASCII characters (low density)
    Basic,
    /// Extended ASCII with more density levels
    Extended,
    /// Block characters for solid fills
    Blocks,
    /// Shading characters for smooth gradients
    Shading,
    /// Dots and stippling
    Dots,
    /// Lines and curves
    Lines,
    /// Braille patterns (high resolution)
    Braille,
    /// Smooth gradient with 64 density levels
    Smooth64,
    /// Ultra-smooth gradient with 128 density levels
    Smooth128,
    /// Maximum smoothness with 256 Braille patterns
    Smooth256,
}

/// Character set for ASCII visualization
///
/// Provides ordered characters from least to most dense for representing
/// different intensity levels in visualizations.
#[derive(Debug, Clone)]
pub struct CharacterSet {
    /// Name of the character set
    pub name: String,
    /// Ordered characters from least to most dense
    pub characters: Vec<char>,
    /// Character set type
    pub set_type: CharacterSetType,
}

impl CharacterSet {
    /// Create a new character set
    ///
    /// # Arguments
    /// * `name` - Name of the character set
    /// * `characters` - Ordered characters from least to most dense
    /// * `set_type` - Type of character set
    pub fn new(name: String, characters: Vec<char>, set_type: CharacterSetType) -> Self {
        Self {
            name,
            characters,
            set_type,
        }
    }

    /// Get character for a given intensity (0.0 to 1.0)
    ///
    /// # Arguments
    /// * `intensity` - Intensity value from 0.0 (empty) to 1.0 (full)
    ///
    /// # Returns
    /// Character representing the intensity
    pub fn get_char(&self, intensity: f32) -> char {
        if self.characters.is_empty() {
            return ' ';
        }

        let clamped = intensity.clamp(0.0, 1.0);
        let index = (clamped * (self.characters.len() - 1) as f32).round() as usize;
        self.characters[index.min(self.characters.len() - 1)]
    }

    /// Get the number of density levels
    pub fn len(&self) -> usize {
        self.characters.len()
    }

    /// Check if the character set is empty
    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }
}

/// Get a predefined character set by type
///
/// # Arguments
/// * `set_type` - Type of character set to retrieve
///
/// # Returns
/// Character set instance
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::character_sets::{get_character_set, CharacterSetType};
///
/// let charset = get_character_set(CharacterSetType::Basic);
/// let ch = charset.get_char(0.5);
/// ```
pub fn get_character_set(set_type: CharacterSetType) -> CharacterSet {
    match set_type {
        CharacterSetType::Basic => basic_set(),
        CharacterSetType::Extended => extended_set(),
        CharacterSetType::Blocks => blocks_set(),
        CharacterSetType::Shading => shading_set(),
        CharacterSetType::Dots => dots_set(),
        CharacterSetType::Lines => lines_set(),
        CharacterSetType::Braille => braille_set(),
        CharacterSetType::Smooth64 => smooth64_set(),
        CharacterSetType::Smooth128 => smooth128_set(),
        CharacterSetType::Smooth256 => smooth256_set(),
    }
}

/// Basic ASCII character set (10 levels)
fn basic_set() -> CharacterSet {
    CharacterSet::new(
        "Basic".to_string(),
        vec![' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'],
        CharacterSetType::Basic,
    )
}

/// Extended ASCII character set (15 levels)
fn extended_set() -> CharacterSet {
    CharacterSet::new(
        "Extended".to_string(),
        vec![
            ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+',
            '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n',
            'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q',
            'p', 'd', 'b', 'k', 'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@',
        ],
        CharacterSetType::Extended,
    )
}

/// Block characters for solid fills (5 levels)
fn blocks_set() -> CharacterSet {
    CharacterSet::new(
        "Blocks".to_string(),
        vec![' ', '░', '▒', '▓', '█'],
        CharacterSetType::Blocks,
    )
}

/// Shading characters for smooth gradients (8 levels)
fn shading_set() -> CharacterSet {
    CharacterSet::new(
        "Shading".to_string(),
        vec![' ', '░', '▒', '▓', '█', '▀', '▄', '▌', '▐'],
        CharacterSetType::Shading,
    )
}

/// Dots and stippling characters (7 levels)
fn dots_set() -> CharacterSet {
    CharacterSet::new(
        "Dots".to_string(),
        vec![' ', '.', '·', '•', '●', '◉', '⬤'],
        CharacterSetType::Dots,
    )
}

/// Lines and curves characters (10 levels)
fn lines_set() -> CharacterSet {
    CharacterSet::new(
        "Lines".to_string(),
        vec![' ', '─', '│', '┌', '┐', '└', '┘', '├', '┤', '┼', '═', '║'],
        CharacterSetType::Lines,
    )
}

/// Braille patterns for high-resolution visualization (8 levels)
///
/// Braille characters provide 2x4 pixel resolution per character cell
fn braille_set() -> CharacterSet {
    CharacterSet::new(
        "Braille".to_string(),
        vec![' ', '⠁', '⠃', '⠇', '⠏', '⠟', '⠿', '⡿', '⣿'],
        CharacterSetType::Braille,
    )
}

/// Smooth 64-level gradient (64 levels)
///
/// Combines multiple character types ordered by perceptual density
/// for smooth gradients without visible banding.
fn smooth64_set() -> CharacterSet {
    // Mix of dots, blocks, and Braille patterns ordered by density
    let chars = vec![
        ' ', '⠀', '⡀', '⢀', '⣀', '⠄', '⡄', '⢄', '⣄', '⠂', '⡂', '⢂', '⣂', '⠆', '⡆', '⢆',
        '⣆', '⠁', '⡁', '⢁', '⣁', '⠅', '⡅', '⢅', '⣅', '⠃', '⡃', '⢃', '⣃', '⠇', '⡇', '⢇',
        '⣇', '░', '⠈', '⡈', '⢈', '⣈', '⠌', '⡌', '⢌', '⣌', '⠊', '⡊', '⢊', '⣊', '⠎', '⡎',
        '⢎', '⣎', '⠉', '⡉', '⢉', '⣉', '⠍', '⡍', '⢍', '⣍', '⠋', '⡋', '⢋', '⣋', '▒', '▓',
    ];

    CharacterSet::new(
        "Smooth 64".to_string(),
        chars,
        CharacterSetType::Smooth64,
    )
}

/// Ultra-smooth 128-level gradient (128 levels)
///
/// Extended Braille patterns with fine density control.
/// Provides nearly imperceptible steps between gradient levels.
fn smooth128_set() -> CharacterSet {
    let mut chars = Vec::with_capacity(128);

    // Generate 128 Braille patterns ordered by dot count (0-8 dots)
    // This creates a smooth progression from empty to full
    for dot_count in 0..=7 {
        // For each dot count, generate patterns
        let patterns_for_count = generate_braille_patterns_with_dots(dot_count);
        for pattern in patterns_for_count.iter().take(16) {
            if chars.len() < 128 {
                chars.push(*pattern);
            }
        }
    }

    // Fill remaining with densest patterns
    while chars.len() < 128 {
        chars.push('⣿');
    }

    CharacterSet::new(
        "Smooth 128".to_string(),
        chars,
        CharacterSetType::Smooth128,
    )
}

/// Maximum smoothness 256-level gradient (256 levels)
///
/// All 256 Braille Unicode patterns (U+2800 to U+28FF) ordered by
/// dot count for the smoothest possible gradient.
fn smooth256_set() -> CharacterSet {
    let mut chars = Vec::with_capacity(256);

    // Generate all 256 Braille patterns ordered by dot count
    for dots in 0..=255 {
        let ch = std::char::from_u32(0x2800 + dots).unwrap_or('⠀');
        chars.push(ch);
    }

    // Sort by dot count for proper density ordering
    chars.sort_by_key(|&c| count_braille_dots(c));

    CharacterSet::new(
        "Smooth 256".to_string(),
        chars,
        CharacterSetType::Smooth256,
    )
}

/// Helper: Generate Braille patterns with exactly N dots
fn generate_braille_patterns_with_dots(dot_count: u8) -> Vec<char> {
    let mut patterns = Vec::new();

    for pattern in 0..=255u8 {
        if pattern.count_ones() == dot_count as u32 {
            let ch = std::char::from_u32(0x2800 + pattern as u32).unwrap_or('⠀');
            patterns.push(ch);
        }
    }

    patterns
}

/// Helper: Count dots in a Braille character
fn count_braille_dots(ch: char) -> u32 {
    let code = ch as u32;
    if (0x2800..=0x28FF).contains(&code) {
        let pattern = (code - 0x2800) as u8;
        pattern.count_ones()
    } else {
        0
    }
}

/// Get all available character sets
///
/// # Returns
/// Vector of all predefined character sets
pub fn get_all_character_sets() -> Vec<CharacterSet> {
    vec![
        basic_set(),
        extended_set(),
        blocks_set(),
        shading_set(),
        dots_set(),
        lines_set(),
        braille_set(),
        smooth64_set(),
        smooth128_set(),
        smooth256_set(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_set() {
        let charset = basic_set();
        assert_eq!(charset.len(), 10);
        assert_eq!(charset.get_char(0.0), ' ');
        assert_eq!(charset.get_char(1.0), '@');
    }

    #[test]
    fn test_extended_set() {
        let charset = extended_set();
        assert!(charset.len() > 10);
        assert_eq!(charset.get_char(0.0), ' ');
    }

    #[test]
    fn test_blocks_set() {
        let charset = blocks_set();
        assert_eq!(charset.len(), 5);
        assert_eq!(charset.get_char(0.0), ' ');
        assert_eq!(charset.get_char(1.0), '█');
    }

    #[test]
    fn test_shading_set() {
        let charset = shading_set();
        assert!(!charset.is_empty());
        assert_eq!(charset.get_char(0.0), ' ');
    }

    #[test]
    fn test_dots_set() {
        let charset = dots_set();
        assert_eq!(charset.len(), 7);
        assert_eq!(charset.get_char(0.0), ' ');
    }

    #[test]
    fn test_lines_set() {
        let charset = lines_set();
        assert!(!charset.is_empty());
        assert_eq!(charset.get_char(0.0), ' ');
    }

    #[test]
    fn test_braille_set() {
        let charset = braille_set();
        assert!(!charset.is_empty());
        assert_eq!(charset.get_char(0.0), ' ');
    }

    #[test]
    fn test_get_char_intensity() {
        let charset = basic_set();
        
        // Test boundary values
        assert_eq!(charset.get_char(0.0), ' ');
        assert_eq!(charset.get_char(1.0), '@');
        
        // Test middle value
        let mid_char = charset.get_char(0.5);
        assert!(mid_char != ' ' && mid_char != '@');
    }

    #[test]
    fn test_get_char_clamping() {
        let charset = basic_set();
        
        // Values outside range should be clamped
        assert_eq!(charset.get_char(-0.5), ' ');
        assert_eq!(charset.get_char(1.5), '@');
    }

    #[test]
    fn test_get_character_set() {
        let types = vec![
            CharacterSetType::Basic,
            CharacterSetType::Extended,
            CharacterSetType::Blocks,
            CharacterSetType::Shading,
            CharacterSetType::Dots,
            CharacterSetType::Lines,
            CharacterSetType::Braille,
        ];

        for set_type in types {
            let charset = get_character_set(set_type);
            assert!(!charset.is_empty());
            assert_eq!(charset.set_type, set_type);
        }
    }

    #[test]
    fn test_get_all_character_sets() {
        let sets = get_all_character_sets();
        assert_eq!(sets.len(), 10); // 7 original + 3 smooth sets (smooth64, smooth128, smooth256)

        for charset in sets {
            assert!(!charset.is_empty());
        }
    }

    #[test]
    fn test_character_set_monotonic() {
        // Test that intensity increases produce different characters
        let charset = basic_set();
        let mut prev_char = charset.get_char(0.0);
        let mut changes = 0;
        
        for i in 1..=10 {
            let intensity = i as f32 / 10.0;
            let ch = charset.get_char(intensity);
            if ch != prev_char {
                changes += 1;
            }
            prev_char = ch;
        }
        
        // Should have multiple different characters across the range
        assert!(changes >= 5);
    }
}

