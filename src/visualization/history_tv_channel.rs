// History TV Channel - Play videos from different historical eras (50s-2000s)
// Navigate with Up/Down to switch between random videos from different decades
// Multi-verse Channel Switcher - Switch between different themed content universes

use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use crate::visualization::{GridBuffer, Visualizer};
use std::time::{Duration, Instant};

/// Represents a content universe/dimension with different themes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Universe {
    RetroTV,        // Classic TV shows and historical content
    SciFi,          // Science fiction shows and movies
    MusicVideos,    // Music videos from different eras
    NewsDocumentary, // News clips and documentaries
    Commercials,    // Just commercials from all eras
    EsotericWeird,  // Strange, avant-garde, experimental content
}

impl Universe {
    /// Get the display name for this universe
    pub fn display_name(&self) -> &'static str {
        match self {
            Universe::RetroTV => "Retro TV Universe",
            Universe::SciFi => "Sci-Fi Universe",
            Universe::MusicVideos => "Music Video Universe",
            Universe::NewsDocumentary => "News & Documentary Universe",
            Universe::Commercials => "Commercials Universe",
            Universe::EsotericWeird => "Esoteric & Weird TV Universe",
        }
    }

    /// Get a short name for display in compact spaces
    pub fn short_name(&self) -> &'static str {
        match self {
            Universe::RetroTV => "RETRO TV",
            Universe::SciFi => "SCI-FI",
            Universe::MusicVideos => "MUSIC VIDEOS",
            Universe::NewsDocumentary => "NEWS & DOCS",
            Universe::Commercials => "COMMERCIALS",
            Universe::EsotericWeird => "ESOTERIC",
        }
    }

    /// Get the next universe (cycle forward)
    pub fn next(&self) -> Self {
        match self {
            Universe::RetroTV => Universe::SciFi,
            Universe::SciFi => Universe::MusicVideos,
            Universe::MusicVideos => Universe::NewsDocumentary,
            Universe::NewsDocumentary => Universe::Commercials,
            Universe::Commercials => Universe::EsotericWeird,
            Universe::EsotericWeird => Universe::RetroTV,
        }
    }

    /// Get the previous universe (cycle backward)
    pub fn previous(&self) -> Self {
        match self {
            Universe::RetroTV => Universe::EsotericWeird,
            Universe::SciFi => Universe::RetroTV,
            Universe::MusicVideos => Universe::SciFi,
            Universe::NewsDocumentary => Universe::MusicVideos,
            Universe::Commercials => Universe::NewsDocumentary,
            Universe::EsotericWeird => Universe::Commercials,
        }
    }
}

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
    pub file_path: String,  // Local file path for video playback
    pub title: String,
    pub year: u16,
    pub description: String,
}

impl VideoEntry {
    pub fn new(file_path: &str, title: &str, year: u16, description: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
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
    /// Create a new catalog with sample videos for a specific universe
    pub fn new_for_universe(universe: Universe) -> Self {
        match universe {
            Universe::RetroTV => Self::new_retro_tv(),
            Universe::SciFi => Self::new_sci_fi(),
            Universe::MusicVideos => Self::new_music_videos(),
            Universe::NewsDocumentary => Self::new_news_documentary(),
            Universe::Commercials => Self::new_commercials(),
            Universe::EsotericWeird => Self::new_esoteric_weird(),
        }
    }

    /// Create Retro TV universe catalog
    /// NOTE: Replace these paths with your own video files!
    /// Example: "/path/to/videos/1950s/soap_commercial.mp4"
    fn new_retro_tv() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "videos/retro/1950s/soap_commercial.mp4",
                    "1950s TV Commercial - Soap Advertisement",
                    1955,
                    "Classic soap commercial from the golden age of television",
                ),
                VideoEntry::new(
                    "videos/retro/1950s/i_love_lucy.mp4",
                    "I Love Lucy - Classic Episode Excerpt",
                    1952,
                    "Iconic sitcom from the 1950s",
                ),
                VideoEntry::new(
                    "videos/retro/1950s/space_race_newsreel.mp4",
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

    /// Create Sci-Fi universe catalog
    fn new_sci_fi() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi1",
                    "The Day the Earth Stood Still - Trailer",
                    1951,
                    "Classic sci-fi film about alien visitor",
                ),
            ],
            sixties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi2",
                    "Star Trek - Original Series Opening",
                    1966,
                    "To boldly go where no man has gone before",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi3",
                    "2001: A Space Odyssey - Trailer",
                    1968,
                    "Kubrick's masterpiece",
                ),
            ],
            seventies: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi4",
                    "Star Wars - Original Trailer",
                    1977,
                    "A New Hope theatrical trailer",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi5",
                    "Alien - Theatrical Trailer",
                    1979,
                    "In space, no one can hear you scream",
                ),
            ],
            eighties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi6",
                    "Blade Runner - Trailer",
                    1982,
                    "Ridley Scott's cyberpunk masterpiece",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi7",
                    "The Terminator - Trailer",
                    1984,
                    "I'll be back",
                ),
            ],
            nineties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi8",
                    "The Matrix - Trailer",
                    1999,
                    "What is the Matrix?",
                ),
            ],
            two_thousands: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=scifi9",
                    "District 9 - Trailer",
                    2009,
                    "You are not welcome here",
                ),
            ],
        }
    }

    /// Create Music Videos universe catalog
    fn new_music_videos() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=music1",
                    "Elvis Presley - Jailhouse Rock",
                    1957,
                    "The King of Rock and Roll",
                ),
            ],
            sixties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=music2",
                    "The Beatles - A Hard Day's Night",
                    1964,
                    "Beatlemania at its peak",
                ),
            ],
            seventies: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=music3",
                    "Queen - Bohemian Rhapsody",
                    1975,
                    "Is this the real life?",
                ),
            ],
            eighties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=music4",
                    "Michael Jackson - Thriller",
                    1983,
                    "The most iconic music video of all time",
                ),
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=music5",
                    "MTV - Video Killed the Radio Star",
                    1981,
                    "First music video on MTV",
                ),
            ],
            nineties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=music6",
                    "Nirvana - Smells Like Teen Spirit",
                    1991,
                    "Grunge revolution",
                ),
            ],
            two_thousands: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=music7",
                    "OK Go - Here It Goes Again",
                    2006,
                    "Viral treadmill music video",
                ),
            ],
        }
    }

    /// Create News & Documentary universe catalog
    fn new_news_documentary() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=news1",
                    "1950s Newsreel - Korean War",
                    1953,
                    "Historical news footage",
                ),
            ],
            sixties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=news2",
                    "Moon Landing Broadcast - 1969",
                    1969,
                    "One small step for man",
                ),
            ],
            seventies: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=news3",
                    "Watergate Scandal - Nixon Resigns",
                    1974,
                    "Historic presidential resignation",
                ),
            ],
            eighties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=news4",
                    "Berlin Wall Falls - 1989",
                    1989,
                    "End of the Cold War era",
                ),
            ],
            nineties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=news5",
                    "CNN - Gulf War Coverage",
                    1991,
                    "24-hour news coverage revolution",
                ),
            ],
            two_thousands: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=news6",
                    "Obama Victory Speech - 2008",
                    2008,
                    "Yes We Can",
                ),
            ],
        }
    }

    /// Create Commercials-only universe catalog
    fn new_commercials() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=comm1",
                    "Colgate Toothpaste - 1950s Ad",
                    1955,
                    "Cleans your breath while it cleans your teeth",
                ),
            ],
            sixties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=comm2",
                    "Coca-Cola - Things Go Better with Coke",
                    1963,
                    "Classic Coke jingle",
                ),
            ],
            seventies: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=comm3",
                    "McDonald's - Big Mac Jingle",
                    1975,
                    "Two all-beef patties, special sauce...",
                ),
            ],
            eighties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=comm4",
                    "Apple - 1984 Super Bowl Ad",
                    1984,
                    "Why 1984 won't be like 1984",
                ),
            ],
            nineties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=comm5",
                    "Budweiser - Whassup?!",
                    1999,
                    "Iconic beer commercial",
                ),
            ],
            two_thousands: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=comm6",
                    "Old Spice - The Man Your Man Could Smell Like",
                    2010,
                    "Viral commercial phenomenon",
                ),
            ],
        }
    }

    /// Create Esoteric & Weird TV universe catalog
    fn new_esoteric_weird() -> Self {
        Self {
            fifties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=weird1",
                    "Test Pattern - TV Static Era",
                    1955,
                    "Late night TV test patterns and color bars",
                ),
            ],
            sixties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=weird2",
                    "The Outer Limits - Opening Sequence",
                    1963,
                    "Do not attempt to adjust your television set",
                ),
            ],
            seventies: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=weird3",
                    "H.R. Pufnstuf - Psychedelic Kids Show",
                    1970,
                    "Trippy children's television",
                ),
            ],
            eighties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=weird4",
                    "Max Headroom Broadcast Signal Intrusion",
                    1987,
                    "Mysterious pirate TV broadcast",
                ),
            ],
            nineties: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=weird5",
                    "Twin Peaks - Red Room Scene",
                    1990,
                    "David Lynch's surreal masterpiece",
                ),
            ],
            two_thousands: vec![
                VideoEntry::new(
                    "https://www.youtube.com/watch?v=weird6",
                    "Adult Swim - Off the Air",
                    2011,
                    "Experimental late night programming",
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
        Self::new_for_universe(Universe::RetroTV)
    }
}

/// History TV Channel Visualizer with Multi-verse Support
pub struct HistoryTVChannelVisualizer {
    color_scheme: ColorScheme,
    current_universe: Universe,
    catalog: VideoCatalog,
    current_era: Era,
    current_video_index: usize,
    pulse: f32,
    animation_phase: f32,
    last_change: Instant,
    message: String,
    auto_play: bool,  // Auto-play mode: automatically play random videos
    auto_play_cross_era: bool,  // Allow jumping to different eras
    auto_play_cross_universe: bool,  // Allow jumping to different universes
}

impl HistoryTVChannelVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        let universe = Universe::RetroTV;
        Self {
            color_scheme,
            current_universe: universe,
            catalog: VideoCatalog::new_for_universe(universe),
            current_era: Era::Fifties,
            current_video_index: 0,
            pulse: 0.0,
            animation_phase: 0.0,
            last_change: Instant::now(),
            message: String::from("Multi-verse TV - Use PgUp/PgDn to switch universes"),
            auto_play: false,
            auto_play_cross_era: true,  // Default: can jump between eras
            auto_play_cross_universe: false,  // Default: stay in same universe
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

    /// Switch to the next universe
    pub fn next_universe(&mut self) {
        self.current_universe = self.current_universe.next();
        self.catalog = VideoCatalog::new_for_universe(self.current_universe);
        self.current_era = Era::Fifties;
        self.current_video_index = 0;
        self.update_message();
        self.last_change = Instant::now();
        tracing::info!(
            "Multi-verse TV: Jumped to {}",
            self.current_universe.display_name()
        );
    }

    /// Switch to the previous universe
    pub fn previous_universe(&mut self) {
        self.current_universe = self.current_universe.previous();
        self.catalog = VideoCatalog::new_for_universe(self.current_universe);
        self.current_era = Era::Fifties;
        self.current_video_index = 0;
        self.update_message();
        self.last_change = Instant::now();
        tracing::info!(
            "Multi-verse TV: Jumped to {}",
            self.current_universe.display_name()
        );
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

    /// Get the current video file path for playback
    pub fn get_current_video_path(&self) -> Option<String> {
        let videos = self.catalog.get_videos(self.current_era);
        videos.get(self.current_video_index).map(|v| v.file_path.clone())
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

    /// Check if current video file exists
    pub fn current_video_exists(&self) -> bool {
        if let Some(path) = self.get_current_video_path() {
            std::path::Path::new(&path).exists()
        } else {
            false
        }
    }

    /// Toggle auto-play mode
    pub fn toggle_auto_play(&mut self) {
        self.auto_play = !self.auto_play;
        tracing::info!(
            "History TV: Auto-play {}",
            if self.auto_play { "ENABLED" } else { "DISABLED" }
        );
    }

    /// Check if auto-play is enabled
    pub fn is_auto_play(&self) -> bool {
        self.auto_play
    }

    /// Toggle cross-era mode for auto-play
    pub fn toggle_cross_era(&mut self) {
        self.auto_play_cross_era = !self.auto_play_cross_era;
        tracing::info!(
            "History TV: Cross-era {}",
            if self.auto_play_cross_era { "ENABLED" } else { "DISABLED" }
        );
    }

    /// Toggle cross-universe mode for auto-play
    pub fn toggle_cross_universe(&mut self) {
        self.auto_play_cross_universe = !self.auto_play_cross_universe;
        tracing::info!(
            "History TV: Cross-universe {}",
            if self.auto_play_cross_universe { "ENABLED" } else { "DISABLED" }
        );
    }

    /// Select a random video for auto-play
    /// Returns the file path of the selected video
    pub fn select_random_video(&mut self) -> Option<String> {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut hasher = RandomState::new().build_hasher();

        // Randomly decide if we should change universe (if allowed)
        if self.auto_play_cross_universe {
            seed.hash(&mut hasher);
            let should_change_universe = (hasher.finish() % 3) == 0; // 33% chance
            if should_change_universe {
                // Pick random universe
                let universe_choices = [
                    Universe::RetroTV,
                    Universe::SciFi,
                    Universe::MusicVideos,
                    Universe::NewsDocumentary,
                    Universe::Commercials,
                    Universe::EsotericWeird,
                ];
                seed.wrapping_add(1).hash(&mut hasher);
                let universe_idx = (hasher.finish() as usize) % universe_choices.len();
                self.current_universe = universe_choices[universe_idx];
                self.catalog = VideoCatalog::new_for_universe(self.current_universe);
                tracing::info!(
                    "Auto-play: Jumped to {}",
                    self.current_universe.display_name()
                );
            }
        }

        // Randomly decide if we should change era (if allowed)
        if self.auto_play_cross_era {
            seed.wrapping_add(2).hash(&mut hasher);
            let should_change_era = (hasher.finish() % 2) == 0; // 50% chance
            if should_change_era {
                let era_choices = [
                    Era::Fifties,
                    Era::Sixties,
                    Era::Seventies,
                    Era::Eighties,
                    Era::Nineties,
                    Era::TwoThousands,
                ];
                seed.wrapping_add(3).hash(&mut hasher);
                let era_idx = (hasher.finish() as usize) % era_choices.len();
                self.current_era = era_choices[era_idx];
                tracing::info!("Auto-play: Jumped to era {}", self.current_era.display_name());
            }
        }

        // Select random video from current era
        let videos = self.catalog.get_videos(self.current_era);
        if videos.is_empty() {
            return None;
        }

        seed.wrapping_add(4).hash(&mut hasher);
        self.current_video_index = (hasher.finish() as usize) % videos.len();

        self.update_message();
        self.last_change = Instant::now();

        if let Some(video) = self.get_current_video() {
            tracing::info!(
                "Auto-play: Selected '{}' ({}) from {} - {}",
                video.title,
                video.year,
                self.current_universe.display_name(),
                self.current_era.display_name()
            );
        }

        self.get_current_video_path()
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

        // Draw title with universe
        let title = format!("╣ {} ╠", self.current_universe.short_name());
        Self::draw_centered(grid, 1, &title);

        // Draw full universe name
        Self::draw_centered(grid, 2, self.current_universe.display_name());

        // Draw current era
        let era_text = format!("Era: {}", self.current_era.display_name());
        Self::draw_centered(grid, 4, &era_text);

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
        Self::draw_centered(grid, grid.height().saturating_sub(6), "Controls:");
        Self::draw_centered(grid, grid.height().saturating_sub(5), "↑↓ Video | ←→ Era | PgUp/PgDn Universe | SPACE Auto-Play");

        // Show auto-play status
        let auto_play_status = if self.auto_play {
            let mode = if self.auto_play_cross_universe {
                "ALL UNIVERSES"
            } else if self.auto_play_cross_era {
                "ALL ERAS"
            } else {
                "SAME ERA"
            };
            format!("🎲 AUTO-PLAY ON ({}) 🎲", mode)
        } else {
            String::from("AUTO-PLAY OFF (Press SPACE to enable)")
        };
        Self::draw_centered(grid, grid.height().saturating_sub(4), &auto_play_status);

        // Show play status
        let play_status = if self.current_video_exists() {
            "ENTER Play Video (ASCII Braille) | Q Quit"
        } else {
            "File not found - Add videos to play! | Q Quit"
        };
        Self::draw_centered(grid, grid.height().saturating_sub(3), play_status);
        Self::draw_centered(grid, grid.height().saturating_sub(2), &format!("Universe: {} of 6", (self.current_universe as usize) + 1));

        // Draw pulsing indicator (shows audio reactivity)
        let pulse_chars = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];
        let pulse_idx = ((self.animation_phase / std::f32::consts::TAU) * pulse_chars.len() as f32) as usize % pulse_chars.len();
        let pulse_x = grid.width().saturating_sub(3);
        let pulse_y = grid.height().saturating_sub(1);
        if pulse_x < grid.width() && pulse_y < grid.height() {
            grid.set_cell(pulse_x, pulse_y, pulse_chars[pulse_idx]);
        }

        // Show file path at bottom
        if let Some(path) = self.get_current_video_path() {
            let path_display = if path.len() > grid.width().saturating_sub(10) {
                format!("...{}", &path[path.len().saturating_sub(grid.width() - 13)..])
            } else {
                path
            };
            Self::draw_centered(grid, grid.height().saturating_sub(1), &format!("File: {}", path_display));
        }
    }

    fn name(&self) -> &str {
        "Multi-verse History TV"
    }
}
