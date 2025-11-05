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
/// Decodes frames with FFmpeg, maps to Braille, and renders to the terminal.
#[cfg(feature = "video")]
pub fn run_video_playback(path: &str) -> Result<()> {
    use anyhow::{Context, Result};
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use ffmpeg_next as ffmpeg;
    use ffmpeg::{
        codec,
        format,
        media::Type,
        software::scaling::{context::Context as Scaler, flag::Flags},
        util::{frame::video::Video, pixel::Pixel},
    };
    use image::{imageops, ImageBuffer, RgbImage};
    use std::time::{Duration, Instant};

    ffmpeg::init().context("ffmpeg init failed")?;

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

    // Set up renderer and grids
    let mut renderer = TerminalRenderer::new()?;
    let (w_cells0, h_cells0) = renderer.dimensions();
    let (mut w_cells, mut h_cells) = (w_cells0 as usize, h_cells0 as usize);
    let mut grid = GridBuffer::new(w_cells, h_cells);
    let mut braille = BrailleGrid::new(w_cells, h_cells);

    // Target dot resolution for thresholding (updated on resize)
    let mut target_dot_w = braille.dot_width();
    let mut target_dot_h = braille.dot_height();

    // Determine playback FPS
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

            // Resize to the Braille dot grid and convert to luma
            let resized = imageops::resize(
                &img,
                target_dot_w as u32,
                target_dot_h as u32,
                imageops::FilterType::Triangle,
            );
            let gray = image::DynamicImage::ImageRgb8(resized).to_luma8();

            // Blit to Braille, then into the GridBuffer
            braille.clear();
            blit_luma_to_braille(
                gray.as_raw(),
                gray.width() as usize,
                gray.height() as usize,
                128,
                &mut braille,
            );

            for cy in 0..h_cells {
                for cx in 0..w_cells {
                    grid.set_cell(cx, cy, braille.get_char(cx, cy));
                }
            }

            renderer.render(&grid)?;

            // Input and resize handling
            while event::poll(Duration::from_millis(0))? {
                match event::read()? {
                    Event::Key(k) if k.kind == KeyEventKind::Press => {
                        match k.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                decoder.send_eof()?;
                                renderer.cleanup()?;
                                return Ok(());
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

    // Drain decoder
    decoder.send_eof()?;
    let mut frame = Video::empty();
    while decoder.receive_frame(&mut frame).is_ok() {}

    renderer.cleanup()?;
    Ok(())
}

/// Map an 8-bit luminance image onto a BrailleGrid via nearest-neighbor scaling
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

