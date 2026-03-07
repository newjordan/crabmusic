// CrabMusic - Real-time ASCII music visualizer
// Main application entry point

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

// Module declarations
mod audio;
mod braille_quality;
mod config;
mod dsp;
#[allow(dead_code)]
mod effects;
mod error;
mod grid_postprocess;
#[allow(dead_code)]
mod img;
mod rendering;
mod runtime_controls;
mod video;
mod visualization;

#[cfg(windows)]
use audio::WasapiLoopbackDevice;
use audio::{
    AudioCaptureDevice, AudioOutputDevice, AudioRingBuffer, CpalAudioDevice, SilentAudioDevice,
};
use config::{AppConfig, BrailleColorMode, InternetArchiveConfig, RenderingConfig};
use dsp::DspProcessor;
use effects::EffectPipeline;
use grid_postprocess::GridBraillePostProcessor;
use rendering::TerminalRenderer;
use runtime_controls::{
    apply_quality_action, quality_control_action_from_key, quality_summary, ColorMode,
    QualityControlAction,
};
use visualization::{
    character_sets::CharacterSet, color_schemes::ColorScheme, primitives::PrimitivesVisualizer,
    ray_tracer::RenderMode, GravityWellVisualizer, GridBuffer, GridTunnelVisualizer,
    ImageChannelVisualizer, NightNightVisualizer, ObjViewerVisualizer, OscilloscopeConfig,
    OscilloscopeVisualizer, Raycaster3DVisualizer, SineWaveVisualizer, SpectrogramVisualizer,
    SpectrumConfig, SpectrumMapping, SpectrumVisualizer, StarfieldVisualizer,
    TerrainLandscapeVisualizer, TriggerSlope, VideoChannelVisualizer, Visualizer, WaveformMode,
    WaveformTunnelVisualizer, XYOscilloscopeConfig, XYOscilloscopeVisualizer,
};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    /// Audio device name
    #[arg(short, long)]
    device: Option<String>,

    /// List available audio devices
    #[arg(long)]
    list_devices: bool,

    /// Test mode (no audio)
    #[arg(long)]
    test: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Debug logging
    #[arg(long)]
    debug: bool,

    /// Play video/GIF file (path to video file)
    #[arg(long)]
    video: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizerMode {
    SineWave,
    Spectrum,
    Oscilloscope,
    XYOscilloscope,
    Raycaster3D,
    ObjViewer,
    Primitives,
    GridTunnel,
    GravityWell,
    WaveformTunnel,
    TerrainLandscape,
    Starfield,
    Spectrogram,
    NightNight,
    Image,
    Video,
    InternetArchive,
    ArchiveCooking,
    ArchivePublicAccess,
    ArchiveIndustrial,
    ArchiveEducational,
    ArchiveLocalNews,
}

impl VisualizerMode {
    fn next(&self) -> Self {
        match self {
            VisualizerMode::SineWave => VisualizerMode::Spectrum,
            VisualizerMode::Spectrum => VisualizerMode::Oscilloscope,
            VisualizerMode::Oscilloscope => VisualizerMode::XYOscilloscope,
            VisualizerMode::XYOscilloscope => VisualizerMode::Raycaster3D,
            VisualizerMode::Raycaster3D => VisualizerMode::ObjViewer,
            VisualizerMode::ObjViewer => VisualizerMode::Primitives,
            VisualizerMode::Primitives => VisualizerMode::GridTunnel,
            VisualizerMode::GridTunnel => VisualizerMode::GravityWell,
            VisualizerMode::GravityWell => VisualizerMode::WaveformTunnel,
            VisualizerMode::WaveformTunnel => VisualizerMode::TerrainLandscape,
            VisualizerMode::TerrainLandscape => VisualizerMode::Starfield,
            VisualizerMode::Starfield => VisualizerMode::Spectrogram,
            VisualizerMode::Spectrogram => VisualizerMode::NightNight,
            VisualizerMode::NightNight => VisualizerMode::Image,
            VisualizerMode::Image => VisualizerMode::Video,
            VisualizerMode::Video => VisualizerMode::InternetArchive,
            VisualizerMode::InternetArchive => VisualizerMode::ArchiveCooking,
            VisualizerMode::ArchiveCooking => VisualizerMode::ArchivePublicAccess,
            VisualizerMode::ArchivePublicAccess => VisualizerMode::ArchiveIndustrial,
            VisualizerMode::ArchiveIndustrial => VisualizerMode::ArchiveEducational,
            VisualizerMode::ArchiveEducational => VisualizerMode::ArchiveLocalNews,
            VisualizerMode::ArchiveLocalNews => VisualizerMode::SineWave,
        }
    }

    fn prev(&self) -> Self {
        match self {
            VisualizerMode::SineWave => VisualizerMode::ArchiveLocalNews,
            VisualizerMode::Spectrum => VisualizerMode::SineWave,
            VisualizerMode::Oscilloscope => VisualizerMode::Spectrum,
            VisualizerMode::XYOscilloscope => VisualizerMode::Oscilloscope,
            VisualizerMode::Raycaster3D => VisualizerMode::XYOscilloscope,
            VisualizerMode::ObjViewer => VisualizerMode::Raycaster3D,
            VisualizerMode::Primitives => VisualizerMode::ObjViewer,
            VisualizerMode::GridTunnel => VisualizerMode::Primitives,
            VisualizerMode::GravityWell => VisualizerMode::GridTunnel,
            VisualizerMode::WaveformTunnel => VisualizerMode::GravityWell,
            VisualizerMode::TerrainLandscape => VisualizerMode::WaveformTunnel,
            VisualizerMode::Starfield => VisualizerMode::TerrainLandscape,
            VisualizerMode::Spectrogram => VisualizerMode::Starfield,
            VisualizerMode::NightNight => VisualizerMode::Spectrogram,
            VisualizerMode::Image => VisualizerMode::NightNight,
            VisualizerMode::Video => VisualizerMode::Image,
            VisualizerMode::InternetArchive => VisualizerMode::Video,
            VisualizerMode::ArchiveCooking => VisualizerMode::InternetArchive,
            VisualizerMode::ArchivePublicAccess => VisualizerMode::ArchiveCooking,
            VisualizerMode::ArchiveIndustrial => VisualizerMode::ArchivePublicAccess,
            VisualizerMode::ArchiveEducational => VisualizerMode::ArchiveIndustrial,
            VisualizerMode::ArchiveLocalNews => VisualizerMode::ArchiveEducational,
        }
    }

    #[allow(dead_code)]
    fn name(&self) -> &'static str {
        match self {
            VisualizerMode::SineWave => "Sine Wave",
            VisualizerMode::Spectrum => "Spectrum",
            VisualizerMode::Oscilloscope => "Oscilloscope",
            VisualizerMode::XYOscilloscope => "XY Oscilloscope",
            VisualizerMode::Raycaster3D => "Raycaster 3D",
            VisualizerMode::ObjViewer => "3D Model Viewer",
            VisualizerMode::Primitives => "Primitives 3D",
            VisualizerMode::GridTunnel => "Grid Tunnel",
            VisualizerMode::GravityWell => "Gravity Well",
            VisualizerMode::WaveformTunnel => "Waveform Tunnel",
            VisualizerMode::TerrainLandscape => "Terrain Landscape",
            VisualizerMode::Starfield => "Starfield",
            VisualizerMode::Spectrogram => "Spectrogram",
            VisualizerMode::NightNight => "Night Night",
            VisualizerMode::Image => "Image Mode",
            VisualizerMode::Video => "Video Mode",
            VisualizerMode::InternetArchive => "Archive TV",
            VisualizerMode::ArchiveCooking => "Archive Cooking",
            VisualizerMode::ArchivePublicAccess => "Archive Public Access",
            VisualizerMode::ArchiveIndustrial => "Archive Industrial Films",
            VisualizerMode::ArchiveEducational => "Archive Educational",
            VisualizerMode::ArchiveLocalNews => "Archive Local News",
        }
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn count() -> usize {
        22 // Total number of visualizer modes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ArchiveChannelKind {
    Tv,
    Cooking,
    PublicAccess,
    Industrial,
    Educational,
    LocalNews,
}

impl ArchiveChannelKind {
    fn from_visualizer_mode(mode: VisualizerMode) -> Option<Self> {
        match mode {
            VisualizerMode::InternetArchive => Some(Self::Tv),
            VisualizerMode::ArchiveCooking => Some(Self::Cooking),
            VisualizerMode::ArchivePublicAccess => Some(Self::PublicAccess),
            VisualizerMode::ArchiveIndustrial => Some(Self::Industrial),
            VisualizerMode::ArchiveEducational => Some(Self::Educational),
            VisualizerMode::ArchiveLocalNews => Some(Self::LocalNews),
            _ => None,
        }
    }

    fn visualizer_mode(self) -> VisualizerMode {
        match self {
            Self::Tv => VisualizerMode::InternetArchive,
            Self::Cooking => VisualizerMode::ArchiveCooking,
            Self::PublicAccess => VisualizerMode::ArchivePublicAccess,
            Self::Industrial => VisualizerMode::ArchiveIndustrial,
            Self::Educational => VisualizerMode::ArchiveEducational,
            Self::LocalNews => VisualizerMode::ArchiveLocalNews,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Tv => "Archive TV",
            Self::Cooking => "Archive Cooking",
            Self::PublicAccess => "Archive Public Access",
            Self::Industrial => "Archive Industrial Films",
            Self::Educational => "Archive Educational",
            Self::LocalNews => "Archive Local News",
        }
    }

    fn idle_label(self) -> &'static str {
        match self {
            Self::Tv => "Random old TV from Internet Archive • press U to retune",
            Self::Cooking => "Random vintage cooking shows • press U to retune",
            Self::PublicAccess => "Random public-access oddities • press U to retune",
            Self::Industrial => "Random industrial and training films • press U to retune",
            Self::Educational => "Random educational films • press U to retune",
            Self::LocalNews => "Random local news oddities • press U to retune",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveRequestContext {
    ManualHotkey,
    RotationMode,
    PlaybackEnded,
}

impl ArchiveRequestContext {
    fn requires_active_channel_match(self) -> bool {
        matches!(self, Self::RotationMode | Self::PlaybackEnded)
    }
}

fn is_quit_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
}

fn is_channel_navigation_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Right | KeyCode::Left | KeyCode::Char('v') | KeyCode::Char('V')
    )
}

fn should_process_key_event(
    code: KeyCode,
    kind: KeyEventKind,
    time_since_last_press: Duration,
    debounce_ms: u64,
) -> bool {
    if is_channel_navigation_key(code) {
        return kind == KeyEventKind::Press;
    }

    if is_quit_key(code) {
        return true;
    }

    time_since_last_press.as_millis() >= debounce_ms as u128
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AudioRuntimeFlags {
    use_loopback: bool,
    microphone_enabled: bool,
}

fn runtime_audio_flags(capture_enabled: bool, requested_use_loopback: bool) -> AudioRuntimeFlags {
    if capture_enabled {
        AudioRuntimeFlags {
            use_loopback: requested_use_loopback,
            microphone_enabled: !requested_use_loopback,
        }
    } else {
        AudioRuntimeFlags {
            use_loopback: false,
            microphone_enabled: false,
        }
    }
}

fn create_silent_audio_device(
    buffer_capacity: usize,
    sample_rate: u32,
    channels: u16,
) -> Result<Box<dyn AudioCaptureDevice>> {
    Ok(Box::new(SilentAudioDevice::new(
        Arc::new(AudioRingBuffer::new(buffer_capacity)),
        sample_rate,
        channels,
    )?))
}

fn create_audio_capture_device(
    args: &Args,
    buffer_capacity: usize,
    sample_rate: u32,
    channels: u16,
    requested_use_loopback: bool,
    configured_device_name: Option<String>,
) -> Result<Box<dyn AudioCaptureDevice>> {
    if args.test {
        return create_silent_audio_device(buffer_capacity, sample_rate, channels);
    }

    let ring_buffer = Arc::new(AudioRingBuffer::new(buffer_capacity));

    if requested_use_loopback {
        #[cfg(windows)]
        {
            return Ok(Box::new(WasapiLoopbackDevice::new(ring_buffer)?));
        }

        #[cfg(not(windows))]
        {
            anyhow::bail!("Audio loopback capture is only supported on Windows")
        }
    }

    Ok(Box::new(CpalAudioDevice::new_with_device(
        ring_buffer,
        args.device.clone().or(configured_device_name),
    )?))
}

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

struct Application {
    audio_device: Box<dyn AudioCaptureDevice>,
    audio_output: Option<AudioOutputDevice>,
    dsp_processor: DspProcessor,
    visualizer: Box<dyn Visualizer>,
    visualizer_mode: VisualizerMode,
    renderer: TerminalRenderer,
    effect_pipeline: EffectPipeline,
    color_scheme: ColorScheme,
    target_fps: u32,
    microphone_enabled: bool,
    show_channel_number: bool,
    sensitivity_multiplier: f32,
    last_key_press: Instant,
    key_debounce_ms: u64,
    file_prompt_active: bool,
    file_prompt_buffer: String,
    file_prompt_error: Option<String>,
    paste_suppress_deadline: Option<Instant>,

    // Raycaster3D specific
    ray3d_mode: RenderMode,
    ray3d_wire_step_rad: f32,
    ray3d_wire_tol_rad: f32,
    ray3d_rotation_speed_y: f32,
    ray3d_auto_rotate: bool,
    ray3d_brightness_boost: f32,

    // ObjViewer specific
    model_viewer_auto_rotate: bool,

    // Spectrum specific
    spectrum_mapping: SpectrumMapping,
    spectrum_range_preset_index: usize,
    spectrum_peak_hold: bool,
    show_labels: bool,

    // Oscilloscope specific
    osc_show_grid: bool,
    osc_waveform_mode: WaveformMode,
    osc_trigger_slope: TriggerSlope,

    current_charset: CharacterSet,
    live_quality: braille_quality::BrailleQualitySettings,
    live_quality_defaults: braille_quality::BrailleQualitySettings,
    live_color_mode: ColorMode,
    live_color_mode_default: ColorMode,
    live_postprocess: GridBraillePostProcessor,

    // Config
    audio_buffer_capacity: usize,
    use_loopback: bool,
    rendering_config: RenderingConfig,

    // Internet Archive specific
    internet_archive: InternetArchiveConfig,
    archive_stream_tx: mpsc::Sender<Result<crate::video::internet_archive::ArchiveStream>>,
    archive_stream_rx: mpsc::Receiver<Result<crate::video::internet_archive::ArchiveStream>>,
    is_loading_archive: bool,
    pending_archive_channel: ArchiveChannelKind,
    archive_request_context: ArchiveRequestContext,
    last_archive_identifiers: HashMap<ArchiveChannelKind, String>,
}

fn color_mode_from_config(color_mode: BrailleColorMode) -> ColorMode {
    match color_mode {
        BrailleColorMode::Off => ColorMode::Off,
        BrailleColorMode::Grayscale => ColorMode::Grayscale,
        BrailleColorMode::Full => ColorMode::Full,
    }
}

impl Application {
    fn fallback_to_audio_off(&mut self) -> Result<()> {
        let audio_config = self.audio_device.get_config();
        self.audio_output = None;
        self.use_loopback = false;
        self.microphone_enabled = false;
        self.audio_device = create_silent_audio_device(
            self.audio_buffer_capacity,
            audio_config.sample_rate,
            audio_config.channels,
        )?;
        Ok(())
    }

    fn new(config: AppConfig, args: &Args) -> Result<Self> {
        let renderer = TerminalRenderer::new()?;
        let rendering_config = config.rendering.clone();
        let requested_use_loopback = config.audio.resolved_use_loopback();
        let mut audio_flags = runtime_audio_flags(true, requested_use_loopback);

        let audio_device = match create_audio_capture_device(
            args,
            config.audio.buffer_capacity,
            config.audio.sample_rate,
            config.audio.channels,
            requested_use_loopback,
            config.audio.device_name.clone(),
        ) {
            Ok(device) => device,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "Failed to initialize audio capture; falling back to audio-off mode"
                );
                audio_flags = runtime_audio_flags(false, requested_use_loopback);
                create_silent_audio_device(
                    config.audio.buffer_capacity,
                    config.audio.sample_rate,
                    config.audio.channels,
                )?
            }
        };

        // Initialize audio output if microphone is enabled (for passthrough)
        // or if we want to support playback in the future
        let audio_output = if audio_flags.microphone_enabled {
            match AudioOutputDevice::new_with_device(config.audio.output_device_name.clone()) {
                Ok(device) => Some(device),
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        "Failed to initialize audio playback; continuing with playback off"
                    );
                    None
                }
            }
        } else {
            None
        };

        let dsp_processor = DspProcessor::new(config.audio.sample_rate, config.dsp.fft_size)?;

        let color_scheme = ColorScheme::default();
        let current_charset =
            CharacterSet::from_name(&config.visualization.character_set).unwrap_or_default();
        let live_quality = rendering_config.live_braille_quality();
        let live_color_mode = color_mode_from_config(rendering_config.live_color_mode());

        // Default visualizer
        let sine_config = visualization::SineWaveConfig {
            amplitude_sensitivity: config.visualization.sine_wave.amplitude,
            frequency_sensitivity: config.visualization.sine_wave.frequency,
            thickness_sensitivity: 5.0, // Default value
            base_frequency: 1.0,
            smoothing_factor: config.visualization.sine_wave.smoothing,
            phase_speed: 0.1,
        };

        let visualizer = Box::new(SineWaveVisualizer::new(
            sine_config,
            current_charset.clone(),
        ));

        // Create Internet Archive stream channel
        let (tx, rx) = mpsc::channel(1);

        Ok(Self {
            audio_device,
            audio_output,
            dsp_processor,
            visualizer,
            visualizer_mode: VisualizerMode::SineWave,
            renderer,
            effect_pipeline: EffectPipeline::new(),
            color_scheme,
            target_fps: config.rendering.target_fps,
            microphone_enabled: audio_flags.microphone_enabled,
            show_channel_number: false,
            sensitivity_multiplier: 1.0,
            last_key_press: Instant::now(),
            key_debounce_ms: 200,
            file_prompt_active: false,
            file_prompt_buffer: String::new(),
            file_prompt_error: None,
            paste_suppress_deadline: None,

            ray3d_mode: RenderMode::Wireframe {
                step_rad: visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
                tol_rad: visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            },
            ray3d_wire_step_rad: visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
            ray3d_wire_tol_rad: visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            ray3d_rotation_speed_y: 0.5,
            ray3d_auto_rotate: true,
            ray3d_brightness_boost: 0.0,

            model_viewer_auto_rotate: true,

            spectrum_mapping: SpectrumMapping::NoteBars,
            spectrum_range_preset_index: 0,
            spectrum_peak_hold: true,
            show_labels: true,

            osc_show_grid: true,
            osc_waveform_mode: WaveformMode::Line,
            osc_trigger_slope: TriggerSlope::Positive,

            current_charset,
            live_quality,
            live_quality_defaults: live_quality,
            live_color_mode,
            live_color_mode_default: live_color_mode,
            live_postprocess: GridBraillePostProcessor,
            audio_buffer_capacity: config.audio.buffer_capacity,
            use_loopback: audio_flags.use_loopback,
            rendering_config,

            internet_archive: config.internet_archive.clone(),
            archive_stream_tx: tx,
            archive_stream_rx: rx,
            is_loading_archive: false,
            pending_archive_channel: ArchiveChannelKind::Tv,
            archive_request_context: ArchiveRequestContext::ManualHotkey,
            last_archive_identifiers: HashMap::new(),
        })
    }

    fn play_video_stream(&mut self, path: &str) -> Result<()> {
        let archive_channel = ArchiveChannelKind::from_visualizer_mode(self.visualizer_mode);
        let prepared = video::prepare_video_input(path)
            .with_context(|| format!("Failed to prepare video input: {path}"))?;
        if let Some(label) = &prepared.display_label {
            tracing::info!("{label}");
        }
        self.renderer
            .cleanup()
            .context("Failed to hand off terminal to video playback")?;

        let playback_result = video::run_video_playback_once_with_config(
            &prepared.playback_target,
            &self.rendering_config,
            archive_channel.is_some(),
        );

        self.renderer = TerminalRenderer::new().context("Failed to restore main renderer")?;
        self.last_key_press = Instant::now();

        match playback_result.with_context(|| format!("Failed to play video input: {path}"))? {
            video::VideoPlaybackExit::UserQuit => Ok(()),
            video::VideoPlaybackExit::NextChannel => {
                self.next_visualizer_mode();
                Ok(())
            }
            video::VideoPlaybackExit::PreviousChannel => {
                self.prev_visualizer_mode();
                Ok(())
            }
            video::VideoPlaybackExit::RetuneArchive => {
                if let Some(channel) =
                    ArchiveChannelKind::from_visualizer_mode(self.visualizer_mode)
                {
                    self.request_archive_stream_for(channel, ArchiveRequestContext::ManualHotkey);
                }
                Ok(())
            }
            video::VideoPlaybackExit::EndOfStream => {
                if let Some(channel) = archive_channel {
                    if self.visualizer_mode == channel.visualizer_mode() {
                        self.request_archive_stream_for(
                            channel,
                            ArchiveRequestContext::PlaybackEnded,
                        );
                    }
                }
                Ok(())
            }
        }
    }

    fn next_visualizer_mode(&mut self) {
        self.visualizer_mode = self.visualizer_mode.next();
        self.recreate_visualizer();
        self.autotune_archive_mode();
    }

    fn prev_visualizer_mode(&mut self) {
        self.visualizer_mode = self.visualizer_mode.prev();
        self.recreate_visualizer();
        self.autotune_archive_mode();
    }

    fn autotune_archive_mode(&mut self) {
        if let Some(channel) = ArchiveChannelKind::from_visualizer_mode(self.visualizer_mode) {
            self.request_archive_stream_for(channel, ArchiveRequestContext::RotationMode);
        }
    }

    fn request_archive_stream(&mut self, context: ArchiveRequestContext) {
        let channel = ArchiveChannelKind::from_visualizer_mode(self.visualizer_mode)
            .unwrap_or(ArchiveChannelKind::Tv);
        self.request_archive_stream_for(channel, context);
    }

    fn request_archive_stream_for(
        &mut self,
        channel: ArchiveChannelKind,
        context: ArchiveRequestContext,
    ) {
        let queries = match channel {
            ArchiveChannelKind::Tv => &self.internet_archive.tv_queries,
            ArchiveChannelKind::Cooking => &self.internet_archive.cooking_queries,
            ArchiveChannelKind::PublicAccess => &self.internet_archive.public_access_queries,
            ArchiveChannelKind::Industrial => &self.internet_archive.industrial_queries,
            ArchiveChannelKind::Educational => &self.internet_archive.educational_queries,
            ArchiveChannelKind::LocalNews => &self.internet_archive.local_news_queries,
        };

        if queries.is_empty() {
            tracing::warn!(
                "No Internet Archive queries configured for {}",
                channel.name()
            );
            return;
        }

        if self.is_loading_archive {
            tracing::warn!("Already loading an Internet Archive video.");
            return;
        }

        self.is_loading_archive = true;
        self.pending_archive_channel = channel;
        self.archive_request_context = context;
        let excluded_identifier = self.last_archive_identifiers.get(&channel).cloned();

        tracing::info!(
            "Requesting random Internet Archive video for {} ({})...",
            channel.name(),
            match context {
                ArchiveRequestContext::ManualHotkey => "manual",
                ArchiveRequestContext::RotationMode => "rotation",
                ArchiveRequestContext::PlaybackEnded => "autoplay",
            }
        );

        let queries = queries.to_vec();
        let rows_per_page = self.internet_archive.rows_per_page;
        let max_pages = self.internet_archive.max_pages;
        let tx = self.archive_stream_tx.clone();
        tokio::spawn(async move {
            let result = match tokio::task::spawn_blocking(move || {
                crate::video::internet_archive::get_random_video_stream_excluding(
                    &queries,
                    rows_per_page,
                    max_pages,
                    excluded_identifier.as_deref(),
                )
            })
            .await
            {
                Ok(result) => result,
                Err(e) => Err(anyhow!("Internet Archive fetch task failed: {}", e)),
            };

            if let Err(e) = tx.send(result).await {
                tracing::error!(
                    "Failed to send Internet Archive stream back to main thread: {}",
                    e
                );
            }
        });
    }

    fn recreate_visualizer(&mut self) {
        self.visualizer = match self.visualizer_mode {
            VisualizerMode::SineWave => {
                let mut config = visualization::SineWaveConfig::default();
                config.amplitude_sensitivity *= self.sensitivity_multiplier;
                let mut viz = SineWaveVisualizer::new(config, self.current_charset.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Spectrum => {
                let config = SpectrumConfig::default();
                let sample_rate = self.audio_device.get_config().sample_rate;
                let mut viz =
                    SpectrumVisualizer::new(config, sample_rate, self.current_charset.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                viz.set_mapping(match self.spectrum_mapping {
                    SpectrumMapping::NoteBars => visualization::SpectrumMapping::NoteBars,
                    SpectrumMapping::LogBars => visualization::SpectrumMapping::LogBars,
                });
                viz.set_peak_hold(self.spectrum_peak_hold);
                viz.set_show_labels(self.show_labels);
                Box::new(viz)
            }
            VisualizerMode::Oscilloscope => {
                let config = OscilloscopeConfig::default();
                let mut viz = OscilloscopeVisualizer::new(config);
                viz.set_color_scheme(self.color_scheme.clone());
                viz.set_show_grid(self.osc_show_grid);
                Box::new(viz)
            }
            VisualizerMode::XYOscilloscope => {
                let config = XYOscilloscopeConfig::default();
                let mut viz = XYOscilloscopeVisualizer::new(config);
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Raycaster3D => {
                let mode = match self.ray3d_mode {
                    RenderMode::Wireframe { .. } => RenderMode::Wireframe {
                        step_rad: self.ray3d_wire_step_rad,
                        tol_rad: self.ray3d_wire_tol_rad,
                    },
                    RenderMode::Solid => RenderMode::Solid,
                };
                let mut viz = Raycaster3DVisualizer::new_with(mode, self.ray3d_brightness_boost);
                viz.set_rotation_speed_y(self.ray3d_rotation_speed_y);
                viz.set_auto_rotate(self.ray3d_auto_rotate);
                Box::new(viz)
            }
            VisualizerMode::ObjViewer => {
                let mut viz = ObjViewerVisualizer::new_with_model_index(0);
                viz.set_auto_rotate(self.model_viewer_auto_rotate);
                Box::new(viz)
            }
            VisualizerMode::Primitives => {
                let mut viz = PrimitivesVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::GridTunnel => {
                let mut viz = GridTunnelVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::GravityWell => {
                let mut viz = GravityWellVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::WaveformTunnel => {
                let mut viz = WaveformTunnelVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::TerrainLandscape => {
                let mut viz = TerrainLandscapeVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Starfield => {
                let mut viz = StarfieldVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Spectrogram => {
                let mut viz = SpectrogramVisualizer::new(
                    self.color_scheme.clone(),
                    visualization::ScrollDirection::Up,
                );
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::NightNight => {
                Box::new(NightNightVisualizer::new(self.color_scheme.clone()))
            }
            VisualizerMode::Image => {
                let mut viz = ImageChannelVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::Video => {
                let mut viz = VideoChannelVisualizer::new(self.color_scheme.clone());
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::InternetArchive => {
                let mut viz = VideoChannelVisualizer::new_named(
                    self.color_scheme.clone(),
                    ArchiveChannelKind::Tv.name(),
                    ArchiveChannelKind::Tv.idle_label(),
                );
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::ArchiveCooking => {
                let mut viz = VideoChannelVisualizer::new_named(
                    self.color_scheme.clone(),
                    ArchiveChannelKind::Cooking.name(),
                    ArchiveChannelKind::Cooking.idle_label(),
                );
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::ArchivePublicAccess => {
                let mut viz = VideoChannelVisualizer::new_named(
                    self.color_scheme.clone(),
                    ArchiveChannelKind::PublicAccess.name(),
                    ArchiveChannelKind::PublicAccess.idle_label(),
                );
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::ArchiveIndustrial => {
                let mut viz = VideoChannelVisualizer::new_named(
                    self.color_scheme.clone(),
                    ArchiveChannelKind::Industrial.name(),
                    ArchiveChannelKind::Industrial.idle_label(),
                );
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::ArchiveEducational => {
                let mut viz = VideoChannelVisualizer::new_named(
                    self.color_scheme.clone(),
                    ArchiveChannelKind::Educational.name(),
                    ArchiveChannelKind::Educational.idle_label(),
                );
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
            VisualizerMode::ArchiveLocalNews => {
                let mut viz = VideoChannelVisualizer::new_named(
                    self.color_scheme.clone(),
                    ArchiveChannelKind::LocalNews.name(),
                    ArchiveChannelKind::LocalNews.idle_label(),
                );
                viz.set_color_scheme(self.color_scheme.clone());
                Box::new(viz)
            }
        };
    }
    fn next_color_scheme(&mut self) {
        self.color_scheme = self.color_scheme.next();
        // Update visualizer with new color scheme
        // Note: Some visualizers might need explicit update
        if let Some(viz) =
            (&mut *self.visualizer as &mut dyn std::any::Any).downcast_mut::<SineWaveVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) =
            (&mut *self.visualizer as &mut dyn std::any::Any).downcast_mut::<SpectrumVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
            .downcast_mut::<OscilloscopeVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
            .downcast_mut::<ImageChannelVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
            .downcast_mut::<VideoChannelVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) =
            (&mut *self.visualizer as &mut dyn std::any::Any).downcast_mut::<PrimitivesVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) =
            (&mut *self.visualizer as &mut dyn std::any::Any).downcast_mut::<GridTunnelVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
            .downcast_mut::<GravityWellVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        } else if let Some(viz) =
            (&mut *self.visualizer as &mut dyn std::any::Any).downcast_mut::<StarfieldVisualizer>()
        {
            viz.set_color_scheme(self.color_scheme.clone());
        }
        // Raycaster and ObjViewer might use their own coloring or ignore it
    }

    fn toggle_effects(&mut self) {
        self.effect_pipeline.toggle_enabled();
    }

    fn toggle_effect(&mut self, name: &str) {
        self.effect_pipeline.toggle_effect(name);
    }

    fn decrease_effect_intensity(&mut self) {
        self.effect_pipeline.decrease_all_intensities();
    }

    fn increase_effect_intensity(&mut self) {
        self.effect_pipeline.increase_all_intensities();
    }

    fn toggle_microphone(&mut self) {
        self.microphone_enabled = !self.microphone_enabled;
    }

    fn increase_sensitivity(&mut self) {
        self.sensitivity_multiplier = (self.sensitivity_multiplier + 0.1).min(5.0);
        self.recreate_visualizer();
    }

    fn decrease_sensitivity(&mut self) {
        self.sensitivity_multiplier = (self.sensitivity_multiplier - 0.1).max(0.1);
        self.recreate_visualizer();
    }

    fn set_sensitivity_preset(&mut self, level: u8) {
        self.sensitivity_multiplier = level as f32 * 0.5;
        self.recreate_visualizer();
    }

    /// Apply character set mapping and colors to the grid
    #[allow(dead_code)] // Reserved for future charset mapping feature
    fn apply_charset_to_grid(&self, grid: &mut GridBuffer) {
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let cell = grid.get_cell(x, y);
                // Map intensity (0.0 = space, 1.0 = filled) based on character
                let intensity = match cell.character {
                    ' ' => 0.0,
                    '.' => 0.1,
                    ':' => 0.2,
                    '-' => 0.3,
                    '=' => 0.4,
                    '+' => 0.5,
                    '*' => 0.6,
                    '#' => 0.7,
                    '%' => 0.8,
                    '@' => 0.9,
                    '█' => 1.0,
                    '▓' => 0.75,
                    '▒' => 0.5,
                    '░' => 0.25,
                    _ => 0.5, // Default for unknown characters
                };
                let new_char = self.current_charset.get_char(intensity);

                // Apply color based on intensity
                if let Some(color) = self.color_scheme.get_color(intensity) {
                    grid.set_cell_with_color(x, y, new_char, color);
                } else {
                    grid.set_cell(x, y, new_char);
                }
            }
        }
    }

    /// Try to load a file into the current channel (Image/Video/Archive TV)
    fn try_load_current_channel_path(&mut self, path: &str) -> Result<(), String> {
        match self.visualizer_mode {
            VisualizerMode::Image => {
                if let Some(v) = (&mut *self.visualizer as &mut dyn std::any::Any)
                    .downcast_mut::<crate::visualization::ImageChannelVisualizer>()
                {
                    v.try_load(path)
                } else {
                    Err("internal visualizer type mismatch (Image)".into())
                }
            }
            VisualizerMode::Video => {
                if let Some(v) = (&mut *self.visualizer as &mut dyn std::any::Any)
                    .downcast_mut::<crate::visualization::VideoChannelVisualizer>()
                {
                    v.try_load(path)
                } else {
                    Err("internal visualizer type mismatch (Video)".into())
                }
            }
            VisualizerMode::InternetArchive
            | VisualizerMode::ArchiveCooking
            | VisualizerMode::ArchivePublicAccess
            | VisualizerMode::ArchiveIndustrial
            | VisualizerMode::ArchiveEducational
            | VisualizerMode::ArchiveLocalNews => {
                if let Some(v) = (&mut *self.visualizer as &mut dyn std::any::Any)
                    .downcast_mut::<crate::visualization::VideoChannelVisualizer>()
                {
                    v.try_load(path)
                } else {
                    Err("internal visualizer type mismatch (Internet Archive)".into())
                }
            }
            _ => Err("File input is only available in Image/Video/Archive TV channels".into()),
        }
    }

    /// Add UI overlay with renderer info and controls
    fn add_ui_overlay(&self, grid: &mut GridBuffer) {
        if self.is_loading_archive {
            let loading_text = format!("Tuning {}...", self.pending_archive_channel.name());
            let start_x = (grid.width().saturating_sub(loading_text.len())) / 2;
            let y = grid.height() / 2;
            for (i, ch) in loading_text.chars().enumerate() {
                let x = start_x + i;
                if x < grid.width() {
                    grid.set_cell(x, y, ch);
                }
            }
            return; // Don't draw other UI while loading
        }
        let visualizer_name = self.visualizer.name();
        let scheme_type = self.color_scheme.scheme_type();
        let color_scheme_name = scheme_type.name();
        let mic_status = if self.microphone_enabled {
            "MIC:ON"
        } else {
            "MIC:OFF"
        };

        // Build effect status string with individual effect states
        let mut fx_parts = Vec::new();
        if self.effect_pipeline.is_enabled() {
            fx_parts.push("FX:ON".to_string());
        } else {
            fx_parts.push("FX:OFF".to_string());
        }

        // Show individual effect states and intensities
        for effect_name in self.effect_pipeline.effect_names() {
            if let Some(effect) = self.effect_pipeline.get_effect(effect_name) {
                let short_name = match effect_name {
                    "Bloom" => "B",
                    "Scanline" => "S",
                    "Phosphor" => "P",
                    _ => &effect_name[0..1],
                };
                let intensity_pct = (effect.intensity() * 100.0) as u8;
                if effect.is_enabled() {
                    fx_parts.push(format!("{}:{}%", short_name, intensity_pct));
                } else {
                    fx_parts.push(format!("{}:off", short_name));
                }
            }
        }
        let fx_status = fx_parts.join(" ");

        // Optional channel prefix (e.g., "CH 3/11: ")
        let channel_prefix = if self.show_channel_number {
            format!(
                "CH {}/{}: ",
                self.visualizer_mode.index() + 1,
                VisualizerMode::count()
            )
        } else {
            String::new()
        };

        let info_text = if self.visualizer_mode == VisualizerMode::Oscilloscope {
            format!(
                    " {}{} | {} | {} | {} | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity G:grid F:fill T:trigger M:mic Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        } else if self.visualizer_mode == VisualizerMode::Spectrum {
            let map_name = match self.spectrum_mapping {
                SpectrumMapping::NoteBars => "NOTES",
                SpectrumMapping::LogBars => "LOG",
            };
            if matches!(self.spectrum_mapping, SpectrumMapping::NoteBars) {
                let (range_label, _min, _max) = match self.spectrum_range_preset_index % 3 {
                    0 => ("A2-A5", 110.0, 880.0),
                    1 => ("A1-A5", 55.0, 880.0),
                    _ => ("A1-A6", 55.0, 1760.0),
                };
                format!(
                    " {}{} | {} | {} | {} | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity P:peaks L:labels N:map({}) R:range({}) M:mic +/-:sens Q:quit ",
                    channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status, map_name, range_label
                )
            } else {
                format!(
                    " {}{} | {} | {} | {} | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity P:peaks L:labels N:map({}) M:mic +/-:sens Q:quit ",
                    channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status, map_name
                )
            }
        } else if self.visualizer_mode == VisualizerMode::Raycaster3D {
            let step_deg = self.ray3d_wire_step_rad.to_degrees();
            let tol = self.ray3d_wire_tol_rad;
            let mode_name = match self.ray3d_mode {
                crate::visualization::ray_tracer::RenderMode::Wireframe { .. } => "WF",
                crate::visualization::ray_tracer::RenderMode::Solid => "SOL",
            };
            let auto_label = if self.ray3d_auto_rotate { "ON" } else { "OFF" };
            let rot_speed = self.ray3d_rotation_speed_y;
            format!(
                " {}{}({}) | {} | {} | {} | W:mode G/H:step({:.0}°) T/Y:thick({:.3}) J/K:rot({:.1}) R:auto({}) Up/Down:bright ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, mode_name, color_scheme_name, mic_status, fx_status, step_deg, tol, rot_speed, auto_label
            )
        } else if self.visualizer_mode == VisualizerMode::ObjViewer {
            let (model_name, line_px, dot_px) = if let Some(viz) = (&*self.visualizer
                as &dyn std::any::Any)
                .downcast_ref::<crate::visualization::ObjViewerVisualizer>()
            {
                let (lp, dp) = viz.wire_px().unwrap_or((1, 2));
                (viz.model_name(), lp, dp)
            } else {
                ("Unknown", 1, 2)
            };
            let auto_label = if self.model_viewer_auto_rotate {
                "ON"
            } else {
                "OFF"
            };
            format!(
                " {}{} | {} | {} | {} | Model: {} | W:mode A/D:yaw J/K:pitch ,/.:roll G/H:line({}px) T/Y:dot({}px) Z/X:zoom F:focus R:auto({}) Up/Down:switch ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status, model_name, line_px, dot_px, auto_label
            )
        } else if self.visualizer_mode == VisualizerMode::Primitives {
            format!(
                " {}{} | {} | {} | {} | Bass:core pulse Mid:orbit Treble:glow | A/D:yaw J/K:pitch ,/.:roll Z/X:zoom R:auto | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        } else if self.visualizer_mode == VisualizerMode::GridTunnel {
            format!(
                " {}{} | {} | {} | {} | Bass:pulse Mid:roll Treble:glow | A/D:twist J/K:speed ,/.:glow Z/X:zoom R:auto | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        } else if self.visualizer_mode == VisualizerMode::GravityWell {
            format!(
                " {}{} | {} | {} | {} | Bass:well depth Mid:lens twist Treble:spark ring | dual-view plunge/orbit/swallow | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        } else if self.visualizer_mode == VisualizerMode::Starfield {
            format!(
                " {}{} | {} | {} | {} | Bass:warp Mid:twist Treble:trails | A/D:twist J/K:speed ,/.:trails Z/X:fov R:auto | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        } else {
            format!(
                " {}{} | {} | {} | {} | ←/→ V:chan I:num U:archive O:color E:fx B:bloom S:scan H:phosphor []:intensity M:mic +/-:sens Q:quit ",
                channel_prefix, visualizer_name, color_scheme_name, mic_status, fx_status
            )
        };

        // Draw info bar at the top
        let start_x = (grid.width().saturating_sub(info_text.len())) / 2;
        for (i, ch) in info_text.chars().enumerate() {
            let x = start_x + i;
            if x < grid.width() {
                grid.set_cell(x, 0, ch);
            }
        }

        if grid.height() > 1
            && !matches!(
                self.visualizer_mode,
                VisualizerMode::Image | VisualizerMode::Video
            )
        {
            let quality_text = quality_summary(self.live_quality, self.live_color_mode);
            let quality_x = (grid.width().saturating_sub(quality_text.len())) / 2;
            for (i, ch) in quality_text.chars().enumerate() {
                let x = quality_x + i;
                if x < grid.width() {
                    grid.set_cell(x, 1, ch);
                }
            }
        }

        // Secondary hint: Image/Video temporarily disabled
        if matches!(
            self.visualizer_mode,
            VisualizerMode::Image | VisualizerMode::Video
        ) {
            let y = 1usize;
            let hint = "White noise mode: image/video temporarily disabled";
            for (i, ch) in hint.chars().enumerate() {
                if i < grid.width() {
                    grid.set_cell(i, y, ch);
                } else {
                    break;
                }
            }
        }
    }

    fn handle_live_quality_action(&mut self, action: QualityControlAction) {
        if action == QualityControlAction::Reset {
            self.live_quality = self.live_quality_defaults;
            self.live_color_mode = self.live_color_mode_default;
            tracing::info!("Live quality controls reset");
            return;
        }

        apply_quality_action(action, &mut self.live_quality, &mut self.live_color_mode);
        tracing::info!(
            "{}",
            quality_summary(self.live_quality, self.live_color_mode)
        );
    }

    /// Run the main application loop
    async fn run(mut self) -> Result<()> {
        tracing::info!("Starting main loop at {} FPS", self.target_fps);

        // Start audio capture
        if let Err(error) = self.audio_device.start_capture() {
            tracing::warn!(
                error = %error,
                "Failed to start audio capture; falling back to audio-off mode"
            );
            self.fallback_to_audio_off()?;
            self.audio_device
                .start_capture()
                .context("Failed to start fallback silent audio capture")?;
        }

        // Start audio output (playback) if enabled
        if let Some(mut audio_output) = self.audio_output.take() {
            match audio_output.start_playback() {
                Ok(()) => {
                    self.audio_output = Some(audio_output);
                }
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        "Failed to start audio playback; continuing with playback off"
                    );
                }
            }
        }

        // Calculate frame time
        let frame_duration = Duration::from_secs_f32(1.0 / self.target_fps as f32);

        // Performance tracking
        let mut frame_count = 0;
        let mut fps_timer = Instant::now();
        let mut total_frame_time = Duration::ZERO;
        let mut max_frame_time = Duration::ZERO;
        let mut min_frame_time = Duration::from_secs(1);

        loop {
            let frame_start = Instant::now();

            // Check for shutdown signal
            if SHUTDOWN.load(Ordering::Relaxed) {
                tracing::info!("Shutdown signal received");
                break;
            }

            // Check for completed Internet Archive fetches
            match self.archive_stream_rx.try_recv() {
                Ok(Ok(stream)) => {
                    self.is_loading_archive = false;
                    let pending_channel = self.pending_archive_channel;
                    let request_context = self.archive_request_context;
                    tracing::info!(
                        "Tuned {} stream: {} ({}) -> {}",
                        pending_channel.name(),
                        stream.title,
                        stream.identifier,
                        stream.stream_url
                    );
                    self.last_archive_identifiers
                        .insert(pending_channel, stream.identifier.clone());

                    if self.visualizer_mode == pending_channel.visualizer_mode() {
                        if let Err(e) =
                            self.try_load_current_channel_path(stream.stream_url.as_str())
                        {
                            tracing::warn!(
                                "Failed to update {} channel status: {}",
                                pending_channel.name(),
                                e
                            );
                        }
                    }

                    if request_context.requires_active_channel_match()
                        && self.visualizer_mode != pending_channel.visualizer_mode()
                    {
                        tracing::info!(
                            "Skipping {} autoplay because the user left that archive channel before tuning completed",
                            pending_channel.name(),
                        );
                        if let Some(active_channel) =
                            ArchiveChannelKind::from_visualizer_mode(self.visualizer_mode)
                        {
                            self.request_archive_stream_for(
                                active_channel,
                                ArchiveRequestContext::RotationMode,
                            );
                        }
                        continue;
                    }

                    if let Err(e) = self.play_video_stream(stream.stream_url.as_str()) {
                        tracing::error!("Failed to play Internet Archive stream: {}", e);
                    }
                }
                Ok(Err(e)) => {
                    self.is_loading_archive = false;
                    tracing::error!("Failed to fetch Internet Archive video: {}", e);
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    // No message yet
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    // Should not happen
                    tracing::error!("Internet Archive stream channel disconnected.");
                }
            }

            // Check for keyboard/paste input
            if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    match ev {
                        Event::Paste(_s) => {
                            // Image/Video inline path input temporarily disabled; ignore paste
                        }
                        Event::Key(KeyEvent { code, kind, .. }) => {
                            // When file prompt is active, handle editing without debounce
                            if self.file_prompt_active {
                                match code {
                                    KeyCode::Esc => {
                                        self.file_prompt_active = false;
                                        self.file_prompt_buffer.clear();
                                        self.file_prompt_error = None;
                                        self.paste_suppress_deadline = None;
                                    }
                                    KeyCode::Enter => {
                                        let candidate_owned = self.file_prompt_buffer.clone();
                                        let path =
                                            candidate_owned.trim().trim_matches('"').to_string();
                                        if path.is_empty() {
                                            self.file_prompt_error = Some("Empty path".to_string());
                                        } else {
                                            match self.try_load_current_channel_path(&path) {
                                                Ok(_) => {
                                                    self.file_prompt_active = false;
                                                    self.file_prompt_error = None;
                                                    self.paste_suppress_deadline = None;
                                                }
                                                Err(err) => {
                                                    self.file_prompt_error = Some(err);
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        self.file_prompt_buffer.pop();
                                    }
                                    KeyCode::Char(c) => {
                                        // Avoid duplicating paste content from terminals that also emit Char events
                                        if let Some(deadline) = self.paste_suppress_deadline {
                                            if Instant::now() <= deadline { /* skip */
                                            } else {
                                                self.file_prompt_buffer.push(c);
                                            }
                                        } else {
                                            self.file_prompt_buffer.push(c);
                                        }
                                    }
                                    _ => {}
                                }
                                // Skip normal key handling when in prompt
                            } else {
                                // Normal key handling with debounce
                                let now = Instant::now();
                                let time_since_last_press = now.duration_since(self.last_key_press);
                                let is_quit_key = is_quit_key(code);
                                let is_navigation_key = is_channel_navigation_key(code);
                                if should_process_key_event(
                                    code,
                                    kind,
                                    time_since_last_press,
                                    self.key_debounce_ms,
                                ) {
                                    if !is_quit_key && !is_navigation_key {
                                        self.last_key_press = now;
                                    }
                                    if let Some(action) = quality_control_action_from_key(code) {
                                        self.handle_live_quality_action(action);
                                        continue;
                                    }
                                    match code {
                                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                            tracing::info!("Quit key pressed");
                                            break;
                                        }
                                        KeyCode::Char('u') | KeyCode::Char('U') => {
                                            self.request_archive_stream(
                                                ArchiveRequestContext::ManualHotkey,
                                            );
                                        }
                                        // Enter: no-op (image/video inline path input disabled temporarily)
                                        KeyCode::Enter => {}
                                        KeyCode::Char('o') | KeyCode::Char('O') => {
                                            self.next_color_scheme();
                                        }
                                        KeyCode::Char('e') | KeyCode::Char('E') => {
                                            self.toggle_effects();
                                        }
                                        KeyCode::Char('b') | KeyCode::Char('B') => {
                                            self.toggle_effect("Bloom");
                                        }
                                        KeyCode::Char('s') | KeyCode::Char('S') => {
                                            self.toggle_effect("Scanline");
                                        }
                                        KeyCode::Char('h') | KeyCode::Char('H') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Increase wireframe grid step (sparser)
                                                let step_prev = self.ray3d_wire_step_rad;
                                                self.ray3d_wire_step_rad = (self
                                                    .ray3d_wire_step_rad
                                                    + (2.0_f32.to_radians()))
                                                .min(45.0_f32.to_radians());
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe step: {:.1}° (was {:.1}°)",
                                                    self.ray3d_wire_step_rad.to_degrees(),
                                                    step_prev.to_degrees()
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (step_prev, _tol) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_step = (step_prev + 2.0_f32.to_radians()).min(45.0_f32.to_radians());
                                                    viz.set_wire_step_rad(new_step);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe step: {:.1}° (was {:.1}°)",
                                                        new_step.to_degrees(),
                                                        step_prev.to_degrees()
                                                    );
                                                }
                                            } else {
                                                self.toggle_effect("Phosphor");
                                            }
                                        }
                                        KeyCode::Char('[') | KeyCode::Char('{') => {
                                            self.decrease_effect_intensity();
                                        }
                                        KeyCode::Char(']') | KeyCode::Char('}') => {
                                            self.increase_effect_intensity();
                                        }
                                        KeyCode::Char('m') | KeyCode::Char('M') => {
                                            self.toggle_microphone();
                                        }
                                        KeyCode::Right => {
                                            self.next_visualizer_mode();
                                        }
                                        KeyCode::Left => {
                                            self.prev_visualizer_mode();
                                        }
                                        KeyCode::Char('v') | KeyCode::Char('V') => {
                                            self.next_visualizer_mode();
                                        }
                                        KeyCode::Char('i') | KeyCode::Char('I') => {
                                            self.show_channel_number = !self.show_channel_number;
                                        }
                                        KeyCode::Char('+') | KeyCode::Char('=') => {
                                            self.increase_sensitivity();
                                        }
                                        KeyCode::Char('-') | KeyCode::Char('_') => {
                                            self.decrease_sensitivity();
                                        }
                                        KeyCode::Char('1') => self.set_sensitivity_preset(1),
                                        KeyCode::Char('2') => self.set_sensitivity_preset(2),
                                        KeyCode::Char('3') => self.set_sensitivity_preset(3),
                                        KeyCode::Char('4') => self.set_sensitivity_preset(4),
                                        KeyCode::Char('5') => self.set_sensitivity_preset(5),
                                        KeyCode::Char('6') => self.set_sensitivity_preset(6),
                                        KeyCode::Char('7') => self.set_sensitivity_preset(7),
                                        KeyCode::Char('8') => self.set_sensitivity_preset(8),
                                        KeyCode::Char('9') => self.set_sensitivity_preset(9),
                                        // Raycaster 3D specific controls
                                        KeyCode::Char('w') | KeyCode::Char('W') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_mode = match self.ray3d_mode {
                                                    crate::visualization::ray_tracer::RenderMode::Wireframe { .. } => crate::visualization::ray_tracer::RenderMode::Solid,
                                                    crate::visualization::ray_tracer::RenderMode::Solid => crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    },
                                                };
                                                self.recreate_visualizer();
                                                let mode_name = match self.ray3d_mode { crate::visualization::ray_tracer::RenderMode::Wireframe { .. } => "WIREFRAME", crate::visualization::ray_tracer::RenderMode::Solid => "SOLID" };
                                                tracing::info!(
                                                    "Raycaster 3D mode toggled: {}",
                                                    mode_name
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.toggle_render_mode();
                                                    tracing::info!("OBJ Viewer: render mode toggled");
                                                }
                                            }
                                        }
                                        KeyCode::Up => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_brightness_boost =
                                                    (self.ray3d_brightness_boost + 0.05).min(0.7);
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D brightness boost: +{:.2}",
                                                    self.ray3d_brightness_boost
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                // Next model
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.next_model();
                                                    tracing::info!("OBJ Viewer: {}", viz.model_name());
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_brightness_boost =
                                                    (self.ray3d_brightness_boost - 0.05).max(-0.3);
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D brightness boost: +{:.2}",
                                                    self.ray3d_brightness_boost
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                // Previous model
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.prev_model();
                                                    tracing::info!("OBJ Viewer: {}", viz.model_name());
                                                }
                                            }
                                        }
                                        KeyCode::Char('g') | KeyCode::Char('G') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Decrease wireframe step (denser grid)
                                                let step_prev = self.ray3d_wire_step_rad;
                                                self.ray3d_wire_step_rad = (self
                                                    .ray3d_wire_step_rad
                                                    - (2.0_f32.to_radians()))
                                                .max(2.0_f32.to_radians());
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe step: {:.1} deg (was {:.1} deg)",
                                                    self.ray3d_wire_step_rad.to_degrees(),
                                                    step_prev.to_degrees()
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (step_prev, _tol) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_step = (step_prev - 2.0_f32.to_radians()).max(2.0_f32.to_radians());
                                                    viz.set_wire_step_rad(new_step);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe step: {:.1} deg (was {:.1} deg)",
                                                        new_step.to_degrees(),
                                                        step_prev.to_degrees()
                                                    );
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Oscilloscope
                                            {
                                                self.osc_show_grid = !self.osc_show_grid;
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Toggled oscilloscope grid: {}",
                                                    self.osc_show_grid
                                                );
                                            }
                                        }
                                        KeyCode::Char('f') | KeyCode::Char('F') => {
                                            if self.visualizer_mode == VisualizerMode::Oscilloscope
                                            {
                                                self.osc_waveform_mode = match self
                                                    .osc_waveform_mode
                                                {
                                                    WaveformMode::Line => WaveformMode::Filled,
                                                    WaveformMode::Filled => {
                                                        WaveformMode::LineAndFill
                                                    }
                                                    WaveformMode::LineAndFill => WaveformMode::Line,
                                                };
                                                self.recreate_visualizer();
                                                tracing::info!("Toggled oscilloscope fill mode");
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.focus_fit();
                                                    tracing::info!("OBJ Viewer: focus fit");
                                                }
                                            }
                                        }
                                        KeyCode::Char('l') | KeyCode::Char('L') => {
                                            if self.visualizer_mode == VisualizerMode::Spectrum {
                                                self.show_labels = !self.show_labels;
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Labels toggled: {}",
                                                    if self.show_labels { "ON" } else { "OFF" }
                                                );
                                            }
                                        }
                                        KeyCode::Char('p') | KeyCode::Char('P') => {
                                            if self.visualizer_mode == VisualizerMode::Spectrum {
                                                self.spectrum_peak_hold = !self.spectrum_peak_hold;
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Peak hold toggled: {}",
                                                    if self.spectrum_peak_hold {
                                                        "ON"
                                                    } else {
                                                        "OFF"
                                                    }
                                                );
                                            }
                                        }
                                        KeyCode::Char('n') | KeyCode::Char('N') => {
                                            if self.visualizer_mode == VisualizerMode::Spectrum {
                                                self.spectrum_mapping = match self.spectrum_mapping
                                                {
                                                    SpectrumMapping::NoteBars => {
                                                        SpectrumMapping::LogBars
                                                    }
                                                    SpectrumMapping::LogBars => {
                                                        SpectrumMapping::NoteBars
                                                    }
                                                };
                                                self.recreate_visualizer();
                                                let name = match self.spectrum_mapping {
                                                    SpectrumMapping::NoteBars => "NOTES",
                                                    SpectrumMapping::LogBars => "LOG",
                                                };
                                                tracing::info!(
                                                    "Spectrum mapping toggled: {}",
                                                    name
                                                );
                                            }
                                        }
                                        KeyCode::Char('t') | KeyCode::Char('T') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Decrease tolerance (thinner lines)
                                                let tol_prev = self.ray3d_wire_tol_rad;
                                                self.ray3d_wire_tol_rad =
                                                    (self.ray3d_wire_tol_rad - 0.005).max(0.002);
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                    self.ray3d_wire_tol_rad,
                                                    tol_prev
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (_step, tol_prev) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_tol = (tol_prev - 0.005).max(0.002);
                                                    viz.set_wire_tol_rad(new_tol);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                        new_tol,
                                                        tol_prev
                                                    );
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Oscilloscope
                                            {
                                                self.osc_trigger_slope = match self
                                                    .osc_trigger_slope
                                                {
                                                    TriggerSlope::Positive => {
                                                        TriggerSlope::Negative
                                                    }
                                                    TriggerSlope::Negative => TriggerSlope::Both,
                                                    TriggerSlope::Both => TriggerSlope::Positive,
                                                };
                                                self.recreate_visualizer();
                                                tracing::info!("Toggled oscilloscope trigger mode");
                                            }
                                        }
                                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                // Increase tolerance (thicker lines)
                                                let tol_prev = self.ray3d_wire_tol_rad;
                                                self.ray3d_wire_tol_rad =
                                                    (self.ray3d_wire_tol_rad + 0.005).min(0.15);
                                                if let crate::visualization::ray_tracer::RenderMode::Wireframe { .. } = self.ray3d_mode {
                                                    self.ray3d_mode = crate::visualization::ray_tracer::RenderMode::Wireframe {
                                                        step_rad: self.ray3d_wire_step_rad,
                                                        tol_rad: self.ray3d_wire_tol_rad,
                                                    };
                                                }
                                                self.recreate_visualizer();
                                                tracing::info!(
                                                    "Raycaster 3D wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                    self.ray3d_wire_tol_rad,
                                                    tol_prev
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let (_step, tol_prev) = viz.wire_params().unwrap_or((crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD));
                                                    let new_tol = (tol_prev + 0.005).min(0.15);
                                                    viz.set_wire_tol_rad(new_tol);
                                                    tracing::info!(
                                                        "OBJ Viewer wireframe thickness (tol): {:.3} rad (was {:.3} rad)",
                                                        new_tol,
                                                        tol_prev
                                                    );
                                                }
                                            }
                                        }
                                        KeyCode::Char('j') | KeyCode::Char('J') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                let prev = self.ray3d_rotation_speed_y;
                                                self.ray3d_rotation_speed_y =
                                                    (self.ray3d_rotation_speed_y - 0.1).max(0.0);
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::Raycaster3DVisualizer>()
                                                {
                                                    viz.set_rotation_speed_y(self.ray3d_rotation_speed_y);
                                                }
                                                tracing::info!("Raycaster 3D rotation speed: {:.2} rad/s (was {:.2})", self.ray3d_rotation_speed_y, prev);
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.pitch_up(step);
                                                    tracing::info!("OBJ Viewer: pitch up ({:.1}°)", 5.0);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.pitch_up(5.0_f32.to_radians());
                                                    tracing::info!("Primitives: pitch up (5.0°)");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    let bias = viz.speed_down();
                                                    tracing::info!("Grid Tunnel speed bias: {:+.3}", bias);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    let bias = viz.speed_down();
                                                    tracing::info!("Starfield speed bias: {:+.3}", bias);
                                                }
                                            }
                                        }
                                        KeyCode::Char('k') | KeyCode::Char('K') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                let prev = self.ray3d_rotation_speed_y;
                                                self.ray3d_rotation_speed_y =
                                                    (self.ray3d_rotation_speed_y + 0.1).min(5.0);
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::Raycaster3DVisualizer>()
                                                {
                                                    viz.set_rotation_speed_y(self.ray3d_rotation_speed_y);
                                                }
                                                tracing::info!("Raycaster 3D rotation speed: {:.2} rad/s (was {:.2})", self.ray3d_rotation_speed_y, prev);
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.pitch_down(step);
                                                    tracing::info!("OBJ Viewer: pitch down ({:.1}°)", 5.0);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.pitch_down(5.0_f32.to_radians());
                                                    tracing::info!("Primitives: pitch down (5.0°)");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    let bias = viz.speed_up();
                                                    tracing::info!("Grid Tunnel speed bias: {:+.3}", bias);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    let bias = viz.speed_up();
                                                    tracing::info!("Starfield speed bias: {:+.3}", bias);
                                                }
                                            }
                                        }

                                        KeyCode::Char('r') | KeyCode::Char('R') => {
                                            if self.visualizer_mode == VisualizerMode::Raycaster3D {
                                                self.ray3d_auto_rotate = !self.ray3d_auto_rotate;
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::Raycaster3DVisualizer>()
                                                {
                                                    viz.set_auto_rotate(self.ray3d_auto_rotate);
                                                }
                                                tracing::info!(
                                                    "Raycaster 3D auto-rotate: {}",
                                                    if self.ray3d_auto_rotate {
                                                        "ON"
                                                    } else {
                                                        "OFF"
                                                    }
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                self.model_viewer_auto_rotate =
                                                    !self.model_viewer_auto_rotate;
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.set_auto_rotate(self.model_viewer_auto_rotate);
                                                }
                                                tracing::info!(
                                                    "OBJ Viewer auto-rotate: {}",
                                                    if self.model_viewer_auto_rotate {
                                                        "ON"
                                                    } else {
                                                        "OFF"
                                                    }
                                                );
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    let enabled = viz.toggle_auto_rotate();
                                                    tracing::info!(
                                                        "Primitives auto-rotate: {}",
                                                        if enabled { "ON" } else { "OFF" }
                                                    );
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    let enabled = viz.toggle_auto_roll();
                                                    tracing::info!(
                                                        "Grid Tunnel auto-roll: {}",
                                                        if enabled { "ON" } else { "OFF" }
                                                    );
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    let enabled = viz.toggle_auto_rotate();
                                                    tracing::info!(
                                                        "Starfield auto-twist: {}",
                                                        if enabled { "ON" } else { "OFF" }
                                                    );
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Spectrum
                                                && matches!(
                                                    self.spectrum_mapping,
                                                    SpectrumMapping::NoteBars
                                                )
                                            {
                                                self.spectrum_range_preset_index =
                                                    (self.spectrum_range_preset_index + 1) % 3;
                                                self.recreate_visualizer();
                                                let (label, _min, _max) =
                                                    match self.spectrum_range_preset_index % 3 {
                                                        0 => ("A2-A5", 110.0, 880.0),
                                                        1 => ("A1-A5", 55.0, 880.0),
                                                        _ => ("A1-A6", 55.0, 1760.0),
                                                    };
                                                tracing::info!(
                                                    "Spectrum note range preset: {}",
                                                    label
                                                );
                                            }
                                        }
                                        KeyCode::Char('a') | KeyCode::Char('A') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.yaw_left(step);
                                                    tracing::info!("OBJ Viewer: yaw left ({:.1}°)", 5.0);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.yaw_left(5.0_f32.to_radians());
                                                    tracing::info!("Primitives: yaw left (5.0°)");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    viz.roll_left(5.0_f32.to_radians());
                                                    tracing::info!("Grid Tunnel: twist left");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    viz.rotate_left(5.0_f32.to_radians());
                                                    tracing::info!("Starfield: twist left");
                                                }
                                            }
                                        }
                                        KeyCode::Char('d') | KeyCode::Char('D') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.yaw_right(step);
                                                    tracing::info!("OBJ Viewer: yaw right ({:.1}°)", 5.0);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.yaw_right(5.0_f32.to_radians());
                                                    tracing::info!("Primitives: yaw right (5.0°)");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    viz.roll_right(5.0_f32.to_radians());
                                                    tracing::info!("Grid Tunnel: twist right");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    viz.rotate_right(5.0_f32.to_radians());
                                                    tracing::info!("Starfield: twist right");
                                                }
                                            }
                                        }
                                        KeyCode::Char(',') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.roll_ccw(step);
                                                    tracing::info!("OBJ Viewer: roll CCW ({:.1}°)", 5.0);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.roll_ccw(5.0_f32.to_radians());
                                                    tracing::info!("Primitives: roll CCW (5.0°)");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    let bias = viz.glow_down();
                                                    tracing::info!("Grid Tunnel glow bias: {:+.2}", bias);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    let gain = viz.trails_down();
                                                    tracing::info!("Starfield trail gain: {:.2}", gain);
                                                }
                                            }
                                        }
                                        KeyCode::Char('.') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    let step = 5.0_f32.to_radians();
                                                    viz.roll_cw(step);
                                                    tracing::info!("OBJ Viewer: roll CW ({:.1}°)", 5.0);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.roll_cw(5.0_f32.to_radians());
                                                    tracing::info!("Primitives: roll CW (5.0°)");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    let bias = viz.glow_up();
                                                    tracing::info!("Grid Tunnel glow bias: {:+.2}", bias);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    let gain = viz.trails_up();
                                                    tracing::info!("Starfield trail gain: {:.2}", gain);
                                                }
                                            }
                                        }

                                        KeyCode::Char('z') | KeyCode::Char('Z') => {
                                            if self.visualizer_mode
                                                == VisualizerMode::XYOscilloscope
                                            {
                                                tracing::info!("XY Oscilloscope zoom control (use +/- for sensitivity)");
                                            } else if self.visualizer_mode
                                                == VisualizerMode::ObjViewer
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.zoom_in();
                                                    tracing::info!("OBJ Viewer: zoom in");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.zoom_in();
                                                    tracing::info!("Primitives: zoom in");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    let zoom = viz.zoom_in();
                                                    tracing::info!("Grid Tunnel zoom: {:.2}", zoom);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    let zoom = viz.zoom_in();
                                                    tracing::info!("Starfield FOV scale: {:.2}", zoom);
                                                }
                                            }
                                        }
                                        KeyCode::Char('x') | KeyCode::Char('X') => {
                                            if self.visualizer_mode == VisualizerMode::ObjViewer {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::ObjViewerVisualizer>()
                                                {
                                                    viz.zoom_out();
                                                    tracing::info!("OBJ Viewer: zoom out");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Primitives
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::PrimitivesVisualizer>()
                                                {
                                                    viz.zoom_out();
                                                    tracing::info!("Primitives: zoom out");
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::GridTunnel
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::GridTunnelVisualizer>()
                                                {
                                                    let zoom = viz.zoom_out();
                                                    tracing::info!("Grid Tunnel zoom: {:.2}", zoom);
                                                }
                                            } else if self.visualizer_mode
                                                == VisualizerMode::Starfield
                                            {
                                                if let Some(viz) = (&mut *self.visualizer as &mut dyn std::any::Any)
                                                    .downcast_mut::<crate::visualization::StarfieldVisualizer>()
                                                {
                                                    let zoom = viz.zoom_out();
                                                    tracing::info!("Starfield FOV scale: {:.2}", zoom);
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Check if audio capture is still active
            if !self.audio_device.is_capturing() {
                tracing::error!("Audio capture stopped unexpectedly. This may indicate:");
                tracing::error!("  - Audio device was disconnected");
                tracing::error!("  - Audio system crashed or restarted");
                tracing::error!("  - Permission was revoked");
                tracing::error!("Exiting...");
                break;
            }

            // 1. Read audio samples from ring buffer (only if microphone is enabled)
            let audio_params = if let Some(audio_buffer) = self.audio_device.read_samples() {
                // Debug: Log that we're receiving audio (disabled for production)
                // if frame_count % 60 == 0 {
                //     tracing::debug!(
                //         "Received audio buffer: {} samples, {} channels",
                //         audio_buffer.samples.len(),
                //         audio_buffer.channels
                //     );
                // }

                // 1a. If microphone passthrough is enabled, write to output so you can hear it
                if self.microphone_enabled {
                    if let Some(ref audio_output) = self.audio_output {
                        audio_output.write_samples(&audio_buffer);
                    }
                }

                // 2. Process audio only when appropriate source is active
                // - Loopback: always process (system audio) WITHOUT amplitude squelch
                // - Mic: process only when microphone_enabled is true (WITH squelch)
                if self.use_loopback {
                    self.dsp_processor.process(&audio_buffer)
                } else if self.microphone_enabled {
                    let mut audio_params = self.dsp_processor.process(&audio_buffer);
                    const SQUELCH_THRESHOLD: f32 = 0.005; // conservative floor for mic noise
                    if audio_params.amplitude < SQUELCH_THRESHOLD {
                        audio_params = dsp::AudioParameters::default();
                    }
                    audio_params
                } else {
                    // Mic is OFF and we're not in loopback: feed silence so visuals decay to zero
                    dsp::AudioParameters::default()
                }
            } else {
                // No new audio available this frame: feed silence so visuals decay
                // Debug: Log when no audio is available (disabled for production)
                // if frame_count % 300 == 0 {
                //     tracing::debug!("No audio buffer available from ring buffer");
                // }
                dsp::AudioParameters::default()
            };

            // 3. Update visualizer with audio parameters
            self.visualizer.update(&audio_params);

            // 4. Render visualization to grid
            let (width, height) = self.renderer.dimensions();
            let mut grid = GridBuffer::new(width as usize, height as usize);
            self.visualizer.render(&mut grid);

            // 5. Apply post-processing effects
            self.effect_pipeline.apply(&mut grid, &audio_params);

            // 6. Apply app-wide braille quality controls for parity with image/video paths
            self.live_postprocess
                .apply(&mut grid, self.live_quality, self.live_color_mode);

            // 7. All visualizers now use Braille rendering directly!
            // No need to apply character set mapping - Braille gives 8× resolution

            // 8. Add UI overlay (character set name and controls)
            self.add_ui_overlay(&mut grid);

            // 9. Update terminal display
            self.renderer
                .render(&grid)
                .context("Failed to render frame")?;

            // Frame timing
            frame_count += 1;
            let frame_elapsed = frame_start.elapsed();

            // Track performance metrics
            total_frame_time += frame_elapsed;
            max_frame_time = max_frame_time.max(frame_elapsed);
            min_frame_time = min_frame_time.min(frame_elapsed);

            // FPS tracking and diagnostics (log every second)
            if fps_timer.elapsed() >= Duration::from_secs(1) {
                let actual_fps = frame_count;
                let avg_frame_time = total_frame_time / frame_count;
                let target_frame_time = frame_duration;

                // Log performance metrics (only warnings, not regular debug)
                if actual_fps < self.target_fps * 9 / 10 {
                    // Warn if FPS drops below 90% of target
                    tracing::warn!(
                        "Performance: FPS={} (target={}), avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                        actual_fps,
                        self.target_fps,
                        avg_frame_time.as_secs_f32() * 1000.0,
                        min_frame_time.as_secs_f32() * 1000.0,
                        max_frame_time.as_secs_f32() * 1000.0
                    );
                }
                // Disabled regular performance debug logging for production
                // else {
                //     tracing::debug!(
                //         "Performance: FPS={} (target={}), avg={:.2}ms, min={:.2}ms, max={:.2}ms",
                //         actual_fps,
                //         self.target_fps,
                //         avg_frame_time.as_secs_f32() * 1000.0,
                //         min_frame_time.as_secs_f32() * 1000.0,
                //         max_frame_time.as_secs_f32() * 1000.0
                //     );
                // }

                // Warn if frame time exceeds target significantly
                if max_frame_time > target_frame_time * 2 {
                    tracing::warn!(
                        "Frame time spike detected: {:.2}ms (target: {:.2}ms)",
                        max_frame_time.as_secs_f32() * 1000.0,
                        target_frame_time.as_secs_f32() * 1000.0
                    );
                }

                // Reset counters
                frame_count = 0;
                fps_timer = Instant::now();
                total_frame_time = Duration::ZERO;
                max_frame_time = Duration::ZERO;
                min_frame_time = Duration::from_secs(1);
            }

            // Sleep to maintain target FPS
            if let Some(sleep_time) = frame_duration.checked_sub(frame_elapsed) {
                tokio::time::sleep(sleep_time).await;
            } else {
                // Frame took longer than target - log at trace level
                tracing::trace!(
                    "Frame overrun: {:.2}ms (target: {:.2}ms)",
                    frame_elapsed.as_secs_f32() * 1000.0,
                    frame_duration.as_secs_f32() * 1000.0
                );
            }
        }

        tracing::info!("Shutting down application...");

        // Stop audio capture
        tracing::debug!("Stopping audio capture...");
        self.audio_device
            .stop_capture()
            .context("Failed to stop audio capture")?;
        tracing::info!("Audio capture stopped");

        // Stop audio output if enabled
        if let Some(audio_output) = self.audio_output.as_mut() {
            tracing::debug!("Stopping audio output...");
            audio_output
                .stop_playback()
                .context("Failed to stop audio playback")?;
            tracing::info!("Audio output stopped");
        }

        // Cleanup terminal
        tracing::debug!("Cleaning up terminal...");
        self.renderer
            .cleanup()
            .context("Failed to cleanup terminal")?;
        tracing::info!("Terminal cleanup complete");

        Ok(())
    }
    /// Run in test mode with test patterns
    fn run_test_mode(mut self) -> Result<()> {
        tracing::info!("Running in TEST MODE - rendering test patterns");
        tracing::info!("Press 'q', 'Q', or ESC to quit");

        let (width, height) = self.renderer.dimensions();
        let mut grid = GridBuffer::new(width as usize, height as usize);

        // Test pattern 1: Grid lines
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let ch = if x % 10 == 0 || y % 5 == 0 {
                    '+'
                } else if x % 5 == 0 {
                    '|'
                } else if y % 2 == 0 {
                    '-'
                } else {
                    ' '
                };
                grid.set_cell(x, y, ch);
            }
        }

        // Test pattern 2: Sine wave with known parameters
        let mut phase = 0.0_f32;

        loop {
            // Check for quit
            if SHUTDOWN.load(Ordering::Relaxed) {
                break;
            }

            if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            tracing::info!("Quit key pressed");
                            break;
                        }
                        KeyCode::Char('1') => {
                            tracing::info!("Test pattern 1: Grid");
                            // Already set above
                        }
                        KeyCode::Char('2') => {
                            tracing::info!("Test pattern 2: Sine wave");
                            grid.clear();
                            let center_y = grid.height() / 2;
                            for x in 0..grid.width() {
                                let norm_x = x as f32 / grid.width() as f32;
                                let wave_y = center_y as f32
                                    + (norm_x * 4.0 * std::f32::consts::PI + phase).sin()
                                        * (grid.height() as f32 * 0.3);
                                let y = wave_y as usize;
                                if y < grid.height() {
                                    grid.set_cell(x, y, '█');
                                    if y > 0 {
                                        grid.set_cell(x, y - 1, '▓');
                                    }
                                    if y + 1 < grid.height() {
                                        grid.set_cell(x, y + 1, '▓');
                                    }
                                }
                            }
                            phase += 0.1;
                        }
                        KeyCode::Char('3') => {
                            tracing::info!("Test pattern 3: Checkerboard");
                            grid.clear();
                            for y in 0..grid.height() {
                                for x in 0..grid.width() {
                                    let ch = if (x + y) % 2 == 0 { '█' } else { ' ' };
                                    grid.set_cell(x, y, ch);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Render
            self.renderer.render(&grid)?;
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }

        // Cleanup
        self.renderer.cleanup()?;
        Ok(())
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool, debug: bool) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    // Determine log level - during visualization, suppress all logs to avoid
    // corrupting the terminal display. Only show logs when not in TUI mode.
    // Since we're always in TUI mode when visualizing, use "off" for normal runs.
    let filter = if debug {
        // Debug mode: log to file instead (future enhancement)
        // For now, completely suppress to prevent terminal corruption
        EnvFilter::new("off")
    } else if verbose {
        // Verbose mode: also suppress during TUI
        EnvFilter::new("off")
    } else {
        // Normal mode: no logging during visualization
        EnvFilter::new("off")
    };

    // Configure logging format
    // IMPORTANT: Writing to stderr while in alternate screen mode causes
    // visual corruption (artifacts, stacking frames). We disable logging
    // entirely during visualization. For debugging, run with --test mode
    // or redirect logs to a file.
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(fmt::time::uptime())
        .with_writer(std::io::sink)  // Write to sink (discard all output)
        .with_ansi(false)
        .with_level(true)
        .init();

    Ok(())
}

/// Setup Ctrl+C handler for graceful shutdown
fn setup_shutdown_handler() -> Result<()> {
    ctrlc::set_handler(move || {
        tracing::info!("Received Ctrl+C, shutting down...");
        SHUTDOWN.store(true, Ordering::Relaxed);
    })
    .context("Failed to set Ctrl+C handler")?;

    Ok(())
}

/// Print version information
fn print_version_info() {
    println!("CrabMusic v{}", env!("CARGO_PKG_VERSION"));
    println!("Real-time ASCII music visualizer for terminal");
    println!();
    println!("Build information:");
    println!("  Rust version: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!("  Target: {}", std::env::consts::ARCH);
    println!("  OS: {}", std::env::consts::OS);
    println!();
    println!("Features:");
    println!("  - Real-time audio capture and visualization");
    println!("  - 7 character sets (basic, extended, blocks, shading, dots, lines, braille)");
    println!("  - Audio passthrough (hear what you visualize)");
    println!("  - Hot-reload configuration");
    println!("  - Cross-platform (Linux, macOS, Windows)");
    println!();
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("License: {}", env!("CARGO_PKG_LICENSE"));
}

/// List available audio devices
fn list_audio_devices() -> Result<()> {
    use cpal::traits::{DeviceTrait, HostTrait};

    println!("Available audio devices:");
    println!();

    let host = cpal::default_host();
    println!("Host: {:?}", host.id());
    println!();

    // List input devices
    println!("Input devices (microphones and loopback):");
    let input_devices = host
        .input_devices()
        .context("Failed to enumerate input devices")?;

    for (i, device) in input_devices.enumerate() {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        let is_default = host
            .default_input_device()
            .and_then(|d| d.name().ok())
            .map(|n| n == name)
            .unwrap_or(false);

        let name_lower = name.to_lowercase();
        let is_loopback = name_lower.contains("stereo mix")
            || name_lower.contains("loopback")
            || name_lower.contains("monitor")
            || name_lower.contains("what u hear")
            || name_lower.contains("wave out");

        let mut tags = Vec::new();
        if is_default {
            tags.push("default");
        }
        if is_loopback {
            tags.push("LOOPBACK");
        }

        if tags.is_empty() {
            println!("  {}. {}", i + 1, name);
        } else {
            println!("  {}. {} [{}]", i + 1, name, tags.join(", "));
        }
    }

    println!();

    // List output devices
    println!("Output devices (speakers/headphones):");
    let output_devices = host
        .output_devices()
        .context("Failed to enumerate output devices")?;

    for (i, device) in output_devices.enumerate() {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        let is_default = host
            .default_output_device()
            .and_then(|d| d.name().ok())
            .map(|n| n == name)
            .unwrap_or(false);

        if is_default {
            println!("  {}. {} [default]", i + 1, name);
        } else {
            println!("  {}. {}", i + 1, name);
        }
    }

    println!();
    println!("To use a specific device, use: --device \"device name\"");
    println!("For system audio capture, look for devices marked [LOOPBACK]");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.verbose, args.debug)?;

    // Print version info
    if args.verbose {
        print_version_info();
    }

    // List devices if requested
    if args.list_devices {
        list_audio_devices()?;
        return Ok(());
    }

    // Load configuration
    let config = AppConfig::load_or_default(&args.config)?;

    // Play video if requested
    if let Some(video_path) = &args.video {
        let prepared = video::prepare_video_input(video_path)
            .with_context(|| format!("Failed to prepare video input: {video_path}"))?;
        if let Some(label) = &prepared.display_label {
            tracing::info!("{label}");
        }
        return video::run_video_playback_with_config(&prepared.playback_target, &config.rendering);
    }

    // Setup shutdown handler
    setup_shutdown_handler()?;

    // Create application
    let app = Application::new(config, &args)?;

    // Run application
    if args.test {
        app.run_test_mode()?;
    } else {
        app.run().await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        is_channel_navigation_key, runtime_audio_flags, should_process_key_event, AudioRuntimeFlags,
    };
    use super::{ArchiveChannelKind, ArchiveRequestContext, VisualizerMode};
    use crossterm::event::{KeyCode, KeyEventKind};
    use std::time::Duration;

    #[test]
    fn archive_modes_are_in_visualizer_rotation() {
        assert_eq!(
            VisualizerMode::Video.next(),
            VisualizerMode::InternetArchive
        );
        assert_eq!(
            VisualizerMode::GridTunnel.next(),
            VisualizerMode::GravityWell
        );
        assert_eq!(
            VisualizerMode::GravityWell.next(),
            VisualizerMode::WaveformTunnel
        );
        assert_eq!(
            VisualizerMode::WaveformTunnel.prev(),
            VisualizerMode::GravityWell
        );
        assert_eq!(
            VisualizerMode::InternetArchive.next(),
            VisualizerMode::ArchiveCooking
        );
        assert_eq!(
            VisualizerMode::ArchiveCooking.next(),
            VisualizerMode::ArchivePublicAccess
        );
        assert_eq!(
            VisualizerMode::ArchivePublicAccess.next(),
            VisualizerMode::ArchiveIndustrial
        );
        assert_eq!(
            VisualizerMode::ArchiveIndustrial.next(),
            VisualizerMode::ArchiveEducational
        );
        assert_eq!(
            VisualizerMode::ArchiveEducational.next(),
            VisualizerMode::ArchiveLocalNews
        );
        assert_eq!(
            VisualizerMode::ArchiveLocalNews.next(),
            VisualizerMode::SineWave
        );
        assert_eq!(
            VisualizerMode::ArchiveCooking.prev(),
            VisualizerMode::InternetArchive
        );
        assert_eq!(
            VisualizerMode::ArchivePublicAccess.prev(),
            VisualizerMode::ArchiveCooking
        );
        assert_eq!(
            VisualizerMode::ArchiveIndustrial.prev(),
            VisualizerMode::ArchivePublicAccess
        );
        assert_eq!(
            VisualizerMode::ArchiveEducational.prev(),
            VisualizerMode::ArchiveIndustrial
        );
        assert_eq!(
            VisualizerMode::ArchiveLocalNews.prev(),
            VisualizerMode::ArchiveEducational
        );
        assert_eq!(
            VisualizerMode::SineWave.prev(),
            VisualizerMode::ArchiveLocalNews
        );
        assert_eq!(VisualizerMode::InternetArchive.name(), "Archive TV");
        assert_eq!(VisualizerMode::ArchiveCooking.name(), "Archive Cooking");
        assert_eq!(
            VisualizerMode::ArchivePublicAccess.name(),
            "Archive Public Access"
        );
        assert_eq!(
            VisualizerMode::ArchiveIndustrial.name(),
            "Archive Industrial Films"
        );
        assert_eq!(
            VisualizerMode::ArchiveEducational.name(),
            "Archive Educational"
        );
        assert_eq!(
            VisualizerMode::ArchiveLocalNews.name(),
            "Archive Local News"
        );
        assert_eq!(VisualizerMode::GravityWell.name(), "Gravity Well");
        assert_eq!(VisualizerMode::count(), 22);
    }

    #[test]
    fn archive_channel_kind_maps_to_archive_modes() {
        assert_eq!(
            ArchiveChannelKind::from_visualizer_mode(VisualizerMode::InternetArchive),
            Some(ArchiveChannelKind::Tv)
        );
        assert_eq!(
            ArchiveChannelKind::from_visualizer_mode(VisualizerMode::ArchiveCooking),
            Some(ArchiveChannelKind::Cooking)
        );
        assert_eq!(
            ArchiveChannelKind::from_visualizer_mode(VisualizerMode::ArchivePublicAccess),
            Some(ArchiveChannelKind::PublicAccess)
        );
        assert_eq!(
            ArchiveChannelKind::from_visualizer_mode(VisualizerMode::ArchiveIndustrial),
            Some(ArchiveChannelKind::Industrial)
        );
        assert_eq!(
            ArchiveChannelKind::from_visualizer_mode(VisualizerMode::ArchiveEducational),
            Some(ArchiveChannelKind::Educational)
        );
        assert_eq!(
            ArchiveChannelKind::from_visualizer_mode(VisualizerMode::ArchiveLocalNews),
            Some(ArchiveChannelKind::LocalNews)
        );
        assert_eq!(
            ArchiveChannelKind::Tv.visualizer_mode(),
            VisualizerMode::InternetArchive
        );
    }

    #[test]
    fn archive_autoplay_contexts_require_active_channel_match() {
        assert!(!ArchiveRequestContext::ManualHotkey.requires_active_channel_match());
        assert!(ArchiveRequestContext::RotationMode.requires_active_channel_match());
        assert!(ArchiveRequestContext::PlaybackEnded.requires_active_channel_match());
    }

    #[test]
    fn channel_navigation_keys_are_immediate_on_press() {
        assert!(is_channel_navigation_key(KeyCode::Left));
        assert!(is_channel_navigation_key(KeyCode::Right));
        assert!(is_channel_navigation_key(KeyCode::Char('v')));

        assert!(should_process_key_event(
            KeyCode::Right,
            KeyEventKind::Press,
            Duration::from_millis(10),
            200,
        ));
        assert!(should_process_key_event(
            KeyCode::Char('V'),
            KeyEventKind::Press,
            Duration::from_millis(10),
            200,
        ));
    }

    #[test]
    fn channel_navigation_repeat_and_release_are_ignored() {
        assert!(!should_process_key_event(
            KeyCode::Left,
            KeyEventKind::Repeat,
            Duration::from_millis(500),
            200,
        ));
        assert!(!should_process_key_event(
            KeyCode::Right,
            KeyEventKind::Release,
            Duration::from_millis(500),
            200,
        ));
    }

    #[test]
    fn non_navigation_keys_still_respect_debounce() {
        assert!(!should_process_key_event(
            KeyCode::Char('o'),
            KeyEventKind::Press,
            Duration::from_millis(50),
            200,
        ));
        assert!(should_process_key_event(
            KeyCode::Char('o'),
            KeyEventKind::Press,
            Duration::from_millis(250),
            200,
        ));
    }

    #[test]
    fn runtime_audio_flags_disable_audio_when_capture_is_unavailable() {
        assert_eq!(
            runtime_audio_flags(false, true),
            AudioRuntimeFlags {
                use_loopback: false,
                microphone_enabled: false,
            }
        );
    }

    #[test]
    fn runtime_audio_flags_preserve_requested_mode_when_capture_is_available() {
        assert_eq!(
            runtime_audio_flags(true, true),
            AudioRuntimeFlags {
                use_loopback: true,
                microphone_enabled: false,
            }
        );
        assert_eq!(
            runtime_audio_flags(true, false),
            AudioRuntimeFlags {
                use_loopback: false,
                microphone_enabled: true,
            }
        );
    }
}
