# CrabMusic Configuration Examples

This directory contains example configuration files for different use cases and music genres.

## Available Configurations

### `config.minimal.yaml`
Minimal configuration with only essential settings. Good starting point for customization.

**Use case:** Simple setup, learning the configuration format

```bash
crabmusic --config examples/config.minimal.yaml
```

### `config.performance.yaml`
Optimized for low CPU usage and battery life.

**Features:**
- Lower FPS (30)
- Larger buffers
- Smaller FFT size
- Simple character set
- High smoothing

**Use case:** Older hardware, laptops on battery, background visualization

```bash
crabmusic --config examples/config.performance.yaml
```

### `config.quality.yaml`
Optimized for best visual quality and responsiveness.

**Features:**
- High FPS (60)
- Larger FFT for better frequency resolution
- Smaller hop size for smoother updates
- Braille character set for high resolution
- Minimal smoothing for fast response

**Use case:** Modern hardware, primary visualization, best experience

```bash
crabmusic --config examples/config.quality.yaml
```

### `config.bass-heavy.yaml`
Optimized for bass-heavy music genres.

**Features:**
- Large FFT for bass resolution
- Lower frequency range focus
- Emphasized amplitude
- Thick wave lines
- Block characters for impact

**Use case:** EDM, hip-hop, dubstep, electronic music

```bash
crabmusic --config examples/config.bass-heavy.yaml
```

### `config.classical.yaml`
Optimized for classical music and acoustic instruments.

**Features:**
- Higher sample rate
- Mid-high frequency focus
- Smooth transitions
- Extended character set
- Gentle amplitude scaling

**Use case:** Classical, jazz, acoustic, orchestral music

```bash
crabmusic --config examples/config.classical.yaml
```

## Creating Your Own Configuration

1. Copy the default configuration:
   ```bash
   cp config.default.yaml config.yaml
   ```

2. Edit `config.yaml` with your preferred settings

3. Run CrabMusic with your configuration:
   ```bash
   crabmusic --config config.yaml
   ```

4. Enable hot-reload to see changes in real-time:
   ```bash
   crabmusic --config config.yaml --hot-reload
   ```

## Configuration Parameters

### Audio Settings

- **sample_rate**: Audio sample rate in Hz (44100, 48000)
- **channels**: Number of channels (1=mono, 2=stereo)
- **buffer_size**: Audio buffer size in samples (affects latency)
- **buffer_capacity**: Ring buffer capacity (should be 4-8x buffer_size)
- **device_name**: Audio device name (null = default)

### DSP Settings

- **fft_size**: FFT window size (512, 1024, 2048, 4096, 8192)
- **hop_size**: Samples between FFT windows
- **window_type**: Window function (hann, hamming, blackman, blackman_harris)
- **frequency_range**: Min/max frequencies to analyze (Hz)
- **smoothing**: Smoothing factor (0.0-1.0)

### Visualization Settings

- **amplitude_scale**: Wave height multiplier (0.1-10.0)
- **frequency_scale**: Wave frequency multiplier (0.1-10.0)
- **phase_offset**: Horizontal wave shift (0.0-6.28 radians)
- **smoothing_factor**: Animation smoothing (0.0-1.0)
- **character_set**: Rendering style (basic, extended, blocks, shading, dots, lines, braille)
- **thickness**: Wave line thickness (1-10)

### Rendering Settings

- **target_fps**: Target frames per second (1-120)
- **enable_differential**: Only update changed cells (recommended: true)
- **enable_double_buffer**: Reduce screen tearing (recommended: true)
- **min_width/min_height**: Minimum terminal dimensions

## Tips

### Performance Tuning

**For better performance:**
- Lower `target_fps` (30 instead of 60)
- Increase `buffer_size` (4096 instead of 2048)
- Decrease `fft_size` (1024 instead of 2048)
- Increase `hop_size` (1024 instead of 512)
- Use simpler character sets (basic, blocks)

**For better quality:**
- Higher `target_fps` (60)
- Decrease `buffer_size` (1024 instead of 2048)
- Increase `fft_size` (4096 instead of 2048)
- Decrease `hop_size` (256 instead of 512)
- Use detailed character sets (extended, braille)

### Music Genre Recommendations

**Electronic/EDM:**
- Use `config.bass-heavy.yaml`
- Character set: blocks or shading
- High amplitude_scale (1.5-2.0)
- Thick lines (2-3)

**Classical/Jazz:**
- Use `config.classical.yaml`
- Character set: extended or lines
- Lower amplitude_scale (0.8-1.0)
- Thin lines (1)

**Rock/Pop:**
- Use default configuration
- Character set: blocks or basic
- Standard amplitude_scale (1.0)
- Medium thickness (1-2)

**Ambient/Chill:**
- High smoothing_factor (0.9)
- Character set: dots or shading
- Lower amplitude_scale (0.7)
- Thin lines (1)

## Troubleshooting

### Visualization too jittery
- Increase `smoothing` in DSP settings
- Increase `smoothing_factor` in visualization settings
- Increase `hop_size`

### Visualization too slow to respond
- Decrease `smoothing` in DSP settings
- Decrease `smoothing_factor` in visualization settings
- Decrease `buffer_size`
- Decrease `hop_size`

### High CPU usage
- Use `config.performance.yaml` as a starting point
- Lower `target_fps`
- Increase `buffer_size` and `hop_size`
- Decrease `fft_size`

### Low frame rate
- Lower `target_fps` to match your hardware
- Disable `enable_double_buffer`
- Use simpler character sets

## See Also

- [Main README](../README.md) - Installation and usage
- [Configuration Guide](../docs/configuration.md) - Detailed configuration documentation
- [Troubleshooting Guide](../docs/troubleshooting.md) - Common issues and solutions

