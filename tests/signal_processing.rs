//! Signal-processing tests driven by synthetic tones and WAV round-trips.
//!
//! These cover the path used by file input: generate a tone, encode it as WAV,
//! decode it, and check the FFT, band split, amplitude, windows, and smoothing.

use crabmusic::audio::{encode_wav_pcm16, parse_wav, AudioBuffer, WavData};
use crabmusic::dsp::smoothing::ExponentialSmoother;
use crabmusic::dsp::windowing::{generate_window, WindowType};
use crabmusic::dsp::DspProcessor;
use crabmusic::preview::synthetic_test_tone;
use std::f32::consts::PI;

fn sine(frequency: f32, sample_rate: u32, seconds: f32) -> Vec<f32> {
    let count = (sample_rate as f32 * seconds) as usize;
    (0..count)
        .map(|i| (2.0 * PI * frequency * i as f32 / sample_rate as f32).sin())
        .collect()
}

fn peak_frequency(samples: &[f32], sample_rate: u32) -> f32 {
    let mut processor = DspProcessor::new(sample_rate, 4096).unwrap();
    let buffer = AudioBuffer::with_samples(samples.to_vec(), sample_rate, 1);
    let spectrum = processor.process_buffer(&buffer);
    let (bin, _) = spectrum
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    processor.bin_to_frequency(bin)
}

#[test]
fn wav_round_trip_keeps_the_detected_pitch() {
    let sample_rate = 44_100;
    let original = sine(440.0, sample_rate, 0.2);
    let encoded = encode_wav_pcm16(&WavData {
        sample_rate,
        channels: 1,
        samples: original,
    });
    let decoded = parse_wav(&encoded).unwrap();
    let detected = peak_frequency(&decoded.samples, sample_rate);
    assert!(
        (detected - 440.0).abs() < 20.0,
        "expected ~440 Hz, got {detected}"
    );
}

#[test]
fn synthetic_tone_puts_energy_in_bass_and_mid() {
    let sample_rate = 44_100;
    let tone = synthetic_test_tone(sample_rate, 0.25);
    let mut processor = DspProcessor::new(sample_rate, 4096).unwrap();
    let params = processor.process(&AudioBuffer::with_samples(tone.clone(), sample_rate, 1));

    assert!(params.amplitude > 0.05, "amplitude {}", params.amplitude);
    assert!(params.bass > 0.05, "bass {}", params.bass);
    // Band values are averages across every bin in the range, so a wide mid
    // band looks smaller than the narrow bass peak even when both tones are present.
    assert!(
        params.bass > params.mid,
        "bass {} should exceed averaged mid {}",
        params.bass,
        params.mid
    );
    assert!(
        params.mid > params.treble,
        "mid {} should exceed treble {}",
        params.mid,
        params.treble
    );
    let fundamental = peak_frequency(&tone, sample_rate);
    assert!(
        (fundamental - 82.41).abs() < 15.0,
        "expected the 82 Hz bass tone to dominate, got {fundamental}"
    );
    assert!(params.spectrum.iter().any(|bin| *bin > 0.5));
    assert_eq!(params.waveform.len(), 512);
}

#[test]
fn louder_tone_has_higher_rms_than_a_quiet_one() {
    let sample_rate = 44_100;
    let quiet: Vec<f32> = sine(440.0, sample_rate, 0.1)
        .into_iter()
        .map(|s| s * 0.1)
        .collect();
    let loud = sine(440.0, sample_rate, 0.1);
    let mut quiet_dsp = DspProcessor::new(sample_rate, 2048).unwrap();
    let mut loud_dsp = DspProcessor::new(sample_rate, 2048).unwrap();
    let quiet_amp = quiet_dsp
        .process(&AudioBuffer::with_samples(quiet, sample_rate, 1))
        .amplitude;
    let loud_amp = loud_dsp
        .process(&AudioBuffer::with_samples(loud, sample_rate, 1))
        .amplitude;
    assert!(loud_amp > quiet_amp * 2.0, "{loud_amp} vs {quiet_amp}");
}

#[test]
fn silence_then_onset_reports_a_beat() {
    let sample_rate = 44_100;
    let mut processor = DspProcessor::new(sample_rate, 2048).unwrap();
    let silence = AudioBuffer::with_samples(vec![0.0; 2048], sample_rate, 1);
    for _ in 0..5 {
        let params = processor.process(&silence);
        assert!(!params.beat);
        assert!(params.amplitude < 0.01);
    }

    let onset = AudioBuffer::with_samples(
        sine(82.0, sample_rate, 2048.0 / sample_rate as f32),
        sample_rate,
        1,
    );
    let params = processor.process(&onset);
    assert!(params.beat, "expected an onset after silence");
    assert!(params.bass > params.treble);
}

#[test]
fn hann_window_is_zero_at_the_edges_and_unity_in_the_middle() {
    let window = generate_window(WindowType::Hann, 1024);
    assert_eq!(window.len(), 1024);
    assert!(window[0].abs() < 1e-5);
    assert!(window[window.len() - 1].abs() < 1e-5);
    let peak = window.iter().copied().fold(0.0f32, f32::max);
    assert!((peak - 1.0).abs() < 0.01, "peak {peak}");
    assert!(window.iter().all(|v| (0.0..=1.0).contains(v)));
}

#[test]
fn rectangular_window_is_flat() {
    let window = generate_window(WindowType::Rectangular, 16);
    assert!(window.iter().all(|v| *v == 1.0));
}

#[test]
fn exponential_smoother_moves_toward_the_new_value() {
    let mut smoother = ExponentialSmoother::new(0.3);
    let first = smoother.smooth(1.0);
    let second = smoother.smooth(0.5);
    assert!((first - 1.0).abs() < 1e-5);
    assert!(second > 0.5 && second < 1.0);
}
