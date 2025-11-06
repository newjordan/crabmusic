// Color schemes for visualizations
// Maps intensity values (0.0-1.0) to colors

use super::Color;

/// Color scheme type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSchemeType {
    /// No colors (monochrome)
    Monochrome,
    /// Rainbow gradient (red → orange → yellow → green → blue → purple)
    Rainbow,
    /// Heat map (black → red → orange → yellow → white)
    HeatMap,
    /// Blue to purple gradient
    BluePurple,
    /// Green to yellow gradient
    GreenYellow,
    /// Cyan to magenta gradient
    CyanMagenta,
}

impl ColorSchemeType {
    /// Get all available color scheme types
    pub fn all() -> Vec<ColorSchemeType> {
        vec![
            ColorSchemeType::Monochrome,
            ColorSchemeType::Rainbow,
            ColorSchemeType::HeatMap,
            ColorSchemeType::BluePurple,
            ColorSchemeType::GreenYellow,
            ColorSchemeType::CyanMagenta,
        ]
    }

    /// Get the name of the color scheme
    pub fn name(&self) -> &str {
        match self {
            ColorSchemeType::Monochrome => "Monochrome",
            ColorSchemeType::Rainbow => "Rainbow",
            ColorSchemeType::HeatMap => "Heat Map",
            ColorSchemeType::BluePurple => "Blue-Purple",
            ColorSchemeType::GreenYellow => "Green-Yellow",
            ColorSchemeType::CyanMagenta => "Cyan-Magenta",
        }
    }
}

/// Color scheme for mapping intensity to colors
#[derive(Clone)]
pub struct ColorScheme {
    scheme_type: ColorSchemeType,
}

impl ColorScheme {
    /// Create a new color scheme
    pub fn new(scheme_type: ColorSchemeType) -> Self {
        Self { scheme_type }
    }

    /// Get color for an intensity value (0.0-1.0)
    ///
    /// # Arguments
    /// * `intensity` - Intensity value from 0.0 (low) to 1.0 (high)
    ///
    /// # Returns
    /// Color for the given intensity, or None for monochrome
    pub fn get_color(&self, intensity: f32) -> Option<Color> {
        let intensity = intensity.clamp(0.0, 1.0);

        match self.scheme_type {
            ColorSchemeType::Monochrome => None,
            ColorSchemeType::Rainbow => Some(Self::rainbow_gradient(intensity)),
            ColorSchemeType::HeatMap => Some(Self::heat_map_gradient(intensity)),
            ColorSchemeType::BluePurple => Some(Self::blue_purple_gradient(intensity)),
            ColorSchemeType::GreenYellow => Some(Self::green_yellow_gradient(intensity)),
            ColorSchemeType::CyanMagenta => Some(Self::cyan_magenta_gradient(intensity)),
        }
    }

    /// Rainbow gradient: red → orange → yellow → green → blue → purple
    fn rainbow_gradient(t: f32) -> Color {
        // Use HSV color space for smooth rainbow
        let hue = t * 300.0; // 0-300 degrees (red to purple)
        Self::hsv_to_rgb(hue, 1.0, 1.0)
    }

    /// Heat map gradient: black → red → orange → yellow → white
    fn heat_map_gradient(t: f32) -> Color {
        if t < 0.25 {
            // Black to red
            let t = t * 4.0;
            Color::new((t * 255.0) as u8, 0, 0)
        } else if t < 0.5 {
            // Red to orange
            let t = (t - 0.25) * 4.0;
            Color::new(255, (t * 165.0) as u8, 0)
        } else if t < 0.75 {
            // Orange to yellow
            let t = (t - 0.5) * 4.0;
            Color::new(255, (165.0 + t * 90.0) as u8, 0)
        } else {
            // Yellow to white
            let t = (t - 0.75) * 4.0;
            Color::new(255, 255, (t * 255.0) as u8)
        }
    }

    /// Blue to purple gradient
    fn blue_purple_gradient(t: f32) -> Color {
        let r = (t * 128.0) as u8;
        let g = 0;
        let b = (255.0 - t * 128.0) as u8;
        Color::new(r, g, b)
    }

    /// Green to yellow gradient
    fn green_yellow_gradient(t: f32) -> Color {
        let r = (t * 255.0) as u8;
        let g = 255;
        let b = 0;
        Color::new(r, g, b)
    }

    /// Cyan to magenta gradient
    fn cyan_magenta_gradient(t: f32) -> Color {
        let r = (t * 255.0) as u8;
        let g = (255.0 - t * 255.0) as u8;
        let b = 255;
        Color::new(r, g, b)
    }

    /// Convert HSV to RGB
    /// H: 0-360, S: 0-1, V: 0-1
    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h_prime < 1.0 {
            (c, x, 0.0)
        } else if h_prime < 2.0 {
            (x, c, 0.0)
        } else if h_prime < 3.0 {
            (0.0, c, x)
        } else if h_prime < 4.0 {
            (0.0, x, c)
        } else if h_prime < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Color::new(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    /// Get the scheme type
    pub fn scheme_type(&self) -> ColorSchemeType {
        self.scheme_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monochrome_returns_none() {
        let scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        assert_eq!(scheme.get_color(0.5), None);
    }

    #[test]
    fn test_rainbow_gradient() {
        let scheme = ColorScheme::new(ColorSchemeType::Rainbow);

        // Low intensity should be reddish
        let low = scheme.get_color(0.0).unwrap();
        assert!(low.r > 200);

        // High intensity should be purplish
        let high = scheme.get_color(1.0).unwrap();
        assert!(high.r > 100 && high.b > 100);
    }

    #[test]
    fn test_heat_map_gradient() {
        let scheme = ColorScheme::new(ColorSchemeType::HeatMap);

        // Low intensity should be dark
        let low = scheme.get_color(0.0).unwrap();
        assert_eq!(low.r, 0);
        assert_eq!(low.g, 0);
        assert_eq!(low.b, 0);

        // High intensity should be bright
        let high = scheme.get_color(1.0).unwrap();
        assert!(high.r > 200 && high.g > 200 && high.b > 200);
    }

    #[test]
    fn test_intensity_clamping() {
        let scheme = ColorScheme::new(ColorSchemeType::Rainbow);

        // Values outside 0-1 should be clamped
        let below = scheme.get_color(-0.5);
        let above = scheme.get_color(1.5);

        assert!(below.is_some());
        assert!(above.is_some());
    }

    #[test]
    fn test_all_schemes() {
        let schemes = ColorSchemeType::all();
        assert_eq!(schemes.len(), 6);

        // Test that all schemes can be created and used
        for scheme_type in schemes {
            let scheme = ColorScheme::new(scheme_type);
            let _color = scheme.get_color(0.5);
            // Should not panic
        }
    }

    #[test]
    fn test_scheme_names() {
        assert_eq!(ColorSchemeType::Monochrome.name(), "Monochrome");
        assert_eq!(ColorSchemeType::Rainbow.name(), "Rainbow");
        assert_eq!(ColorSchemeType::HeatMap.name(), "Heat Map");
    }
}
