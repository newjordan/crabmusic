//! Video playback module (feature-gated)
//!
//! Provides a CLI entrypoint for video mode and utilities to convert
//! image luminance buffers into BrailleGrid output.

use anyhow::Result;

use crate::braille_quality::{self, BrailleQualitySettings};
use crate::config::RenderingConfig;
use crate::rendering::TerminalRenderer;
use crate::runtime_controls::{
    apply_quality_action, quality_control_action_from_key, ColorMode, QualityControlAction,
};
use crate::visualization::braille::BrailleGrid;
use crate::visualization::{Color, GridBuffer};
#[cfg(feature = "video")]
use std::time::Duration;

#[cfg(feature = "video")]
const AUTO_THRESHOLD_REFRESH_INTERVAL: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedVideoInput {
    pub playback_target: String,
    pub display_label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoPlaybackExit {
    UserQuit,
    EndOfStream,
    NextChannel,
    PreviousChannel,
    RetuneArchive,
}

fn playback_exit_from_key(
    code: crossterm::event::KeyCode,
    kind: crossterm::event::KeyEventKind,
    allow_app_navigation: bool,
) -> Option<VideoPlaybackExit> {
    if kind != crossterm::event::KeyEventKind::Press {
        return None;
    }

    match code {
        crossterm::event::KeyCode::Char('q')
        | crossterm::event::KeyCode::Char('Q')
        | crossterm::event::KeyCode::Esc => Some(VideoPlaybackExit::UserQuit),
        crossterm::event::KeyCode::Right if allow_app_navigation => {
            Some(VideoPlaybackExit::NextChannel)
        }
        crossterm::event::KeyCode::Left if allow_app_navigation => {
            Some(VideoPlaybackExit::PreviousChannel)
        }
        crossterm::event::KeyCode::Char('u') | crossterm::event::KeyCode::Char('U')
            if allow_app_navigation =>
        {
            Some(VideoPlaybackExit::RetuneArchive)
        }
        _ => None,
    }
}

fn playback_navigation_hint(
    allow_app_navigation: bool,
    show_archive_retune_hint: bool,
) -> &'static str {
    if !allow_app_navigation {
        ""
    } else if show_archive_retune_hint {
        " | ←/→ chan | u retune"
    } else {
        " | ←/→ chan"
    }
}

#[cfg(all(feature = "video", windows))]
pub mod webcam;

#[cfg(feature = "video")]
pub mod internet_archive;

#[cfg(feature = "video")]
pub mod youtube;

/// Run video playback mode.
///
/// When compiled without the `video` feature, this runs a short animated
/// demo using the Braille grid and instructs how to enable real video.
#[cfg(not(feature = "video"))]
#[allow(dead_code)]
pub fn run_video_playback(path: &str) -> Result<()> {
    run_video_playback_with_config(path, &RenderingConfig::default())
}

#[cfg(not(feature = "video"))]
pub fn run_video_playback_with_config(_path: &str, _rendering: &RenderingConfig) -> Result<()> {
    // Friendly stub: animate a short moving pattern so users see the plumbing
    // works even when the real video feature is disabled.
    let mut renderer = TerminalRenderer::new()?;
    let (w, h) = renderer.dimensions();
    let (w, h) = (w as usize, h as usize);

    let mut grid = GridBuffer::new(w, h);
    let mut braille = BrailleGrid::new(w, h);

    use std::time::{Duration, Instant};
    let start = Instant::now();

    loop {
        braille.clear();
        let t = start.elapsed().as_secs_f32();
        let dots_w = braille.dot_width();
        let dots_h = braille.dot_height();

        // Simple animated thresholded sine pattern
        for y in 0..dots_h {
            for x in 0..dots_w {
                let v = ((x as f32 * 0.15 + y as f32 * 0.08 + t * 4.0).sin() * 0.5 + 0.5) * 255.0;
                if v as u8 > 160 {
                    braille.set_dot(x, y);
                }
            }
        }

        // Blit BrailleGrid characters into GridBuffer
        for cy in 0..h {
            for cx in 0..w {
                let ch = braille.get_char(cx, cy);
                grid.set_cell(cx, cy, ch);
            }
        }

        renderer.render_fast(&mut grid)?;
        std::thread::sleep(Duration::from_millis(33)); // ~30 FPS

        // Run for ~3 seconds then exit
        if start.elapsed() > Duration::from_secs(3) {
            break;
        }
    }

    renderer.cleanup()?;
    tracing::warn!(
        "Video feature not enabled. Build with `--features video` and add ffmpeg-next to play actual videos."
    );
    Ok(())
}

#[cfg(not(feature = "video"))]
pub fn run_video_playback_once_with_config(
    path: &str,
    rendering: &RenderingConfig,
    show_archive_retune_hint: bool,
) -> Result<VideoPlaybackExit> {
    let _ = show_archive_retune_hint;
    run_video_playback_with_config(path, rendering)?;
    Ok(VideoPlaybackExit::EndOfStream)
}

#[cfg(not(feature = "video"))]
pub fn prepare_video_input(input: &str) -> Result<PreparedVideoInput> {
    Ok(PreparedVideoInput {
        playback_target: input.to_string(),
        display_label: None,
    })
}

/// Implementation when `video` feature is enabled.

#[cfg(feature = "video")]
#[cfg(feature = "video")]
#[derive(Debug, Clone, Copy)]
struct VideoRenderSettings {
    color_mode: ColorMode,
    letterbox: bool,
    manual_threshold: u8,
    auto_threshold: bool,
    temporal_blend_preset: usize,
    temporal_hysteresis_preset: usize,
    quality: BrailleQualitySettings,
}

#[cfg(feature = "video")]
impl Default for VideoRenderSettings {
    fn default() -> Self {
        Self::from_rendering_config(&RenderingConfig::default())
    }
}

#[cfg(feature = "video")]
impl VideoRenderSettings {
    fn from_rendering_config(rendering: &RenderingConfig) -> Self {
        Self {
            color_mode: ColorMode::Off,
            letterbox: true,
            manual_threshold: 128,
            auto_threshold: false,
            temporal_blend_preset: rendering.video_temporal_blend_preset(),
            temporal_hysteresis_preset: rendering.video_temporal_hysteresis_preset(),
            quality: rendering.video_braille_quality(),
        }
    }

    fn temporal_blend(self) -> f32 {
        braille_quality::TEMPORAL_BLEND_PRESETS[self
            .temporal_blend_preset
            .min(braille_quality::TEMPORAL_BLEND_PRESETS.len() - 1)]
    }

    fn cycle_temporal_blend(&mut self) {
        self.temporal_blend_preset =
            (self.temporal_blend_preset + 1) % braille_quality::TEMPORAL_BLEND_PRESETS.len();
    }

    fn temporal_hysteresis(self) -> u8 {
        braille_quality::TEMPORAL_HYSTERESIS_PRESETS[self
            .temporal_hysteresis_preset
            .min(braille_quality::TEMPORAL_HYSTERESIS_PRESETS.len() - 1)]
    }

    fn cycle_temporal_hysteresis(&mut self) {
        self.temporal_hysteresis_preset = (self.temporal_hysteresis_preset + 1)
            % braille_quality::TEMPORAL_HYSTERESIS_PRESETS.len();
    }

    fn reset_quality_controls(&mut self, defaults: &Self) {
        *self = *defaults;
    }

    fn handle_quality_action(&mut self, action: QualityControlAction, defaults: &Self) {
        if action == QualityControlAction::Reset {
            self.reset_quality_controls(defaults);
        } else {
            apply_quality_action(action, &mut self.quality, &mut self.color_mode);
        }
    }
}

/// Decodes frames with FFmpeg, maps to Braille, and renders to the terminal.
#[cfg(feature = "video")]
#[allow(dead_code)]
pub fn run_video_playback(path: &str) -> Result<()> {
    run_video_playback_with_config(path, &RenderingConfig::default())
}

/// Decodes frames with FFmpeg, maps to Braille, and renders to the terminal.
#[cfg(feature = "video")]
pub fn run_video_playback_with_config(path: &str, rendering: &RenderingConfig) -> Result<()> {
    run_video_playback_internal(path, rendering, true, false, false).map(|_| ())
}

#[cfg(feature = "video")]
pub fn run_video_playback_once_with_config(
    path: &str,
    rendering: &RenderingConfig,
    show_archive_retune_hint: bool,
) -> Result<VideoPlaybackExit> {
    run_video_playback_internal(path, rendering, false, true, show_archive_retune_hint)
}

#[cfg(feature = "video")]
pub fn prepare_video_input(input: &str) -> Result<PreparedVideoInput> {
    if let Some(resolved) = youtube::resolve_youtube_input(input)? {
        return Ok(PreparedVideoInput {
            playback_target: resolved.stream_url.to_string(),
            display_label: Some(format!(
                "Resolved YouTube video: {} ({})",
                resolved.title, resolved.webpage_url
            )),
        });
    }

    Ok(PreparedVideoInput {
        playback_target: input.to_string(),
        display_label: None,
    })
}

/// Decodes frames with FFmpeg, maps to Braille, and renders to the terminal.
#[cfg(feature = "video")]
fn run_video_playback_internal(
    path: &str,
    rendering: &RenderingConfig,
    loop_forever: bool,
    allow_app_navigation: bool,
    show_archive_retune_hint: bool,
) -> Result<VideoPlaybackExit> {
    use anyhow::Context;
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use ffmpeg::{
        codec, format,
        media::Type,
        software::scaling::{context::Context as Scaler, flag::Flags},
        util::{format::pixel::Pixel, frame::video::Video},
    };
    use ffmpeg_next as ffmpeg;
    use std::time::{Duration, Instant};

    ffmpeg::init().context("ffmpeg init failed")?;

    // Set up renderer and grids once
    let mut renderer = TerminalRenderer::new()?;
    let (w_cells0, h_cells0) = renderer.dimensions();
    let (mut w_cells, mut h_cells) = (w_cells0 as usize, h_cells0 as usize);
    let mut grid = GridBuffer::new(w_cells, h_cells);
    let mut braille = BrailleGrid::new(w_cells, h_cells);

    // Target dot resolution for thresholding (updated on resize)
    let mut target_dot_w = braille.dot_width();
    let mut target_dot_h = braille.dot_height();
    let mut gray_luma = vec![0u8; target_dot_w * target_dot_h];
    let mut toned_gray = vec![0u8; target_dot_w * target_dot_h];

    // Visual controls state
    let default_settings = VideoRenderSettings::from_rendering_config(rendering);
    let mut settings = default_settings;

    // HUD visibility
    let mut show_hud: bool = true;
    let mut dot_luma = Vec::new();
    let mut blended_dot_luma = Vec::new();
    let mut hysteresis_dot_luma = Vec::new();
    let mut previous_dot_luma = Vec::new();
    let mut previous_dot_mask = Vec::new();
    let mut cached_auto_threshold = settings.manual_threshold;
    let mut auto_threshold_frames_since_refresh = AUTO_THRESHOLD_REFRESH_INTERVAL;
    // Default: loop playback forever until the user quits
    loop {
        reset_temporal_history(
            &mut previous_dot_luma,
            &mut previous_dot_mask,
            &mut auto_threshold_frames_since_refresh,
        );
        // Open input and find the best video stream
        let mut ictx = format::input(&path).with_context(|| format!("open input {}", path))?;
        let input = ictx
            .streams()
            .best(Type::Video)
            .context("no video stream")?;
        let video_stream_index = input.index();

        // Set up decoder
        let ctx_decoder = codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = ctx_decoder.decoder().video()?;

        let src_w = decoder.width();
        let src_h = decoder.height();
        let (initial_fit_w, initial_fit_h) = compute_fit_dimensions(
            src_w as usize,
            src_h as usize,
            target_dot_w,
            target_dot_h,
            settings.letterbox,
        );

        // Convert to RGB24 for simple pipeline
        let mut scaler = Scaler::get(
            decoder.format(),
            src_w,
            src_h,
            Pixel::RGB24,
            initial_fit_w as u32,
            initial_fit_h as u32,
            Flags::BILINEAR,
        )?;
        let mut frame = Video::empty();
        let mut rgb_frame = Video::empty();
        let mut scaler_output_w = initial_fit_w;
        let mut scaler_output_h = initial_fit_h;

        // Determine playback FPS for pacing
        let afr = input.avg_frame_rate();
        let fps = if afr.denominator() != 0 {
            afr.numerator() as f64 / afr.denominator() as f64
        } else {
            24.0
        };
        let frame_duration =
            Duration::from_secs_f64(if fps > 0.0 { 1.0 / fps } else { 1.0 / 24.0 });
        let mut last_frame_time = Instant::now();
        let mut perf_timer = Instant::now();
        let mut perf_frames = 0usize;
        let mut decode_copy_total = Duration::ZERO;
        let mut resize_total = Duration::ZERO;
        let mut braille_total = Duration::ZERO;
        let mut color_total = Duration::ZERO;
        let mut render_total = Duration::ZERO;

        // Packet -> frame loop
        for (stream, packet) in ictx.packets() {
            if stream.index() != video_stream_index {
                continue;
            }
            decoder.send_packet(&packet)?;

            while decoder.receive_frame(&mut frame).is_ok() {
                let decode_start = Instant::now();
                let dst_w = target_dot_w;
                let dst_h = target_dot_h;
                let (fit_w_us, fit_h_us) = compute_fit_dimensions(
                    src_w as usize,
                    src_h as usize,
                    dst_w,
                    dst_h,
                    settings.letterbox,
                );

                if fit_w_us != scaler_output_w || fit_h_us != scaler_output_h {
                    scaler.cached(
                        decoder.format(),
                        src_w,
                        src_h,
                        Pixel::RGB24,
                        fit_w_us as u32,
                        fit_h_us as u32,
                        Flags::BILINEAR,
                    );
                    rgb_frame = Video::empty();
                    scaler_output_w = fit_w_us;
                    scaler_output_h = fit_h_us;
                }

                // Convert to RGB24 directly at the fitted output size
                scaler.run(&frame, &mut rgb_frame)?;
                let data = rgb_frame.data(0);
                let stride = rgb_frame.stride(0) as usize;
                decode_copy_total += decode_start.elapsed();

                let resize_start = Instant::now();
                let fit_rgb = data;
                let fit_rgb_stride = stride;

                // Center the fitted image in the destination (black bars around if needed)
                let off_x = ((dst_w as isize - fit_w_us as isize) / 2).max(0) as usize;
                let off_y = ((dst_h as isize - fit_h_us as isize) / 2).max(0) as usize;

                if fit_w_us == dst_w && fit_h_us == dst_h {
                    rgb_to_luma_strided(
                        fit_rgb,
                        fit_w_us,
                        fit_h_us,
                        fit_rgb_stride,
                        &mut gray_luma,
                    );
                } else {
                    rgb_to_luma_letterboxed_strided(
                        fit_rgb,
                        fit_w_us,
                        fit_h_us,
                        fit_rgb_stride,
                        &mut gray_luma,
                        dst_w,
                        dst_h,
                        off_x,
                        off_y,
                    );
                }
                resize_total += resize_start.elapsed();

                let braille_start = Instant::now();
                // Blit to Braille dots using manual or auto (Otsu) threshold
                let dot_w = braille.dot_width();
                let dot_h = braille.dot_height();
                braille_quality::preprocess_luma_to_dot_grid_into(
                    &gray_luma,
                    dst_w,
                    dst_h,
                    dot_w,
                    dot_h,
                    settings.quality,
                    &mut dot_luma,
                );
                braille_quality::blend_dot_luma_with_previous_into(
                    &dot_luma,
                    (!previous_dot_luma.is_empty()).then_some(previous_dot_luma.as_slice()),
                    settings.temporal_blend(),
                    &mut blended_dot_luma,
                );
                let used_threshold: u8 = if settings.auto_threshold {
                    if auto_threshold_frames_since_refresh >= AUTO_THRESHOLD_REFRESH_INTERVAL {
                        cached_auto_threshold = otsu_threshold(&blended_dot_luma);
                        auto_threshold_frames_since_refresh = 0;
                    }
                    auto_threshold_frames_since_refresh += 1;
                    cached_auto_threshold
                } else {
                    auto_threshold_frames_since_refresh = AUTO_THRESHOLD_REFRESH_INTERVAL;
                    settings.manual_threshold
                };
                braille_quality::apply_temporal_hysteresis_into(
                    &blended_dot_luma,
                    (!previous_dot_mask.is_empty()).then_some(previous_dot_mask.as_slice()),
                    settings.temporal_hysteresis(),
                    &mut hysteresis_dot_luma,
                );
                braille_quality::render_dot_luma_to_braille(
                    &hysteresis_dot_luma,
                    dot_w,
                    dot_h,
                    used_threshold,
                    settings.quality.dither_mode,
                    &mut braille,
                );
                std::mem::swap(&mut previous_dot_luma, &mut blended_dot_luma);
                braille_quality::capture_braille_dot_mask_into(&braille, &mut previous_dot_mask);
                braille_total += braille_start.elapsed();

                // Write characters and colors according to color mode
                let color_start = Instant::now();
                match settings.color_mode {
                    ColorMode::Off => {
                        for cy in 0..h_cells {
                            for cx in 0..w_cells {
                                let ch = braille.get_char(cx, cy);
                                grid.set_cell(cx, cy, ch);
                            }
                        }
                    }
                    ColorMode::Grayscale => {
                        braille_quality::apply_tone_curve_into(
                            &gray_luma,
                            settings.quality,
                            &mut toned_gray,
                        );
                        write_grayscale_braille_to_grid(
                            &mut grid,
                            &braille,
                            &toned_gray,
                            w_cells,
                            h_cells,
                            dst_w,
                        );
                    }
                    ColorMode::Full => {
                        write_full_color_braille_to_grid(
                            &mut grid,
                            &braille,
                            fit_rgb,
                            fit_w_us,
                            fit_h_us,
                            fit_rgb_stride,
                            w_cells,
                            h_cells,
                            off_x,
                            off_y,
                        );
                    }
                }
                // Tiny HUD overlay before render
                if show_hud {
                    draw_video_hud(
                        &mut grid,
                        path,
                        allow_app_navigation,
                        show_archive_retune_hint,
                        used_threshold,
                        settings.auto_threshold,
                        settings.letterbox,
                        settings.color_mode,
                        settings.quality,
                        settings.temporal_blend(),
                        settings.temporal_hysteresis(),
                    );
                }
                color_total += color_start.elapsed();

                let render_start = Instant::now();
                renderer.render_fast(&mut grid)?;
                render_total += render_start.elapsed();

                perf_frames += 1;
                let perf_elapsed = perf_timer.elapsed();
                if perf_elapsed >= Duration::from_secs(1) {
                    let actual_fps = perf_frames as f64 / perf_elapsed.as_secs_f64();
                    let avg_decode_ms = average_stage_ms(decode_copy_total, perf_frames);
                    let avg_resize_ms = average_stage_ms(resize_total, perf_frames);
                    let avg_braille_ms = average_stage_ms(braille_total, perf_frames);
                    let avg_color_ms = average_stage_ms(color_total, perf_frames);
                    let avg_render_ms = average_stage_ms(render_total, perf_frames);

                    if actual_fps < fps * 0.9 {
                        tracing::warn!(
                            "Video playback slow: fps={:.1}/{:.1} decode={:.2}ms resize={:.2}ms braille={:.2}ms color={:.2}ms render={:.2}ms",
                            actual_fps,
                            fps,
                            avg_decode_ms,
                            avg_resize_ms,
                            avg_braille_ms,
                            avg_color_ms,
                            avg_render_ms
                        );
                    } else {
                        tracing::debug!(
                            "Video playback timings: fps={:.1}/{:.1} decode={:.2}ms resize={:.2}ms braille={:.2}ms color={:.2}ms render={:.2}ms",
                            actual_fps,
                            fps,
                            avg_decode_ms,
                            avg_resize_ms,
                            avg_braille_ms,
                            avg_color_ms,
                            avg_render_ms
                        );
                    }

                    perf_timer = Instant::now();
                    perf_frames = 0;
                    decode_copy_total = Duration::ZERO;
                    resize_total = Duration::ZERO;
                    braille_total = Duration::ZERO;
                    color_total = Duration::ZERO;
                    render_total = Duration::ZERO;
                }

                // Input and resize handling
                while event::poll(Duration::from_millis(0))? {
                    match event::read()? {
                        Event::Key(k) => {
                            if let Some(exit) =
                                playback_exit_from_key(k.code, k.kind, allow_app_navigation)
                            {
                                decoder.send_eof()?;
                                renderer.cleanup()?;
                                return Ok(exit);
                            }

                            match k.code {
                                // Color mode cycle: only on press (avoid rapid cycling on repeat)
                                KeyCode::Char('c') | KeyCode::Char('C')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.color_mode = match settings.color_mode {
                                        ColorMode::Off => ColorMode::Grayscale,
                                        ColorMode::Grayscale => ColorMode::Full,
                                        ColorMode::Full => ColorMode::Off,
                                    };
                                }
                                // Letterbox toggle: only on press
                                KeyCode::Char('l') | KeyCode::Char('L')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.letterbox = !settings.letterbox;
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                // Toggle HUD (F1): only on press
                                KeyCode::F(1) if k.kind == KeyEventKind::Press => {
                                    show_hud = !show_hud;
                                }

                                // Toggle effects pipeline: only on press
                                // Image extraction threshold controls: respond to press and repeat
                                KeyCode::Char('+') | KeyCode::Char('=')
                                    if matches!(
                                        k.kind,
                                        KeyEventKind::Press | KeyEventKind::Repeat
                                    ) =>
                                {
                                    if settings.manual_threshold < 250 {
                                        settings.manual_threshold =
                                            settings.manual_threshold.saturating_add(5);
                                        reset_temporal_history(
                                            &mut previous_dot_luma,
                                            &mut previous_dot_mask,
                                            &mut auto_threshold_frames_since_refresh,
                                        );
                                    }
                                }
                                KeyCode::Char('-') | KeyCode::Char('_')
                                    if matches!(
                                        k.kind,
                                        KeyEventKind::Press | KeyEventKind::Repeat
                                    ) =>
                                {
                                    if settings.manual_threshold > 5 {
                                        settings.manual_threshold =
                                            settings.manual_threshold.saturating_sub(5);
                                        reset_temporal_history(
                                            &mut previous_dot_luma,
                                            &mut previous_dot_mask,
                                            &mut auto_threshold_frames_since_refresh,
                                        );
                                    }
                                }
                                // Auto threshold toggle: only on press
                                KeyCode::Char('a') | KeyCode::Char('A')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.auto_threshold = !settings.auto_threshold;
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                code if k.kind == KeyEventKind::Press
                                    && quality_control_action_from_key(code).is_some() =>
                                {
                                    settings.handle_quality_action(
                                        quality_control_action_from_key(code).unwrap(),
                                        &default_settings,
                                    );
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('d') | KeyCode::Char('D')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_dither();
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('p') | KeyCode::Char('P')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_preset();
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('g') | KeyCode::Char('G')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_gamma();
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('v') | KeyCode::Char('V')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_contrast();
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('z') | KeyCode::Char('Z')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_exposure();
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('t') | KeyCode::Char('T')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.cycle_temporal_blend();
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('y') | KeyCode::Char('Y')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.cycle_temporal_hysteresis();
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                KeyCode::Char('0') if k.kind == KeyEventKind::Press => {
                                    settings.reset_quality_controls(&default_settings);
                                    reset_temporal_history(
                                        &mut previous_dot_luma,
                                        &mut previous_dot_mask,
                                        &mut auto_threshold_frames_since_refresh,
                                    );
                                }
                                _ => {}
                            }
                        }
                        Event::Resize(new_w, new_h) => {
                            // Rebuild buffers to new terminal size and update dot targets
                            w_cells = new_w as usize;
                            h_cells = new_h as usize;
                            grid = GridBuffer::new(w_cells, h_cells);
                            braille = BrailleGrid::new(w_cells, h_cells);
                            target_dot_w = braille.dot_width();

                            target_dot_h = braille.dot_height();
                            gray_luma.resize(target_dot_w * target_dot_h, 0);
                            toned_gray.resize(target_dot_w * target_dot_h, 0);
                            reset_temporal_history(
                                &mut previous_dot_luma,
                                &mut previous_dot_mask,
                                &mut auto_threshold_frames_since_refresh,
                            );
                        }
                        _ => {}
                    }
                }

                // Frame pacing to target FPS
                let elapsed = last_frame_time.elapsed();
                if frame_duration > elapsed {
                    std::thread::sleep(frame_duration - elapsed);
                }
                last_frame_time = Instant::now();
            }
        }

        // Drain decoder before restarting
        decoder.send_eof()?;
        let mut frame = Video::empty();
        while decoder.receive_frame(&mut frame).is_ok() {}

        if !loop_forever {
            renderer.cleanup()?;
            return Ok(VideoPlaybackExit::EndOfStream);
        }

        // Loop repeats: reopen the input and continue playback
    }
}

#[cfg(feature = "video")]
fn reset_temporal_history(
    previous_dot_luma: &mut Vec<u8>,
    previous_dot_mask: &mut Vec<u8>,
    auto_threshold_frames_since_refresh: &mut usize,
) {
    previous_dot_luma.clear();
    previous_dot_mask.clear();
    *auto_threshold_frames_since_refresh = AUTO_THRESHOLD_REFRESH_INTERVAL;
}

#[cfg(feature = "video")]
fn rgb_to_luma_strided(
    rgb: &[u8],
    src_w: usize,
    src_h: usize,
    src_stride: usize,
    gray: &mut Vec<u8>,
) {
    gray.resize(src_w * src_h, 0);
    for y in 0..src_h {
        let src_row = &rgb[y * src_stride..y * src_stride + src_w * 3];
        let dst_row = &mut gray[y * src_w..(y + 1) * src_w];
        for (dst, chunk) in dst_row.iter_mut().zip(src_row.chunks_exact(3)) {
            *dst = rgb_triplet_to_luma(chunk[0], chunk[1], chunk[2]);
        }
    }
}

#[cfg(feature = "video")]
fn compute_fit_dimensions(
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
    letterbox: bool,
) -> (usize, usize) {
    if !letterbox {
        return (dst_w.max(1), dst_h.max(1));
    }

    let src_aspect = src_w as f32 / src_h.max(1) as f32;
    let dst_aspect = dst_w as f32 / dst_h.max(1) as f32;
    if src_aspect > dst_aspect {
        let fit_w = dst_w.max(1);
        let fit_h = ((dst_w as f32 / src_aspect).round().max(1.0) as usize).min(dst_h.max(1));
        (fit_w, fit_h)
    } else {
        let fit_h = dst_h.max(1);
        let fit_w = ((dst_h as f32 * src_aspect).round().max(1.0) as usize).min(dst_w.max(1));
        (fit_w, fit_h)
    }
}

#[cfg(feature = "video")]
fn rgb_to_luma_letterboxed_strided(
    rgb: &[u8],
    src_w: usize,
    src_h: usize,
    src_stride: usize,
    gray: &mut Vec<u8>,
    dst_w: usize,
    dst_h: usize,
    off_x: usize,
    off_y: usize,
) {
    gray.resize(dst_w * dst_h, 0);
    gray.fill(0);

    for y in 0..src_h {
        let src_row = &rgb[y * src_stride..y * src_stride + src_w * 3];
        let dst_row = &mut gray[(off_y + y) * dst_w + off_x..(off_y + y) * dst_w + off_x + src_w];
        for (dst, chunk) in dst_row.iter_mut().zip(src_row.chunks_exact(3)) {
            *dst = rgb_triplet_to_luma(chunk[0], chunk[1], chunk[2]);
        }
    }
}

#[cfg(feature = "video")]
fn write_grayscale_braille_to_grid(
    grid: &mut GridBuffer,
    braille: &BrailleGrid,
    toned_gray: &[u8],
    w_cells: usize,
    h_cells: usize,
    dst_w: usize,
) {
    for cy in 0..h_cells {
        let row0 = (cy * 4) * dst_w;
        let row1 = row0 + dst_w;
        let row2 = row1 + dst_w;
        let row3 = row2 + dst_w;
        for cx in 0..w_cells {
            let ch = braille.get_char(cx, cy);
            let x0 = cx * 2;
            let acc = toned_gray[row0 + x0] as u32
                + toned_gray[row0 + x0 + 1] as u32
                + toned_gray[row1 + x0] as u32
                + toned_gray[row1 + x0 + 1] as u32
                + toned_gray[row2 + x0] as u32
                + toned_gray[row2 + x0 + 1] as u32
                + toned_gray[row3 + x0] as u32
                + toned_gray[row3 + x0 + 1] as u32;
            let v = (acc / 8) as u8;
            grid.set_cell_with_color(cx, cy, ch, Color::new(v, v, v));
        }
    }
}

#[cfg(feature = "video")]
fn write_full_color_braille_to_grid(
    grid: &mut GridBuffer,
    braille: &BrailleGrid,
    src_rgb: &[u8],
    src_w: usize,
    src_h: usize,
    src_stride: usize,
    w_cells: usize,
    h_cells: usize,
    off_x: usize,
    off_y: usize,
) {
    let src_x_end = off_x + src_w;
    let src_y_end = off_y + src_h;
    for cy in 0..h_cells {
        let dst_y0 = cy * 4;
        for cx in 0..w_cells {
            let ch = braille.get_char(cx, cy);
            let dst_x0 = cx * 2;
            let mut r_acc: u32 = 0;
            let mut g_acc: u32 = 0;
            let mut b_acc: u32 = 0;
            for oy in 0..4 {
                let dst_y = dst_y0 + oy;
                if dst_y < off_y || dst_y >= src_y_end {
                    continue;
                }
                let src_y = dst_y - off_y;
                let row_off = src_y * src_stride;
                for ox in 0..2 {
                    let dst_x = dst_x0 + ox;
                    if dst_x < off_x || dst_x >= src_x_end {
                        continue;
                    }
                    let src_x = dst_x - off_x;
                    let idx = row_off + src_x * 3;
                    r_acc += src_rgb[idx] as u32;
                    g_acc += src_rgb[idx + 1] as u32;
                    b_acc += src_rgb[idx + 2] as u32;
                }
            }
            grid.set_cell_with_color(
                cx,
                cy,
                ch,
                Color::new((r_acc / 8) as u8, (g_acc / 8) as u8, (b_acc / 8) as u8),
            );
        }
    }
}

#[cfg(feature = "video")]
#[inline]
fn rgb_triplet_to_luma(r: u8, g: u8, b: u8) -> u8 {
    let r = r as u32;
    let g = g as u32;
    let b = b as u32;
    ((r * 77 + g * 150 + b * 29) >> 8) as u8
}

#[cfg(feature = "video")]
fn average_stage_ms(total: Duration, frames: usize) -> f32 {
    if frames == 0 {
        0.0
    } else {
        total.as_secs_f32() * 1000.0 / frames as f32
    }
}

/// Map an 8-bit luminance image onto a BrailleGrid via nearest-neighbor scaling

#[cfg(feature = "video")]
pub fn draw_centered(grid: &mut GridBuffer, text: &str) {
    let start_x = (grid.width().saturating_sub(text.len())) / 2;
    for (i, ch) in text.chars().enumerate() {
        let x = start_x + i;
        if x < grid.width() {
            grid.set_cell(x, 0, ch);
        }
    }
}

#[cfg(feature = "video")]
fn draw_video_hud(
    grid: &mut GridBuffer,
    path: &str,
    allow_app_navigation: bool,
    show_archive_retune_hint: bool,
    used_threshold: u8,
    auto_thresh: bool,
    letterbox: bool,
    color_mode: ColorMode,
    quality: BrailleQualitySettings,
    temporal_blend: f32,
    temporal_hysteresis: u8,
) {
    use std::path::Path;
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("(none)");
    let color_str = match color_mode {
        ColorMode::Off => "OFF",
        ColorMode::Grayscale => "GRAY",
        ColorMode::Full => "FULL",
    };
    let navigation_hint = playback_navigation_hint(allow_app_navigation, show_archive_retune_hint);
    let status = format!(
        "{}{} | +/- thr={} | a auto={} | F1-7 quality | p {} | d {} | g {:.2} | v {:.2} | z {:.2} | t {:.2} | y {} | l letterbox={} | c color={}",
        name,
        navigation_hint,
        used_threshold,
        if auto_thresh { "ON" } else { "OFF" },
        quality.preset_label(),
        quality.dither_mode.short_name(),
        quality.gamma(),
        quality.contrast(),
        quality.exposure(),
        temporal_blend,
        temporal_hysteresis,
        if letterbox { "ON" } else { "OFF" },
        color_str
    );
    draw_centered(grid, &status);
}

/// and binary thresholding.
///
/// - luma: length = img_w * img_h
/// - threshold: 0..=255; pixels >= threshold set their corresponding dot
#[allow(dead_code)]
pub fn blit_luma_to_braille(
    luma: &[u8],
    img_w: usize,
    img_h: usize,
    threshold: u8,
    braille: &mut BrailleGrid,
) {
    braille_quality::blit_luma_to_braille_with_quality(
        luma,
        img_w,
        img_h,
        threshold,
        BrailleQualitySettings::default(),
        braille,
    );
}

/// Compute an Otsu threshold from an 8-bit luma slice
pub fn otsu_threshold(luma: &[u8]) -> u8 {
    braille_quality::otsu_threshold(luma)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEventKind};

    #[test]
    fn test_blit_luma_to_braille_full_on() {
        // One Braille cell => 2x4 dots; source image 2x4 with all 255 => '⣿'
        let mut braille = BrailleGrid::new(1, 1);
        let luma = vec![255u8; 2 * 4];
        blit_luma_to_braille(&luma, 2, 4, 128, &mut braille);
        assert_eq!(braille.get_char(0, 0), '⣿');
    }

    #[test]
    fn test_blit_luma_to_braille_checker() {
        // Checkerboard on 2x4 should set alternating dots
        let mut braille = BrailleGrid::new(1, 1);
        let luma = vec![
            255, 0, // y0: dots (1,4)
            0, 255, // y1: (2,5)
            255, 0, // y2: (3,6)
            0, 255, // y3: (7,8)
        ];
        blit_luma_to_braille(&luma, 2, 4, 128, &mut braille);
        // Dots 1,5,3,8 => pattern bits 1,16,4,128 => 0b10010101 = 0x95
        let ch = braille.get_char(0, 0);
        // Ensure we didn't get empty or full; smoke assertion
        assert_ne!(ch, '⠀');
        assert_ne!(ch, '⣿');
    }

    #[test]
    fn embedded_playback_maps_navigation_keys_to_exit_actions() {
        assert_eq!(
            playback_exit_from_key(KeyCode::Right, KeyEventKind::Press, true),
            Some(VideoPlaybackExit::NextChannel)
        );
        assert_eq!(
            playback_exit_from_key(KeyCode::Left, KeyEventKind::Press, true),
            Some(VideoPlaybackExit::PreviousChannel)
        );
        assert_eq!(
            playback_exit_from_key(KeyCode::Char('u'), KeyEventKind::Press, true),
            Some(VideoPlaybackExit::RetuneArchive)
        );
    }

    #[test]
    fn standalone_playback_ignores_app_navigation_keys() {
        assert_eq!(
            playback_exit_from_key(KeyCode::Right, KeyEventKind::Press, false),
            None
        );
        assert_eq!(
            playback_exit_from_key(KeyCode::Char('u'), KeyEventKind::Press, false),
            None
        );
        assert_eq!(
            playback_exit_from_key(KeyCode::Char('q'), KeyEventKind::Press, false),
            Some(VideoPlaybackExit::UserQuit)
        );
    }

    #[test]
    fn playback_exit_requires_initial_key_press() {
        assert_eq!(
            playback_exit_from_key(KeyCode::Right, KeyEventKind::Repeat, true),
            None
        );
        assert_eq!(
            playback_exit_from_key(KeyCode::Char('q'), KeyEventKind::Release, true),
            None
        );
    }

    #[test]
    fn playback_navigation_hint_shows_archive_controls_when_available() {
        assert_eq!(
            playback_navigation_hint(true, true),
            " | ←/→ chan | u retune"
        );
        assert_eq!(playback_navigation_hint(true, false), " | ←/→ chan");
    }

    #[test]
    fn playback_navigation_hint_is_hidden_for_standalone_playback() {
        assert_eq!(playback_navigation_hint(false, false), "");
        assert_eq!(playback_navigation_hint(false, true), "");
    }

    #[cfg(feature = "video")]
    #[test]
    fn letterboxed_luma_writes_into_offset_region() {
        let rgb = vec![255, 0, 0, 0, 255, 0];
        let mut gray = Vec::new();
        rgb_to_luma_letterboxed_strided(&rgb, 2, 1, 6, &mut gray, 4, 2, 1, 1);

        assert_eq!(gray.len(), 8);
        assert_eq!(gray[0], 0);
        assert_eq!(gray[5], rgb_triplet_to_luma(255, 0, 0));
        assert_eq!(gray[6], rgb_triplet_to_luma(0, 255, 0));
    }

    #[cfg(feature = "video")]
    #[test]
    fn strided_rgb_to_luma_skips_padding() {
        let rgb = vec![
            255, 0, 0, 0, 255, 0, 9, 9, 9, 0, 0, 255, 255, 255, 255, 7, 7, 7,
        ];
        let mut gray = Vec::new();
        rgb_to_luma_strided(&rgb, 2, 2, 9, &mut gray);

        assert_eq!(
            gray,
            vec![
                rgb_triplet_to_luma(255, 0, 0),
                rgb_triplet_to_luma(0, 255, 0),
                rgb_triplet_to_luma(0, 0, 255),
                rgb_triplet_to_luma(255, 255, 255)
            ]
        );
    }

    #[cfg(feature = "video")]
    #[test]
    fn compute_fit_dimensions_preserves_aspect_when_letterboxed() {
        assert_eq!(
            compute_fit_dimensions(1920, 1080, 100, 100, true),
            (100, 56)
        );
        assert_eq!(
            compute_fit_dimensions(1080, 1920, 100, 100, true),
            (56, 100)
        );
        assert_eq!(compute_fit_dimensions(640, 480, 80, 40, false), (80, 40));
    }
}
