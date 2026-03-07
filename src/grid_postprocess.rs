use crate::braille_quality::{self, BrailleQualitySettings};
use crate::runtime_controls::ColorMode;
use crate::visualization::{braille::BrailleGrid, Color, GridBuffer, GridCell};

const BRAILLE_DOT_MAP: [(usize, usize); 8] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (1, 0),
    (1, 1),
    (1, 2),
    (0, 3),
    (1, 3),
];

pub struct GridBraillePostProcessor {
    dot_luma: Vec<u8>,
    toned: Vec<u8>,
    source_cells: Vec<GridCell>,
    braille: BrailleGrid,
}

impl Default for GridBraillePostProcessor {
    fn default() -> Self {
        Self {
            dot_luma: Vec::new(),
            toned: Vec::new(),
            source_cells: Vec::new(),
            braille: BrailleGrid::new(0, 0),
        }
    }
}

impl GridBraillePostProcessor {
    pub fn apply(
        &mut self,
        grid: &mut GridBuffer,
        quality: BrailleQualitySettings,
        color_mode: ColorMode,
    ) {
        if grid.width() == 0 || grid.height() == 0 {
            return;
        }

        let width = grid.width();
        let height = grid.height();
        let cell_count = width * height;
        let dot_width = width * 2;
        let dot_height = height * 4;
        let dot_count = dot_width * dot_height;

        self.dot_luma.resize(dot_count, 0);
        self.dot_luma.fill(0);
        self.toned.resize(dot_count, 0);
        self.source_cells.resize(cell_count, GridCell::empty());
        if self.braille.width() != width || self.braille.height() != height {
            self.braille = BrailleGrid::new(width, height);
        }

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let cell = *grid.get_cell(x, y);
                self.source_cells[idx] = cell;
                if should_preserve_cell(cell.character) {
                    continue;
                }
                write_cell_to_dots(&mut self.dot_luma, dot_width, x, y, cell);
            }
        }

        braille_quality::apply_tone_curve_into(&self.dot_luma, quality, &mut self.toned);
        let threshold = braille_quality::otsu_threshold(&self.toned);
        braille_quality::render_dot_luma_to_braille(
            &self.toned,
            dot_width,
            dot_height,
            threshold,
            quality.dither_mode,
            &mut self.braille,
        );

        grid.clear();
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let source = self.source_cells[idx];
                if should_preserve_cell(source.character) {
                    let cell = source;
                    if let Some(color) = cell.foreground_color {
                        grid.set_cell_with_color(x, y, cell.character, color);
                    } else {
                        grid.set_cell(x, y, cell.character);
                    }
                    continue;
                }

                let ch = self.braille.get_char(x, y);
                if ch == ' ' || ch == '⠀' {
                    continue;
                }

                let color = resolve_output_color(color_mode, source, &self.toned, dot_width, x, y);
                if let Some(color) = color {
                    grid.set_cell_with_color(x, y, ch, color);
                } else {
                    grid.set_cell(x, y, ch);
                }
            }
        }
    }
}

fn write_cell_to_dots(
    dot_luma: &mut [u8],
    dot_width: usize,
    cell_x: usize,
    cell_y: usize,
    cell: GridCell,
) {
    let intensity = cell_luminance(cell);
    let origin_x = cell_x * 2;
    let origin_y = cell_y * 4;

    if let Some(pattern) = braille_pattern(cell.character) {
        for (bit, (dx, dy)) in BRAILLE_DOT_MAP.iter().enumerate() {
            if (pattern & (1 << bit)) != 0 {
                dot_luma[(origin_y + dy) * dot_width + origin_x + dx] = intensity;
            }
        }
        return;
    }

    match cell.character {
        ' ' => {}
        '█' => fill_rect(dot_luma, dot_width, origin_x, origin_y, 2, 4, intensity),
        '▓' => fill_rect(
            dot_luma,
            dot_width,
            origin_x,
            origin_y,
            2,
            4,
            intensity.saturating_mul(5) / 6,
        ),
        '▒' => fill_rect(dot_luma, dot_width, origin_x, origin_y, 2, 4, intensity / 2),
        '░' => fill_rect(dot_luma, dot_width, origin_x, origin_y, 2, 4, intensity / 4),
        '▀' => fill_rect(dot_luma, dot_width, origin_x, origin_y, 2, 2, intensity),
        '▄' => fill_rect(dot_luma, dot_width, origin_x, origin_y + 2, 2, 2, intensity),
        '▌' => fill_rect(dot_luma, dot_width, origin_x, origin_y, 1, 4, intensity),
        '▐' => fill_rect(dot_luma, dot_width, origin_x + 1, origin_y, 1, 4, intensity),
        _ => fill_rect(dot_luma, dot_width, origin_x, origin_y, 2, 4, intensity),
    }
}

fn fill_rect(
    dot_luma: &mut [u8],
    dot_width: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    value: u8,
) {
    for dy in 0..h {
        for dx in 0..w {
            dot_luma[(y + dy) * dot_width + x + dx] = value;
        }
    }
}

fn resolve_output_color(
    color_mode: ColorMode,
    source: GridCell,
    toned: &[u8],
    dot_width: usize,
    cell_x: usize,
    cell_y: usize,
) -> Option<Color> {
    match color_mode {
        ColorMode::Off => None,
        ColorMode::Full => source
            .foreground_color
            .or_else(|| grayscale_for_cell(toned, dot_width, cell_x, cell_y)),
        ColorMode::Grayscale => grayscale_for_cell(toned, dot_width, cell_x, cell_y),
    }
}

fn grayscale_for_cell(
    toned: &[u8],
    dot_width: usize,
    cell_x: usize,
    cell_y: usize,
) -> Option<Color> {
    let origin_x = cell_x * 2;
    let origin_y = cell_y * 4;
    let mut total = 0u32;
    for dy in 0..4 {
        for dx in 0..2 {
            total += toned[(origin_y + dy) * dot_width + origin_x + dx] as u32;
        }
    }
    let avg = (total / 8) as u8;
    if avg == 0 {
        None
    } else {
        Some(Color::new(avg, avg, avg))
    }
}

fn cell_luminance(cell: GridCell) -> u8 {
    if let Some(color) = cell.foreground_color {
        ((color.r as u32 * 2126 + color.g as u32 * 7152 + color.b as u32 * 722) / 10_000) as u8
    } else if cell.character == ' ' {
        0
    } else {
        255
    }
}

fn braille_pattern(ch: char) -> Option<u8> {
    let code = ch as u32;
    if (0x2800..=0x28ff).contains(&code) {
        Some((code - 0x2800) as u8)
    } else {
        None
    }
}

fn should_preserve_cell(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            ':' | ';'
                | ','
                | '.'
                | '!'
                | '?'
                | '/'
                | '\\'
                | '-'
                | '_'
                | '+'
                | '='
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '"'
                | '\''
                | '#'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braille_pattern_round_trips_unicode_braille() {
        assert_eq!(braille_pattern('⠓'), Some(0b0001_0011));
    }

    #[test]
    fn postprocess_keeps_visible_output() {
        let mut grid = GridBuffer::new(16, 8);
        for x in 0..grid.width() {
            grid.set_cell_with_color(x, 3, '█', Color::new(255, 128, 32));
        }
        let mut processor = GridBraillePostProcessor::default();
        processor.apply(
            &mut grid,
            BrailleQualitySettings::default(),
            ColorMode::Full,
        );
        let visible = (0..grid.width())
            .filter(|&x| grid.get_cell(x, 3).character != ' ')
            .count();
        assert!(visible > 4);
    }

    #[test]
    fn postprocess_reuses_and_resizes_cached_buffers() {
        let mut processor = GridBraillePostProcessor::default();

        let mut small = GridBuffer::new(4, 2);
        small.set_cell(0, 0, '█');
        processor.apply(
            &mut small,
            BrailleQualitySettings::default(),
            ColorMode::Off,
        );
        assert_eq!(processor.dot_luma.len(), 4 * 2 * 8);
        assert_eq!(processor.source_cells.len(), 4 * 2);
        assert_eq!(processor.braille.width(), 4);
        assert_eq!(processor.braille.height(), 2);

        let mut large = GridBuffer::new(6, 3);
        large.set_cell(1, 1, '█');
        processor.apply(
            &mut large,
            BrailleQualitySettings::default(),
            ColorMode::Off,
        );
        assert_eq!(processor.dot_luma.len(), 6 * 3 * 8);
        assert_eq!(processor.source_cells.len(), 6 * 3);
        assert_eq!(processor.braille.width(), 6);
        assert_eq!(processor.braille.height(), 3);
    }
}
