// History TV Channel - Play videos from different historical eras (50s-2000s)
// Navigate with Up/Down to switch between random videos from different decades

use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use crate::visualization::{GridBuffer, Visualizer};
use std::time::{Duration, Instant};

/// Represents a historical era/decade
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Era {
    Fifties,      // 1950s
    Sixties,      // 1960s
    Seventies,    // 1970s
    Eighties,     // 1980s
    Nineties,     // 1990s
    TwoThousands, // 2000s
}

impl Era {
    /// Get the display name for this era
    pub fn display_name(&self) -> &'static str {
        match self {
            Era::Fifties => "1950s",
            Era::Sixties => "1960s",
            Era::Seventies => "1970s",
            Era::Eighties => "1980s",
            Era::Nineties => "1990s",
            Era::TwoThousands => "2000s",
        }
    }

    /// Get the next era (cycle forward)
    pub fn next(&self) -> Self {
        match self {
            Era::Fifties => Era::Sixties,
            Era::Sixties => Era::Seventies,
            Era::Seventies => Era::Eighties,
            Era::Eighties => Era::Nineties,
            Era::Nineties => Era::TwoThousands,
            Era::TwoThousands => Era::Fifties,
        }
    }

    /// Get the previous era (cycle backward)
    pub fn previous(&self) -> Self {
        match self {
            Era::Fifties => Era::TwoThousands,
            Era::Sixties => Era::Fifties,
            Era::Seventies => Era::Sixties,
            Era::Eighties => Era::Seventies,
            Era::Nineties => Era::Eighties,
            Era::TwoThousands => Era::Nineties,
        }
    }
}

/// Video metadata for the catalog
#[derive(Debug, Clone)]
pub struct VideoEntry {
    pub url: String,
    pub title: String,
    pub year: u16,
    pub description: String,
}

impl VideoEntry {
    pub fn new(url: &str, title: &str, year: u16, description: &str) -> Self {
        Self {
            url: url.to_string(),
            title: title.to_string(),
            year,
            description: description.to_string(),
        }
    }
}

/// Video catalog organized by era
pub struct VideoCatalog {
    fifties: Vec<VideoEntry>,
    sixties: Vec<VideoEntry>,
    seventies: Vec<VideoEntry>,
    eighties: Vec<VideoEntry>,
    nineties: Vec<VideoEntry>,
    two_thousands: Vec<VideoEntry>,
}

impl VideoCatalog {
    /// Create a new catalog with sample videos
    pub fn new() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=dQw4w9WgXcQ", // Placeholder
                    "1950s TV Commercial - Soap Advertisement",
                    1955,
                    "Classic soap commercial from the golden age of television",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example2",
                    "I Love Lucy - Classic Episode Excerpt",
                    1952,
                    "Iconic sitcom from the 1950s",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example3",
                    "1950s Newsreel - Space Race Begins",
                    1957,
                    "Historical newsreel footage",
                ),
            ],
            sixties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example4",
                    "1960s Coca-Cola Commercial",
                    1965,
                    "Vintage soda advertisement",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example5",
                    "The Twilight Zone - Opening Sequence",
                    1963,
                    "Classic sci-fi anthology series",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example6",
                    "Moon Landing Broadcast - 1969",
                    1969,
                    "Historic moment in television history",
                ),
            ],
            seventies: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example7",
                    "1970s Disco Commercial - Bell Bottoms",
                    1975,
                    "Groovy fashion advertisement",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example8",
                    "Saturday Night Live - Original Cast",
                    1976,
                    "Comedy sketch from the premiere season",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example9",
                    "Star Wars TV Spot - 1977",
                    1977,
                    "Original theatrical trailer",
                ),
            ],
            eighties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example10",
                    "MTV Launch - Video Killed the Radio Star",
                    1981,
                    "The first music video on MTV",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example11",
                    "1980s Nintendo Commercial - NES",
                    1985,
                    "Classic video game advertisement",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example12",
                    "Max Headroom - TV Excerpt",
                    1987,
                    "Cyberpunk TV personality",
                ),
            ],
            nineties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example13",
                    "Seinfeld - Classic Scene",
                    1993,
                    "Show about nothing",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example14",
                    "1990s Internet Commercial - AOL",
                    1996,
                    "You've got mail!",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example15",
                    "Friends - Central Perk Scene",
                    1998,
                    "Iconic 90s sitcom",
                ),
            ],
            two_thousands: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example16",
                    "iPod Commercial - Silhouettes",
                    2003,
                    "Revolutionary music player ad",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example17",
                    "YouTube Launch Video",
                    2005,
                    "The first video uploaded to YouTube",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=example18",
                    "iPhone Announcement - Steve Jobs",
                    2007,
                    "Historic product reveal",
                ),
            ],
        }
    }

    /// Get videos for a specific era
    pub fn get_videos(&self, era: Era) -> &Vec<VideoEntry> {
        match era {
            Era::Fifties => &self.fifties,
            Era::Sixties => &self.sixties,
            Era::Seventies => &self.seventies,
            Era::Eighties => &self.eighties,
            Era::Nineties => &self.nineties,
            Era::TwoThousands => &self.two_thousands,
        }
    }

    /// Get a mutable reference to videos for a specific era
    pub fn get_videos_mut(&mut self, era: Era) -> &mut Vec<VideoEntry> {
        match era {
            Era::Fifties => &mut self.fifties,
            Era::Sixties => &mut self.sixties,
            Era::Seventies => &mut self.seventies,
            Era::Eighties => &mut self.eighties,
            Era::Nineties => &mut self.nineties,
            Era::TwoThousands => &mut self.two_thousands,
        }
    }
}

impl Default for VideoCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// History TV Channel Visualizer
pub struct HistoryTVChannelVisualizer {
    color_scheme: ColorScheme,
    catalog: VideoCatalog,
    current_era: Era,
    current_video_index: usize,
    pulse: f32,
    animation_phase: f32,
    last_change: Instant,
    message: String,
}

impl HistoryTVChannelVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        Self {
            color_scheme,
            catalog: VideoCatalog::new(),
            current_era: Era::Fifties,
            current_video_index: 0,
            pulse: 0.0,
            animation_phase: 0.0,
            last_change: Instant::now(),
            message: String::from("History TV Channel - Use Up/Down to change videos"),
        }
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    /// Change to next video in current era
    pub fn next_video(&mut self) {
        let videos = self.catalog.get_videos(self.current_era);
        if !videos.is_empty() {
            self.current_video_index = (self.current_video_index + 1) % videos.len();
            self.update_message();
            self.last_change = Instant::now();
            tracing::info!(
                "History TV: Switched to next video in {}",
                self.current_era.display_name()
            );
        }
    }

    /// Change to previous video in current era
    pub fn previous_video(&mut self) {
        let videos = self.catalog.get_videos(self.current_era);
        if !videos.is_empty() {
            if self.current_video_index == 0 {
                self.current_video_index = videos.len() - 1;
            } else {
                self.current_video_index -= 1;
            }
            self.update_message();
            self.last_change = Instant::now();
            tracing::info!(
                "History TV: Switched to previous video in {}",
                self.current_era.display_name()
            );
        }
    }

    /// Change to a random video in the next era
    pub fn next_era(&mut self) {
        self.current_era = self.current_era.next();
        self.random_video_in_era();
        self.last_change = Instant::now();
        tracing::info!("History TV: Changed to era {}", self.current_era.display_name());
    }

    /// Change to a random video in the previous era
    pub fn previous_era(&mut self) {
        self.current_era = self.current_era.previous();
        self.random_video_in_era();
        self.last_change = Instant::now();
        tracing::info!("History TV: Changed to era {}", self.current_era.display_name());
    }

    /// Jump to a random video in the current era
    fn random_video_in_era(&mut self) {
        let videos = self.catalog.get_videos(self.current_era);
        if !videos.is_empty() {
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hash, Hasher};

            // Use current time as seed for randomness
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let mut hasher = RandomState::new().build_hasher();
            seed.hash(&mut hasher);
            let random_index = (hasher.finish() as usize) % videos.len();

            self.current_video_index = random_index;
            self.update_message();
        }
    }

    /// Update the status message based on current video
    fn update_message(&mut self) {
        let videos = self.catalog.get_videos(self.current_era);
        if let Some(video) = videos.get(self.current_video_index) {
            self.message = format!(
                "{} - {} ({}) | Up/Down: Change Video | Left/Right: Change Era",
                self.current_era.display_name(),
                video.title,
                video.year
            );
        }
    }

    /// Get the current video URL (for future video playback integration)
    pub fn get_current_video_url(&self) -> Option<String> {
        let videos = self.catalog.get_videos(self.current_era);
        videos.get(self.current_video_index).map(|v| v.url.clone())
    }

    /// Get the current video metadata
    pub fn get_current_video(&self) -> Option<&VideoEntry> {
        let videos = self.catalog.get_videos(self.current_era);
        videos.get(self.current_video_index)
    }

    /// Add a new video to the catalog
    pub fn add_video(&mut self, era: Era, video: VideoEntry) {
        self.catalog.get_videos_mut(era).push(video);
    }

    fn draw_centered(grid: &mut GridBuffer, row: usize, text: &str) {
        if row >= grid.height() {
            return;
        }
        let start_x = (grid.width().saturating_sub(text.len())) / 2;
        for (i, ch) in text.chars().enumerate() {
            let x = start_x + i;
            if x < grid.width() {
                grid.set_cell(x, row, ch);
            }
        }
    }

    fn draw_left_aligned(grid: &mut GridBuffer, row: usize, col: usize, text: &str) {
        if row >= grid.height() {
            return;
        }
        for (i, ch) in text.chars().enumerate() {
            let x = col + i;
            if x < grid.width() {
                grid.set_cell(x, row, ch);
            }
        }
    }
}

impl Visualizer for HistoryTVChannelVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Gentle pulse based on amplitude
        let target = params.amplitude.clamp(0.0, 1.0);
        self.pulse = self.pulse + (target - self.pulse) * 0.15;

        // Animation phase for visual effects
        self.animation_phase += 0.05;
        if self.animation_phase > std::f32::consts::TAU {
            self.animation_phase -= std::f32::consts::TAU;
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();

        let center_y = grid.height() / 2;
        let center_x = grid.width() / 2;

        // Draw retro TV frame
        let frame_width = (grid.width() as f32 * 0.8) as usize;
        let frame_height = (grid.height() as f32 * 0.7) as usize;
        let frame_x = (grid.width() - frame_width) / 2;
        let frame_y = (grid.height() - frame_height) / 2;

        // Top and bottom borders
        for x in frame_x..frame_x + frame_width {
            if x < grid.width() {
                if frame_y < grid.height() {
                    grid.set_cell(x, frame_y, '═');
                }
                let bottom_y = frame_y + frame_height - 1;
                if bottom_y < grid.height() {
                    grid.set_cell(x, bottom_y, '═');
                }
            }
        }

        // Left and right borders
        for y in frame_y..frame_y + frame_height {
            if y < grid.height() {
                if frame_x < grid.width() {
                    grid.set_cell(frame_x, y, '║');
                }
                let right_x = frame_x + frame_width - 1;
                if right_x < grid.width() {
                    grid.set_cell(right_x, y, '║');
                }
            }
        }

        // Corners
        if frame_x < grid.width() && frame_y < grid.height() {
            grid.set_cell(frame_x, frame_y, '╔');
        }
        let top_right_x = frame_x + frame_width - 1;
        if top_right_x < grid.width() && frame_y < grid.height() {
            grid.set_cell(top_right_x, frame_y, '╗');
        }
        let bottom_y = frame_y + frame_height - 1;
        if frame_x < grid.width() && bottom_y < grid.height() {
            grid.set_cell(frame_x, bottom_y, '╚');
        }
        if top_right_x < grid.width() && bottom_y < grid.height() {
            grid.set_cell(top_right_x, bottom_y, '╝');
        }

        // Draw title
        let title = "╣ HISTORY TV CHANNEL ╠";
        Self::draw_centered(grid, 1, title);

        // Draw current era
        let era_text = format!("Era: {}", self.current_era.display_name());
        Self::draw_centered(grid, 3, &era_text);

        // Draw current video info
        if let Some(video) = self.get_current_video() {
            let video_title = format!("▶ {}", video.title);
            Self::draw_centered(grid, center_y - 2, &video_title);

            let year_text = format!("Year: {}", video.year);
            Self::draw_centered(grid, center_y, &year_text);

            // Draw description (wrapped if needed)
            let desc_max_width = frame_width.saturating_sub(4);
            if video.description.len() <= desc_max_width {
                Self::draw_centered(grid, center_y + 2, &video.description);
            } else {
                let desc = &video.description[0..desc_max_width.min(video.description.len())];
                Self::draw_centered(grid, center_y + 2, &format!("{}...", desc));
            }

            // Show video number
            let videos = self.catalog.get_videos(self.current_era);
            let video_num = format!(
                "Video {}/{}",
                self.current_video_index + 1,
                videos.len()
            );
            Self::draw_centered(grid, center_y + 4, &video_num);
        }

        // Draw controls at bottom
        Self::draw_centered(grid, grid.height().saturating_sub(3), "Controls:");
        Self::draw_centered(grid, grid.height().saturating_sub(2), "↑↓ Change Video | ←→ Change Era | Q Quit");

        // Draw pulsing indicator (shows audio reactivity)
        let pulse_chars = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];
        let pulse_idx = ((self.animation_phase / std::f32::consts::TAU) * pulse_chars.len() as f32) as usize % pulse_chars.len();
        let pulse_x = grid.width().saturating_sub(3);
        let pulse_y = grid.height().saturating_sub(1);
        if pulse_x < grid.width() && pulse_y < grid.height() {
            grid.set_cell(pulse_x, pulse_y, pulse_chars[pulse_idx]);
        }

        // Note: Actual video playback would require integration with video player
        // For now, this shows the interface and catalog navigation
        let status = "Note: Video playback requires --video flag + file paths";
        Self::draw_centered(grid, grid.height().saturating_sub(1), status);
    }

    fn name(&self) -> &str {
        "History TV Channel"
    }
}
