# History TV Channel Feature

## Overview

The **History TV Channel** is a new visualizer mode in CrabMusic that provides an interactive TV channel-like experience, allowing you to browse and "watch" videos from different historical eras (1950s through 2000s).

This feature transforms CrabMusic into a nostalgic time-traveling TV remote, where you can flip through commercials, TV shows, and iconic moments from the past 50+ years.

## Features

- **6 Historical Eras**: Browse content from the 1950s, 1960s, 1970s, 1980s, 1990s, and 2000s
- **Video Catalog**: Pre-configured collection of historical videos, commercials, and TV content organized by decade
- **Intuitive Navigation**: Simple keyboard controls to switch between videos and eras
- **Retro TV Interface**: ASCII art TV frame with era and video information displayed
- **Audio-Reactive**: Visualizer responds to audio with pulsing animations

## How to Use

### Accessing the History TV Channel

1. Launch CrabMusic: `cargo run --release`
2. Use the **Left/Right arrow keys** to cycle through visualizer channels
3. Navigate to the "History TV Channel" (channel 14)

### Navigation Controls

When in the History TV Channel:

- **↑ (Up Arrow)**: Switch to the next video in the current era
- **↓ (Down Arrow)**: Switch to the previous video in the current era
- **→ (Right Arrow)**: Jump to the next era (e.g., 1950s → 1960s)
- **← (Left Arrow)**: Jump to the previous era (e.g., 1960s → 1950s)
- **Q**: Quit the application

### Switching to Other Channels

To switch from the History TV Channel to other visualizers:
- Use **number keys** to jump directly to a specific channel
- Or use the **'V' key** to cycle forward through channels

## Video Catalog Structure

The History TV Channel comes with a pre-configured catalog of videos organized by era:

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

You can customize the video catalog by modifying the `VideoCatalog` in `src/visualization/history_tv_channel.rs`:

```rust
impl VideoCatalog {
    pub fn new() -> Self {
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

1. **Era Enum**: Represents the 6 historical decades
2. **VideoEntry**: Struct containing video metadata
3. **VideoCatalog**: Organizes videos by era
4. **HistoryTVChannelVisualizer**: Main visualizer implementation

## Future Enhancements

Potential improvements for the History TV Channel:

- **Actual Video Playback**: Integration with the existing video playback system to play the videos
- **Random Shuffle Mode**: Randomly select videos across all eras
- **Favorites System**: Mark and quickly access favorite videos
- **Search Functionality**: Search videos by title, year, or keywords
- **Custom Playlists**: Create and save custom video playlists
- **External Catalog Loading**: Load video catalogs from JSON/YAML files
- **YouTube Integration**: Direct YouTube API integration for streaming
- **Thumbnail Display**: Show video thumbnails using ASCII art

## Notes

- **Video Playback**: Currently, the History TV Channel displays video metadata and provides navigation. Actual video playback would require integration with the `--video` feature and file paths instead of URLs.
- **YouTube URLs**: The sample URLs in the catalog are placeholders. You'll need to replace them with actual video URLs or local file paths.
- **Performance**: The visualizer is lightweight and shouldn't impact performance, even with large video catalogs.

## Credits

The History TV Channel feature was designed to bring a nostalgic, TV-channel-surfing experience to CrabMusic, celebrating the rich history of television, commercials, and pop culture from the 1950s through the 2000s.

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
