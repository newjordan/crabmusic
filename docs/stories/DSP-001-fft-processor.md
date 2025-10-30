# [DSP-001] FFT Processor Implementation

**Epic**: DSP Processing
**Priority**: P0 (Blocking)
**Estimated Effort**: 1.5-2 days
**Status**: Not Started

---

## Description

Implement the core DSP processor that takes audio buffers and performs Fast Fourier Transform (FFT) analysis to extract frequency spectrum data. This is the foundation for all audio parameter extraction.

**Agent Instructions**: Create a DspProcessor that:
- Accepts AudioBuffer from ring buffer
- Applies windowing function (Hann window)
- Performs FFT using rustfft
- Outputs frequency spectrum data
- Is optimized for real-time performance

---

## Acceptance Criteria

- [ ] DspProcessor struct created with FFT planner
- [ ] process_buffer() method accepts AudioBuffer and returns spectrum data
- [ ] Hann windowing applied to reduce spectral leakage
- [ ] FFT plan pre-computed during initialization (not per-frame)
- [ ] Window size configurable (default 2048 samples)
- [ ] Frequency bins correctly mapped to Hz values
- [ ] Spectrum normalized to 0.0-1.0 range
- [ ] Performance: process_buffer() completes in <5ms on target hardware
- [ ] Unit tests with synthetic sine waves validate correct frequency detection
- [ ] Benchmarks created with criterion

---

## Technical Approach

### DspProcessor Structure

Reference: **docs/architecture.md - DSP Processing Component**

```rust
use rustfft::{FftPlanner, num_complex::Complex};

pub struct DspProcessor {
    fft_planner: FftPlanner<f32>,
    window_size: usize,
    sample_rate: u32,
    hann_window: Vec<f32>,
    scratch_buffer: Vec<Complex<f32>>, // Pre-allocated
}

impl DspProcessor {
    pub fn new(sample_rate: u32, window_size: usize) -> Self {
        assert!(window_size.is_power_of_two(), "Window size must be power of 2");

        let mut planner = FftPlanner::new();
        let hann_window = Self::generate_hann_window(window_size);

        Self {
            fft_planner: planner,
            window_size,
            sample_rate,
            hann_window,
            scratch_buffer: vec![Complex::new(0.0, 0.0); window_size],
        }
    }

    pub fn process_buffer(&mut self, buffer: &AudioBuffer) -> Vec<f32> {
        // Apply Hann window
        // Perform FFT
        // Convert to magnitude spectrum
        // Normalize
    }
}
```

### Hann Window Generation

```rust
fn generate_hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| {
            let factor = 2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32;
            0.5 * (1.0 - factor.cos())
        })
        .collect()
}
```

### FFT Processing

```rust
pub fn process_buffer(&mut self, buffer: &AudioBuffer) -> Vec<f32> {
    // 1. Apply Hann window to reduce spectral leakage
    for (i, &sample) in buffer.samples.iter().enumerate() {
        self.scratch_buffer[i] = Complex::new(sample * self.hann_window[i], 0.0);
    }

    // 2. Perform FFT
    let fft = self.fft_planner.plan_fft_forward(self.window_size);
    fft.process(&mut self.scratch_buffer);

    // 3. Convert to magnitude spectrum (only first half, FFT is symmetric)
    let spectrum: Vec<f32> = self.scratch_buffer[..self.window_size / 2]
        .iter()
        .map(|c| (c.re * c.re + c.im * c.im).sqrt())
        .collect();

    // 4. Normalize to 0.0-1.0 range
    let max_magnitude = spectrum.iter().cloned().fold(0.0f32, f32::max);
    if max_magnitude > 0.0 {
        spectrum.iter().map(|&m| m / max_magnitude).collect()
    } else {
        spectrum
    }
}
```

### Frequency Bin Mapping

Helper function to map bin index to frequency:
```rust
pub fn bin_to_frequency(&self, bin: usize) -> f32 {
    bin as f32 * self.sample_rate as f32 / self.window_size as f32
}
```

---

## Dependencies

- **Depends on**:
  - FOUND-001 (project exists)
  - AUDIO-003 (AudioBuffer definition)
- **Blocks**: DSP-002 (frequency band extraction needs spectrum)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "DSP Processing Component"
- **Tech Stack**: docs/architecture.md - rustfft 6.1+ entry
- **Performance**: docs/architecture.md - "Performance Optimization"

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_detects_440hz_sine_wave() {
        // Generate pure 440 Hz tone
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048);

        let spectrum = processor.process_buffer(&buffer);

        // Find peak frequency
        let peak_bin = spectrum.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        let peak_freq = processor.bin_to_frequency(peak_bin);

        // Should be within 1 Hz of 440
        assert!((peak_freq - 440.0).abs() < 1.0);
    }

    #[test]
    fn test_hann_window_symmetry() {
        let window = DspProcessor::generate_hann_window(1024);
        assert_eq!(window[0], window[1023]); // Symmetric
        assert!(window[512] > window[0]); // Peak in middle
    }

    #[test]
    fn test_zero_input_produces_zero_spectrum() {
        let buffer = AudioBuffer::new(vec![0.0; 2048], 44100, 1);
        let mut processor = DspProcessor::new(44100, 2048);

        let spectrum = processor.process_buffer(&buffer);

        assert!(spectrum.iter().all(|&s| s == 0.0));
    }
}
```

### Benchmarks

Create `benches/fft_benchmark.rs`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fft_benchmark(c: &mut Criterion) {
    let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);
    let mut processor = DspProcessor::new(44100, 2048);

    c.bench_function("fft_process_buffer", |b| {
        b.iter(|| {
            processor.process_buffer(black_box(&buffer))
        });
    });
}

criterion_group!(benches, fft_benchmark);
criterion_main!(benches);
```

**Performance Target**: <5ms for 2048-sample FFT on modern hardware

---

## Notes for AI Agent

**FFT Window Size Tradeoffs**:
- Larger window = better frequency resolution, worse time resolution
- Smaller window = better time resolution, worse frequency resolution
- 2048 samples @ 44.1kHz = ~46ms of audio, ~21 Hz frequency resolution
- This is good balance for music visualization

**Pre-computation Critical**:
- FFT planner MUST be created once during init
- Hann window MUST be pre-computed
- Scratch buffer MUST be pre-allocated
- **No allocations during process_buffer()** for real-time performance

**Spectrum Normalization**:
- Normalize per-frame so visualization scales automatically
- Alternative: Use fixed normalization based on expected max (less dynamic but more consistent)

**Testing with Synthetic Audio**:
Helper function for tests:
```rust
fn generate_sine_wave(freq: f32, amplitude: f32, sample_rate: u32, num_samples: usize) -> AudioBuffer {
    let samples: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * std::f32::consts::PI * freq * t).sin()
        })
        .collect();

    AudioBuffer::new(samples, sample_rate, 1)
}
```

**Success Indicator**: Benchmark shows <5ms, unit tests pass with 440 Hz detection accurate to <1 Hz
