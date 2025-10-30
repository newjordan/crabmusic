// FFT performance benchmarks

use crabmusic::audio::AudioBuffer;
use crabmusic::dsp::DspProcessor;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Helper function to generate synthetic sine wave for benchmarking
fn generate_sine_wave(
    freq: f32,
    amplitude: f32,
    sample_rate: u32,
    num_samples: usize,
) -> AudioBuffer {
    let samples: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * std::f32::consts::PI * freq * t).sin()
        })
        .collect();

    AudioBuffer::with_samples(samples, sample_rate, 1)
}

fn fft_benchmark(c: &mut Criterion) {
    let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);
    let mut processor = DspProcessor::new(44100, 2048).expect("Failed to create processor");

    c.bench_function("fft_process_buffer_2048", |b| {
        b.iter(|| processor.process_buffer(black_box(&buffer)));
    });
}

fn fft_benchmark_1024(c: &mut Criterion) {
    let buffer = generate_sine_wave(440.0, 1.0, 44100, 1024);
    let mut processor = DspProcessor::new(44100, 1024).expect("Failed to create processor");

    c.bench_function("fft_process_buffer_1024", |b| {
        b.iter(|| processor.process_buffer(black_box(&buffer)));
    });
}

fn fft_benchmark_4096(c: &mut Criterion) {
    let buffer = generate_sine_wave(440.0, 1.0, 44100, 4096);
    let mut processor = DspProcessor::new(44100, 4096).expect("Failed to create processor");

    c.bench_function("fft_process_buffer_4096", |b| {
        b.iter(|| processor.process_buffer(black_box(&buffer)));
    });
}

criterion_group!(
    benches,
    fft_benchmark,
    fft_benchmark_1024,
    fft_benchmark_4096
);
criterion_main!(benches);
