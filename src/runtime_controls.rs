use crossterm::event::KeyCode;

use crate::braille_quality::BrailleQualitySettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Off,
    Grayscale,
    Full,
}

impl ColorMode {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Grayscale,
            Self::Grayscale => Self::Full,
            Self::Full => Self::Off,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Grayscale => "GRAY",
            Self::Full => "FULL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityControlAction {
    CycleColorMode,
    CyclePreset,
    CycleDither,
    CycleGamma,
    CycleContrast,
    CycleExposure,
    Reset,
}

pub fn quality_control_action_from_key(code: KeyCode) -> Option<QualityControlAction> {
    match code {
        KeyCode::F(1) => Some(QualityControlAction::CycleColorMode),
        KeyCode::F(2) => Some(QualityControlAction::CyclePreset),
        KeyCode::F(3) => Some(QualityControlAction::CycleDither),
        KeyCode::F(4) => Some(QualityControlAction::CycleGamma),
        KeyCode::F(5) => Some(QualityControlAction::CycleContrast),
        KeyCode::F(6) => Some(QualityControlAction::CycleExposure),
        KeyCode::F(7) => Some(QualityControlAction::Reset),
        _ => None,
    }
}

pub fn apply_quality_action(
    action: QualityControlAction,
    quality: &mut BrailleQualitySettings,
    color_mode: &mut ColorMode,
) {
    match action {
        QualityControlAction::CycleColorMode => *color_mode = color_mode.next(),
        QualityControlAction::CyclePreset => quality.cycle_preset(),
        QualityControlAction::CycleDither => quality.cycle_dither(),
        QualityControlAction::CycleGamma => quality.cycle_gamma(),
        QualityControlAction::CycleContrast => quality.cycle_contrast(),
        QualityControlAction::CycleExposure => quality.cycle_exposure(),
        QualityControlAction::Reset => {}
    }
}

pub fn quality_summary(quality: BrailleQualitySettings, color_mode: ColorMode) -> String {
    let preset = quality
        .active_preset
        .map(|preset| preset.short_name())
        .unwrap_or("CUS");
    format!(
        "Q:{} {} {} g{:.2} c{:.2} e{:.2} F1-7",
        color_mode.short_name(),
        preset,
        quality.dither_mode.short_name(),
        quality.gamma(),
        quality.contrast(),
        quality.exposure()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::braille_quality::QualityPreset;

    #[test]
    fn f_keys_map_to_quality_actions() {
        assert_eq!(
            quality_control_action_from_key(KeyCode::F(3)),
            Some(QualityControlAction::CycleDither)
        );
        assert_eq!(
            quality_control_action_from_key(KeyCode::F(7)),
            Some(QualityControlAction::Reset)
        );
    }

    #[test]
    fn quality_summary_includes_mode_and_preset() {
        let summary = quality_summary(
            BrailleQualitySettings::from_preset(QualityPreset::Motion),
            ColorMode::Full,
        );
        assert!(summary.contains("FULL"));
        assert!(summary.contains("MOT"));
    }
}
