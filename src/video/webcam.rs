#[cfg(feature = "video")]
use anyhow::{Context, Result};
#[cfg(feature = "video")]
use image::{imageops, ImageBuffer, RgbImage};
#[cfg(feature = "video")]
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
#[cfg(feature = "video")]
use std::{cmp::Ordering, time::Duration};

#[cfg(feature = "video")]
use crate::rendering::TerminalRenderer;
#[cfg(feature = "video")]
use crate::visualization::braille::BrailleGrid;
#[cfg(feature = "video")]
use crate::visualization::{Color, GridBuffer};

#[cfg(feature = "video")]
use super::{blit_luma_to_braille, draw_centered, otsu_threshold, ColorMode};

#[cfg(feature = "video")]
pub fn run_webcam_capture(device_index: usize) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    const FPS_SMOOTHING_WINDOW: usize = 30;

    // 1. Get terminal dimensions first to inform camera resolution choice
    let mut renderer = TerminalRenderer::new()?;
    let (w_cells0, h_cells0) = renderer.dimensions();
    let (mut w_cells, mut h_cells) = (w_cells0 as usize, h_cells0 as usize);

    // Calculate target pixel resolution (Braille uses 2x4 dots per cell)
    let target_pixel_w = w_cells * 2;
    let target_pixel_h = h_cells * 4;

    tracing::info!(
        "Terminal: {}x{} cells = {}x{} pixels (Braille dots)",
        w_cells,
        h_cells,
        target_pixel_w,
        target_pixel_h
    );

    // 2. Initialize Camera with SMART RESOLUTION GOVERNOR
    let index = CameraIndex::Index(device_index as u32);
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(index, requested).context("Failed to open camera")?;

    // Smart Governor: Find camera mode based on terminal size and advertised FPS.
    // Strategy: prefer the highest frame-rate mode that still provides enough
    // detail for the terminal, while avoiding oversized formats that waste CPU.
    let optimal_width = (target_pixel_w * 2) as u32; // 2x for quality
    let optimal_height = (target_pixel_h * 2) as u32;
    let minimum_width = target_pixel_w as u32;
    let minimum_height = target_pixel_h as u32;

    if let Ok(formats) = camera.compatible_camera_formats() {
        tracing::info!("Available camera formats: {}", formats.len());
        let max_fps = formats
            .iter()
            .map(|f| f.frame_rate())
            .max()
            .unwrap_or(1)
            .max(1);

        // Score each format based on how well it matches our needs
        let mut scored_formats: Vec<_> = formats
            .iter()
            .map(|f| {
                let res = f.resolution();
                let w = res.width();
                let h = res.height();
                let fps = f.frame_rate().max(1);
                let score = score_webcam_format(
                    w,
                    h,
                    fps,
                    max_fps,
                    optimal_width,
                    optimal_height,
                    minimum_width,
                    minimum_height,
                );
                let scale_factor =
                    ((w as f32 / optimal_width as f32) + (h as f32 / optimal_height as f32)) / 2.0;

                tracing::debug!(
                    "Format {}x{}@{}fps: scale={:.2}x, score={:.1}",
                    w,
                    h,
                    fps,
                    scale_factor,
                    score
                );

                (f, score, fps)
            })
            .collect();

        // Sort by score (best first)
        scored_formats.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| b.2.cmp(&a.2))
        });

        if let Some((best_format, score, fps)) = scored_formats.first() {
            let res = best_format.resolution();
            tracing::info!(
                "🎯 Smart Governor selected: {}x{} @ {} FPS (score: {:.1}, optimal was {}x{})",
                res.width(),
                res.height(),
                fps,
                score,
                optimal_width,
                optimal_height
            );
            if let Err(error) = camera.set_camera_format((*best_format).clone()) {
                tracing::warn!(
                    "Failed to set selected camera format {}x{} @ {} FPS: {}",
                    res.width(),
                    res.height(),
                    fps,
                    error
                );
            }
        }
    }

    camera
        .open_stream()
        .context("Failed to open camera stream")?;

    let actual_format = camera.camera_format();
    tracing::info!(
        "Webcam stream opened at {}x{} @ {} FPS",
        actual_format.resolution().width(),
        actual_format.resolution().height(),
        actual_format.frame_rate()
    );

    // 3. Setup rendering buffers
    let mut grid = GridBuffer::new(w_cells, h_cells);
    let mut braille = BrailleGrid::new(w_cells, h_cells);

    let mut target_dot_w = braille.dot_width();
    let mut target_dot_h = braille.dot_height();

    // 4. State
    let mut color_mode = ColorMode::Off;
    let mut manual_threshold: u8 = 128;
    let mut auto_thresh: bool = false;
    let mut show_hud: bool = true;

    // FPS tracking for performance monitoring
    use std::time::Instant;
    let mut last_frame_time = Instant::now();
    let mut frame_times = [0.0_f32; FPS_SMOOTHING_WINDOW];
    let mut frame_time_sum = 0.0_f32;
    let mut frame_time_count = 0usize;
    let mut frame_time_index = 0usize;
    let mut luma_bytes = vec![0u8; target_dot_w * target_dot_h];

    // 4. Loop - optimized for performance
    loop {
        let frame = camera.frame().context("Failed to get frame")?;
        let decoded = frame
            .decode_image::<RgbFormat>()
            .context("Failed to decode frame")?;

        // Convert to RgbImage
        let img: RgbImage =
            ImageBuffer::from_raw(decoded.width(), decoded.height(), decoded.into_raw())
                .context("Failed to create image buffer")?;

        let dst_w = target_dot_w;
        let dst_h = target_dot_h;

        // Use Nearest neighbor for much faster resizing (vs Triangle/Lanczos)
        let resized = imageops::resize(
            &img,
            dst_w as u32,
            dst_h as u32,
            imageops::FilterType::Nearest,
        );

        // Compute luma inline for better performance
        let rgb_bytes = resized.as_raw();
        if luma_bytes.len() != dst_w * dst_h {
            luma_bytes.resize(dst_w * dst_h, 0);
        }

        // Fast RGB to grayscale conversion (ITU-R BT.601 formula)
        for (dst, chunk) in luma_bytes.iter_mut().zip(rgb_bytes.chunks_exact(3)) {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            // Weighted average: 0.299*R + 0.587*G + 0.114*B
            let luma = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
            *dst = luma;
        }

        // Threshold
        let used_threshold: u8 = if auto_thresh {
            otsu_threshold(&luma_bytes)
        } else {
            manual_threshold
        };

        blit_luma_to_braille(&luma_bytes, dst_w, dst_h, used_threshold, &mut braille);

        // Color mapping - optimized
        match color_mode {
            ColorMode::Off => {
                for cy in 0..h_cells {
                    for cx in 0..w_cells {
                        let ch = braille.get_char(cx, cy);
                        grid.set_cell(cx, cy, ch);
                    }
                }
            }
            ColorMode::Grayscale => {
                for cy in 0..h_cells {
                    for cx in 0..w_cells {
                        let ch = braille.get_char(cx, cy);
                        let x = (cx * 2).min(dst_w - 1);
                        let y = (cy * 4).min(dst_h - 1);
                        let idx = y * dst_w + x;
                        let v = luma_bytes[idx];
                        grid.set_cell_with_color(cx, cy, ch, Color::new(v, v, v));
                    }
                }
            }
            ColorMode::Full => {
                for cy in 0..h_cells {
                    for cx in 0..w_cells {
                        let ch = braille.get_char(cx, cy);
                        let x = (cx * 2).min(dst_w - 1);
                        let y = (cy * 4).min(dst_h - 1);
                        let idx = (y * dst_w + x) * 3;
                        let r = rgb_bytes[idx];
                        let g = rgb_bytes[idx + 1];
                        let b = rgb_bytes[idx + 2];
                        grid.set_cell_with_color(cx, cy, ch, Color::new(r, g, b));
                    }
                }
            }
        }

        // Calculate FPS (rolling average over last 30 frames)
        let now = Instant::now();
        let frame_time = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;

        if frame_time_count == FPS_SMOOTHING_WINDOW {
            frame_time_sum -= frame_times[frame_time_index];
        } else {
            frame_time_count += 1;
        }
        frame_times[frame_time_index] = frame_time;
        frame_time_sum += frame_time;
        frame_time_index = (frame_time_index + 1) % FPS_SMOOTHING_WINDOW;

        let avg_frame_time = if frame_time_count > 0 {
            frame_time_sum / frame_time_count as f32
        } else {
            0.0
        };
        let fps = if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        };

        // HUD with FPS
        if show_hud {
            let status = format!(
                "WEBCAM [{:.1} FPS] | +/- thr={} | a auto={} | c color={}",
                fps,
                used_threshold,
                if auto_thresh { "ON" } else { "OFF" },
                match color_mode {
                    ColorMode::Off => "OFF",
                    ColorMode::Grayscale => "GRAY",
                    ColorMode::Full => "FULL",
                }
            );
            draw_centered(&mut grid, &status);
        }

        renderer.render_fast(&mut grid)?;

        // Input
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(k) => match k.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') if k.kind == KeyEventKind::Press => {
                        color_mode = match color_mode {
                            ColorMode::Off => ColorMode::Grayscale,
                            ColorMode::Grayscale => ColorMode::Full,
                            ColorMode::Full => ColorMode::Off,
                        };
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') if k.kind == KeyEventKind::Press => {
                        auto_thresh = !auto_thresh;
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        manual_threshold = manual_threshold.saturating_add(5);
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        manual_threshold = manual_threshold.saturating_sub(5);
                    }
                    KeyCode::F(1) if k.kind == KeyEventKind::Press => {
                        show_hud = !show_hud;
                    }
                    _ => {}
                },
                Event::Resize(new_w, new_h) => {
                    w_cells = new_w as usize;
                    h_cells = new_h as usize;
                    grid = GridBuffer::new(w_cells, h_cells);
                    braille = BrailleGrid::new(w_cells, h_cells);
                    target_dot_w = braille.dot_width();
                    target_dot_h = braille.dot_height();
                    luma_bytes.resize(target_dot_w * target_dot_h, 0);
                }
                _ => {}
            }
        }
    }

    renderer.cleanup()?;
    Ok(())
}

#[cfg(feature = "video")]
fn score_webcam_format(
    width: u32,
    height: u32,
    fps: u32,
    max_fps: u32,
    optimal_width: u32,
    optimal_height: u32,
    minimum_width: u32,
    minimum_height: u32,
) -> f32 {
    let width_scale = width as f32 / optimal_width.max(1) as f32;
    let height_scale = height as f32 / optimal_height.max(1) as f32;
    let scale_factor = (width_scale + height_scale) / 2.0;

    let oversize_penalty = if scale_factor > 1.35 {
        (scale_factor - 1.35) * 90.0
    } else {
        0.0
    };

    let undersize_penalty = if scale_factor < 0.9 {
        (0.9 - scale_factor) * 170.0
    } else {
        0.0
    };

    let minimum_floor_penalty = if width < minimum_width || height < minimum_height {
        let width_shortfall = (minimum_width as f32 / width.max(1) as f32 - 1.0).max(0.0);
        let height_shortfall = (minimum_height as f32 / height.max(1) as f32 - 1.0).max(0.0);
        (width_shortfall + height_shortfall) * 140.0
    } else {
        0.0
    };

    let megapixel_penalty = ((width as u64 * height as u64) as f32 / 1_000_000.0) * 12.0;
    let fps_penalty = if max_fps > 0 {
        (1.0 - (fps as f32 / max_fps as f32)).max(0.0) * 120.0
    } else {
        0.0
    };

    let common_bonus = match (width, height) {
        (320, 240) | (640, 480) | (800, 600) => -12.0,
        (1280, 720) | (1920, 1080) if scale_factor > 2.0 => 35.0,
        _ => 0.0,
    };

    oversize_penalty
        + undersize_penalty
        + minimum_floor_penalty
        + megapixel_penalty
        + fps_penalty
        + common_bonus
}
