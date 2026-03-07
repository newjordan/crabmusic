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

#[derive(Default)]
pub struct GridBraillePostProcessor;

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
        let dot_width = width * 2;
        let dot_height = height * 4;
        let mut dot_luma = vec![0u8; dot_width * dot_height];
        let mut source_cells = Vec::with_capacity(width * height);
        let mut protected_cells = vec![None; width * height];

        for y in 0..height {
            for x in 0..width {
                let cell = *grid.get_cell(x, y);
                source_cells.push(cell);
                if should_preserve_cell(cell.character) {
                    protected_cells[y * width + x] = Some(cell);
                    continue;
                }
                write_cell_to_dots(&mut dot_luma, dot_width, x, y, cell);
            }
        }

        let toned = braille_quality::apply_tone_curve(&dot_luma, quality);
        let threshold = braille_quality::otsu_threshold(&toned);
        let mut braille = BrailleGrid::new(width, height);
        braille_quality::render_dot_luma_to_braille(
            &toned,
            dot_width,
            dot_height,
            threshold,
            quality.dither_mode,
            &mut braille,
        );

        grid.clear();
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if let Some(cell) = protected_cells[idx] {
                    if let Some(color) = cell.foreground_color {
                        grid.set_cell_with_color(x, y, cell.character, color);
                    } else {
                        grid.set_cell(x, y, cell.character);
                    }
                    continue;
                }

                let ch = braille.get_char(x, y);
                if ch == ' ' || ch == '⠀' {
                    continue;
                }

                let color =
                    resolve_output_color(color_mode, source_cells[idx], &toned, dot_width, x, y);
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
        let mut processor = GridBraillePostProcessor;
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
}
