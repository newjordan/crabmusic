# Windows Audio Setup Guide

This guide shows you how to capture system audio (music, games, etc.) on Windows **without installing any additional software**.

## Option 1: Enable Stereo Mix (Built-in, No Extra Software)

Windows has a built-in feature called **Stereo Mix** that captures system audio. It's free and already on your computer.

### Step 1: Enable Stereo Mix

1. **Right-click the speaker icon** in your system tray (bottom-right corner)
2. Click **"Sounds"** or **"Sound settings"**
3. Click **"Recording"** tab (or "More sound settings" → "Recording")
4. **Right-click in the empty space** and check:
   - ☑ **Show Disabled Devices**
   - ☑ **Show Disconnected Devices**
5. You should now see **"Stereo Mix"** in the list
6. **Right-click "Stereo Mix"** → **"Enable"**
7. **Right-click "Stereo Mix"** → **"Set as Default Device"** (optional)
8. Click **"OK"**

### Step 2: Run CrabMusic

```bash
# List devices to confirm Stereo Mix is available
cargo run --release -- --list-devices

# Run with Stereo Mix
cargo run --release -- --device "Stereo Mix"
```

### Step 3: Play Music and Visualize!

- Open Spotify, YouTube, or any music player
- Play some music
- Watch CrabMusic visualize it in real-time! 🎵

### Troubleshooting Stereo Mix

**"I don't see Stereo Mix"**
- Some audio drivers don't include Stereo Mix (especially Realtek HD Audio on newer systems)
- Try updating your audio drivers from your motherboard/laptop manufacturer
- If still not available, see Option 2 below

**"Stereo Mix is there but not working"**
- Make sure it's **enabled** (not grayed out)
- Make sure it's **not muted**
- Try setting it as the **default recording device**
- Restart your computer after enabling it

**"Audio quality is poor"**
- Right-click Stereo Mix → Properties → Advanced
- Set format to **"2 channel, 16 bit, 48000 Hz (DVD Quality)"** or higher
- Click Apply

---

## Option 2: Use VoiceMeeter (Free, More Features)

If Stereo Mix isn't available or doesn't work well, **VoiceMeeter** is a free virtual audio mixer that's more reliable.

### Why VoiceMeeter?

- ✅ Free and lightweight
- ✅ Works on all Windows systems
- ✅ Better audio quality than Stereo Mix
- ✅ More control over audio routing
- ✅ Can mix multiple sources

### Setup VoiceMeeter

1. **Download VoiceMeeter** (free): https://vb-audio.com/Voicemeeter/
2. **Install and restart** your computer
3. **Open VoiceMeeter**
4. **Configure audio routing:**
   - Set your speakers as **Hardware Out A1**
   - In Windows Sound Settings, set **VoiceMeeter Input** as default playback device
   - In VoiceMeeter, enable **A1** button for Hardware Input 1

5. **Run CrabMusic:**
```bash
cargo run --release -- --device "VoiceMeeter Output"
```

---

## Option 3: VB-Audio Virtual Cable (What You Tried)

This is what you tried before. It works but requires more setup:

1. Download VB-Audio Virtual Cable: https://vb-audio.com/Cable/
2. Install and restart
3. Set CABLE Input as default playback device in Windows
4. Run: `cargo run --release -- --device "CABLE Output"`

**Downside:** You won't hear audio unless you also set up playback routing.

---

## Comparison

| Method | Pros | Cons | Recommended? |
|--------|------|------|--------------|
| **Stereo Mix** | Built-in, no install, simple | Not available on all systems, lower quality | ✅ **Try this first** |
| **VoiceMeeter** | Free, reliable, good quality | Requires install, more complex | ✅ **Best if Stereo Mix unavailable** |
| **VB-Audio Cable** | Works everywhere | Requires install, complex routing, no audio playback | ❌ Not recommended |

---

## Quick Start (TL;DR)

### If you have Stereo Mix:
```bash
# 1. Enable Stereo Mix in Windows Sound Settings
# 2. Run CrabMusic
cargo run --release -- --device "Stereo Mix"
```

### If you don't have Stereo Mix:
```bash
# 1. Install VoiceMeeter (free): https://vb-audio.com/Voicemeeter/
# 2. Configure VoiceMeeter (see above)
# 3. Run CrabMusic
cargo run --release -- --device "VoiceMeeter Output"
```

---

## Testing Your Setup

```bash
# 1. List all available devices
cargo run --release -- --list-devices

# 2. Look for one of these:
#    - "Stereo Mix"
#    - "VoiceMeeter Output"
#    - "CABLE Output"

# 3. Run with your device
cargo run --release -- --device "YOUR_DEVICE_NAME"

# 4. Play some music and watch it visualize!
```

---

## Future: Native WASAPI Loopback

We're planning to add **native Windows WASAPI loopback support** in a future release. This will allow direct system audio capture without any setup or external software.

**Planned features:**
- `--loopback` flag for automatic system audio capture
- No Stereo Mix or VoiceMeeter needed
- Works on all Windows systems
- Better performance and quality

**Status:** Planned for v0.2.0

---

## Need Help?

If you're having trouble with audio setup:

1. Run `cargo run --release -- --list-devices` and share the output
2. Check which audio driver you have (Realtek, Conexant, etc.)
3. Try the troubleshooting steps above
4. Open an issue on GitHub with your system details

---

## Summary

**Easiest solution:** Enable Stereo Mix (if available)
**Most reliable:** Install VoiceMeeter (free)
**Future:** Native WASAPI loopback (coming soon)

Choose the option that works best for your system! 🦀🎵

