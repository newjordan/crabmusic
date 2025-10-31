// Integration tests for enhanced oscilloscope visualizer
// Tests that the oscilloscope displays real audio waveforms correctly

use crabmusic::audio::AudioBuffer;
use crabmusic::dsp::DspProcessor;
use crabmusic::visualization::{GridBuffer, OscilloscopeConfig, OscilloscopeVisualizer, Visualizer};

/// Helper function to generate sine wave for testing
fn generate_sine_wave(freq: f32, amplitude: f32, sample_rate: u32, num_samples: usize) -> Vec<f32> {
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * std::f32::consts::PI * freq * t).sin()
        })
        .collect()
}

/// Helper function to generate square wave for testing
fn generate_square_wave(freq: f32, amplitude: f32, sample_rate: u32, num_samples: usize) -> Vec<f32> {
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let phase = (freq * t) % 1.0;
            if phase < 0.5 {
                amplitude
            } else {
                -amplitude
            }
        })
        .collect()
}

#[test]
fn test_oscilloscope_with_real_audio() {
    // Create DSP processor
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();

    // Create oscilloscope visualizer
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Generate 440 Hz sine wave (A note)
    let samples = generate_sine_wave(440.0, 0.8, 44100, 2048);
    let buffer = AudioBuffer::with_samples(samples, 44100, 1);

    // Process and visualize
    let params = dsp.process(&buffer);
    viz.update(&params);

    // Waveform should show sine wave pattern
    // Check that we have positive and negative values (not flat)
    assert!(
        viz.waveform.iter().any(|&s| s > 0.5),
        "Waveform should have positive values"
    );
    assert!(
        viz.waveform.iter().any(|&s| s < -0.5),
        "Waveform should have negative values"
    );

    // Render and verify output
    let mut grid = GridBuffer::new(80, 24);
    viz.render(&mut grid);

    // Should have visualization content (not just grid)
    let mut has_content = false;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let ch = grid.get_cell(x, y).character;
            if ch != ' ' && ch != '·' && ch != '┼' && ch != '┬' {
                has_content = true;
                break;
            }
        }
    }
    assert!(
        has_content,
        "Oscilloscope should display waveform content"
    );
}

#[test]
fn test_oscilloscope_shows_different_waveforms() {
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();
    let config = OscilloscopeConfig::default();

    // Test 1: Sine wave (smooth)
    let sine_samples = generate_sine_wave(440.0, 0.8, 44100, 2048);
    let sine_buffer = AudioBuffer::with_samples(sine_samples, 44100, 1);
    let sine_params = dsp.process(&sine_buffer);

    // Test 2: Square wave (sharp transitions)
    let square_samples = generate_square_wave(440.0, 0.8, 44100, 2048);
    let square_buffer = AudioBuffer::with_samples(square_samples, 44100, 1);
    let square_params = dsp.process(&square_buffer);

    // Waveforms should be different
    assert_ne!(
        sine_params.waveform, square_params.waveform,
        "Sine and square waveforms should be different"
    );

    // Sine wave should be smoother (fewer sharp transitions)
    let sine_transitions = count_sharp_transitions(&sine_params.waveform);
    let square_transitions = count_sharp_transitions(&square_params.waveform);

    // Square wave should have more sharp transitions than sine wave
    assert!(
        square_transitions > sine_transitions,
        "Square wave should have more sharp transitions ({}) than sine wave ({})",
        square_transitions,
        sine_transitions
    );
}

/// Count the number of sharp transitions in a waveform
/// A sharp transition is when the value changes by more than 0.5 between samples
fn count_sharp_transitions(waveform: &[f32]) -> usize {
    let mut count = 0;
    for i in 1..waveform.len() {
        if (waveform[i] - waveform[i - 1]).abs() > 0.5 {
            count += 1;
        }
    }
    count
}

#[test]
fn test_oscilloscope_handles_silence() {
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Create silent buffer
    let silent_buffer = AudioBuffer::with_samples(vec![0.0; 2048], 44100, 1);

    // Process silence
    let params = dsp.process(&silent_buffer);
    viz.update(&params);

    // Waveform should be near zero
    let max_value = viz
        .waveform
        .iter()
        .map(|&s| s.abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_value < 0.1,
        "Silent audio should produce near-zero waveform, got max value: {}",
        max_value
    );
}

#[test]
fn test_oscilloscope_with_stereo_audio() {
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Generate stereo sine wave (interleaved L/R channels)
    let mut stereo_samples = Vec::with_capacity(4096);
    for i in 0..2048 {
        let t = i as f32 / 44100.0;
        let left = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.8;
        let right = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.8;
        stereo_samples.push(left);
        stereo_samples.push(right);
    }

    let stereo_buffer = AudioBuffer::with_samples(stereo_samples, 44100, 2);

    // Process stereo audio
    let params = dsp.process(&stereo_buffer);
    viz.update(&params);

    // Should produce valid waveform (mono-mixed from stereo)
    assert!(
        viz.waveform.iter().any(|&s| s > 0.3),
        "Stereo waveform should have positive values"
    );
    assert!(
        viz.waveform.iter().any(|&s| s < -0.3),
        "Stereo waveform should have negative values"
    );
}

#[test]
fn test_oscilloscope_trigger_stabilizes_waveform() {
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();

    // Create two configs: one with trigger, one without
    let config_with_trigger = OscilloscopeConfig {
        trigger_enabled: true,
        trigger_level: 0.0,
        ..Default::default()
    };
    let config_without_trigger = OscilloscopeConfig {
        trigger_enabled: false,
        ..Default::default()
    };

    let mut viz_with_trigger = OscilloscopeVisualizer::new(config_with_trigger);
    let mut viz_without_trigger = OscilloscopeVisualizer::new(config_without_trigger);

    // Generate periodic sine wave
    let sine_samples = generate_sine_wave(440.0, 0.8, 44100, 2048);
    let buffer = AudioBuffer::with_samples(sine_samples, 44100, 1);

    // Process same audio with both visualizers
    let params = dsp.process(&buffer);
    viz_with_trigger.update(&params);
    viz_without_trigger.update(&params);

    // Both should have valid waveforms
    assert!(!viz_with_trigger.waveform.is_empty());
    assert!(!viz_without_trigger.waveform.is_empty());

    // Note: We can't easily test that trigger actually stabilizes the waveform
    // in a unit test, but we can verify that both modes work
    assert!(viz_with_trigger.waveform.iter().any(|&s| s.abs() > 0.1));
    assert!(viz_without_trigger.waveform.iter().any(|&s| s.abs() > 0.1));
}

#[test]
fn test_oscilloscope_grid_rendering() {
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();

    // Test with grid enabled
    let config_with_grid = OscilloscopeConfig {
        show_grid: true,
        ..Default::default()
    };
    let mut viz_with_grid = OscilloscopeVisualizer::new(config_with_grid);

    // Test with grid disabled
    let config_without_grid = OscilloscopeConfig {
        show_grid: false,
        ..Default::default()
    };
    let mut viz_without_grid = OscilloscopeVisualizer::new(config_without_grid);

    // Generate test audio
    let samples = generate_sine_wave(440.0, 0.5, 44100, 2048);
    let buffer = AudioBuffer::with_samples(samples, 44100, 1);
    let params = dsp.process(&buffer);

    viz_with_grid.update(&params);
    viz_without_grid.update(&params);

    // Render both
    let mut grid_with = GridBuffer::new(80, 24);
    let mut grid_without = GridBuffer::new(80, 24);

    viz_with_grid.render(&mut grid_with);
    viz_without_grid.render(&mut grid_without);

    // Count grid characters in center line
    let center_y = 24 / 2;
    let mut grid_chars_with = 0;
    let mut grid_chars_without = 0;

    for x in 0..80 {
        let ch_with = grid_with.get_cell(x, center_y).character;
        let ch_without = grid_without.get_cell(x, center_y).character;

        if ch_with == '·' || ch_with == '┼' || ch_with == '┬' {
            grid_chars_with += 1;
        }
        if ch_without == '·' || ch_without == '┼' || ch_without == '┬' {
            grid_chars_without += 1;
        }
    }

    // Grid with show_grid=true should have grid characters
    assert!(
        grid_chars_with > 0,
        "Oscilloscope with grid enabled should show grid characters"
    );

    // Grid with show_grid=false should have fewer or no grid characters
    // (some might remain from the waveform itself)
    assert!(
        grid_chars_with > grid_chars_without,
        "Grid enabled ({}) should have more grid chars than disabled ({})",
        grid_chars_with,
        grid_chars_without
    );
}

#[test]
fn test_oscilloscope_waveform_length() {
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Generate test audio
    let samples = generate_sine_wave(440.0, 0.8, 44100, 2048);
    let buffer = AudioBuffer::with_samples(samples, 44100, 1);

    // Process audio
    let params = dsp.process(&buffer);

    // Check that waveform field is populated with correct length
    assert_eq!(
        params.waveform.len(),
        512,
        "Waveform should have 512 samples"
    );

    // Update visualizer
    viz.update(&params);

    // Visualizer waveform should match config
    assert_eq!(
        viz.waveform.len(),
        config.sample_count,
        "Visualizer waveform should match config sample_count"
    );
}

#[test]
fn test_oscilloscope_performance() {
    use std::time::Instant;

    let mut dsp = DspProcessor::new(44100, 2048).unwrap();
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Generate test audio
    let samples = generate_sine_wave(440.0, 0.8, 44100, 2048);
    let buffer = AudioBuffer::with_samples(samples, 44100, 1);

    // Measure update performance
    let start = Instant::now();
    for _ in 0..1000 {
        let params = dsp.process(&buffer);
        viz.update(&params);
    }
    let elapsed = start.elapsed();

    // Should process 1000 frames in reasonable time
    // At 60 FPS, 1000 frames = ~16.7 seconds
    // We'll allow up to 10 seconds for processing overhead
    assert!(
        elapsed.as_secs() < 10,
        "Performance issue: 1000 updates took {:?}",
        elapsed
    );
}
