# [VIZ-008] Braille Video Playback - Brownfield Enhancement

**Epic**: Visualization Engine
**Priority**: P3 (Cool Enhancement - Experimental Feature)
**Estimated Effort**: 2-3 days (16-24 hours)
**Status**: Draft

---

## Description

Add **video playback capability** to CrabMusic using the existing Braille rendering infrastructure, enabling users to watch videos in the terminal with a stunning retro CRT-style aesthetic. The Braille dot matrix provides high-resolution rendering perfect for artistic video playback.

**Current System**: CrabMusic uses Braille characters (2×4 dot patterns) for high-resolution audio visualization (oscilloscope, spectrum analyzer) with up to 160×96 pixel resolution in standard 80×24 terminals.

**The Enhancement**: Extend the Braille system for video playback that:
- Decodes video files using FFmpeg (supports MP4, AVI, MKV, WebM, etc.)
- Converts video frames to Braille dot patterns with dithering
- Supports configurable resolution (160×96 up to 512×512 or higher)
- Renders with color (RGB per cell) or monochrome (grayscale dithering)
- Maintains 24-30 FPS playback for smooth video
- Provides playback controls (play, pause, seek, speed)
- Creates retro CRT/ASCII art aesthetic

**Visual Style**: Braille dots create an authentic retro feel - like watching video on a 1970s terminal or oscilloscope display. Dithering mimics CRT phosphor patterns, and per-cell coloring provides surprisingly good color fidelity.

**Key Benefit**: Unique artistic video experience, retro aesthetic demonstrations, music video visualizations in terminal, demo scene culture, live performance visuals.

---

## User Story

**As a** CrabMusic user,
**I want** to play video files in the terminal using Braille character rendering,
**So that** I can enjoy a unique retro aesthetic experience, watch music videos with terminal visuals, or create artistic live performance displays.

---

## Story Context

**Existing System Integration:**
- Integrates with: BrailleGrid (`src/visualization/braille.rs`)
- Technology: Rust with FFmpeg for decoding, existing Braille infrastructure
- Follows pattern: Visualizer trait, GridBuffer rendering pipeline
- Touch points: BrailleGrid, TerminalRenderer, visualization mode switching

**Existing Braille Infrastructure (VIZ-007)**:
- BrailleGrid provides 2×4 dot patterns per terminal cell
- High-resolution: 80×24 terminal = 160×96 dots (4× vertical resolution)
- Color support: RGB per cell via terminal ANSI codes
- Optimized rendering: Bresenham line drawing, efficient bit operations
- Memory efficient: ~5 bytes per cell (pattern + color)

**Video Playback Approach**:
- Use FFmpeg (via ffmpeg-next crate) for universal codec support
- Decode video frames to RGB images
- Resize frames to match Braille dot resolution
- Convert image to Braille patterns using threshold + dithering
- Average pixel colors per 2×4 block for cell coloring
- Maintain frame timing with adaptive quality

**Resolution Flexibility**:
- **Standard (80×24 terminal)**: 160×96 pixels
- **Large (160×48 terminal)**: 320×192 pixels
- **Extra Large (240×72 terminal)**: 480×288 pixels
- **Huge (256×128 terminal)**: 512×512 pixels ✓
- **Custom**: User-specified or auto-detected from terminal size

---

## Acceptance Criteria

**Functional Requirements:**

1. VideoPlayer struct created with FFmpeg decoder integration
2. Supports common video formats: MP4, AVI, MKV, WebM, MOV
3. Frame decoding to RGB pixel data via FFmpeg
4. Configurable target resolution (auto, fixed, or scaled)
5. Frame resizing with quality interpolation (bilinear/bicubic)
6. Image-to-Braille conversion with threshold algorithm
7. Floyd-Steinberg dithering for improved visual quality
8. Color mode: RGB per cell (averaged from 2×4 pixel block)
9. Monochrome mode: Grayscale with dithering
10. Playback timing maintains target FPS (24-30 FPS)
11. Frame buffering with background decoding thread
12. Playback controls: Play, pause, resume, stop
13. Seeking support: Jump to specific time
14. Speed control: 0.5×, 1×, 1.5×, 2× playback speed

**Integration Requirements:**

15. Existing BrailleGrid infrastructure used (no duplication)
16. Integrates with TerminalRenderer via GridBuffer
17. New video module: `src/video/mod.rs`
18. Mode switching: Audio viz → Video → Back to audio
19. Configuration via config file or CLI args
20. No breaking changes to existing visualizers

**Quality Requirements:**

21. Performance: 24-30 FPS at 512×512 resolution
22. Memory usage: <5MB for frame buffering (10 frames)
23. CPU usage: <30% single core at target resolution
24. Unit tests for frame conversion algorithms
25. Integration tests with sample video files
26. Manual testing with various resolutions and formats
27. Error handling for unsupported formats/codecs
28. Graceful degradation on performance constraints

**Configuration Options:**

29. Resolution mode: auto (match terminal), fixed (WxH), scale (2.0×)
30. Color mode: color (RGB), monochrome (grayscale)
31. Dithering: enabled/disabled, algorithm selection
32. Frame skip: auto-skip frames if falling behind
33. Quality: low (fast), medium (balanced), high (slow but prettier)

---

## Technical Notes

**Integration Approach:**
- Create new `src/video/` module with VideoPlayer, VideoDecoder, FrameConverter
- VideoPlayer owns BrailleGrid instance for video frames
- Use crossbeam channels for background frame decoding
- Frame converter handles: resize → dither → braille conversion → color mapping
- Existing TerminalRenderer handles display (no changes needed)

**Existing Pattern Reference:**
- Follow Visualizer trait pattern (src/visualization/mod.rs)
- Similar to OscilloscopeVisualizer structure
- Use BrailleGrid infrastructure (src/visualization/braille.rs)
- Integrate with main loop in src/main.rs

**Key Constraints:**
- FFmpeg must be installed on user system (dependency)
- Terminal must support RGB ANSI codes for color
- Performance scales with resolution (higher = slower)
- Terminal refresh rate may limit FPS
- Font size affects visual quality of Braille dots
- Braille rendering is default visualization system

**Algorithm Overview**:

```rust
// Frame processing pipeline
let frame = decoder.next_frame()?; // FFmpeg decode
let resized = resize_image(&frame, target_width, target_height);
let dithered = apply_dithering(&resized, DitherAlgorithm::FloydSteinberg);
let braille_grid = convert_to_braille(&dithered, use_color);
render_to_terminal(&braille_grid);
```

**Dithering Algorithm (Floyd-Steinberg)**:
```rust
// Error diffusion for better visual quality
for y in 0..height {
    for x in 0..width {
        let old_pixel = pixels[y][x];
        let new_pixel = if old_pixel > threshold { 1 } else { 0 };
        pixels[y][x] = new_pixel;

        let error = old_pixel - new_pixel;
        pixels[y][x+1] += error * 7/16;      // Right
        pixels[y+1][x-1] += error * 3/16;    // Bottom-left
        pixels[y+1][x] += error * 5/16;      // Bottom
        pixels[y+1][x+1] += error * 1/16;    // Bottom-right
    }
}
```

---

## Definition of Done

- [ ] VideoPlayer struct implemented with FFmpeg integration
- [ ] Video decoding works for common formats (MP4, AVI, MKV)
- [ ] Frame resizing with quality interpolation
- [ ] Image-to-Braille conversion algorithm working
- [ ] Floyd-Steinberg dithering implemented
- [ ] Color mode: RGB per cell based on pixel averaging
- [ ] Monochrome mode: Grayscale with dithering
- [ ] Configurable resolution (auto, fixed, scaled)
- [ ] Playback timing maintains 24-30 FPS
- [ ] Background decoding thread with frame buffering
- [ ] Playback controls: play, pause, seek, speed
- [ ] Integration with main application (mode switching)
- [ ] Configuration via CLI or config file
- [ ] Unit tests pass (frame conversion, dithering)
- [ ] Integration tests pass (sample video playback)
- [ ] Performance validated (24-30 FPS at 512×512)
- [ ] Manual testing with various formats
- [ ] Documentation: usage guide, configuration options
- [ ] Error handling for edge cases
- [ ] Braille visualization confirmed as default

---

## Risk and Compatibility Check

**Minimal Risk Assessment:**

**Primary Risk**: FFmpeg dependency requires system installation (users must have FFmpeg)
**Mitigation**: Clear installation instructions, detect FFmpeg presence, graceful error messages
**Rollback**: Video feature is optional, can be disabled via feature flag

**Secondary Risk**: Performance may vary by terminal emulator and system
**Mitigation**: Adaptive quality settings, frame skipping, resolution auto-detection
**Rollback**: Allow disabling video mode if performance is unacceptable

**Compatibility Verification:**

- [x] No breaking changes to existing visualizers
- [x] Video playback is additive feature (optional)
- [x] BrailleGrid API unchanged
- [x] Terminal rendering pipeline unchanged
- [x] Memory usage reasonable (<5MB overhead)
- [x] Braille rendering system remains default

---

## Implementation Phases

### Phase 1: FFmpeg Integration & Video Decoder

**File**: `src/video/decoder.rs`

Create video decoder using FFmpeg:

```rust
use ffmpeg_next as ffmpeg;
use image::RgbImage;

/// Video decoder using FFmpeg
pub struct VideoDecoder {
    /// FFmpeg input context
    input: ffmpeg::format::context::Input,
    /// Video stream index
    video_stream_index: usize,
    /// Video decoder
    decoder: ffmpeg::decoder::Video,
    /// Scaler for format conversion to RGB
    scaler: ffmpeg::software::scaling::Context,
    /// Current frame number
    frame_number: usize,
    /// Total frames in video
    total_frames: usize,
    /// Frames per second
    fps: f32,
}

impl VideoDecoder {
    /// Open a video file
    ///
    /// # Arguments
    /// * `path` - Path to video file
    ///
    /// # Returns
    /// VideoDecoder instance
    ///
    /// # Errors
    /// Returns error if file cannot be opened or no video stream found
    pub fn open(path: &str) -> Result<Self, VideoError> {
        ffmpeg::init()?;

        let input = ffmpeg::format::input(&path)?;

        // Find video stream
        let video_stream = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(VideoError::NoVideoStream)?;

        let video_stream_index = video_stream.index();

        // Create decoder
        let context = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;
        let decoder = context.decoder().video()?;

        // Create scaler for RGB conversion
        let scaler = ffmpeg::software::scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ffmpeg::format::Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            ffmpeg::software::scaling::Flags::BILINEAR,
        )?;

        // Calculate FPS and total frames
        let fps = video_stream.avg_frame_rate().0 as f32 / video_stream.avg_frame_rate().1 as f32;
        let duration = video_stream.duration() as f32 * f32::from(video_stream.time_base());
        let total_frames = (duration * fps) as usize;

        Ok(Self {
            input,
            video_stream_index,
            decoder,
            scaler,
            frame_number: 0,
            total_frames,
            fps,
        })
    }

    /// Get next frame from video
    ///
    /// # Returns
    /// RgbImage containing frame pixels, or None if end of video
    pub fn next_frame(&mut self) -> Result<Option<RgbImage>, VideoError> {
        for (stream, packet) in self.input.packets() {
            if stream.index() == self.video_stream_index {
                self.decoder.send_packet(&packet)?;

                let mut decoded = ffmpeg::util::frame::video::Video::empty();

                if self.decoder.receive_frame(&mut decoded).is_ok() {
                    // Scale to RGB
                    let mut rgb_frame = ffmpeg::util::frame::video::Video::empty();
                    self.scaler.run(&decoded, &mut rgb_frame)?;

                    // Convert to image::RgbImage
                    let width = rgb_frame.width();
                    let height = rgb_frame.height();
                    let data = rgb_frame.data(0).to_vec();

                    let image = RgbImage::from_raw(width, height, data)
                        .ok_or(VideoError::ImageConversionFailed)?;

                    self.frame_number += 1;
                    return Ok(Some(image));
                }
            }
        }

        // End of video
        Ok(None)
    }

    /// Get video metadata
    pub fn metadata(&self) -> VideoMetadata {
        VideoMetadata {
            fps: self.fps,
            total_frames: self.total_frames,
            width: self.decoder.width(),
            height: self.decoder.height(),
            current_frame: self.frame_number,
        }
    }

    /// Seek to specific frame
    pub fn seek_to_frame(&mut self, frame: usize) -> Result<(), VideoError> {
        // Calculate timestamp
        let timestamp = (frame as f32 / self.fps * 1000.0) as i64;

        // Seek in FFmpeg
        self.input.seek(timestamp, ..timestamp)?;
        self.frame_number = frame;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub fps: f32,
    pub total_frames: usize,
    pub width: u32,
    pub height: u32,
    pub current_frame: usize,
}
```

---

### Phase 2: Frame to Braille Converter

**File**: `src/video/converter.rs`

Convert image frames to Braille patterns:

```rust
use crate::visualization::{BrailleGrid, Color};
use image::{RgbImage, Rgb};

/// Convert image frame to Braille grid
pub struct FrameConverter {
    /// Dithering enabled
    use_dithering: bool,
    /// Color mode enabled
    use_color: bool,
    /// Brightness threshold (0-255)
    threshold: u8,
}

impl FrameConverter {
    pub fn new(use_dithering: bool, use_color: bool, threshold: u8) -> Self {
        Self {
            use_dithering,
            use_color,
            threshold,
        }
    }

    /// Convert RGB image to Braille grid
    ///
    /// # Arguments
    /// * `image` - Input image (must match grid dot resolution)
    /// * `grid` - Output BrailleGrid to populate
    pub fn convert(&self, image: &RgbImage, grid: &mut BrailleGrid) {
        let dot_width = grid.dot_width();
        let dot_height = grid.dot_height();

        // Apply dithering if enabled
        let processed = if self.use_dithering {
            self.apply_floyd_steinberg_dithering(image)
        } else {
            image.clone()
        };

        // Clear grid
        grid.clear();

        // Convert pixels to Braille dots
        for dot_y in 0..dot_height {
            for dot_x in 0..dot_width {
                if dot_x >= processed.width() as usize || dot_y >= processed.height() as usize {
                    continue;
                }

                let pixel = processed.get_pixel(dot_x as u32, dot_y as u32);
                let brightness = self.calculate_brightness(pixel);

                // Set dot if above threshold
                if brightness > self.threshold {
                    if self.use_color {
                        let color = Color::new(pixel[0], pixel[1], pixel[2]);
                        grid.set_dot_with_color(dot_x, dot_y, color);
                    } else {
                        grid.set_dot(dot_x, dot_y);
                    }
                }
            }
        }

        // If color mode, average colors per cell
        if self.use_color {
            self.apply_cell_colors(image, grid);
        }
    }

    /// Calculate perceived brightness (luminance)
    fn calculate_brightness(&self, pixel: &Rgb<u8>) -> u8 {
        // Weighted average for human perception
        let r = pixel[0] as f32 * 0.299;
        let g = pixel[1] as f32 * 0.587;
        let b = pixel[2] as f32 * 0.114;
        (r + g + b) as u8
    }

    /// Apply Floyd-Steinberg dithering to image
    fn apply_floyd_steinberg_dithering(&self, image: &RgbImage) -> RgbImage {
        let mut dithered = image.clone();
        let width = image.width() as usize;
        let height = image.height() as usize;

        for y in 0..height {
            for x in 0..width {
                let pixel = dithered.get_pixel(x as u32, y as u32);
                let old_brightness = self.calculate_brightness(pixel);

                // Quantize to binary
                let new_brightness = if old_brightness > self.threshold { 255 } else { 0 };

                // Calculate error
                let error = old_brightness as i32 - new_brightness as i32;

                // Set quantized pixel
                let scale = new_brightness as f32 / old_brightness.max(1) as f32;
                let new_pixel = Rgb([
                    (pixel[0] as f32 * scale) as u8,
                    (pixel[1] as f32 * scale) as u8,
                    (pixel[2] as f32 * scale) as u8,
                ]);
                dithered.put_pixel(x as u32, y as u32, new_pixel);

                // Diffuse error to neighbors
                if x + 1 < width {
                    self.add_error(&mut dithered, x + 1, y, error * 7 / 16);
                }
                if y + 1 < height {
                    if x > 0 {
                        self.add_error(&mut dithered, x - 1, y + 1, error * 3 / 16);
                    }
                    self.add_error(&mut dithered, x, y + 1, error * 5 / 16);
                    if x + 1 < width {
                        self.add_error(&mut dithered, x + 1, y + 1, error / 16);
                    }
                }
            }
        }

        dithered
    }

    /// Add error to pixel (for dithering)
    fn add_error(&self, image: &mut RgbImage, x: usize, y: usize, error: i32) {
        let pixel = image.get_pixel(x as u32, y as u32);
        let adjust = |v: u8| -> u8 {
            ((v as i32 + error).max(0).min(255)) as u8
        };

        let new_pixel = Rgb([
            adjust(pixel[0]),
            adjust(pixel[1]),
            adjust(pixel[2]),
        ]);
        image.put_pixel(x as u32, y as u32, new_pixel);
    }

    /// Apply averaged colors to cells
    fn apply_cell_colors(&self, image: &RgbImage, grid: &mut BrailleGrid) {
        let cell_width = grid.width();
        let cell_height = grid.height();

        for cell_y in 0..cell_height {
            for cell_x in 0..cell_width {
                // Average color of 2×4 pixel block
                let color = self.average_block_color(image, cell_x * 2, cell_y * 4);

                // Set cell color if cell has any dots
                if !grid.is_empty(cell_x, cell_y) {
                    // Update color (need to set at least one dot with color)
                    let dot_x = cell_x * 2;
                    let dot_y = cell_y * 4;
                    grid.set_dot_with_color(dot_x, dot_y, color);
                }
            }
        }
    }

    /// Average color of 2×4 pixel block
    fn average_block_color(&self, image: &RgbImage, x: usize, y: usize) -> Color {
        let mut r_sum = 0u32;
        let mut g_sum = 0u32;
        let mut b_sum = 0u32;
        let mut count = 0u32;

        for dy in 0..4 {
            for dx in 0..2 {
                let px = (x + dx) as u32;
                let py = (y + dy) as u32;

                if px < image.width() && py < image.height() {
                    let pixel = image.get_pixel(px, py);
                    r_sum += pixel[0] as u32;
                    g_sum += pixel[1] as u32;
                    b_sum += pixel[2] as u32;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Color::new(
                (r_sum / count) as u8,
                (g_sum / count) as u8,
                (b_sum / count) as u8,
            )
        } else {
            Color::new(0, 0, 0)
        }
    }
}
```

---

### Phase 3: Video Player with Playback Control

**File**: `src/video/player.rs`

Main video player with timing control:

```rust
use super::{VideoDecoder, FrameConverter, VideoError};
use crate::visualization::BrailleGrid;
use image::imageops;
use std::time::{Duration, Instant};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::thread;

/// Video playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

/// Video player configuration
#[derive(Debug, Clone)]
pub struct VideoConfig {
    /// Target resolution (width, height) in dots
    pub resolution: (usize, usize),
    /// Enable color rendering
    pub use_color: bool,
    /// Enable dithering
    pub use_dithering: bool,
    /// Brightness threshold (0-255)
    pub threshold: u8,
    /// Target FPS (0 = use video's native FPS)
    pub target_fps: f32,
    /// Frame buffer size
    pub buffer_size: usize,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            resolution: (512, 512),  // High quality default
            use_color: true,
            use_dithering: true,
            threshold: 128,
            target_fps: 0.0,  // Use native FPS
            buffer_size: 10,
        }
    }
}

/// Video player
pub struct VideoPlayer {
    /// Video decoder
    decoder: VideoDecoder,
    /// Frame converter
    converter: FrameConverter,
    /// Braille grid for rendering
    grid: BrailleGrid,
    /// Configuration
    config: VideoConfig,
    /// Playback state
    state: PlaybackState,
    /// Playback speed multiplier
    speed: f32,
    /// Frame receiver (from background decoder)
    frame_rx: Option<Receiver<Option<image::RgbImage>>>,
    /// Decoder control sender
    control_tx: Option<Sender<PlayerControl>>,
}

enum PlayerControl {
    Pause,
    Resume,
    Stop,
    Seek(usize),
}

impl VideoPlayer {
    /// Create new video player
    pub fn new(video_path: &str, config: VideoConfig) -> Result<Self, VideoError> {
        let decoder = VideoDecoder::open(video_path)?;

        let converter = FrameConverter::new(
            config.use_dithering,
            config.use_color,
            config.threshold,
        );

        // Create Braille grid based on resolution
        let (dot_width, dot_height) = config.resolution;
        let cell_width = dot_width / 2;
        let cell_height = dot_height / 4;
        let grid = BrailleGrid::new(cell_width, cell_height);

        Ok(Self {
            decoder,
            converter,
            grid,
            config,
            state: PlaybackState::Stopped,
            speed: 1.0,
            frame_rx: None,
            control_tx: None,
        })
    }

    /// Start video playback
    pub fn play(&mut self) -> Result<(), VideoError> {
        self.state = PlaybackState::Playing;

        // Start background decoder thread
        self.start_decoder_thread();

        Ok(())
    }

    /// Get next frame for display
    pub fn next_frame(&mut self) -> Result<Option<&BrailleGrid>, VideoError> {
        if self.state != PlaybackState::Playing {
            return Ok(None);
        }

        // Get frame from buffer
        if let Some(rx) = &self.frame_rx {
            match rx.try_recv() {
                Ok(Some(frame)) => {
                    // Resize to target resolution
                    let resized = imageops::resize(
                        &frame,
                        self.config.resolution.0 as u32,
                        self.config.resolution.1 as u32,
                        imageops::FilterType::Lanczos3,
                    );

                    // Convert to Braille
                    self.converter.convert(&resized, &mut self.grid);

                    Ok(Some(&self.grid))
                }
                Ok(None) => {
                    // End of video
                    self.state = PlaybackState::Stopped;
                    Ok(None)
                }
                Err(_) => {
                    // No frame ready yet
                    Ok(Some(&self.grid))  // Return previous frame
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
            if let Some(tx) = &self.control_tx {
                let _ = tx.send(PlayerControl::Pause);
            }
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.state = PlaybackState::Playing;
            if let Some(tx) = &self.control_tx {
                let _ = tx.send(PlayerControl::Resume);
            }
        }
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        if let Some(tx) = &self.control_tx {
            let _ = tx.send(PlayerControl::Stop);
        }
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.1).min(4.0);
    }

    /// Get playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Get FPS (actual playback FPS considering speed)
    pub fn fps(&self) -> f32 {
        let native_fps = if self.config.target_fps > 0.0 {
            self.config.target_fps
        } else {
            self.decoder.metadata().fps
        };
        native_fps * self.speed
    }

    /// Start background decoder thread
    fn start_decoder_thread(&mut self) {
        let (frame_tx, frame_rx) = bounded(self.config.buffer_size);
        let (control_tx, control_rx) = bounded(1);

        let mut decoder = self.decoder.clone();  // Assumes VideoDecoder implements Clone

        thread::spawn(move || {
            loop {
                // Check for control messages
                if let Ok(control) = control_rx.try_recv() {
                    match control {
                        PlayerControl::Pause => {
                            // Wait for resume or stop
                            match control_rx.recv() {
                                Ok(PlayerControl::Resume) => continue,
                                Ok(PlayerControl::Stop) | Err(_) => break,
                                _ => {}
                            }
                        }
                        PlayerControl::Stop => break,
                        PlayerControl::Seek(frame) => {
                            if decoder.seek_to_frame(frame).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                // Decode next frame
                match decoder.next_frame() {
                    Ok(Some(frame)) => {
                        if frame_tx.send(Some(frame)).is_err() {
                            break;  // Receiver dropped
                        }
                    }
                    Ok(None) => {
                        // End of video
                        let _ = frame_tx.send(None);
                        break;
                    }
                    Err(_) => break,
                }
            }
        });

        self.frame_rx = Some(frame_rx);
        self.control_tx = Some(control_tx);
    }
}
```

---

### Phase 4: Integration with Main Application

**File**: `src/main.rs` (modifications)

Add video mode to main loop:

```rust
enum AppMode {
    AudioVisualization,
    VideoPlayback,
}

// In main loop:
match app_mode {
    AppMode::AudioVisualization => {
        // Existing audio visualization code
    }
    AppMode::VideoPlayback => {
        if let Some(video_player) = &mut video_player {
            if let Ok(Some(grid)) = video_player.next_frame() {
                // Convert BrailleGrid to GridBuffer
                let mut grid_buffer = GridBuffer::new(grid.width(), grid.height());

                for y in 0..grid.height() {
                    for x in 0..grid.width() {
                        let ch = grid.get_char(x, y);
                        let color = grid.get_color(x, y);

                        if ch != '⠀' {  // Not empty
                            grid_buffer.set_cell_with_color(x, y, ch, color);
                        }
                    }
                }

                renderer.render(&grid_buffer)?;

                // Frame timing
                let frame_time = Duration::from_secs_f32(1.0 / video_player.fps());
                thread::sleep(frame_time);
            }
        }
    }
}

// Keyboard controls for video mode
match key {
    KeyCode::Char('v') => {
        // Toggle video mode
        app_mode = match app_mode {
            AppMode::AudioVisualization => AppMode::VideoPlayback,
            AppMode::VideoPlayback => AppMode::AudioVisualization,
        };
    }
    KeyCode::Char(' ') if app_mode == AppMode::VideoPlayback => {
        // Play/pause toggle
        if video_player.state() == PlaybackState::Playing {
            video_player.pause();
        } else {
            video_player.resume();
        }
    }
    KeyCode::Char('+') if app_mode == AppMode::VideoPlayback => {
        // Increase speed
        let new_speed = video_player.speed * 1.2;
        video_player.set_speed(new_speed);
    }
    KeyCode::Char('-') if app_mode == AppMode::VideoPlayback => {
        // Decrease speed
        let new_speed = video_player.speed * 0.8;
        video_player.set_speed(new_speed);
    }
    // ... other controls
}
```

---

## Dependencies

**New Dependencies to Add:**

```toml
[dependencies]
# Existing dependencies...

# Video playback
ffmpeg-next = "7.0"          # FFmpeg bindings for video decoding
image = "0.25"                # Image processing and resizing
crossbeam = "0.8"             # Efficient channels for frame buffering

# Optional: for better dithering algorithms
imageproc = "0.25"            # Additional image processing utilities
```

**System Dependencies:**
- FFmpeg must be installed on user system
- Installation instructions for major platforms:
  - **Linux**: `sudo apt install ffmpeg` (Ubuntu/Debian) or `sudo dnf install ffmpeg` (Fedora)
  - **macOS**: `brew install ffmpeg`
  - **Windows**: Download from ffmpeg.org or `choco install ffmpeg`

---

## Dependencies & Integration

- **Depends on**:
  - VIZ-007 (Braille infrastructure - BrailleGrid, dot patterns)
  - Existing TerminalRenderer and GridBuffer
  - System FFmpeg installation
- **Blocks**: None (this is an optional enhancement)
- **Enables**:
  - Music video playback in terminal
  - Retro video aesthetic demonstrations
  - Live performance visuals
  - Demo scene / art installations
  - VJ-style visualizations

---

## Architecture References

- **Visualization Engine**: docs/architecture/README.md - Visualization component
- **Braille System**: src/visualization/braille.rs - High-resolution rendering
- **Source Tree**: docs/architecture/source-tree.md - visualization/ module
- **Coding Standards**: docs/architecture/coding-standards.md - Rust style guide
- **Tech Stack**: docs/architecture/tech-stack.md - Visualization tech

---

## Testing Requirements

### Unit Tests

**File**: `src/video/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_converter_threshold() {
        let mut converter = FrameConverter::new(false, false, 128);

        // Create test image: half white, half black
        let mut image = RgbImage::new(4, 4);
        for y in 0..4 {
            for x in 0..2 {
                image.put_pixel(x, y, Rgb([255, 255, 255]));  // White
            }
            for x in 2..4 {
                image.put_pixel(x, y, Rgb([0, 0, 0]));  // Black
            }
        }

        let mut grid = BrailleGrid::new(2, 1);
        converter.convert(&image, &mut grid);

        // Left cell should have dots, right cell should be empty
        assert!(!grid.is_empty(0, 0));
        assert!(grid.is_empty(1, 0));
    }

    #[test]
    fn test_floyd_steinberg_dithering() {
        let converter = FrameConverter::new(true, false, 128);

        // Create gradient image
        let mut image = RgbImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                let brightness = (x * 25) as u8;
                image.put_pixel(x, y, Rgb([brightness, brightness, brightness]));
            }
        }

        let dithered = converter.apply_floyd_steinberg_dithering(&image);

        // Dithered image should have only 0 or 255 values
        for y in 0..10 {
            for x in 0..10 {
                let pixel = dithered.get_pixel(x, y);
                assert!(pixel[0] == 0 || pixel[0] == 255);
            }
        }
    }

    #[test]
    fn test_color_averaging() {
        let converter = FrameConverter::new(false, true, 0);

        // Create test image with known colors
        let mut image = RgbImage::new(2, 4);
        for y in 0..4 {
            for x in 0..2 {
                image.put_pixel(x, y, Rgb([100, 150, 200]));
            }
        }

        let color = converter.average_block_color(&image, 0, 0);

        assert_eq!(color.r, 100);
        assert_eq!(color.g, 150);
        assert_eq!(color.b, 200);
    }
}
```

### Integration Tests

**File**: `tests/video_playback_integration_test.rs`

```rust
use crabmusic::video::{VideoPlayer, VideoConfig};

#[test]
#[ignore]  // Requires sample video file
fn test_video_playback_basic() {
    let config = VideoConfig {
        resolution: (160, 96),
        use_color: false,
        use_dithering: true,
        threshold: 128,
        target_fps: 24.0,
        buffer_size: 5,
    };

    let mut player = VideoPlayer::new("tests/fixtures/sample.mp4", config)
        .expect("Failed to open video");

    player.play().expect("Failed to start playback");

    // Get a few frames
    for _ in 0..10 {
        let frame = player.next_frame().expect("Failed to get frame");
        assert!(frame.is_some());
    }

    player.stop();
}

#[test]
#[ignore]
fn test_video_playback_color() {
    let config = VideoConfig {
        resolution: (320, 192),
        use_color: true,
        use_dithering: true,
        ..Default::default()
    };

    let mut player = VideoPlayer::new("tests/fixtures/sample.mp4", config)
        .expect("Failed to open video");

    player.play().expect("Failed to start playback");

    let frame = player.next_frame().expect("Failed to get frame");
    assert!(frame.is_some());

    let grid = frame.unwrap();

    // Check that some cells have color
    let mut has_color = false;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if grid.get_color(x, y).is_some() {
                has_color = true;
                break;
            }
        }
    }
    assert!(has_color, "Expected some cells to have color");
}
```

### Manual Testing

```bash
# Build with video support
cargo build --release --features video

# Test basic playback (monochrome, standard resolution)
crabmusic --video sample.mp4

# Test high resolution playback (512×512)
crabmusic --video sample.mp4 --resolution 512x512

# Test with color
crabmusic --video sample.mp4 --color

# Test different formats
crabmusic --video sample.avi
crabmusic --video sample.mkv
crabmusic --video sample.webm

# Test playback controls:
# - Space: pause/resume
# - +/-: speed control
# - Q: quit
# - V: switch back to audio mode
```

---

## Performance Validation

### Target Performance Metrics

**Resolution: 512×512 dots (256×128 cells)**
- **Frame decoding**: <5ms
- **Resizing**: <5ms
- **Braille conversion**: <3ms
- **Rendering**: <15ms
- **Total**: <30ms per frame = **33 FPS max** ✓

**Memory Usage:**
- Frame buffer (10 frames @ 512×512 RGB): ~3.2 MB
- BrailleGrid (256×128 cells): ~65 KB
- Decoder overhead: ~1 MB
- **Total**: <5 MB ✓

**CPU Usage:**
- Target: <30% single core
- Expected: 20-25% at 30 FPS

### Benchmarks

```rust
#[bench]
fn bench_frame_conversion_512(b: &mut Bencher) {
    let converter = FrameConverter::new(true, true, 128);
    let image = RgbImage::new(512, 512);
    let mut grid = BrailleGrid::new(256, 128);

    b.iter(|| {
        converter.convert(black_box(&image), black_box(&mut grid))
    });
}
// Expected: <3ms per conversion

#[bench]
fn bench_floyd_steinberg_512(b: &mut Bencher) {
    let converter = FrameConverter::new(true, false, 128);
    let image = RgbImage::new(512, 512);

    b.iter(|| {
        converter.apply_floyd_steinberg_dithering(black_box(&image))
    });
}
// Expected: <5ms per frame
```

---

## Algorithm Theory

### Floyd-Steinberg Dithering

**Purpose**: Improve visual quality by distributing quantization error to neighboring pixels.

**Why it's needed**: Simple threshold creates harsh black/white patterns. Dithering creates illusion of gray levels using dot patterns.

**Algorithm**:
```
For each pixel (x, y):
  old_value = pixel[x][y]
  new_value = quantize(old_value)  // 0 or 255
  error = old_value - new_value

  // Distribute error to neighbors:
  pixel[x+1][y  ] += error × 7/16  (right)
  pixel[x-1][y+1] += error × 3/16  (bottom-left)
  pixel[x  ][y+1] += error × 5/16  (bottom)
  pixel[x+1][y+1] += error × 1/16  (bottom-right)
```

**Result**: Creates visually smooth gradients from binary dot patterns.

### Color Averaging

**Challenge**: Each Braille cell can only have one color, but represents 2×4 pixels.

**Solution**: Average the RGB values of all 8 pixels in the block:

```rust
for each 2×4 pixel block:
  r_sum = sum of all red values
  g_sum = sum of all green values
  b_sum = sum of all blue values

  cell_color = (r_sum/8, g_sum/8, b_sum/8)
```

**Result**: Surprisingly good color fidelity despite low cell resolution.

---

## Retro Aesthetic Enhancements (Stretch Goals)

### CRT-Style Effects

**Scanlines**:
```rust
// Dim alternating rows slightly
for y in (0..height).step_by(2) {
    // Reduce brightness by 20%
    for x in 0..width {
        grid.set_color_dimmed(x, y, 0.8);
    }
}
```

**Phosphor Glow**:
```rust
// Bright pixels "bleed" to neighbors
for bright_pixel in bright_pixels {
    for neighbor in neighbors {
        add_glow(neighbor, brightness * 0.3);
    }
}
```

**Color Palettes**:
- **CGA**: 4-color palette (magenta, cyan, white, black)
- **EGA**: 16-color palette
- **Green terminal**: Map all colors to green phosphor
- **Amber terminal**: Map all colors to amber phosphor

---

## Notes for AI Agent

**This is an experimental but highly achievable feature!**

### Key Points

1. **Braille infrastructure is perfect** - Already has everything needed
2. **FFmpeg is proven** - Used by many Rust video projects
3. **Performance is achievable** - 24-30 FPS at 512×512 is realistic
4. **Dithering is essential** - Makes huge difference in visual quality
5. **Color averaging works** - Surprising fidelity despite cell-based coloring

### Implementation Order

1. **Start simple**: Monochrome, fixed resolution, no dithering
2. **Add dithering**: Immediate visual improvement
3. **Add color**: Per-cell color averaging
4. **Add controls**: Pause, seek, speed
5. **Optimize**: Background decoding, frame buffering
6. **Polish**: Adaptive quality, error handling

### Common Pitfalls

- Don't forget FFmpeg initialization (`ffmpeg::init()`)
- Image dimensions must match Braille dot resolution exactly
- Dithering mutates image - clone first if needed
- Terminal refresh rate can be limiting factor
- Frame timing is critical - use precise Duration
- Color averaging requires checking cell emptiness

### Success Indicators

1. Video plays smoothly at 24-30 FPS
2. Dithering creates smooth gradients (not harsh binary)
3. Colors look reasonably accurate
4. No frame drops or stuttering
5. Memory usage stays under 5MB
6. Terminal remains responsive
7. Retro aesthetic looks **amazing** 🎬

### Time Estimate

**Phase 1 (MVP)**: 8-12 hours
- FFmpeg integration
- Basic frame decoding
- Simple conversion (no dithering)
- Playback loop

**Phase 2 (Quality)**: 4-6 hours
- Floyd-Steinberg dithering
- Color support
- Optimization

**Phase 3 (Controls)**: 4-6 hours
- Pause/resume
- Seeking
- Speed control
- Background decoding

**Total**: 16-24 hours (2-3 days)

### Validation Checklist

- [ ] FFmpeg decodes video correctly
- [ ] Frames resize to target resolution
- [ ] Braille conversion produces recognizable images
- [ ] Dithering improves visual quality
- [ ] Color mode shows reasonable colors
- [ ] Playback maintains 24-30 FPS
- [ ] Controls work (pause, resume, speed)
- [ ] Memory usage under 5MB
- [ ] Retro aesthetic achieved!

**This feature would make CrabMusic absolutely legendary!** 🦀🎬
