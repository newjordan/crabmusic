# CrabMusic Testing Guide

## Quick Start

### Running the Application

```bash
# Run with default settings
cargo run --release

# Or run the compiled binary directly
./target/release/crabmusic
```

### Controls

- **`C`** - Cycle through character sets (7 different styles)
- **`Q`** or **`ESC`** - Quit the application

### Character Sets Available

1. **Basic** - Simple ASCII characters (10 levels): ` . : - = + * # % @`
2. **Extended** - Full ASCII art set (65 levels) for high detail
3. **Blocks** - Block characters (5 levels): ` ░ ▒ ▓ █`
4. **Shading** - Smooth gradients (9 levels) with block elements
5. **Dots** - Stippling effect (7 levels): ` . · • ● ◉ ⬤`
6. **Lines** - Box drawing characters (12 levels)
7. **Braille** - High resolution patterns (9 levels)

## Features to Test

### ✅ Audio Input & Output
- **Audio Capture**: The app captures system audio (microphone or loopback)
- **Audio Playback**: You should **hear** the audio being played back through your speakers/headphones
- **Real-time Visualization**: The sine wave should react to the audio in real-time

### ✅ Character Set Switching
- Press **`C`** repeatedly to cycle through all 7 character sets
- The current character set name is displayed at the top of the screen
- Each character set has a different visual style and density

### ✅ Visual Quality
- **Smoothness**: The animation should be smooth at 60 FPS
- **Responsiveness**: The visualization should react immediately to audio changes
- **Character Mapping**: Each character set should show different visual densities

## Testing Scenarios

### 1. Music Playback Test
1. Start playing music on your computer
2. Run CrabMusic: `cargo run --release`
3. You should:
   - **Hear** the music playing through your speakers
   - **See** the sine wave reacting to the music
   - Notice the wave amplitude changes with volume
   - Notice the wave frequency changes with pitch

### 2. Character Set Test
1. While music is playing, press **`C`** multiple times
2. Observe how each character set renders differently:
   - **Basic**: Simple, clean look
   - **Extended**: Very detailed, complex patterns
   - **Blocks**: Solid, bold appearance
   - **Shading**: Smooth gradients
   - **Dots**: Stippled, textured look
   - **Lines**: Geometric, structured
   - **Braille**: High resolution, subtle

### 3. Silence Test
1. Stop all audio playback
2. The visualization should show a flat line or minimal movement
3. Audio output should be silent

### 4. Dynamic Range Test
1. Play quiet music → should show small wave amplitude
2. Play loud music → should show large wave amplitude
3. The visualization should scale appropriately

## Troubleshooting

### No Audio Capture
- **Windows**: Make sure "Stereo Mix" or similar loopback device is enabled
- **Linux**: Ensure PulseAudio or PipeWire is running
- **macOS**: May need to use a virtual audio device like BlackHole

### No Audio Output
- Check your default audio output device
- Make sure volume is not muted
- Try restarting the application

### Visualization Not Responding
- Make sure audio is actually playing
- Check that the audio device is being captured correctly
- Look at the debug logs (run with `-v` for verbose mode)

### Performance Issues
- The app targets 60 FPS
- If you see stuttering, check CPU usage
- Try reducing the sensitivity: `cargo run --release -- --sensitivity 5.0`

## Command Line Options

```bash
# Show help
cargo run --release -- --help

# Verbose logging
cargo run --release -- -v

# Custom sensitivity
cargo run --release -- --sensitivity 15.0

# Custom FPS
cargo run --release -- --fps 30

# Test mode (no audio, just patterns)
cargo run --release -- --test
```

## Expected Behavior

### ✅ Working Correctly
- Audio plays through speakers/headphones
- Visualization reacts to audio in real-time
- Character sets change when pressing 'C'
- Smooth 60 FPS animation
- Clean exit with 'Q' or ESC

### ❌ Issues to Report
- No audio playback
- Visualization frozen or not responding
- Crashes or errors
- Stuttering or low FPS
- Character sets not changing

## Next Steps

After testing, we'll continue with:
- CONFIG-002: YAML configuration loading
- CONFIG-003: Hot-reload file watching
- More visualizer types (spectrum analyzer, oscilloscope)
- Color support
- Interactive controls

Enjoy testing! 🦀🎵

