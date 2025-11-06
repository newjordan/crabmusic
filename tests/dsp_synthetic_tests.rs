// Comprehensive DSP tests with synthetic audio signals
// Tests FFT accuracy, frequency detection, and parameter extraction

use crabmusic::audio::AudioBuffer;
use crabmusic::dsp::DspProcessor;
use std::f32::consts::PI;

/// Generate a pure sine wave
fn generate_sine_wave(frequency: f32, sample_rate: u32, duration_secs: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * PI * frequency * t).sin()
        })
        .collect()
}

/// Generate a square wave
fn generate_square_wave(frequency: f32, sample_rate: u32, duration_secs: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            if (2.0 * PI * frequency * t).sin() > 0.0 {
                1.0
            } else {
                -1.0
            }
        })
        .collect()
}

/// Generate a sawtooth wave
fn generate_sawtooth_wave(frequency: f32, sample_rate: u32, duration_secs: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let period = sample_rate as f32 / frequency;
    (0..num_samples)
        .map(|i| {
            let phase = (i as f32 % period) / period;
            2.0 * phase - 1.0
        })
        .collect()
}

/// Generate white noise
fn generate_white_noise(sample_rate: u32, duration_secs: f32, seed: u64) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut rng = seed;
    (0..num_samples)
        .map(|_| {
            // Simple LCG random number generator
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let random = ((rng / 65536) % 32768) as f32 / 32768.0;
            2.0 * random - 1.0
        })
        .collect()
}

/// Generate a multi-tone signal (sum of multiple sine waves)
fn generate_multi_tone(
    frequencies: &[f32],
    amplitudes: &[f32],
    sample_rate: u32,
    duration_secs: f32,
) -> Vec<f32> {
    assert_eq!(frequencies.len(), amplitudes.len());
    let num_samples = (sample_rate as f32 * duration_secs) as usize;

    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            frequencies
                .iter()
                .zip(amplitudes.iter())
                .map(|(&freq, &amp)| amp * (2.0 * PI * freq * t).sin())
                .sum()
        })
        .collect()
}

#[test]
fn test_detect_440hz_sine_wave() {
    let sample_rate = 44100;
    let samples = generate_sine_wave(440.0, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 2048).unwrap();
    let spectrum = processor.process_buffer(&buffer);

    // Find peak frequency
    let (peak_bin, _) = spectrum
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();

    let detected_freq = processor.bin_to_frequency(peak_bin);

    // Should detect 440Hz within 20Hz tolerance
    assert!(
        (detected_freq - 440.0).abs() < 20.0,
        "Expected ~440Hz, got {}Hz",
        detected_freq
    );
}

#[test]
fn test_detect_1000hz_sine_wave() {
    let sample_rate = 44100;
    let samples = generate_sine_wave(1000.0, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 2048).unwrap();
    let spectrum = processor.process_buffer(&buffer);

    let (peak_bin, _) = spectrum
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();

    let detected_freq = processor.bin_to_frequency(peak_bin);

    assert!(
        (detected_freq - 1000.0).abs() < 30.0,
        "Expected ~1000Hz, got {}Hz",
        detected_freq
    );
}

#[test]
fn test_multi_tone_detection() {
    let sample_rate = 44100;
    let frequencies = vec![200.0, 500.0, 1500.0];
    let amplitudes = vec![0.5, 0.7, 0.3];
    let samples = generate_multi_tone(&frequencies, &amplitudes, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 4096).unwrap();
    let spectrum = processor.process_buffer(&buffer);

    // Find top 5 peaks (to account for harmonics and nearby bins)
    let mut peaks: Vec<(usize, f32)> = spectrum.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    peaks.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    let detected_freqs: Vec<f32> = peaks
        .iter()
        .take(10)
        .map(|(bin, _)| processor.bin_to_frequency(*bin))
        .collect();

    // Check that we detect frequencies close to the input tones
    // Use wider tolerance due to FFT bin resolution
    for &expected_freq in &frequencies {
        let found = detected_freqs
            .iter()
            .any(|&detected| (detected - expected_freq).abs() < 100.0);
        assert!(
            found,
            "Expected to find {}Hz in detected frequencies: {:?}",
            expected_freq, detected_freqs
        );
    }
}

#[test]
fn test_bass_frequency_extraction() {
    let sample_rate = 44100;
    // Generate bass-heavy signal (60Hz fundamental)
    let samples = generate_sine_wave(60.0, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 2048).unwrap();
    let params = processor.process(&buffer);

    // Bass should be significantly higher than mid and treble
    assert!(params.bass > params.mid);
    assert!(params.bass > params.treble);
    assert!(params.bass > 0.1); // Should have some bass energy
}

#[test]
fn test_treble_frequency_extraction() {
    let sample_rate = 44100;
    // Generate treble-heavy signal (8kHz)
    let samples = generate_sine_wave(8000.0, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 4096).unwrap();
    let params = processor.process(&buffer);

    // Treble should be significantly higher than bass and mid (relative comparison)
    assert!(
        params.treble > params.bass * 2.0,
        "treble: {}, bass: {}",
        params.treble,
        params.bass
    );
    assert!(
        params.treble > params.mid,
        "treble: {}, mid: {}",
        params.treble,
        params.mid
    );
}

#[test]
fn test_mid_frequency_extraction() {
    let sample_rate = 44100;
    // Generate mid-range signal (1kHz)
    let samples = generate_sine_wave(1000.0, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 4096).unwrap();
    let params = processor.process(&buffer);

    // Mid should be significantly higher than bass and treble (relative comparison)
    assert!(
        params.mid > params.bass * 2.0,
        "mid: {}, bass: {}",
        params.mid,
        params.bass
    );
    assert!(
        params.mid > params.treble * 2.0,
        "mid: {}, treble: {}",
        params.mid,
        params.treble
    );
}

#[test]
fn test_amplitude_detection() {
    let sample_rate = 44100;

    // Test with different amplitudes
    let amplitudes = vec![0.1, 0.5, 1.0];
    let mut detected_amps = vec![];

    for &amp in &amplitudes {
        let samples: Vec<f32> = generate_sine_wave(440.0, sample_rate, 0.1)
            .iter()
            .map(|&s| s * amp)
            .collect();
        let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

        let mut processor = DspProcessor::new(sample_rate, 2048).unwrap();
        let params = processor.process(&buffer);

        detected_amps.push(params.amplitude);
    }

    // Detected amplitudes should increase monotonically
    assert!(detected_amps[0] < detected_amps[1]);
    assert!(detected_amps[1] < detected_amps[2]);
}

#[test]
fn test_silence_detection() {
    let sample_rate = 44100;
    let samples = vec![0.0; 4096];
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 2048).unwrap();
    let params = processor.process(&buffer);

    // All parameters should be near zero for silence
    assert!(params.amplitude < 0.01);
    assert!(params.bass < 0.01);
    assert!(params.mid < 0.01);
    assert!(params.treble < 0.01);
}

#[test]
fn test_white_noise_spectrum() {
    let sample_rate = 44100;
    let samples = generate_white_noise(sample_rate, 0.1, 12345);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, 2048).unwrap();
    let params = processor.process(&buffer);

    // White noise should have relatively balanced frequency content
    let max_band = params.bass.max(params.mid).max(params.treble);
    let min_band = params.bass.min(params.mid).min(params.treble);

    // Ratio between max and min band should be less than 3:1
    assert!(max_band / min_band < 3.0);
}

#[test]
fn test_square_wave_harmonics() {
    let sample_rate = 44100;
    let window_size = 4096;
    let samples = generate_square_wave(440.0, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, window_size).unwrap();
    let spectrum = processor.process_buffer(&buffer);

    // Square wave should have strong odd harmonics (440, 1320, 2200, etc.)
    // Convert frequency to bin: bin = freq * window_size / sample_rate
    let fundamental_bin = (440.0 * window_size as f32 / sample_rate as f32) as usize;
    let third_harmonic_bin = (1320.0 * window_size as f32 / sample_rate as f32) as usize;

    // Fundamental should be strong
    assert!(spectrum[fundamental_bin] > 0.3);

    // Third harmonic should be present
    assert!(spectrum[third_harmonic_bin] > 0.1);
}

#[test]
fn test_sawtooth_wave_harmonics() {
    let sample_rate = 44100;
    let window_size = 4096;
    let samples = generate_sawtooth_wave(440.0, sample_rate, 0.1);
    let buffer = AudioBuffer::with_samples(samples, sample_rate, 1);

    let mut processor = DspProcessor::new(sample_rate, window_size).unwrap();
    let spectrum = processor.process_buffer(&buffer);

    // Sawtooth should have both odd and even harmonics
    let fundamental_bin = (440.0 * window_size as f32 / sample_rate as f32) as usize;
    let second_harmonic_bin = (880.0 * window_size as f32 / sample_rate as f32) as usize;

    assert!(spectrum[fundamental_bin] > 0.2);
    assert!(spectrum[second_harmonic_bin] > 0.05);
}
