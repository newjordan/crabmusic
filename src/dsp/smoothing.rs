// Parameter smoothing algorithms for audio DSP
// Provides various smoothing techniques to reduce jitter and noise in extracted parameters

/// Exponential moving average smoother
///
/// Uses exponential weighting to smooth parameter values over time.
/// More recent values have higher weight than older values.
///
/// # Examples
///
/// ```
/// use crabmusic::dsp::smoothing::ExponentialSmoother;
///
/// let mut smoother = ExponentialSmoother::new(0.3);
/// let smoothed1 = smoother.smooth(1.0);
/// let smoothed2 = smoother.smooth(0.5);
/// assert!(smoothed2 > 0.5 && smoothed2 < 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct ExponentialSmoother {
    /// Smoothing factor (0.0 = no smoothing, 1.0 = maximum smoothing)
    alpha: f32,
    /// Current smoothed value
    current: f32,
    /// Whether the smoother has been initialized
    initialized: bool,
}

impl ExponentialSmoother {
    /// Create a new exponential smoother
    ///
    /// # Arguments
    /// * `alpha` - Smoothing factor (0.0 to 1.0)
    ///   - 0.0 = no smoothing (instant response)
    ///   - 1.0 = maximum smoothing (very slow response)
    ///   - Typical values: 0.1 to 0.5
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::smoothing::ExponentialSmoother;
    ///
    /// let smoother = ExponentialSmoother::new(0.3);
    /// ```
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            current: 0.0,
            initialized: false,
        }
    }

    /// Smooth a new value
    ///
    /// # Arguments
    /// * `value` - New input value to smooth
    ///
    /// # Returns
    /// Smoothed value
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::smoothing::ExponentialSmoother;
    ///
    /// let mut smoother = ExponentialSmoother::new(0.3);
    /// let result = smoother.smooth(1.0);
    /// ```
    pub fn smooth(&mut self, value: f32) -> f32 {
        if !self.initialized {
            self.current = value;
            self.initialized = true;
            return value;
        }

        // Exponential moving average: y[n] = alpha * y[n-1] + (1 - alpha) * x[n]
        self.current = self.alpha * self.current + (1.0 - self.alpha) * value;
        self.current
    }

    /// Reset the smoother to uninitialized state
    pub fn reset(&mut self) {
        self.current = 0.0;
        self.initialized = false;
    }

    /// Get the current smoothed value without updating
    pub fn current(&self) -> f32 {
        self.current
    }
}

/// Simple moving average smoother
///
/// Maintains a sliding window of recent values and returns their average.
///
/// # Examples
///
/// ```
/// use crabmusic::dsp::smoothing::MovingAverageSmoother;
///
/// let mut smoother = MovingAverageSmoother::new(3);
/// smoother.smooth(1.0);
/// smoother.smooth(2.0);
/// let result = smoother.smooth(3.0);
/// assert!((result - 2.0).abs() < 0.001);
/// ```
#[derive(Debug, Clone)]
pub struct MovingAverageSmoother {
    /// Window size for averaging
    window_size: usize,
    /// Circular buffer of recent values
    buffer: Vec<f32>,
    /// Current write position in buffer
    position: usize,
    /// Number of values written (for initial fill)
    count: usize,
}

impl MovingAverageSmoother {
    /// Create a new moving average smoother
    ///
    /// # Arguments
    /// * `window_size` - Number of values to average
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::smoothing::MovingAverageSmoother;
    ///
    /// let smoother = MovingAverageSmoother::new(5);
    /// ```
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size: window_size.max(1),
            buffer: vec![0.0; window_size.max(1)],
            position: 0,
            count: 0,
        }
    }

    /// Smooth a new value
    ///
    /// # Arguments
    /// * `value` - New input value to smooth
    ///
    /// # Returns
    /// Smoothed value (average of recent values)
    pub fn smooth(&mut self, value: f32) -> f32 {
        self.buffer[self.position] = value;
        self.position = (self.position + 1) % self.window_size;
        self.count = (self.count + 1).min(self.window_size);

        // Calculate average of filled portion
        let sum: f32 = self.buffer.iter().take(self.count).sum();
        sum / self.count as f32
    }

    /// Reset the smoother
    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.position = 0;
        self.count = 0;
    }
}

/// One-pole low-pass filter
///
/// Simple IIR filter for smoothing with configurable cutoff frequency.
///
/// # Examples
///
/// ```
/// use crabmusic::dsp::smoothing::OnePoleFilter;
///
/// let mut filter = OnePoleFilter::new(44100.0, 10.0);
/// let filtered = filter.process(1.0);
/// ```
#[derive(Debug, Clone)]
pub struct OnePoleFilter {
    /// Filter coefficient
    a: f32,
    /// Previous output value
    y_prev: f32,
}

impl OnePoleFilter {
    /// Create a new one-pole low-pass filter
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `cutoff_freq` - Cutoff frequency in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::smoothing::OnePoleFilter;
    ///
    /// let filter = OnePoleFilter::new(44100.0, 10.0);
    /// ```
    pub fn new(sample_rate: f32, cutoff_freq: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / sample_rate;
        let a = dt / (rc + dt);

        Self { a, y_prev: 0.0 }
    }

    /// Process a new sample
    ///
    /// # Arguments
    /// * `x` - Input sample
    ///
    /// # Returns
    /// Filtered output sample
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.a * x + (1.0 - self.a) * self.y_prev;
        self.y_prev = y;
        y
    }

    /// Reset the filter state
    pub fn reset(&mut self) {
        self.y_prev = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_smoother_initialization() {
        let mut smoother = ExponentialSmoother::new(0.5);
        let result = smoother.smooth(1.0);
        assert_eq!(result, 1.0); // First value should pass through
    }

    #[test]
    fn test_exponential_smoother_smoothing() {
        let mut smoother = ExponentialSmoother::new(0.5);
        smoother.smooth(0.0);
        let result = smoother.smooth(1.0);
        assert!(result > 0.0 && result < 1.0); // Should be between old and new
    }

    #[test]
    fn test_exponential_smoother_reset() {
        let mut smoother = ExponentialSmoother::new(0.5);
        smoother.smooth(1.0);
        smoother.reset();
        let result = smoother.smooth(0.5);
        assert_eq!(result, 0.5); // Should act like first value after reset
    }

    #[test]
    fn test_moving_average_smoother() {
        let mut smoother = MovingAverageSmoother::new(3);
        smoother.smooth(1.0);
        smoother.smooth(2.0);
        let result = smoother.smooth(3.0);
        assert!((result - 2.0).abs() < 0.001); // Average of 1, 2, 3
    }

    #[test]
    fn test_moving_average_window_overflow() {
        let mut smoother = MovingAverageSmoother::new(2);
        smoother.smooth(1.0);
        smoother.smooth(2.0);
        let result = smoother.smooth(3.0);
        assert!((result - 2.5).abs() < 0.001); // Average of 2, 3 (1 dropped)
    }

    #[test]
    fn test_one_pole_filter() {
        let mut filter = OnePoleFilter::new(44100.0, 10.0);
        let result1 = filter.process(1.0);
        let result2 = filter.process(1.0);
        assert!(result1 < 1.0); // Should smooth the step
        assert!(result2 > result1); // Should approach 1.0
    }

    #[test]
    fn test_one_pole_filter_reset() {
        let mut filter = OnePoleFilter::new(44100.0, 10.0);
        filter.process(1.0);
        filter.reset();
        let result = filter.process(0.0);
        assert!(result.abs() < 0.1); // Should be close to 0 after reset
    }
}

