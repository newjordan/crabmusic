# Setting Up Videos for History TV Channel

## Quick Start

The History TV Channel plays **real videos as ASCII Braille art**! Here's how to set it up:

### 1. Organize Your Video Files

Create a directory structure for your videos:

```bash
mkdir -p videos/{retro,scifi,music,news,commercials,esoteric}/{1950s,1960s,1970s,1980s,1990s,2000s}
```

### 2. Add Video Files

Place your video files (`.mp4`, `.avi`, `.mkv`, etc.) in the appropriate directories:

```
videos/
├── retro/
│   ├── 1950s/
│   │   ├── soap_commercial.mp4
│   │   ├── i_love_lucy.mp4
│   │   └── space_race_newsreel.mp4
│   ├── 1960s/
│   │   └── twilight_zone.mp4
│   └── ...
├── scifi/
│   ├── 1950s/
│   │   └── day_earth_stood_still.mp4
│   └── ...
└── music/
    └── ...
```

### 3. Update the Video Catalog

Edit `src/visualization/history_tv_channel.rs` and update the file paths in each universe catalog:

```rust
fn new_retro_tv() -> Self {
    Self {
        fifties: vec![
            VideoEntry::new(
                "videos/retro/1950s/soap_commercial.mp4",  // ← Your actual file path
                "1950s TV Commercial - Soap Advertisement",
                1955,
                "Classic soap commercial",
            ),
            // Add more videos...
        ],
        // ...
    }
}
```

### 4. Build with Video Feature

```bash
cargo build --release --features video
```

**Note:** Requires FFmpeg libraries installed on your system.

### 5. Launch and Play!

```bash
cargo run --release
```

1. Navigate to History TV Channel (channel 14)
2. Browse videos with Up/Down arrows
3. Switch eras with Left/Right arrows
4. Switch universes with PageUp/PageDown
5. **Press ENTER to play the selected video in ASCII Braille!**

## How It Works

When you press ENTER:
1. CrabMusic exits the visualizer temporarily
2. Launches the video player using FFmpeg
3. Decodes video frames in real-time
4. Converts each frame to Braille dot patterns
5. Renders as pure ASCII art in your terminal!
6. Returns to the channel browser when the video ends

## Video Format Support

Supports any format FFmpeg can decode:
- MP4 (H.264, H.265)
- AVI
- MKV
- MOV
- WebM
- And many more!

## Tips

### Finding Public Domain Videos

Great sources for historical content:
- **Internet Archive** (archive.org) - Public domain movies, newsreels, commercials
- **Prelinger Archives** - Vintage educational films and ephemera
- **Library of Congress** - Historical footage
- **NASA Archive** - Space race content
- **British Pathé** - Historical newsreels

### Download Examples

```bash
# Example: Download a public domain 1950s commercial
youtube-dl "https://archive.org/details/..." -o "videos/retro/1950s/commercial.mp4"
```

### Optimal Video Settings

For best ASCII rendering:
- **Resolution**: 720p or lower works great
- **Framerate**: 24-30 FPS
- **Contrast**: High contrast videos look better in ASCII

### Converting Videos

Use FFmpeg to optimize:

```bash
# Convert to optimized format
ffmpeg -i input.mp4 -vf scale=1280:720 -r 24 output.mp4

# Increase contrast for better ASCII rendering
ffmpeg -i input.mp4 -vf "eq=contrast=1.5:brightness=0.1" output.mp4
```

## Customizing Catalogs

### Add More Videos

Edit the catalog methods in `src/visualization/history_tv_channel.rs`:

```rust
fifties: vec![
    VideoEntry::new(
        "videos/retro/1950s/my_video.mp4",
        "My Custom 1950s Video",
        1955,
        "Description of the video",
    ),
    // Add as many as you want!
],
```

### Create New Universe

Add a new universe by:
1. Adding variant to `Universe` enum
2. Creating `new_your_universe()` method
3. Adding it to `new_for_universe()` match

## Troubleshooting

### "File not found"
- Check the file path is correct (relative or absolute)
- Ensure the file exists in the specified location
- Try absolute paths if relative paths don't work

### Video won't play
- Build with `--features video` flag
- Install FFmpeg libraries:
  ```bash
  # Ubuntu/Debian
  sudo apt install libavcodec-dev libavformat-dev libswscale-dev

  # macOS
  brew install ffmpeg
  ```

### Audio not playing
The ASCII video player currently focuses on visual rendering. For audio:
- Play videos in a separate window with audio
- Use CrabMusic's system audio capture to visualize that audio
- Future enhancement: integrate audio playback with video

## Example Setup

Here's a complete example for the Retro TV universe:

```bash
# Create directory
mkdir -p videos/retro/1950s

# Download public domain video (example)
youtube-dl "https://archive.org/details/1950s-commercial" \
  -o "videos/retro/1950s/soap_ad.mp4"

# Update catalog (in history_tv_channel.rs)
VideoEntry::new(
    "videos/retro/1950s/soap_ad.mp4",
    "1950s Soap Commercial",
    1955,
    "Classic TV advertising from the golden age",
),

# Build and run
cargo build --release --features video
cargo run --release
```

## Advanced: Dynamic Catalog Loading

Future enhancement idea: Load catalogs from JSON:

```json
{
  "universe": "RetroTV",
  "era": "1950s",
  "videos": [
    {
      "file_path": "videos/retro/1950s/commercial.mp4",
      "title": "1950s Commercial",
      "year": 1955,
      "description": "Vintage ad"
    }
  ]
}
```

This would allow updating videos without recompiling!

---

**Happy time-traveling through ASCII video history!** 📺✨🎬
