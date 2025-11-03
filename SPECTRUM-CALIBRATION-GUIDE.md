# Spectrum Analyzer Calibration Guide

This guide helps you debug and calibrate the spectrum analyzer for optimal music visualization.

## Labeled Spectrum Analyzer

To enable frequency range labels for debugging:

```bash
# Enable labels via command line
cargo run -- --show-labels --loopback

# Or toggle labels during runtime with 'L' key (Spectrum mode only)
```

The labeled mode shows:
- **Band labels**: SUB, BASS, LMID, MID, HMID, PRES, TREB
- **Frequency values**: Actual Hz/kHz values at key points

## Professional Audio Frequency Ranges

### Standard Frequency Bands

| Band | Range | Characteristics | Musical Elements |
|------|-------|-----------------|------------------|
| **Sub-bass** | 20-60 Hz | Felt more than heard | Kick drum fundamental, bass drops |
| **Bass** | 60-250 Hz | Low-end punch | Bass guitar, kick drum body, toms |
| **Low-mid** | 250-500 Hz | Body/warmth | Guitar body, snare body, male vocals |
| **Mid** | 500-2000 Hz | Presence/clarity | Vocals, most instruments |
| **High-mid** | 2000-4000 Hz | Definition/edge | Vocal clarity, guitar attack |
| **Presence** | 4000-6000 Hz | Air/sparkle | Cymbals attack, "s" sounds |
| **Treble** | 6000-20000 Hz | Brilliance/shimmer | Cymbals, high harmonics |

### Current DSP Implementation

Located in `src/dsp/mod.rs:597-599`:
```rust
let bass = self.extract_band(&spectrum, 20.0, 250.0);
let mid = self.extract_band(&spectrum, 250.0, 4000.0);
let treble = self.extract_band(&spectrum, 4000.0, 20000.0);
```

## Common Calibration Issues

### 1. **Visualization Lags Behind Music** ⚠️ CRITICAL FIX

**Problem**: Smoothing causes lag for drums/beats
**Location**: `src/visualization/spectrum.rs:36`
**Current value**: `smoothing_factor: 0.0` (FIXED - was 0.7!)

**Why smoothing is BAD for live music**:
- With 0.7 smoothing: Takes 5-7 frames (~100ms) to reach full height
- Drums hit and are OVER before the bar reaches peak
- Creates "blurry" feeling where beats smear together

**Current fix**: ZERO smoothing for instant response!
```rust
smoothing_factor: 0.0,  // Instant response - feels crispy!
```

**If too jittery**, add minimal smoothing:
```rust
smoothing_factor: 0.1,  // 10% smoothing - barely noticeable
```

**Professional tools** (Voxengo SPAN, REW) use zero smoothing + peak hold for visual persistence

### 2. **Bars Don't Match Music Intensity**

**Problem**: Amplitude sensitivity needs tuning
**Location**: `src/visualization/spectrum.rs:35`
**Current value**: `amplitude_sensitivity: 2.0`

**Fix**:
```rust
amplitude_sensitivity: 3.0,  // Boost for quiet music
amplitude_sensitivity: 1.0,  // Reduce for loud music
```

**Runtime control**:
- Press `+`/`-` to adjust sensitivity
- Press `1`-`9` for presets (0.5x to 4.5x)

### 3. **High Frequencies Dominate**

**Problem**: No perceptual weighting (human hearing is less sensitive at low/high frequencies)

**Recommended improvement**: Add A-weighting filter in DSP
- Human hearing peaks at 2-5kHz
- Bass and treble need boosting for balanced perception

**Implementation location**: `src/dsp/mod.rs:640-666` (extract_band method)

### 4. **FFT Resolution Issues**

**Current settings** (`src/dsp/mod.rs:425`):
- Window size: 2048 (power of 2)
- Sample rate: 44100 Hz
- Frequency resolution: ~21.5 Hz per bin

**Frequency resolution** = sample_rate / window_size

**Trade-offs**:
| Window Size | Resolution | Time Resolution | Best For |
|-------------|------------|-----------------|----------|
| 512 | ~86 Hz | Fast (11ms) | Rhythm, beats |
| 1024 | ~43 Hz | Medium (23ms) | Balanced |
| 2048 | ~21.5 Hz | Good (46ms) | **Current, recommended** |
| 4096 | ~10.7 Hz | Slow (93ms) | Detailed bass |

**To change**: Add `--fft-size 4096` to command line

### 5. **Bars Don't Align with Musical Elements**

**Problem**: Check logarithmic frequency mapping

**Verification**:
1. Enable labels: `cargo run -- --show-labels --loopback`
2. Play music with known frequency (e.g., 440 Hz A note)
3. Verify the correct bar lights up

**Current implementation**: `src/visualization/spectrum.rs:182-194`
Uses logarithmic scaling (matches human hearing)

## Professional Tools for Comparison

Use these to verify your calibration:

### Free Tools
- **REW (Room EQ Wizard)** - Gold standard for audio analysis
  - Download: https://www.roomeqwizard.com/
  - Features: High-precision FFT, waterfall plots, EQ curves

- **Sonic Visualiser** - Research-grade spectrum analyzer
  - Download: https://www.sonicvisualiser.org/
  - Features: Multiple FFT views, spectrograms, annotations

- **Voxengo SPAN** - Free VST plugin
  - Download: https://www.voxengo.com/product/span/
  - Features: Real-time spectrum, accurate metering

### Test Signals

Generate test tones to verify frequency accuracy:

1. **Online tone generator**: https://www.szynalski.com/tone-generator/
2. **Audacity**: Generate → Tone (built-in)

**Recommended tests**:
- 50 Hz → Should light up SUB/BASS bars
- 100 Hz → Should light up BASS bars
- 440 Hz → Should light up MID bars (A note)
- 1000 Hz → Should light up MID bars
- 8000 Hz → Should light up PRES/TREB bars

## Configuration Recommendations

### For Electronic Music (EDM, Hip-Hop)
- Emphasis on bass response
- Fast attack (low smoothing)

```rust
SpectrumConfig {
    smoothing_factor: 0.3,  // Fast response
    amplitude_sensitivity: 2.5,  // Boost for impact
    peak_hold_enabled: true,  // Show transients
    peak_decay_rate: 0.05,  // Medium decay
    ..Default::default()
}
```

### For Classical/Acoustic Music
- Balanced frequency response
- Smoother transitions

```rust
SpectrumConfig {
    smoothing_factor: 0.5,  // Smooth transitions
    amplitude_sensitivity: 1.5,  // Natural dynamics
    peak_hold_enabled: false,  // Less visual noise
    ..Default::default()
}
```

### For Live Performances
- High responsiveness
- Clear visual feedback

```rust
SpectrumConfig {
    smoothing_factor: 0.2,  // Very responsive
    amplitude_sensitivity: 3.0,  // High visibility
    peak_hold_enabled: true,  // Show peaks
    peak_decay_rate: 0.1,  // Fast decay
    ..Default::default()
}
```

## Advanced Calibration Techniques

### 1. Perceptual Weighting (A-weighting)

Add frequency-dependent gain based on human hearing curves:

```rust
// In extract_bar_from_spectrum(), after calculating raw_height:
let perceptual_weight = calculate_a_weight(freq_center);
let weighted_height = raw_height * perceptual_weight;
```

A-weighting formula (simplified):
- Boost: 2-5kHz (+10 dB)
- Reduce: <500 Hz (-20 dB at 50 Hz)
- Reduce: >10kHz (-10 dB at 15 kHz)

### 2. Logarithmic Amplitude Scaling

Current: Linear amplitude → Can make quiet sounds invisible

**Improvement**: Use dB scale
```rust
let db = 20.0 * (amplitude + 1e-10).log10();  // Avoid log(0)
let normalized = (db + 60.0) / 60.0;  // Map -60dB to 0dB → 0.0 to 1.0
```

### 3. Dynamic Range Compression

**Problem**: Quiet passages invisible, loud passages clipped

**Solution**: Auto-gain adjustment
```rust
// Track recent peak values
let recent_peak = recent_values.max();
let auto_gain = 0.8 / recent_peak.max(0.1);  // Target 80% max
let adjusted = raw_height * auto_gain;
```

### 4. Window Function Comparison

Current: Hann window (src/dsp/mod.rs:487-494)

**Alternative windows**:
- **Hamming**: Better frequency resolution, more spectral leakage
- **Blackman**: Less leakage, worse resolution
- **Flat-top**: Best amplitude accuracy, poor resolution

**When to change**: If frequency accuracy is more important than resolution

## Keyboard Shortcuts

During runtime (Spectrum Analyzer mode):
- `L` - Toggle frequency labels ON/OFF
- `+`/`-` - Increase/decrease sensitivity
- `1`-`9` - Sensitivity presets
- `O` - Cycle color schemes
- `V` - Switch visualizer mode
- `M` - Toggle microphone

## Troubleshooting

### "Bars are flickering/jittery"
- Increase `smoothing_factor` (0.5-0.8)
- Increase `peak_decay_rate` (0.05-0.1)

### "Visualization feels delayed"
- Decrease `smoothing_factor` (0.2-0.4)
- Reduce FFT window size (try 1024)

### "Bass isn't showing up"
- Increase amplitude_sensitivity
- Check audio device sample rate (needs 44.1kHz or 48kHz)
- Verify frequency range (should start at 20 Hz)

### "All bars at same height"
- Audio input might be mono/centered
- Check loopback device selection
- Verify music is actually playing with dynamics

### "High frequencies dominate"
- Reduce amplitude_sensitivity
- Consider adding perceptual weighting
- Check for clipping in audio chain

## References

- **Audio Engineering Society**: Frequency band standards
- **ISO 226:2003**: Equal-loudness contours (Fletcher-Munson curves)
- **IEC 61672**: Sound level meters (A-weighting specification)
- **FFT Best Practices**: http://www.dspguide.com/ch8.htm

---

**Quick Start**: `cargo run -- --show-labels --loopback`
**Documentation**: See `README.md` for full usage instructions
