// Windowing functions for DSP
// Provides various window functions to reduce spectral leakage in FFT analysis

/// Window function type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    /// Rectangular window (no windowing)
    Rectangular,
    /// Hann window (raised cosine)
    Hann,
    /// Hamming window
    Hamming,
    /// Blackman window
    Blackman,
    /// Blackman-Harris window
    BlackmanHarris,
}

/// Generate a window function
///
/// # Arguments
/// * `window_type` - Type of window to generate
/// * `size` - Window size
///
/// # Returns
/// Vector of window coefficients
///
/// # Examples
///
/// ```
/// use crabmusic::dsp::windowing::{generate_window, WindowType};
///
/// let window = generate_window(WindowType::Hann, 1024);
/// assert_eq!(window.len(), 1024);
/// ```
pub fn generate_window(window_type: WindowType, size: usize) -> Vec<f32> {
    match window_type {
        WindowType::Rectangular => generate_rectangular(size),
        WindowType::Hann => generate_hann(size),
        WindowType::Hamming => generate_hamming(size),
        WindowType::Blackman => generate_blackman(size),
        WindowType::BlackmanHarris => generate_blackman_harris(size),
    }
}

/// Generate rectangular window (no windowing)
///
/// All coefficients are 1.0. Provides best frequency resolution
/// but worst spectral leakage.
///
/// # Arguments
/// * `size` - Window size
///
/// # Returns
/// Vector of window coefficients (all 1.0)
fn generate_rectangular(size: usize) -> Vec<f32> {
    vec![1.0; size]
}

/// Generate Hann window
///
/// Raised cosine window. Good general-purpose window with
/// moderate frequency resolution and spectral leakage.
///
/// Formula: w[n] = 0.5 * (1 - cos(2π * n / (N-1)))
///
/// # Arguments
/// * `size` - Window size
///
/// # Returns
/// Vector of Hann window coefficients
pub fn generate_hann(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| {
            let factor = 2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32;
            0.5 * (1.0 - factor.cos())
        })
        .collect()
}

/// Generate Hamming window
///
/// Similar to Hann but with slightly different coefficients.
/// Better frequency resolution than Hann but slightly more leakage.
///
/// Formula: w[n] = 0.54 - 0.46 * cos(2π * n / (N-1))
///
/// # Arguments
/// * `size` - Window size
///
/// # Returns
/// Vector of Hamming window coefficients
pub fn generate_hamming(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| {
            let factor = 2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32;
            0.54 - 0.46 * factor.cos()
        })
        .collect()
}

/// Generate Blackman window
///
/// Provides excellent spectral leakage reduction at the cost
/// of wider main lobe (reduced frequency resolution).
///
/// Formula: w[n] = 0.42 - 0.5 * cos(2π * n / (N-1)) + 0.08 * cos(4π * n / (N-1))
///
/// # Arguments
/// * `size` - Window size
///
/// # Returns
/// Vector of Blackman window coefficients
pub fn generate_blackman(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| {
            let factor = 2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32;
            0.42 - 0.5 * factor.cos() + 0.08 * (2.0 * factor).cos()
        })
        .collect()
}

/// Generate Blackman-Harris window
///
/// Four-term Blackman-Harris window. Provides the best spectral
/// leakage reduction but widest main lobe.
///
/// # Arguments
/// * `size` - Window size
///
/// # Returns
/// Vector of Blackman-Harris window coefficients
pub fn generate_blackman_harris(size: usize) -> Vec<f32> {
    const A0: f32 = 0.35875;
    const A1: f32 = 0.48829;
    const A2: f32 = 0.14128;
    const A3: f32 = 0.01168;

    (0..size)
        .map(|i| {
            let factor = 2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32;
            A0 - A1 * factor.cos() + A2 * (2.0 * factor).cos() - A3 * (3.0 * factor).cos()
        })
        .collect()
}

/// Apply a window function to a signal
///
/// Multiplies each sample by the corresponding window coefficient.
///
/// # Arguments
/// * `signal` - Input signal (will be modified in-place)
/// * `window` - Window coefficients
///
/// # Panics
/// Panics if signal and window have different lengths
///
/// # Examples
///
/// ```
/// use crabmusic::dsp::windowing::{apply_window, generate_window, WindowType};
///
/// let mut signal = vec![1.0; 1024];
/// let window = generate_window(WindowType::Hann, 1024);
/// apply_window(&mut signal, &window);
/// ```
pub fn apply_window(signal: &mut [f32], window: &[f32]) {
    assert_eq!(
        signal.len(),
        window.len(),
        "Signal and window must have the same length"
    );

    for (sample, &coeff) in signal.iter_mut().zip(window.iter()) {
        *sample *= coeff;
    }
}

/// Calculate the coherent gain of a window
///
/// The coherent gain is the average value of the window coefficients.
/// Used for amplitude correction after windowing.
///
/// # Arguments
/// * `window` - Window coefficients
///
/// # Returns
/// Coherent gain value
///
/// # Examples
///
/// ```
/// use crabmusic::dsp::windowing::{generate_window, coherent_gain, WindowType};
///
/// let window = generate_window(WindowType::Hann, 1024);
/// let gain = coherent_gain(&window);
/// assert!(gain > 0.4 && gain < 0.6); // Hann window has ~0.5 coherent gain
/// ```
pub fn coherent_gain(window: &[f32]) -> f32 {
    let sum: f32 = window.iter().sum();
    sum / window.len() as f32
}

/// Calculate the noise equivalent bandwidth of a window
///
/// ENBW represents the effective bandwidth of the window in bins.
/// Used for power spectrum density calculations.
///
/// # Arguments
/// * `window` - Window coefficients
///
/// # Returns
/// Noise equivalent bandwidth in bins
pub fn noise_equivalent_bandwidth(window: &[f32]) -> f32 {
    let sum_squared: f32 = window.iter().map(|&w| w * w).sum();
    let sum: f32 = window.iter().sum();
    window.len() as f32 * sum_squared / (sum * sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangular_window() {
        let window = generate_rectangular(10);
        assert_eq!(window.len(), 10);
        assert!(window.iter().all(|&w| w == 1.0));
    }

    #[test]
    fn test_hann_window() {
        let window = generate_hann(10);
        assert_eq!(window.len(), 10);
        assert!(window[0] < 0.01); // Should be near 0 at edges
        assert!(window[9] < 0.01);
        assert!(window[5] > 0.9); // Should be near 1 at center
    }

    #[test]
    fn test_hamming_window() {
        let window = generate_hamming(10);
        assert_eq!(window.len(), 10);
        assert!(window[0] > 0.0); // Hamming doesn't go to 0 at edges
        assert!(window[5] > 0.9); // Should be near 1 at center
    }

    #[test]
    fn test_blackman_window() {
        let window = generate_blackman(10);
        assert_eq!(window.len(), 10);
        assert!(window[0] < 0.01); // Should be near 0 at edges
        assert!(window[5] > 0.9); // Should be near 1 at center
    }

    #[test]
    fn test_blackman_harris_window() {
        let window = generate_blackman_harris(10);
        assert_eq!(window.len(), 10);
        assert!(window[0] < 0.01); // Should be near 0 at edges
        assert!(window[5] > 0.9); // Should be near 1 at center
    }

    #[test]
    fn test_apply_window() {
        let mut signal = vec![1.0; 10];
        let window = generate_hann(10);
        apply_window(&mut signal, &window);
        
        assert!(signal[0] < 0.1); // Edges should be attenuated
        assert!(signal[5] > 0.9); // Center should be near 1
    }

    #[test]
    fn test_coherent_gain_rectangular() {
        let window = generate_rectangular(100);
        let gain = coherent_gain(&window);
        assert!((gain - 1.0).abs() < 0.001); // Rectangular has gain of 1
    }

    #[test]
    fn test_coherent_gain_hann() {
        let window = generate_hann(100);
        let gain = coherent_gain(&window);
        assert!(gain > 0.4 && gain < 0.6); // Hann has gain ~0.5
    }

    #[test]
    fn test_noise_equivalent_bandwidth() {
        let window = generate_rectangular(100);
        let enbw = noise_equivalent_bandwidth(&window);
        assert!((enbw - 1.0).abs() < 0.001); // Rectangular has ENBW of 1
    }

    #[test]
    fn test_generate_window_enum() {
        let types = vec![
            WindowType::Rectangular,
            WindowType::Hann,
            WindowType::Hamming,
            WindowType::Blackman,
            WindowType::BlackmanHarris,
        ];

        for window_type in types {
            let window = generate_window(window_type, 100);
            assert_eq!(window.len(), 100);
        }
    }
}

