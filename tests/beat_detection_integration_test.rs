// Integration tests for beat detection
// Tests beat detection with synthetic audio patterns that simulate real music

use crabmusic::audio::AudioBuffer;
use crabmusic::dsp::DspProcessor;
use std::f32::consts::PI;

/// Generate a sine wave with given frequency and amplitude
fn generate_sine_wave(
    frequency: f32,
    amplitude: f32,
    sample_rate: u32,
    num_samples: usize,
) -> AudioBuffer {
    let samples: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * PI * frequency * t).sin()
        })
        .collect();
    AudioBuffer::with_samples(samples, sample_rate, 1)
}

#[test]
fn test_beat_detection_with_kick_drum_pattern() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();
    let sample_rate = 44100;
    let window_size = 2048;

    // Simulate kick drum pattern: LOUD-quiet-quiet-quiet-LOUD-quiet-quiet-quiet
    let quiet_samples = vec![0.1; window_size];
    let loud_samples = vec![0.8; window_size];

    let mut beat_count = 0;

    // First kick
    let params = processor.process(&AudioBuffer::with_samples(
        loud_samples.clone(),
        sample_rate,
        1,
    ));
    if params.beat {
        beat_count += 1;
    }

    // Quiet sections
    for _ in 0..3 {
        processor.process(&AudioBuffer::with_samples(
            quiet_samples.clone(),
            sample_rate,
            1,
        ));
    }

    // Second kick
    let params = processor.process(&AudioBuffer::with_samples(
        loud_samples.clone(),
        sample_rate,
        1,
    ));
    if params.beat {
        beat_count += 1;
    }

    assert!(
        beat_count >= 1,
        "Should detect at least one beat in kick pattern"
    );
}

#[test]
fn test_beat_detection_with_sine_wave_pulses() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Generate sine wave pulses (simulated beat pattern)
    // Low amplitude baseline
    let quiet_buffer = generate_sine_wave(440.0, 0.1, 44100, 2048);
    for _ in 0..5 {
        processor.process(&quiet_buffer);
    }

    // High amplitude pulse
    let loud_buffer = generate_sine_wave(440.0, 0.8, 44100, 2048);
    let params = processor.process(&loud_buffer);
    let beat_detected = params.beat;

    assert!(beat_detected, "Should detect beat on amplitude pulse");
}

#[test]
fn test_beat_detection_no_false_positives_in_silence() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Process multiple silent buffers
    let silent_buffer = AudioBuffer::with_samples(vec![0.0; 2048], 44100, 1);
    let mut beats_detected = 0;

    for _ in 0..20 {
        let params = processor.process(&silent_buffer);
        if params.beat {
            beats_detected += 1;
        }
    }

    assert_eq!(beats_detected, 0, "Should not detect any beats in silence");
}

#[test]
fn test_beat_detection_no_false_positives_in_sustained_tone() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Generate sustained tone at constant amplitude
    let sustained = generate_sine_wave(440.0, 0.5, 44100, 2048);
    let mut beats_detected = 0;

    // Process same buffer multiple times (sustained tone)
    for _ in 0..10 {
        let params = processor.process(&sustained);
        if params.beat {
            beats_detected += 1;
        }
    }

    // Should detect at most 1 beat (initial onset), not continuous beats
    assert!(
        beats_detected <= 1,
        "Should not continuously detect beats in sustained tone"
    );
}

#[test]
fn test_beat_detection_respects_cooldown() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Build quiet baseline
    let quiet = AudioBuffer::with_samples(vec![0.1; 2048], 44100, 1);
    for _ in 0..3 {
        processor.process(&quiet);
    }

    // First loud pulse - should trigger beat
    let loud = AudioBuffer::with_samples(vec![0.8; 2048], 44100, 1);
    let params1 = processor.process(&loud);
    assert!(params1.beat, "First pulse should trigger beat");

    // Immediate second loud pulse - should be blocked by cooldown
    let params2 = processor.process(&loud);
    assert!(!params2.beat, "Second pulse should be blocked by cooldown");
}

#[test]
fn test_beat_detection_with_dynamic_range() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Start with quiet music
    let quiet = generate_sine_wave(440.0, 0.1, 44100, 2048);
    for _ in 0..5 {
        processor.process(&quiet);
    }

    // Sudden increase to moderate volume
    let moderate = generate_sine_wave(440.0, 0.3, 44100, 2048);
    let params = processor.process(&moderate);

    // Should detect beat even though absolute amplitude is moderate
    // (dynamic threshold adapts to quiet baseline)
    assert!(
        params.beat,
        "Should detect beat with moderate amplitude after quiet baseline"
    );
}

#[test]
fn test_beat_detection_with_gradual_volume_increase() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Gradually increase volume
    let amplitudes = vec![0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4];
    let mut beats_detected = 0;

    for &amp in &amplitudes {
        let buffer = generate_sine_wave(440.0, amp, 44100, 2048);
        let params = processor.process(&buffer);
        if params.beat {
            beats_detected += 1;
        }
    }

    // Should detect very few beats in gradual increase (not sudden onsets)
    assert!(
        beats_detected < 3,
        "Should not detect many beats in gradual volume increase"
    );
}

#[test]
fn test_beat_detection_bass_transient() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Simulate bass drum: brief loud low-frequency transient
    // Build quiet baseline
    let quiet = generate_sine_wave(100.0, 0.1, 44100, 2048);
    for _ in 0..5 {
        processor.process(&quiet);
    }

    // Bass transient
    let bass_hit = generate_sine_wave(60.0, 0.9, 44100, 2048);
    let params = processor.process(&bass_hit);

    assert!(params.beat, "Should detect beat on bass drum transient");
    assert!(params.bass > 0.3, "Bass frequency band should be active");
}

#[test]
fn test_beat_detection_multiple_instruments() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Simulate mix with bass, mid, and treble content
    let quiet_mix: Vec<f32> = (0..2048)
        .map(|i| {
            let t = i as f32 / 44100.0;
            0.1 * (2.0 * PI * 60.0 * t).sin()   // Bass
                + 0.1 * (2.0 * PI * 440.0 * t).sin()  // Mid
                + 0.1 * (2.0 * PI * 8000.0 * t).sin() // Treble
        })
        .collect();

    let loud_mix: Vec<f32> = (0..2048)
        .map(|i| {
            let t = i as f32 / 44100.0;
            0.8 * (2.0 * PI * 60.0 * t).sin()   // Bass hit
                + 0.3 * (2.0 * PI * 440.0 * t).sin()
                + 0.2 * (2.0 * PI * 8000.0 * t).sin()
        })
        .collect();

    // Build baseline
    for _ in 0..5 {
        processor.process(&AudioBuffer::with_samples(quiet_mix.clone(), 44100, 1));
    }

    // Sudden mix increase
    let params = processor.process(&AudioBuffer::with_samples(loud_mix, 44100, 1));

    assert!(params.beat, "Should detect beat in multi-frequency mix");
}

#[test]
fn test_beat_detection_fast_tempo() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();
    let sample_rate = 44100;
    let window_size = 2048;

    // Simulate fast tempo: 150 BPM = 400ms per beat
    // At ~50ms per buffer, that's ~8 buffers per beat
    let quiet = vec![0.1; window_size];
    let loud = vec![0.7; window_size];

    let mut total_beats = 0;

    // Pattern: loud + 7 quiet, repeated 3 times
    for _ in 0..3 {
        let params = processor.process(&AudioBuffer::with_samples(loud.clone(), sample_rate, 1));
        if params.beat {
            total_beats += 1;
        }

        // Wait through cooldown period
        for _ in 0..7 {
            processor.process(&AudioBuffer::with_samples(quiet.clone(), sample_rate, 1));
        }
    }

    assert!(
        total_beats >= 2,
        "Should detect beats at fast tempo (150 BPM)"
    );
}

#[test]
fn test_beat_detection_slow_tempo() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();
    let sample_rate = 44100;
    let window_size = 2048;

    // Simulate slow tempo: 60 BPM = 1000ms per beat
    let quiet = vec![0.1; window_size];
    let loud = vec![0.7; window_size];

    // Build baseline
    for _ in 0..3 {
        processor.process(&AudioBuffer::with_samples(quiet.clone(), sample_rate, 1));
    }

    // First beat
    let params1 = processor.process(&AudioBuffer::with_samples(loud.clone(), sample_rate, 1));

    // Long wait (simulate 1 second at 60 BPM)
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Reset to quiet
    for _ in 0..3 {
        processor.process(&AudioBuffer::with_samples(quiet.clone(), sample_rate, 1));
    }

    // Second beat
    let params2 = processor.process(&AudioBuffer::with_samples(loud.clone(), sample_rate, 1));

    assert!(
        params1.beat || params2.beat,
        "Should detect beats at slow tempo (60 BPM)"
    );
}
