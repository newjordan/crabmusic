# Audio Capture Guide for CrabMusic

This guide explains how to capture system audio output (music/sounds playing on your computer) rather than microphone input.

## Current Status

**⚠️ Important**: CrabMusic currently captures from the **default input device** (microphone). To visualize system audio output (music, games, etc.), you need platform-specific solutions.

## Platform-Specific Solutions

### Windows (WASAPI)

Windows has **native loopback support** through WASAPI, but it requires special handling that isn't currently implemented in CrabMusic.

**Current Workaround Options:**

1. **Virtual Audio Cable** (Recommended)
   - Install [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) (free)
   - Set Virtual Cable as your default playback device
   - Set Virtual Cable Output as your input device in CrabMusic
   - Use `--device "CABLE Output"` when running CrabMusic

2. **Stereo Mix** (if available)
   - Some audio drivers include a "Stereo Mix" device
   - Enable it in Windows Sound Settings → Recording tab
   - Right-click → Show Disabled Devices
   - Enable "Stereo Mix" if available
   - Use `--device "Stereo Mix"` when running CrabMusic

3. **VoiceMeeter** (Advanced)
   - Install [VoiceMeeter](https://vb-audio.com/Voicemeeter/) (free)
   - Route your audio through VoiceMeeter
   - Use VoiceMeeter's virtual input as capture device

**Future Enhancement:**
We plan to add native WASAPI loopback support in a future update.

### Linux (PulseAudio/PipeWire)

Linux has excellent loopback support through monitor devices.

**PulseAudio:**
```bash
# List all devices including monitors
pactl list sources

# Look for devices ending in ".monitor"
# Example: alsa_output.pci-0000_00_1f.3.analog-stereo.monitor

# Run CrabMusic with monitor device
cargo run --release -- --device "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor"
```

**PipeWire:**
```bash
# List all devices
pw-cli list-objects Node

# Look for monitor devices
# Run CrabMusic with monitor device name
cargo run --release -- --device "monitor_name"
```

### macOS (CoreAudio)

macOS **does not have native loopback support**. You need a virtual audio device.

**Recommended Solutions:**

1. **BlackHole** (Free, Open Source)
   - Install [BlackHole](https://github.com/ExistentialAudio/BlackHole)
   - Create a Multi-Output Device in Audio MIDI Setup
   - Set BlackHole as input device in CrabMusic
   - Use `--device "BlackHole"` when running CrabMusic

2. **Soundflower** (Free, Legacy)
   - Install [Soundflower](https://github.com/mattingalls/Soundflower)
   - Similar setup to BlackHole

## Using CrabMusic with Different Devices

### List Available Devices

```bash
cargo run --release -- --list-devices
```

This will show all available input and output devices. Devices marked `[LOOPBACK]` are suitable for system audio capture.

### Specify a Device

```bash
# Use a specific device by name (partial match works)
cargo run --release -- --device "Stereo Mix"
cargo run --release -- --device "monitor"
cargo run --release -- --device "CABLE Output"
```

### Configure in YAML

Edit your `config.yaml`:

```yaml
audio:
  sample_rate: 44100
  channels: 2
  buffer_capacity: 10
  device_name: "Stereo Mix"  # Or your loopback device name
```

## Troubleshooting

### "No audio device found"

- Run `--list-devices` to see available devices
- Make sure the device is enabled in your system settings
- Check device name spelling (partial match is supported)

### "Permission denied"

**Linux:**
```bash
# Add your user to the audio group
sudo usermod -a -G audio $USER
# Log out and log back in
```

### Visualization not reacting to audio

1. **Check if microphone is muted in CrabMusic**
   - Press 'M' to toggle microphone on/off
   - Make sure it shows `MIC:ON` in the UI

2. **Verify correct device is selected**
   - Run with `--verbose` to see which device is being used
   - Make sure you're using a loopback/monitor device, not a microphone

3. **Check audio levels**
   - Make sure audio is actually playing
   - Increase volume if needed
   - Try adjusting sensitivity: `--sensitivity 2.0`

### Audio is choppy or laggy

- Increase buffer size in config
- Lower FPS: `--fps 30`
- Use performance config: `--config examples/config.performance.yaml`

## Technical Details

### How Audio Capture Works

CrabMusic uses the [CPAL](https://github.com/RustAudio/cpal) library for cross-platform audio capture:

- **Windows**: WASAPI backend
- **Linux**: ALSA/PulseAudio/PipeWire backend
- **macOS**: CoreAudio backend

### Loopback vs. Microphone

- **Microphone**: Captures sound from a physical microphone
- **Loopback/Monitor**: Captures audio output from your system (what you hear)

For music visualization, you typically want **loopback/monitor** devices.

### Why No Native Loopback on Windows?

CPAL doesn't currently expose WASAPI loopback mode directly. This requires:
1. Opening the output device in loopback mode
2. Special WASAPI API calls
3. Platform-specific code

This is planned for a future update.

## Recommended Setup by Use Case

### Music Visualization (Most Common)

**Goal**: Visualize music playing from Spotify, YouTube, etc.

- **Windows**: Use VB-Audio Virtual Cable
- **Linux**: Use monitor device (e.g., `.monitor`)
- **macOS**: Use BlackHole

### Live Performance

**Goal**: Visualize microphone input (singing, instruments)

- Use default microphone device (no special setup needed)
- Adjust sensitivity: `--sensitivity 3.0`

### DJ/Mixing

**Goal**: Visualize mixed audio output

- **Windows**: Use VoiceMeeter for routing
- **Linux**: Use PulseAudio/PipeWire routing
- **macOS**: Use BlackHole with Multi-Output Device

## Future Enhancements

Planned improvements for audio capture:

1. **Native WASAPI Loopback** (Windows)
   - Direct system audio capture without virtual cables
   - Automatic loopback device detection

2. **Automatic Device Detection**
   - Auto-detect loopback/monitor devices
   - Prefer loopback over microphone for visualization

3. **Device Switching**
   - Hot-swap between devices without restarting
   - Press 'D' to cycle through devices

4. **Multiple Input Sources**
   - Mix microphone + system audio
   - Separate visualizations for each source

## Contributing

If you have experience with platform-specific audio APIs and want to help implement native loopback support, please see [CONTRIBUTING.md](../CONTRIBUTING.md).

## References

- [CPAL Documentation](https://docs.rs/cpal/)
- [WASAPI Loopback](https://docs.microsoft.com/en-us/windows/win32/coreaudio/loopback-recording)
- [PulseAudio Monitor Sources](https://www.freedesktop.org/wiki/Software/PulseAudio/Documentation/User/Modules/#module-loopback)
- [BlackHole for macOS](https://github.com/ExistentialAudio/BlackHole)

