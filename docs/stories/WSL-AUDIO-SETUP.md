# WSL Audio Setup Guide

You're running CrabMusic in WSL (Windows Subsystem for Linux), which doesn't have native audio access. Here's how to fix it:

## Option 1: PulseAudio Server (Recommended for Testing)

### Step 1: Install PulseAudio on Windows

1. Download **PulseAudio for Windows**: https://www.freedesktop.org/wiki/Software/PulseAudio/Ports/Windows/Support/
   - Or use this direct link: https://github.com/pgaskin/pulseaudio-win32/releases

2. Extract to `C:\PulseAudio`

3. Edit `C:\PulseAudio\etc\pulse\default.pa` and add:
   ```
   load-module module-native-protocol-tcp auth-ip-acl=127.0.0.1;172.16.0.0/12
   load-module module-waveout sink_name=output source_name=input
   ```

4. Run `C:\PulseAudio\bin\pulseaudio.exe`

### Step 2: Configure WSL

In WSL terminal:

```bash
# Install PulseAudio client in WSL
sudo apt update
sudo apt install pulseaudio

# Get your Windows host IP (from WSL)
export HOST_IP=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')

# Set PulseAudio to connect to Windows
export PULSE_SERVER=tcp:$HOST_IP

# Test it
pactl info
```

### Step 3: Run CrabMusic

```bash
# Set the environment variable each time
export PULSE_SERVER=tcp:$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')

# Run CrabMusic
cargo run -- --show-labels

# Or make it permanent in ~/.bashrc:
echo 'export PULSE_SERVER=tcp:$(cat /etc/resolv.conf | grep nameserver | awk '"'"'{print $2}'"'"')' >> ~/.bashrc
```

---

## Option 2: Use VB-CABLE (Virtual Audio Cable)

### Install VB-CABLE

1. Download VB-CABLE: https://vb-audio.com/Cable/
2. Install it (reboot if needed)
3. Set VB-CABLE as your Windows default playback device
4. Your music player → VB-CABLE → CrabMusic

### Configure Audio Routing

```
Music Player (Spotify/YouTube)
    ↓
VB-CABLE Input (Windows default)
    ↓
VB-CABLE Output (CrabMusic listens here)
```

---

## Option 3: Run Natively on Windows (Best Performance)

Build and run CrabMusic directly on Windows (not WSL):

### Using Windows Terminal with Rust

1. **Install Rust on Windows** (not WSL):
   - Download from: https://rustup.rs/
   - Install Visual Studio Build Tools when prompted

2. **Open Windows PowerShell or CMD** (not WSL!):
   ```powershell
   cd E:\crabmusic

   # Build
   cargo build --release

   # Run with WASAPI loopback (captures all system audio)
   cargo run --release -- --loopback --show-labels
   ```

3. **Play music** in Spotify, YouTube, etc.

**This is the BEST option** - native Windows WASAPI loopback captures ALL system audio automatically!

---

## Option 4: Generate Test Tone (Quick Verification)

If you just want to verify the visualizer works:

```bash
# Install sox in WSL
sudo apt install sox

# Generate 440 Hz tone and pipe to CrabMusic
# (This won't work directly, but shows the concept)
sox -n -t wav - synth 5 sine 440
```

Or use an **online tone generator** in your browser:
- https://www.szynalski.com/tone-generator/
- Play various frequencies and see if bars respond

---

## Troubleshooting

### "Still not detecting audio"

Check if PulseAudio is running:
```bash
# In WSL
pactl list short

# Should show sources/sinks
```

### "Getting noise but not music"

You might be capturing microphone instead of system audio:
```bash
# List all audio sources
pactl list sources short

# Set default source to loopback/monitor
pactl set-default-source <source-name-here>
```

### "Visualizer not reacting"

1. Verify audio is playing in Windows
2. Check volume isn't muted
3. Make sure VB-CABLE or PulseAudio is set as default device
4. Try increasing sensitivity with `+` key

---

## Recommended Quick Solution

**For quick testing RIGHT NOW:**

1. **Build on native Windows** (not WSL):
   ```powershell
   # In Windows PowerShell, NOT WSL
   cd E:\crabmusic
   cargo run --release -- --loopback --show-labels
   ```

2. **Play music** in browser/Spotify

3. **Watch it work** immediately with WASAPI loopback!

No virtual cables, no PulseAudio server needed - Windows WASAPI captures everything.

---

## Which Option Should I Choose?

| Option | Pros | Cons | Best For |
|--------|------|------|----------|
| **Native Windows** | ✅ Works instantly<br>✅ WASAPI loopback<br>✅ Best performance | ❌ Need Rust on Windows | **Production use** |
| **PulseAudio** | ✅ Works in WSL<br>✅ No recompile | ❌ Setup complexity<br>❌ Latency | **Development in WSL** |
| **VB-CABLE** | ✅ Professional tool<br>✅ Works everywhere | ❌ Extra software<br>❌ Route audio manually | **Streaming/recording** |
| **Test Tone** | ✅ Instant verification | ❌ Not real music | **Quick testing** |

**My recommendation**: Build natively on Windows for the best experience!
