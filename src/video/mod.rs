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
use crate::visualization::GridBuffer;

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

        renderer.render(&grid)?;
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
    use image::{imageops, ImageBuffer, RgbImage};
    use std::time::{Duration, Instant};

    use crate::dsp::AudioParameters;
    use crate::effects::EffectPipeline;
    use crate::effects::{
        bloom::BloomEffect, phosphor::PhosphorGlowEffect, scanline::ScanlineEffect,
    };
    use crate::visualization::Color;

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

    // Visual controls state
    let default_settings = VideoRenderSettings::from_rendering_config(rendering);
    let mut settings = default_settings;

    // Effects pipeline (disabled by default; toggle with 'e')
    let mut effect_pipeline = EffectPipeline::new();
    effect_pipeline.add_effect(Box::new(BloomEffect::new(0.7, 2)));
    effect_pipeline.add_effect(Box::new(ScanlineEffect::new(2)));
    effect_pipeline.add_effect(Box::new(PhosphorGlowEffect::new(0.3, 0.7)));
    effect_pipeline.set_enabled(false);
    let mut last_effect: String = "Bloom".to_string();

    // Dummy audio params for effects (video mode has no audio input)
    let audio_params = AudioParameters::default();
    // HUD visibility
    let mut show_hud: bool = true;
    // Default: loop playback forever until the user quits
    loop {
        let mut previous_dot_luma: Option<Vec<u8>> = None;
        let mut previous_dot_mask: Option<Vec<u8>> = None;
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

        // Convert to RGB24 for simple pipeline
        let mut scaler = Scaler::get(
            decoder.format(),
            src_w,
            src_h,
            Pixel::RGB24,
            src_w,
            src_h,
            Flags::BILINEAR,
        )?;

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

        // Packet -> frame loop
        for (stream, packet) in ictx.packets() {
            if stream.index() != video_stream_index {
                continue;
            }
            decoder.send_packet(&packet)?;

            let mut frame = Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                // Convert to RGB24
                let mut rgb_frame = Video::empty();
                scaler.run(&frame, &mut rgb_frame)?;

                let src_w = rgb_frame.width() as usize;
                let src_h = rgb_frame.height() as usize;
                let data = rgb_frame.data(0);
                let stride = rgb_frame.stride(0) as usize;

                // Construct an RgbImage row-by-row (account for stride)
                let mut img: RgbImage = ImageBuffer::new(src_w as u32, src_h as u32);
                for y in 0..src_h {
                    let row = &data[y * stride..y * stride + src_w * 3];
                    let dst = &mut img.as_mut()[y * src_w * 3..(y + 1) * src_w * 3];
                    dst.copy_from_slice(row);
                }

                // Resize to dot grid with optional letterboxing
                let dst_w = target_dot_w;
                let dst_h = target_dot_h;
                let (fit_w, fit_h) = if settings.letterbox {
                    let src_aspect = src_w as f32 / src_h as f32;
                    let dst_aspect = dst_w as f32 / dst_h as f32;
                    if src_aspect > dst_aspect {
                        let w = dst_w as u32;
                        let h = ((dst_w as f32 / src_aspect).round().max(1.0)) as u32;
                        (w.min(dst_w as u32), h.min(dst_h as u32))
                    } else {
                        let h = dst_h as u32;
                        let w = ((dst_h as f32 * src_aspect).round().max(1.0)) as u32;
                        (w.min(dst_w as u32), h.min(dst_h as u32))
                    }
                } else {
                    (dst_w as u32, dst_h as u32)
                };

                let resized_fit =
                    imageops::resize(&img, fit_w, fit_h, imageops::FilterType::Triangle);
                let mut canvas: RgbImage = ImageBuffer::new(dst_w as u32, dst_h as u32);
                // Center the fitted image in the canvas (black bars around)
                let off_x = ((dst_w as i32 - fit_w as i32) / 2).max(0) as usize;
                let off_y = ((dst_h as i32 - fit_h as i32) / 2).max(0) as usize;
                {
                    let src_bytes = resized_fit.as_raw();
                    let dst_bytes = canvas.as_mut();
                    let fit_w_us = fit_w as usize;
                    let fit_h_us = fit_h as usize;
                    for y in 0..fit_h_us {
                        let src_row = &src_bytes[y * fit_w_us * 3..(y + 1) * fit_w_us * 3];
                        let dst_start = ((off_y + y) * dst_w + off_x) * 3;
                        let dst_row = &mut dst_bytes[dst_start..dst_start + fit_w_us * 3];
                        dst_row.copy_from_slice(src_row);
                    }
                }

                // Convert to luma for Braille dot thresholding
                let gray = image::DynamicImage::ImageRgb8(canvas.clone()).to_luma8();

                // Blit to Braille dots using manual or auto (Otsu) threshold
                let dot_w = braille.dot_width();
                let dot_h = braille.dot_height();
                let dot_luma = braille_quality::preprocess_luma_to_dot_grid(
                    gray.as_raw(),
                    dst_w,
                    dst_h,
                    dot_w,
                    dot_h,
                    settings.quality,
                );
                let blended_dot_luma = braille_quality::blend_dot_luma_with_previous(
                    &dot_luma,
                    previous_dot_luma.as_deref(),
                    settings.temporal_blend(),
                );
                let used_threshold: u8 = if settings.auto_threshold {
                    otsu_threshold(&blended_dot_luma)
                } else {
                    settings.manual_threshold
                };
                let hysteresis_dot_luma = braille_quality::apply_temporal_hysteresis(
                    &blended_dot_luma,
                    previous_dot_mask.as_deref(),
                    settings.temporal_hysteresis(),
                );
                braille_quality::render_dot_luma_to_braille(
                    &hysteresis_dot_luma,
                    dot_w,
                    dot_h,
                    used_threshold,
                    settings.quality.dither_mode,
                    &mut braille,
                );
                previous_dot_luma = Some(blended_dot_luma);
                previous_dot_mask = Some(braille_quality::capture_braille_dot_mask(&braille));

                // Write characters and colors according to color mode
                let toned_gray = braille_quality::apply_tone_curve(gray.as_raw(), settings.quality);
                let gray_bytes = toned_gray.as_slice();
                let rgb_bytes = canvas.as_raw();
                for cy in 0..h_cells {
                    for cx in 0..w_cells {
                        let ch = braille.get_char(cx, cy);
                        match settings.color_mode {
                            ColorMode::Off => {
                                grid.set_cell(cx, cy, ch);
                            }
                            ColorMode::Grayscale => {
                                let x0 = cx * 2;
                                let y0 = cy * 4;
                                let mut acc: u32 = 0;
                                let mut count: u32 = 0;
                                for oy in 0..4 {
                                    let y = y0 + oy;
                                    if y >= dst_h {
                                        break;
                                    }
                                    let row_off = y * dst_w;
                                    for ox in 0..2 {
                                        let x = x0 + ox;
                                        if x >= dst_w {
                                            break;
                                        }
                                        acc += gray_bytes[row_off + x] as u32;
                                        count += 1;
                                    }
                                }
                                let v = if count > 0 { (acc / count) as u8 } else { 0 };
                                grid.set_cell_with_color(cx, cy, ch, Color::new(v, v, v));
                            }
                            ColorMode::Full => {
                                let x0 = cx * 2;
                                let y0 = cy * 4;
                                let mut r_acc: u32 = 0;
                                let mut g_acc: u32 = 0;
                                let mut b_acc: u32 = 0;
                                let mut count: u32 = 0;
                                for oy in 0..4 {
                                    let y = y0 + oy;
                                    if y >= dst_h {
                                        break;
                                    }
                                    let row_off = y * dst_w;
                                    for ox in 0..2 {
                                        let x = x0 + ox;
                                        if x >= dst_w {
                                            break;
                                        }
                                        let idx = (row_off + x) * 3;
                                        r_acc += rgb_bytes[idx] as u32;
                                        g_acc += rgb_bytes[idx + 1] as u32;
                                        b_acc += rgb_bytes[idx + 2] as u32;
                                        count += 1;
                                    }
                                }
                                let r = if count > 0 { (r_acc / count) as u8 } else { 0 };
                                let g = if count > 0 { (g_acc / count) as u8 } else { 0 };
                                let b = if count > 0 { (b_acc / count) as u8 } else { 0 };
                                grid.set_cell_with_color(cx, cy, ch, Color::new(r, g, b));
                            }
                        }
                    }
                }

                // Apply effects then render
                effect_pipeline.apply(&mut grid, &audio_params);
                // Tiny HUD overlay after effects, before render
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
                        &effect_pipeline,
                        &last_effect,
                    );
                }

                renderer.render(&grid)?;

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
                                    previous_dot_luma = None;
                                }
                                // Toggle HUD (F1): only on press
                                KeyCode::F(1) if k.kind == KeyEventKind::Press => {
                                    show_hud = !show_hud;
                                }

                                // Toggle effects pipeline: only on press
                                KeyCode::Char('e') | KeyCode::Char('E')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    effect_pipeline.set_enabled(!effect_pipeline.is_enabled());
                                }
                                // Toggle individual effects and set last_effect: only on press
                                KeyCode::Char('b') | KeyCode::Char('B')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    if let Some(eff) = effect_pipeline.get_effect_mut("Bloom") {
                                        eff.set_enabled(!eff.is_enabled());
                                        last_effect = "Bloom".to_string();
                                    }
                                }
                                KeyCode::Char('s') | KeyCode::Char('S')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    if let Some(eff) = effect_pipeline.get_effect_mut("Scanline") {
                                        eff.set_enabled(!eff.is_enabled());
                                        last_effect = "Scanline".to_string();
                                    }
                                }
                                KeyCode::Char('h') | KeyCode::Char('H')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    if let Some(eff) = effect_pipeline.get_effect_mut("Phosphor") {
                                        eff.set_enabled(!eff.is_enabled());
                                        last_effect = "Phosphor".to_string();
                                    }
                                }
                                // Intensity adjust for last-toggled effect: respond to press and repeat
                                KeyCode::Char('[') | KeyCode::Char('{')
                                    if matches!(
                                        k.kind,
                                        KeyEventKind::Press | KeyEventKind::Repeat
                                    ) =>
                                {
                                    if let Some(eff) = effect_pipeline.get_effect_mut(&last_effect)
                                    {
                                        let new_i = (eff.intensity() - 0.1).max(0.0);
                                        eff.set_intensity(new_i);
                                    }
                                }
                                KeyCode::Char(']') | KeyCode::Char('}')
                                    if matches!(
                                        k.kind,
                                        KeyEventKind::Press | KeyEventKind::Repeat
                                    ) =>
                                {
                                    if let Some(eff) = effect_pipeline.get_effect_mut(&last_effect)
                                    {
                                        let new_i = (eff.intensity() + 0.1).min(1.0);
                                        eff.set_intensity(new_i);
                                    }
                                }
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
                                        previous_dot_luma = None;
                                        previous_dot_mask = None;
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
                                        previous_dot_luma = None;
                                        previous_dot_mask = None;
                                    }
                                }
                                // Auto threshold toggle: only on press
                                KeyCode::Char('a') | KeyCode::Char('A')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.auto_threshold = !settings.auto_threshold;
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                code if k.kind == KeyEventKind::Press
                                    && quality_control_action_from_key(code).is_some() =>
                                {
                                    settings.handle_quality_action(
                                        quality_control_action_from_key(code).unwrap(),
                                        &default_settings,
                                    );
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('d') | KeyCode::Char('D')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_dither();
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('p') | KeyCode::Char('P')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_preset();
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('g') | KeyCode::Char('G')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_gamma();
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('v') | KeyCode::Char('V')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_contrast();
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('z') | KeyCode::Char('Z')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.quality.cycle_exposure();
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('t') | KeyCode::Char('T')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.cycle_temporal_blend();
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('y') | KeyCode::Char('Y')
                                    if k.kind == KeyEventKind::Press =>
                                {
                                    settings.cycle_temporal_hysteresis();
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
                                }
                                KeyCode::Char('0') if k.kind == KeyEventKind::Press => {
                                    settings.reset_quality_controls(&default_settings);
                                    previous_dot_luma = None;
                                    previous_dot_mask = None;
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
                            previous_dot_luma = None;
                            previous_dot_mask = None;
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
    pipeline: &crate::effects::EffectPipeline,
    last_effect: &str,
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
    let eff = if pipeline.is_enabled() {
        if let Some(e) = pipeline.get_effect(last_effect) {
            format!("ON {} {:.1}", last_effect, e.intensity())
        } else {
            "ON".to_string()
        }
    } else {
        "OFF".to_string()
    };
    let navigation_hint = playback_navigation_hint(allow_app_navigation, show_archive_retune_hint);
    let status = format!(
        "{}{} | +/- thr={} | a auto={} | F1-7 quality | p {} | d {} | g {:.2} | v {:.2} | z {:.2} | t {:.2} | y {} | l letterbox={} | c color={} | fx={}",
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
        color_str,
        eff
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
}
