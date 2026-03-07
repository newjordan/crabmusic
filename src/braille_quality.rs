use serde::{Deserialize, Serialize};

use crate::visualization::braille::BrailleGrid;

pub const GAMMA_PRESETS: [f32; 4] = [0.85, 1.0, 1.2, 1.45];
pub const CONTRAST_PRESETS: [f32; 4] = [0.85, 1.0, 1.2, 1.4];
pub const EXPOSURE_PRESETS: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
pub const TEMPORAL_BLEND_PRESETS: [f32; 5] = [0.0, 0.15, 0.28, 0.42, 0.58];
pub const TEMPORAL_HYSTERESIS_PRESETS: [u8; 5] = [0, 6, 12, 20, 28];

const BAYER_4X4: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
const BAYER_8X8: [u8; 64] = [
    0, 48, 12, 60, 3, 51, 15, 63, 32, 16, 44, 28, 35, 19, 47, 31, 8, 56, 4, 52, 11, 59, 7, 55, 40,
    24, 36, 20, 43, 27, 39, 23, 2, 50, 14, 62, 1, 49, 13, 61, 34, 18, 46, 30, 33, 17, 45, 29, 10,
    58, 6, 54, 9, 57, 5, 53, 42, 26, 38, 22, 41, 25, 37, 21,
];

// Treat tiny residual luma as true black so dithering does not pepper backgrounds
// that should read as empty space after resize/compression/tone-shaping.
const SHADOW_FLOOR: u8 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DitherMode {
    Threshold,
    Bayer4x4,
    Bayer8x8,
    FloydSteinberg,
    Atkinson,
}

impl DitherMode {
    pub fn next(self) -> Self {
        match self {
            Self::Threshold => Self::Bayer4x4,
            Self::Bayer4x4 => Self::Bayer8x8,
            Self::Bayer8x8 => Self::FloydSteinberg,
            Self::FloydSteinberg => Self::Atkinson,
            Self::Atkinson => Self::Threshold,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Threshold => "THR",
            Self::Bayer4x4 => "B4",
            Self::Bayer8x8 => "B8",
            Self::FloydSteinberg => "FS",
            Self::Atkinson => "ATK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityPreset {
    Motion,
    Cinema,
    Punch,
    Newsprint,
    PlanetKiller,
}

impl QualityPreset {
    pub fn next(self) -> Self {
        match self {
            Self::Motion => Self::Cinema,
            Self::Cinema => Self::Punch,
            Self::Punch => Self::Newsprint,
            Self::Newsprint => Self::PlanetKiller,
            Self::PlanetKiller => Self::Motion,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Motion => "MOT",
            Self::Cinema => "CIN",
            Self::Punch => "PNC",
            Self::Newsprint => "NWS",
            Self::PlanetKiller => "PKR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrailleQualitySettings {
    pub active_preset: Option<QualityPreset>,
    pub dither_mode: DitherMode,
    pub gamma_preset: usize,
    pub contrast_preset: usize,
    pub exposure_preset: usize,
}

impl Default for BrailleQualitySettings {
    fn default() -> Self {
        Self {
            active_preset: None,
            dither_mode: DitherMode::Threshold,
            gamma_preset: 1,
            contrast_preset: 1,
            exposure_preset: 1,
        }
    }
}

impl BrailleQualitySettings {
    pub fn image_default() -> Self {
        Self::from_preset(QualityPreset::Cinema)
    }

    pub fn from_preset(preset: QualityPreset) -> Self {
        match preset {
            QualityPreset::Motion => Self {
                active_preset: Some(preset),
                dither_mode: DitherMode::Bayer4x4,
                gamma_preset: 1,
                contrast_preset: 1,
                exposure_preset: 1,
            },
            QualityPreset::Cinema => Self {
                active_preset: Some(preset),
                dither_mode: DitherMode::FloydSteinberg,
                gamma_preset: 2,
                contrast_preset: 1,
                exposure_preset: 1,
            },
            QualityPreset::Punch => Self {
                active_preset: Some(preset),
                dither_mode: DitherMode::Bayer8x8,
                gamma_preset: 0,
                contrast_preset: 2,
                exposure_preset: 2,
            },
            QualityPreset::Newsprint => Self {
                active_preset: Some(preset),
                dither_mode: DitherMode::Atkinson,
                gamma_preset: 2,
                contrast_preset: 3,
                exposure_preset: 1,
            },
            QualityPreset::PlanetKiller => Self {
                active_preset: Some(preset),
                dither_mode: DitherMode::Atkinson,
                gamma_preset: 0,
                contrast_preset: 3,
                exposure_preset: 3,
            },
        }
    }

    pub fn preset_label(self) -> &'static str {
        self.active_preset
            .map(QualityPreset::short_name)
            .unwrap_or("CUS")
    }

    pub fn cycle_preset(&mut self) {
        let next = self.active_preset.unwrap_or(QualityPreset::Motion).next();
        *self = Self::from_preset(next);
    }

    pub fn with_overrides(
        mut self,
        dither_mode: Option<DitherMode>,
        gamma_preset: Option<usize>,
        contrast_preset: Option<usize>,
        exposure_preset: Option<usize>,
    ) -> Self {
        let baseline = self;
        if let Some(mode) = dither_mode {
            self.dither_mode = mode;
        }
        if let Some(preset) = gamma_preset {
            self.gamma_preset = preset.min(GAMMA_PRESETS.len() - 1);
        }
        if let Some(preset) = contrast_preset {
            self.contrast_preset = preset.min(CONTRAST_PRESETS.len() - 1);
        }
        if let Some(preset) = exposure_preset {
            self.exposure_preset = preset.min(EXPOSURE_PRESETS.len() - 1);
        }

        if self.dither_mode != baseline.dither_mode
            || self.gamma_preset != baseline.gamma_preset
            || self.contrast_preset != baseline.contrast_preset
            || self.exposure_preset != baseline.exposure_preset
        {
            self.active_preset = None;
        }

        self
    }

    pub fn gamma(self) -> f32 {
        GAMMA_PRESETS[self.gamma_preset.min(GAMMA_PRESETS.len() - 1)]
    }

    pub fn contrast(self) -> f32 {
        CONTRAST_PRESETS[self.contrast_preset.min(CONTRAST_PRESETS.len() - 1)]
    }

    pub fn exposure(self) -> f32 {
        EXPOSURE_PRESETS[self.exposure_preset.min(EXPOSURE_PRESETS.len() - 1)]
    }

    pub fn cycle_dither(&mut self) {
        self.active_preset = None;
        self.dither_mode = self.dither_mode.next();
    }

    pub fn cycle_gamma(&mut self) {
        self.active_preset = None;
        self.gamma_preset = (self.gamma_preset + 1) % GAMMA_PRESETS.len();
    }

    pub fn cycle_contrast(&mut self) {
        self.active_preset = None;
        self.contrast_preset = (self.contrast_preset + 1) % CONTRAST_PRESETS.len();
    }

    pub fn cycle_exposure(&mut self) {
        self.active_preset = None;
        self.exposure_preset = (self.exposure_preset + 1) % EXPOSURE_PRESETS.len();
    }
}

pub fn otsu_threshold(luma: &[u8]) -> u8 {
    let mut hist = [0u32; 256];
    for &v in luma {
        hist[v as usize] += 1;
    }
    let total = luma.len() as u32;
    let mut sum_all = 0u64;
    for (i, count) in hist.iter().enumerate() {
        sum_all += (i as u64) * (*count as u64);
    }

    let mut sum_b = 0u64;
    let mut w_b = 0u32;
    let mut max_var = -1.0;
    let mut threshold = 128u8;
    for (t, count) in hist.iter().enumerate() {
        w_b += *count;
        if w_b == 0 {
            continue;
        }
        let w_f = total.saturating_sub(w_b);
        if w_f == 0 {
            break;
        }
        sum_b += (t as u64) * (*count as u64);
        let m_b = sum_b as f64 / w_b as f64;
        let m_f = (sum_all - sum_b) as f64 / w_f as f64;
        let var_between = (w_b as f64) * (w_f as f64) * (m_b - m_f).powi(2);
        if var_between > max_var {
            max_var = var_between;
            threshold = t as u8;
        }
    }
    threshold
}

pub fn apply_tone_curve(luma: &[u8], quality: BrailleQualitySettings) -> Vec<u8> {
    luma.iter()
        .map(|&v| {
            let exposed = ((v as f32 / 255.0) * quality.exposure()).clamp(0.0, 1.0);
            let centered = (exposed - 0.5) * quality.contrast() + 0.5;
            let contrasted = centered.clamp(0.0, 1.0);
            let gamma_corrected = contrasted.powf(1.0 / quality.gamma());
            (gamma_corrected * 255.0).round().clamp(0.0, 255.0) as u8
        })
        .collect()
}

pub fn preprocess_luma_to_dot_grid(
    luma: &[u8],
    img_w: usize,
    img_h: usize,
    dot_w: usize,
    dot_h: usize,
    quality: BrailleQualitySettings,
) -> Vec<u8> {
    sample_to_dot_grid(luma, img_w, img_h, dot_w, dot_h, quality)
}

pub fn blend_dot_luma_with_previous(
    current: &[u8],
    previous: Option<&[u8]>,
    amount: f32,
) -> Vec<u8> {
    let blend = amount.clamp(0.0, 0.95);
    match previous {
        Some(prev) if prev.len() == current.len() && blend > 0.0 => current
            .iter()
            .zip(prev.iter())
            .map(|(&cur, &old)| {
                let mixed = cur as f32 * (1.0 - blend) + old as f32 * blend;
                mixed.round().clamp(0.0, 255.0) as u8
            })
            .collect(),
        _ => current.to_vec(),
    }
}

pub fn apply_temporal_hysteresis(
    current: &[u8],
    previous_mask: Option<&[u8]>,
    strength: u8,
) -> Vec<u8> {
    match previous_mask {
        Some(previous) if previous.len() == current.len() && strength > 0 => current
            .iter()
            .zip(previous.iter())
            .map(|(&cur, &prev)| {
                let bias = if prev > 0 {
                    strength as i16
                } else {
                    -(strength as i16)
                };
                (cur as i16 + bias).clamp(0, 255) as u8
            })
            .collect(),
        _ => current.to_vec(),
    }
}

pub fn render_dot_luma_to_braille(
    sampled: &[u8],
    dot_w: usize,
    dot_h: usize,
    threshold: u8,
    dither_mode: DitherMode,
    braille: &mut BrailleGrid,
) {
    braille.clear();
    match dither_mode {
        DitherMode::Threshold => apply_threshold(sampled, dot_w, dot_h, threshold, braille),
        DitherMode::Bayer4x4 => apply_ordered_dither(sampled, dot_w, dot_h, threshold, braille, 4),
        DitherMode::Bayer8x8 => apply_ordered_dither(sampled, dot_w, dot_h, threshold, braille, 8),
        DitherMode::FloydSteinberg => {
            apply_floyd_steinberg(sampled, dot_w, dot_h, threshold, braille)
        }
        DitherMode::Atkinson => apply_atkinson(sampled, dot_w, dot_h, threshold, braille),
    }
}

pub fn blit_luma_to_braille_with_quality(
    luma: &[u8],
    img_w: usize,
    img_h: usize,
    threshold: u8,
    quality: BrailleQualitySettings,
    braille: &mut BrailleGrid,
) {
    if img_w == 0 || img_h == 0 {
        return;
    }
    let dot_w = braille.dot_width();
    let dot_h = braille.dot_height();
    let sampled = preprocess_luma_to_dot_grid(luma, img_w, img_h, dot_w, dot_h, quality);
    render_dot_luma_to_braille(
        &sampled,
        dot_w,
        dot_h,
        threshold,
        quality.dither_mode,
        braille,
    );
}

pub fn capture_braille_dot_mask(braille: &BrailleGrid) -> Vec<u8> {
    let mut mask = vec![0; braille.dot_width() * braille.dot_height()];
    for y in 0..braille.dot_height() {
        for x in 0..braille.dot_width() {
            if braille.is_dot_set(x, y) {
                mask[y * braille.dot_width() + x] = 255;
            }
        }
    }
    mask
}

fn sample_to_dot_grid(
    luma: &[u8],
    img_w: usize,
    img_h: usize,
    dot_w: usize,
    dot_h: usize,
    quality: BrailleQualitySettings,
) -> Vec<u8> {
    let mut sampled = vec![0u8; dot_w * dot_h];
    for dy in 0..dot_h {
        let sy = (dy * img_h) / dot_h;
        let sy_off = sy * img_w;
        for dx in 0..dot_w {
            let sx = (dx * img_w) / dot_w;
            sampled[dy * dot_w + dx] = luma[sy_off + sx];
        }
    }
    apply_tone_curve(&sampled, quality)
}

#[inline]
fn clamp_shadow_floor(value: u8) -> u8 {
    if value <= SHADOW_FLOOR {
        0
    } else {
        value
    }
}

fn apply_threshold(
    sampled: &[u8],
    dot_w: usize,
    dot_h: usize,
    threshold: u8,
    braille: &mut BrailleGrid,
) {
    for y in 0..dot_h {
        for x in 0..dot_w {
            if clamp_shadow_floor(sampled[y * dot_w + x]) >= threshold {
                braille.set_dot(x, y);
            }
        }
    }
}

fn apply_ordered_dither(
    sampled: &[u8],
    dot_w: usize,
    dot_h: usize,
    threshold: u8,
    braille: &mut BrailleGrid,
    matrix_size: usize,
) {
    let matrix = if matrix_size == 4 {
        &BAYER_4X4[..]
    } else {
        &BAYER_8X8[..]
    };
    let levels = matrix.len() as f32;
    for y in 0..dot_h {
        for x in 0..dot_w {
            let value = clamp_shadow_floor(sampled[y * dot_w + x]);
            if value == 0 {
                continue;
            }
            let idx = (y % matrix_size) * matrix_size + (x % matrix_size);
            let bias = (((matrix[idx] as f32 + 0.5) / levels) - 0.5) * 96.0;
            if value as f32 + bias >= threshold as f32 {
                braille.set_dot(x, y);
            }
        }
    }
}

fn apply_floyd_steinberg(
    sampled: &[u8],
    dot_w: usize,
    dot_h: usize,
    threshold: u8,
    braille: &mut BrailleGrid,
) {
    let mut work = sampled
        .iter()
        .map(|&v| clamp_shadow_floor(v) as f32)
        .collect::<Vec<_>>();
    for y in 0..dot_h {
        for x in 0..dot_w {
            let idx = y * dot_w + x;
            let old = work[idx];
            let new = if old >= threshold as f32 { 255.0 } else { 0.0 };
            if new > 0.0 {
                braille.set_dot(x, y);
            }
            let err = old - new;
            diffuse(&mut work, dot_w, dot_h, x + 1, y, err * 7.0 / 16.0);
            if x > 0 {
                diffuse(&mut work, dot_w, dot_h, x - 1, y + 1, err * 3.0 / 16.0);
            }
            diffuse(&mut work, dot_w, dot_h, x, y + 1, err * 5.0 / 16.0);
            diffuse(&mut work, dot_w, dot_h, x + 1, y + 1, err * 1.0 / 16.0);
        }
    }
}

fn apply_atkinson(
    sampled: &[u8],
    dot_w: usize,
    dot_h: usize,
    threshold: u8,
    braille: &mut BrailleGrid,
) {
    let mut work = sampled
        .iter()
        .map(|&v| clamp_shadow_floor(v) as f32)
        .collect::<Vec<_>>();
    for y in 0..dot_h {
        for x in 0..dot_w {
            let idx = y * dot_w + x;
            let old = work[idx];
            let new = if old >= threshold as f32 { 255.0 } else { 0.0 };
            if new > 0.0 {
                braille.set_dot(x, y);
            }
            let err = (old - new) / 8.0;
            diffuse(&mut work, dot_w, dot_h, x + 1, y, err);
            diffuse(&mut work, dot_w, dot_h, x + 2, y, err);
            if y + 1 < dot_h {
                if x > 0 {
                    diffuse(&mut work, dot_w, dot_h, x - 1, y + 1, err);
                }
                diffuse(&mut work, dot_w, dot_h, x, y + 1, err);
                diffuse(&mut work, dot_w, dot_h, x + 1, y + 1, err);
            }
            diffuse(&mut work, dot_w, dot_h, x, y + 2, err);
        }
    }
}

fn diffuse(work: &mut [f32], dot_w: usize, dot_h: usize, x: usize, y: usize, delta: f32) {
    if x < dot_w && y < dot_h {
        let idx = y * dot_w + x;
        work[idx] = (work[idx] + delta).clamp(0.0, 255.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_curve_can_lift_midtones() {
        let mut quality = BrailleQualitySettings::default();
        quality.gamma_preset = 3;
        let adjusted = apply_tone_curve(&[96], quality);
        assert!(adjusted[0] > 96);
    }

    #[test]
    fn exposure_can_brighten_input() {
        let mut quality = BrailleQualitySettings::default();
        quality.exposure_preset = 3;
        let adjusted = apply_tone_curve(&[96], quality);
        assert!(adjusted[0] > 96);
    }

    #[test]
    fn bayer_dither_uses_partial_pattern_for_flat_midtone() {
        let mut braille = BrailleGrid::new(1, 1);
        let quality = BrailleQualitySettings {
            dither_mode: DitherMode::Bayer4x4,
            ..BrailleQualitySettings::default()
        };
        blit_luma_to_braille_with_quality(&vec![128u8; 8], 2, 4, 128, quality, &mut braille);
        let ch = braille.get_char(0, 0);
        assert_ne!(ch, '⠀');
        assert_ne!(ch, '⣿');
    }

    #[test]
    fn floyd_steinberg_keeps_full_white_full() {
        let mut braille = BrailleGrid::new(1, 1);
        let quality = BrailleQualitySettings {
            dither_mode: DitherMode::FloydSteinberg,
            ..BrailleQualitySettings::default()
        };
        blit_luma_to_braille_with_quality(&vec![255u8; 8], 2, 4, 128, quality, &mut braille);
        assert_eq!(braille.get_char(0, 0), '⣿');
    }

    #[test]
    fn atkinson_dither_uses_partial_pattern_for_flat_midtone() {
        let mut braille = BrailleGrid::new(1, 1);
        let quality = BrailleQualitySettings {
            dither_mode: DitherMode::Atkinson,
            ..BrailleQualitySettings::default()
        };
        blit_luma_to_braille_with_quality(&[128u8; 8], 2, 4, 128, quality, &mut braille);
        let ch = braille.get_char(0, 0);
        assert_ne!(ch, '⠀');
        assert_ne!(ch, '⣿');
    }

    #[test]
    fn temporal_blend_preserves_previous_energy() {
        let current = vec![255u8; 8];
        let previous = vec![0u8; 8];
        let blended = blend_dot_luma_with_previous(&current, Some(&previous), 0.5);
        assert_eq!(blended[0], 128);
    }

    #[test]
    fn shadow_floor_keeps_dark_backgrounds_blank_across_dither_modes() {
        for dither_mode in [
            DitherMode::Threshold,
            DitherMode::Bayer4x4,
            DitherMode::Bayer8x8,
            DitherMode::FloydSteinberg,
            DitherMode::Atkinson,
        ] {
            let mut braille = BrailleGrid::new(1, 1);
            let quality = BrailleQualitySettings {
                dither_mode,
                ..BrailleQualitySettings::default()
            };
            blit_luma_to_braille_with_quality(&[SHADOW_FLOOR; 8], 2, 4, 128, quality, &mut braille);
            assert_eq!(
                braille.get_char(0, 0),
                '⠀',
                "expected blank background for {:?}",
                dither_mode
            );
        }
    }

    #[test]
    fn temporal_hysteresis_favors_previous_on_pixels() {
        let current = vec![120u8; 4];
        let previous = vec![255u8, 0, 255, 0];
        let adjusted = apply_temporal_hysteresis(&current, Some(&previous), 12);
        assert_eq!(adjusted, vec![132u8, 108, 132, 108]);
    }

    #[test]
    fn capture_braille_dot_mask_reports_active_dots() {
        let mut braille = BrailleGrid::new(1, 1);
        braille.set_dot(0, 0);
        braille.set_dot(1, 3);
        let mask = capture_braille_dot_mask(&braille);
        assert_eq!(mask, vec![255, 0, 0, 0, 0, 0, 0, 255]);
    }

    #[test]
    fn quality_preset_round_trip_sets_expected_mode() {
        let quality = BrailleQualitySettings::from_preset(QualityPreset::Newsprint);
        assert_eq!(quality.active_preset, Some(QualityPreset::Newsprint));
        assert_eq!(quality.dither_mode, DitherMode::Atkinson);
    }

    #[test]
    fn manual_tweak_marks_quality_as_custom() {
        let mut quality = BrailleQualitySettings::image_default();
        quality.cycle_contrast();
        assert_eq!(quality.active_preset, None);
        assert_eq!(quality.preset_label(), "CUS");
    }

    #[test]
    fn override_helper_keeps_preset_when_values_match() {
        let quality = BrailleQualitySettings::from_preset(QualityPreset::Motion).with_overrides(
            Some(DitherMode::Bayer4x4),
            Some(1),
            Some(1),
            Some(1),
        );
        assert_eq!(quality.active_preset, Some(QualityPreset::Motion));
    }

    #[test]
    fn override_helper_marks_custom_when_values_change() {
        let quality = BrailleQualitySettings::from_preset(QualityPreset::Motion).with_overrides(
            Some(DitherMode::Atkinson),
            None,
            None,
            None,
        );
        assert_eq!(quality.active_preset, None);
        assert_eq!(quality.dither_mode, DitherMode::Atkinson);
    }

    fn render_snapshot(quality: BrailleQualitySettings) -> String {
        let luma: [u8; 64] = [
            8, 32, 64, 96, 128, 160, 192, 224, 16, 48, 80, 112, 144, 176, 208, 240, 24, 56, 88,
            120, 152, 184, 216, 248, 0, 24, 60, 90, 126, 156, 190, 220, 12, 44, 76, 108, 140, 172,
            204, 236, 20, 52, 84, 116, 148, 180, 212, 244, 28, 58, 92, 124, 158, 188, 218, 250, 4,
            36, 68, 100, 132, 164, 196, 228,
        ];
        let sampled = preprocess_luma_to_dot_grid(&luma, 8, 8, 8, 8, quality);
        let mut braille = BrailleGrid::new(4, 2);
        render_dot_luma_to_braille(&sampled, 8, 8, 128, quality.dither_mode, &mut braille);

        let mut out = String::new();
        for y in 0..braille.height() {
            for x in 0..braille.width() {
                out.push(braille.get_char(x, y));
            }
            if y + 1 < braille.height() {
                out.push('\n');
            }
        }
        out
    }

    #[test]
    fn quality_preset_snapshots_match_expected_golden_output() {
        assert_eq!(
            render_snapshot(BrailleQualitySettings::from_preset(QualityPreset::Motion)),
            "⠀⠠⣺⣿\n⠀⠠⣾⣿"
        );
        assert_eq!(
            render_snapshot(BrailleQualitySettings::from_preset(QualityPreset::Cinema)),
            "⠀⢎⢞⣿\n⠠⢣⢻⣾"
        );
        assert_eq!(
            render_snapshot(BrailleQualitySettings::from_preset(QualityPreset::Punch)),
            "⠀⠨⣾⣿\n⠀⠨⣾⣿"
        );
        assert_eq!(
            render_snapshot(BrailleQualitySettings::from_preset(
                QualityPreset::Newsprint
            )),
            "⠀⠰⣻⣿\n⠀⠸⢷⣿"
        );
        assert_eq!(
            render_snapshot(BrailleQualitySettings::from_preset(
                QualityPreset::PlanetKiller
            )),
            "⠀⠰⣿⣿\n⠀⠼⣿⣿"
        );
    }
}
