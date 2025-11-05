//! Video playback module (feature-gated)
//!
//! Provides a CLI entrypoint for video mode and utilities to convert
//! image luminance buffers into BrailleGrid output.

use anyhow::Result;

use crate::rendering::TerminalRenderer;
use crate::visualization::{GridBuffer};
use crate::visualization::braille::BrailleGrid;

/// Run video playback mode.
///
/// When compiled without the `video` feature, this runs a short animated
/// demo using the Braille grid and instructs how to enable real video.
#[cfg(not(feature = "video"))]
pub fn run_video_playback(_path: &str) -> Result<()> {
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
                if v as u8 > 160 { braille.set_dot(x, y); }
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
        if start.elapsed() > Duration::from_secs(3) { break; }
    }

    renderer.cleanup()?;
    tracing::warn!(
        "Video feature not enabled. Build with `--features video` and add ffmpeg-next to play actual videos."
    );
    Ok(())
}

/// Implementation when `video` feature is enabled.

#[cfg(feature = "video")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorMode {
    Off,
    Grayscale,
    Full,
}

/// Decodes frames with FFmpeg, maps to Braille, and renders to the terminal.
#[cfg(feature = "video")]
pub fn run_video_playback(path: &str) -> Result<()> {
    use anyhow::Context;
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use ffmpeg_next as ffmpeg;
    use ffmpeg::{
        codec,
        format,
        media::Type,
        software::scaling::{context::Context as Scaler, flag::Flags},
        util::{frame::video::Video, format::pixel::Pixel},
    };
    use image::{imageops, ImageBuffer, RgbImage};
    use std::time::{Duration, Instant};

    use crate::dsp::AudioParameters;
    use crate::effects::{EffectPipeline};
    use crate::effects::{bloom::BloomEffect, scanline::ScanlineEffect, phosphor::PhosphorGlowEffect};
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
    let mut color_mode = ColorMode::Off; // Off → Grayscale → Full
    let mut letterbox = true;            // Preserve aspect ratio by default

    // Effects pipeline (disabled by default; toggle with 'e')
    let mut effect_pipeline = EffectPipeline::new();
    effect_pipeline.add_effect(Box::new(BloomEffect::new(0.7, 2)));
    effect_pipeline.add_effect(Box::new(ScanlineEffect::new(2)));
    effect_pipeline.add_effect(Box::new(PhosphorGlowEffect::new(0.3, 0.7)));
    effect_pipeline.set_enabled(false);
    let mut last_effect: String = "Bloom".to_string();

    // Dummy audio params for effects (video mode has no audio input)
    let audio_params = AudioParameters::default();
    // Threshold controls for image extraction (manual/auto)
    let mut manual_threshold: u8 = 128;
    let mut auto_thresh: bool = false;

    // HUD visibility and last used threshold for on-screen display
    let mut show_hud: bool = true;
    let mut last_used_threshold: u8 = manual_threshold;


    // Default: loop playback forever until the user quits
    loop {
        // Open input and find the best video stream
        let mut ictx = format::input(&path).with_context(|| format!("open input {}", path))?;
        let input = ictx.streams().best(Type::Video).context("no video stream")?;
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
        let frame_duration = Duration::from_secs_f64(if fps > 0.0 { 1.0 / fps } else { 1.0 / 24.0 });
        let mut last_frame_time = Instant::now();

        // Packet -> frame loop
        for (stream, packet) in ictx.packets() {
            if stream.index() != video_stream_index { continue; }
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
                let (fit_w, fit_h) = if letterbox {
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

                let resized_fit = imageops::resize(
                    &img,
                    fit_w,
                    fit_h,
                    imageops::FilterType::Triangle,
                );
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
                braille.clear();
                let used_threshold: u8 = if auto_thresh {
                    otsu_threshold(gray.as_raw())
                } else {
                    manual_threshold
                };
                // Remember for HUD display
                last_used_threshold = used_threshold;

                blit_luma_to_braille(
                    gray.as_raw(),
                    dst_w,
                    dst_h,
                    used_threshold,
                    &mut braille,
                );

                // Write characters and colors according to color mode
                let gray_bytes = gray.as_raw();
                let rgb_bytes = canvas.as_raw();
                for cy in 0..h_cells {
                    for cx in 0..w_cells {
                        let ch = braille.get_char(cx, cy);
                        match color_mode {
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
                                    if y >= dst_h { break; }
                                    let row_off = y * dst_w;
                                    for ox in 0..2 {
                                        let x = x0 + ox;
                                        if x >= dst_w { break; }
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
                                    if y >= dst_h { break; }
                                    let row_off = y * dst_w;
                                    for ox in 0..2 {
                                        let x = x0 + ox;
                                        if x >= dst_w { break; }
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
                        last_used_threshold,
                        auto_thresh,
                        letterbox,
                        color_mode,
                        &effect_pipeline,
                        &last_effect,
                    );
                }

                renderer.render(&grid)?;

                // Input and resize handling
                while event::poll(Duration::from_millis(0))? {
                    match event::read()? {
                        Event::Key(k) => {
                            match k.code {
                                // Quit: only on initial press
                                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc if k.kind == KeyEventKind::Press => {
                                    decoder.send_eof()?;
                                    renderer.cleanup()?;
                                    return Ok(());
                                }
                                // Color mode cycle: only on press (avoid rapid cycling on repeat)
                                KeyCode::Char('c') | KeyCode::Char('C') if k.kind == KeyEventKind::Press => {
                                    color_mode = match color_mode {
                                        ColorMode::Off => ColorMode::Grayscale,
                                        ColorMode::Grayscale => ColorMode::Full,
                                        ColorMode::Full => ColorMode::Off,
                                    };
                                }
                                // Letterbox toggle: only on press
                                KeyCode::Char('l') | KeyCode::Char('L') if k.kind == KeyEventKind::Press => {
                                    letterbox = !letterbox;
                                }
                                // Toggle HUD (F1): only on press
                                KeyCode::F(1) if k.kind == KeyEventKind::Press => {
                                    show_hud = !show_hud;
                                }

                                // Toggle effects pipeline: only on press
                                KeyCode::Char('e') | KeyCode::Char('E') if k.kind == KeyEventKind::Press => {
                                    effect_pipeline.set_enabled(!effect_pipeline.is_enabled());
                                }
                                // Toggle individual effects and set last_effect: only on press
                                KeyCode::Char('b') | KeyCode::Char('B') if k.kind == KeyEventKind::Press => {
                                    if let Some(eff) = effect_pipeline.get_effect_mut("Bloom") {
                                        eff.set_enabled(!eff.is_enabled());
                                        last_effect = "Bloom".to_string();
                                    }
                                }
                                KeyCode::Char('s') | KeyCode::Char('S') if k.kind == KeyEventKind::Press => {
                                    if let Some(eff) = effect_pipeline.get_effect_mut("Scanline") {
                                        eff.set_enabled(!eff.is_enabled());
                                        last_effect = "Scanline".to_string();
                                    }
                                }
                                KeyCode::Char('h') | KeyCode::Char('H') if k.kind == KeyEventKind::Press => {
                                    if let Some(eff) = effect_pipeline.get_effect_mut("Phosphor") {
                                        eff.set_enabled(!eff.is_enabled());
                                        last_effect = "Phosphor".to_string();
                                    }
                                }
                                // Intensity adjust for last-toggled effect: respond to press and repeat
                                KeyCode::Char('[') | KeyCode::Char('{') if matches!(k.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                                    if let Some(eff) = effect_pipeline.get_effect_mut(&last_effect) {
                                        let new_i = (eff.intensity() - 0.1).max(0.0);
                                        eff.set_intensity(new_i);
                                    }
                                }
                                KeyCode::Char(']') | KeyCode::Char('}') if matches!(k.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                                    if let Some(eff) = effect_pipeline.get_effect_mut(&last_effect) {
                                        let new_i = (eff.intensity() + 0.1).min(1.0);
                                        eff.set_intensity(new_i);
                                    }
                                }
                                // Image extraction threshold controls: respond to press and repeat
                                KeyCode::Char('+') | KeyCode::Char('=') if matches!(k.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                                    if manual_threshold < 250 { manual_threshold = manual_threshold.saturating_add(5); }
                                }
                                KeyCode::Char('-') | KeyCode::Char('_') if matches!(k.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                                    if manual_threshold > 5 { manual_threshold = manual_threshold.saturating_sub(5); }
                                }
                                // Auto threshold toggle: only on press
                                KeyCode::Char('a') | KeyCode::Char('A') if k.kind == KeyEventKind::Press => {
                                    auto_thresh = !auto_thresh;
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

        // Loop repeats: reopen the input and continue playback
    }
}

/// Map an 8-bit luminance image onto a BrailleGrid via nearest-neighbor scaling

#[cfg(feature = "video")]
fn draw_centered(grid: &mut GridBuffer, text: &str) {
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
    used_threshold: u8,
    auto_thresh: bool,
    letterbox: bool,
    color_mode: ColorMode,
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
    let status = format!(
        "{} | +/- thr={} | a auto={} | l letterbox={} | c color={} | fx={}",
        name,
        used_threshold,
        if auto_thresh { "ON" } else { "OFF" },
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
pub fn blit_luma_to_braille(
    luma: &[u8],
    img_w: usize,
    img_h: usize,
    threshold: u8,
    braille: &mut BrailleGrid,
) {
    if img_w == 0 || img_h == 0 { return; }

    let dot_w = braille.dot_width();
    let dot_h = braille.dot_height();

    for dy in 0..dot_h {
        // Map dot row to source row (nearest-neighbor)
        let sy = (dy * img_h) / dot_h;
        let sy_off = sy * img_w;
        for dx in 0..dot_w {
            let sx = (dx * img_w) / dot_w;
            let v = luma[sy_off + sx];


            if v >= threshold { braille.set_dot(dx, dy); }
        }
    }
}

/// Compute an Otsu threshold from an 8-bit luma slice
fn otsu_threshold(luma: &[u8]) -> u8 {
    let mut hist = [0u32; 256];
    for &v in luma { hist[v as usize] += 1; }
    let total: u32 = luma.len() as u32;
    let mut sum_all: u64 = 0;
    for i in 0..256 { sum_all += (i as u64) * (hist[i] as u64); }

    let mut sum_b: u64 = 0;
    let mut w_b: u32 = 0;
    let mut max_var: f64 = -1.0;
    let mut threshold: u8 = 128;

    for t in 0..256 {
        w_b += hist[t] as u32;
        if w_b == 0 { continue; }
        let w_f = total - w_b;
        if w_f == 0 { break; }
        sum_b += (t as u64) * (hist[t] as u64);
        let m_b = sum_b as f64 / w_b as f64;
        let m_f = (sum_all - sum_b) as f64 / w_f as f64;
        let var_between = (w_b as f64) * (w_f as f64) * (m_b - m_f).powi(2);
        if var_between > max_var { max_var = var_between; threshold = t as u8; }
    }
    threshold
}


#[cfg(test)]
mod tests {
    use super::*;

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
            255, 0,   // y0: dots (1,4)
            0, 255,   // y1: (2,5)
            255, 0,   // y2: (3,6)
            0, 255,   // y3: (7,8)


        ];
        blit_luma_to_braille(&luma, 2, 4, 128, &mut braille);
        // Dots 1,5,3,8 => pattern bits 1,16,4,128 => 0b10010101 = 0x95
        let ch = braille.get_char(0, 0);
        // Ensure we didn't get empty or full; smoke assertion
        assert_ne!(ch, '⠀');
        assert_ne!(ch, '⣿');
    }
}

