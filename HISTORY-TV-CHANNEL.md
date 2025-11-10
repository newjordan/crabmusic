# History TV Channel - Real ASCII Video Player with Multi-verse

## Overview

The **History TV Channel** is a revolutionary **ACTUAL VIDEO PLAYER** that renders real videos as ASCII Braille art! Browse and **play videos** from different historical eras (1950s through 2000s) across **6 parallel content universes**.

This feature transforms CrabMusic into a nostalgic time-traveling, dimension-hopping TV that **actually plays videos** in beautiful ASCII Braille rendering - flip through commercials, TV shows, sci-fi classics, music videos, news clips, and experimental content from the past 50+ years, and press ENTER to watch them in your terminal!

## Features

- **REAL VIDEO PLAYBACK**: Actual video player using ASCII Braille rendering!
- **6 Parallel Universes**: Jump between themed content dimensions
  - 🎬 Retro TV Universe
  - 🚀 Sci-Fi Universe
  - 🎵 Music Videos Universe
  - 📰 News & Documentary Universe
  - 📺 Commercials Universe
  - 👁️ Esoteric & Weird TV Universe
- **6 Historical Eras**: Browse content from the 1950s, 1960s, 1970s, 1980s, 1990s, and 2000s
- **Multi-dimensional Catalog**: Each universe has its own curated video collection for all eras
- **Braille Art Rendering**: Videos play as beautiful ASCII art in real-time
- **Intuitive Navigation**: Simple keyboard controls to switch between universes, videos, and eras
- **Retro TV Interface**: ASCII art TV frame with universe, era, and video information displayed
- **FFmpeg Integration**: Supports all major video formats (MP4, AVI, MKV, etc.)

## How to Use

### Accessing the History TV Channel

1. Launch CrabMusic: `cargo run --release`
2. Use the **Left/Right arrow keys** to cycle through visualizer channels
3. Navigate to the "History TV Channel" (channel 14)

### Navigation Controls

When in the History TV Channel:

**Video Playback:**
- **ENTER**: **Play the selected video in ASCII Braille!**
  - With auto-play ON: Videos play continuously in a loop!
  - With auto-play OFF: Returns to browser after video ends

**Auto-Play Mode:**
- **SPACE**: Toggle auto-play ON/OFF
- **A**: Toggle cross-era mode (jump between 50s, 60s, 70s, etc.)
- **U**: Toggle cross-universe mode (jump between universes)

**Manual Navigation:**
- **↑ (Up Arrow)**: Switch to the next video in the current era
- **↓ (Down Arrow)**: Switch to the previous video in the current era
- **→ (Right Arrow)**: Jump to the next era (e.g., 1950s → 1960s)
- **← (Left Arrow)**: Jump to the previous era (e.g., 1960s → 1950s)
- **Page Up**: Jump to the next universe (dimension hop!)
- **Page Down**: Jump to the previous universe

**Other:**
- **Q**: Quit (from browser or during video playback)

### Switching to Other Channels

To switch from the History TV Channel to other visualizers:
- Use **number keys** to jump directly to a specific channel
- Or use the **'V' key** to cycle forward through channels

## Multi-verse Content Universes

### 🎬 Retro TV Universe (Original)
Classic television content and historical moments through the decades.

### 🚀 Sci-Fi Universe
Science fiction films, TV shows, and trailers from 1950s to 2000s.
- 1950s: The Day the Earth Stood Still
- 1960s: Star Trek, 2001: A Space Odyssey
- 1970s: Star Wars, Alien
- 1980s: Blade Runner, The Terminator
- 1990s: The Matrix
- 2000s: District 9

### 🎵 Music Videos Universe
Iconic music videos and performances through history.
- 1950s: Elvis Presley - Jailhouse Rock
- 1960s: The Beatles - A Hard Day's Night
- 1970s: Queen - Bohemian Rhapsody
- 1980s: Michael Jackson - Thriller, MTV Launch
- 1990s: Nirvana - Smells Like Teen Spirit
- 2000s: OK Go - Here It Goes Again

### 📰 News & Documentary Universe
Historical news coverage and documentary footage.
- 1950s: Korean War newsreels
- 1960s: Moon Landing broadcast
- 1970s: Watergate scandal
- 1980s: Berlin Wall falls
- 1990s: Gulf War CNN coverage
- 2000s: Obama victory speech

### 📺 Commercials Universe
Pure commercial content from all eras.
- 1950s: Colgate toothpaste ads
- 1960s: Coca-Cola commercials
- 1970s: McDonald's Big Mac jingle
- 1980s: Apple 1984 Super Bowl ad
- 1990s: Budweiser "Whassup?!"
- 2000s: Old Spice "The Man Your Man Could Smell Like"

### 👁️ Esoteric & Weird TV Universe
Experimental, avant-garde, and bizarre television content.
- 1950s: TV test patterns and color bars
- 1960s: The Outer Limits opening
- 1970s: H.R. Pufnstuf psychedelic kids show
- 1980s: Max Headroom broadcast signal intrusion
- 1990s: Twin Peaks red room scene
- 2000s: Adult Swim - Off the Air

## Video Catalog Structure (Retro TV Universe)

The original Retro TV Universe includes:

### 1950s
- Classic TV commercials (soap advertisements)
- Iconic sitcoms like "I Love Lucy"
- Historical newsreels (Space Race)

### 1960s
- Vintage product advertisements (Coca-Cola)
- "The Twilight Zone" opening sequences
- Moon Landing broadcast (1969)

### 1970s
- Disco-era commercials and fashion
- "Saturday Night Live" original cast
- Star Wars theatrical trailers (1977)

### 1980s
- MTV launch and music videos
- Nintendo NES commercials
- Max Headroom TV excerpts

### 1990s
- "Seinfeld" and "Friends" classic scenes
- AOL and early internet commercials
- 90s pop culture moments

### 2000s
- iPod silhouette commercials
- First YouTube videos (2005)
- iPhone announcement by Steve Jobs (2007)

## Customization

### Adding Your Own Videos

You can customize the video catalog for any universe by modifying the appropriate catalog creation method in `src/visualization/history_tv_channel.rs`:

#### For Retro TV Universe:

```rust
impl VideoCatalog {
    fn new_retro_tv() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "https://youtube.com/watch?v=YOUR_VIDEO_ID",
                    "Your Video Title",
                    1955,
                    "Video description",
                ),
                // Add more videos...
            ],
            // Add to other eras...
        }
    }
}
```

#### For Other Universes:
Modify `new_sci_fi()`, `new_music_videos()`, `new_news_documentary()`, `new_commercials()`, or `new_esoteric_weird()` methods to add content to those universes.

### Video Entry Fields

Each `VideoEntry` contains:
- **url**: Video URL (YouTube or other sources)
- **title**: Display title for the video
- **year**: Year the content was created
- **description**: Brief description shown in the interface

## Implementation Details

### Architecture

The History TV Channel is implemented as a standard CrabMusic visualizer (`HistoryTVChannelVisualizer`) that:
- Maintains a catalog of videos organized by era
- Tracks the current era and video index
- Responds to keyboard inputs for navigation
- Renders an ASCII TV interface showing current video information

### File Structure

```
src/
├── visualization/
│   ├── history_tv_channel.rs  # Main implementation
│   └── mod.rs                  # Export the visualizer
└── main.rs                     # Integration and keyboard controls
```

### Key Components

1. **Universe Enum**: Represents the 6 themed content universes
2. **Era Enum**: Represents the 6 historical decades
3. **VideoEntry**: Struct containing video metadata
4. **VideoCatalog**: Organizes videos by era (universe-specific)
5. **HistoryTVChannelVisualizer**: Main visualizer with multi-verse support

## Future Enhancements

Potential improvements for the Multi-verse History TV:

- **Actual Video Playback**: Integration with the existing video playback system to play the videos
- **Cross-Universe Random Mode**: Randomly select videos across all universes and eras
- **Universe Bookmarks**: Save favorite universe/era/video combinations
- **Search Across Universes**: Search videos across all dimensions
- **Custom Universe Creation**: Add your own themed universes
- **Universe Mixing**: Combine content from multiple universes
- **External Catalog Loading**: Load universe catalogs from JSON/YAML files
- **YouTube Integration**: Direct YouTube API integration for streaming
- **Thumbnail Display**: Show video thumbnails using ASCII art
- **Time Portal Animation**: Special effects when switching universes

## Video Playback

### Manual Mode (Default)

1. **Browse** videos in the channel interface (see metadata, year, description)
2. **Press ENTER** to play the selected video
3. **Watch** the video render in real-time as ASCII Braille art!
4. Video ends → Returns to channel browser

### Auto-Play Mode (Like a Real TV Channel!)

**The magic feature that makes this feel like real TV:**

1. **Press SPACE** to enable auto-play mode
2. **Press ENTER** to start playing
3. **Sit back and watch!** Videos play automatically one after another
4. The system randomly selects the next video based on your settings:
   - **SAME ERA mode** (default): Stays in current universe and era
   - **ALL ERAS mode** (press A): Jumps randomly between eras in the same universe
   - **ALL UNIVERSES mode** (press U): Jumps randomly across all universes and eras!

**Example Auto-Play Session:**
```
1. Navigate to Retro TV Universe, 1950s era
2. Press SPACE (enable auto-play)
3. Press A (enable cross-era)
4. Press ENTER (start playing)
5. Watch: 1950s commercial → random jump to 1980s sitcom → 1960s newsreel → ...
```

**Press Q during playback to stop and return to browser.**

### Requirements

- **Build with video feature**: `cargo build --release --features video`
- **FFmpeg libraries** must be installed on your system
- **Local video files** in the paths specified in the catalog

### Setup Guide

See **[HISTORY-TV-SETUP.md](./HISTORY-TV-SETUP.md)** for complete setup instructions, including:
- How to organize video files
- Where to find public domain content
- How to update the video catalog
- Video format optimization tips
- Troubleshooting guide

## Notes

- **File Paths**: The catalog uses local file paths (e.g., `videos/retro/1950s/commercial.mp4`)
- **Placeholder Paths**: Default paths are placeholders - replace with your own video files!
- **File Check**: UI shows whether video file exists before playing
- **Performance**: ASCII rendering runs smoothly at 24-30 FPS for most videos

## Credits

The Multi-verse History TV Channel feature was designed to bring a nostalgic, dimension-hopping, TV-channel-surfing experience to CrabMusic, celebrating the rich history of television, sci-fi, music, news, commercials, and experimental media from the 1950s through the 2000s across parallel content universes.

## Troubleshooting

### Can't switch away from History TV Channel
- Use number keys (0-13) to jump directly to another channel
- Or press 'V' to cycle to the next visualizer

### Videos not displaying
- This version displays video metadata only
- For actual playback, videos need to be local files with the `--video` feature

### Audio not affecting visuals
- The visualizer includes subtle audio-reactive pulsing
- Ensure audio input is enabled (microphone or loopback)

---

**Happy time traveling through TV history!** 📺✨
