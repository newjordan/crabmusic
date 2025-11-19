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
use std::time::Duration;

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

    // 1. Get terminal dimensions first to inform camera resolution choice
    let mut renderer = TerminalRenderer::new()?;
    let (w_cells0, h_cells0) = renderer.dimensions();
    let (mut w_cells, mut h_cells) = (w_cells0 as usize, h_cells0 as usize);
    
    // Calculate target pixel resolution (Braille uses 2x4 dots per cell)
    let target_pixel_w = w_cells * 2;
    let target_pixel_h = h_cells * 4;
    
    tracing::info!(
        "Terminal: {}x{} cells = {}x{} pixels (Braille dots)",
        w_cells, h_cells, target_pixel_w, target_pixel_h
    );

    // 2. Initialize Camera with SMART RESOLUTION GOVERNOR
    let index = CameraIndex::Index(device_index as u32);
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(index, requested).context("Failed to open camera")?;
    
    // Smart Governor: Find optimal camera resolution based on terminal size
    // Strategy: Use ~2x terminal resolution for quality, but not more (diminishing returns)
    let optimal_width = (target_pixel_w * 2) as u32;  // 2x for quality
    let optimal_height = (target_pixel_h * 2) as u32;
    
    if let Ok(formats) = camera.compatible_camera_formats() {
        tracing::info!("Available camera formats: {}", formats.len());
        
        // Score each format based on how well it matches our needs
        let mut scored_formats: Vec<_> = formats.iter().map(|f| {
            let res = f.resolution();
            let w = res.width();
            let h = res.height();
            
            // Calculate how much we'd need to scale (prefer minimal scaling)
            let scale_factor = ((w as f32 / optimal_width as f32) + 
                               (h as f32 / optimal_height as f32)) / 2.0;
            
            // Penalty for being too large (wasted processing)
            let size_penalty = if scale_factor > 1.5 {
                (scale_factor - 1.5) * 100.0
            } else {
                0.0
            };
            
            // Penalty for being too small (quality loss)
            let quality_penalty = if scale_factor < 0.8 {
                (0.8 - scale_factor) * 150.0
            } else {
                0.0
            };
            
            // Prefer common resolutions (they're usually better optimized)
            let common_bonus = match (w, h) {
                (640, 480) | (320, 240) | (800, 600) => -20.0,
                (1280, 720) | (1920, 1080) => if scale_factor > 2.0 { 50.0 } else { 0.0 },
                _ => 0.0,
            };
            
            // Total score (lower is better)
            let score = size_penalty + quality_penalty + common_bonus + 
                       ((w * h) as f32 / 1_000_000.0) * 10.0; // Slight penalty for total pixels
            
            tracing::debug!(
                "Format {}x{}: scale={:.2}x, score={:.1} (size_pen={:.1}, qual_pen={:.1})",
                w, h, scale_factor, score, size_penalty, quality_penalty
            );
            
            (f, score)
        }).collect();
        
        // Sort by score (best first)
        scored_formats.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some((best_format, score)) = scored_formats.first() {
            let res = best_format.resolution();
            tracing::info!(
                "🎯 Smart Governor selected: {}x{} (score: {:.1}, optimal was {}x{})",
                res.width(), res.height(), score, optimal_width, optimal_height
            );
            let _ = camera.set_camera_format((*best_format).clone());
        }
    }
    
    camera.open_stream().context("Failed to open camera stream")?;

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
    let mut last_used_threshold: u8 = manual_threshold;
    
    // FPS tracking for performance monitoring
    use std::time::Instant;
    let mut frame_times: Vec<f32> = Vec::with_capacity(30);
    let mut last_frame_time = Instant::now();

    // 4. Loop - optimized for performance
    loop {
        let frame = camera.frame().context("Failed to get frame")?;
        let decoded = frame.decode_image::<RgbFormat>().context("Failed to decode frame")?;
        
        // Convert to RgbImage
        let img: RgbImage = ImageBuffer::from_raw(decoded.width(), decoded.height(), decoded.into_raw())
            .context("Failed to create image buffer")?;

        let dst_w = target_dot_w;
        let dst_h = target_dot_h;
        
        // Use Nearest neighbor for much faster resizing (vs Triangle/Lanczos)
        let resized = imageops::resize(&img, dst_w as u32, dst_h as u32, imageops::FilterType::Nearest);
        
        // Compute luma inline for better performance
        let rgb_bytes = resized.as_raw();
        let mut luma_bytes = Vec::with_capacity(dst_w * dst_h);
        
        // Fast RGB to grayscale conversion (ITU-R BT.601 formula)
        for chunk in rgb_bytes.chunks_exact(3) {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            // Weighted average: 0.299*R + 0.587*G + 0.114*B
            let luma = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
            luma_bytes.push(luma);
        }

        // Threshold
        braille.clear();
        let used_threshold: u8 = if auto_thresh {
            otsu_threshold(&luma_bytes)
        } else {
            manual_threshold
        };
        last_used_threshold = used_threshold;

        blit_luma_to_braille(&luma_bytes, dst_w, dst_h, used_threshold, &mut braille);

        // Color mapping - optimized
        for cy in 0..h_cells {
            for cx in 0..w_cells {
                let ch = braille.get_char(cx, cy);
                match color_mode {
                    ColorMode::Off => {
                        grid.set_cell(cx, cy, ch);
                    }
                    ColorMode::Grayscale => {
                        // Simple sampling
                        let x = (cx * 2).min(dst_w - 1);
                        let y = (cy * 4).min(dst_h - 1);
                        let idx = y * dst_w + x;
                        let v = luma_bytes[idx];
                        grid.set_cell_with_color(cx, cy, ch, Color::new(v, v, v));
                    }
                    ColorMode::Full => {
                        let x = (cx * 2).min(dst_w - 1);
                        let y = (cy * 4).min(dst_h - 1);
                        let idx = (y * dst_w + x) * 3;
                        let r = rgb_bytes[idx];
                        let g = rgb_bytes[idx+1];
                        let b = rgb_bytes[idx+2];
                        grid.set_cell_with_color(cx, cy, ch, Color::new(r, g, b));
                    }
                }
            }
        }

        // Calculate FPS (rolling average over last 30 frames)
        let frame_time = last_frame_time.elapsed().as_secs_f32();
        last_frame_time = Instant::now();
        
        frame_times.push(frame_time);
        if frame_times.len() > 30 {
            frame_times.remove(0);
        }
        
        let avg_frame_time = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
        let fps = if avg_frame_time > 0.0 { 1.0 / avg_frame_time } else { 0.0 };

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

        renderer.render(&grid)?;

        // Input
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(k) => {
                    match k.code {
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
                    }
                }
                Event::Resize(new_w, new_h) => {
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
    }

    renderer.cleanup()?;
    Ok(())
}
