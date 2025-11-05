//! Image import and drag-and-drop viewing (Braille renderer)
//!
//! Two entrypoints:
//! - render_image(path): load + convert to Braille + render
//! - drop_loop(): waits for terminal paste events (drag-and-drop pastes paths)

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, EnableBracketedPaste, DisableBracketedPaste};
use crossterm::execute;
use std::io::stdout;

use crate::rendering::TerminalRenderer;
use crate::visualization::{GridBuffer, Color};
use crate::visualization::braille::BrailleGrid;
use crate::video::blit_luma_to_braille;


#[cfg(feature = "image")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorMode {
    Off,
    Grayscale,
    Full,
}

#[cfg(feature = "image")]
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

#[cfg(feature = "image")]
fn prepare_luma_dot_sized(
    img: &image::DynamicImage,
    dot_w: u32,
    dot_h: u32,
    letterbox: bool,
) -> image::GrayImage {
    use image::{imageops, DynamicImage, GenericImageView, GrayImage};
    if !letterbox {
        let resized = imageops::resize(&img.to_rgb8(), dot_w, dot_h, imageops::FilterType::Triangle);
        return DynamicImage::ImageRgb8(resized).to_luma8();
    }

    // Letterbox: preserve aspect ratio and center within dot area
    let (src_w, src_h) = img.dimensions();
    if src_w == 0 || src_h == 0 || dot_w == 0 || dot_h == 0 {
        return GrayImage::new(dot_w, dot_h);
    }
    let src_aspect = src_w as f32 / src_h as f32;
    let dst_aspect = dot_w as f32 / dot_h as f32;

    let (fit_w, fit_h) = if src_aspect > dst_aspect {
        // Fit by width
        let w = dot_w;
        let h = ((dot_w as f32 / src_aspect).round() as u32).max(1);
        (w, h.min(dot_h))
    } else {
        // Fit by height
        let h = dot_h;
        let w = ((dot_h as f32 * src_aspect).round() as u32).max(1);
        (w.min(dot_w), h)
    };

    let resized = imageops::resize(&img.to_rgb8(), fit_w, fit_h, imageops::FilterType::Triangle);
    let gray = DynamicImage::ImageRgb8(resized).to_luma8();

    // Compose into full-size gray image with black bars
    let mut out = GrayImage::new(dot_w, dot_h);
    let off_x = ((dot_w - fit_w) / 2) as i64;
    let off_y = ((dot_h - fit_h) / 2) as i64;
    imageops::overlay(&mut out, &gray, off_x, off_y);
    out
}

#[cfg(feature = "image")]
fn prepare_rgb_dot_sized(
    img: &image::DynamicImage,
    dot_w: u32,
    dot_h: u32,
    letterbox: bool,
) -> image::RgbImage {
    use image::{imageops, DynamicImage, GenericImageView, RgbImage};
    if !letterbox {
        return imageops::resize(&img.to_rgb8(), dot_w, dot_h, imageops::FilterType::Triangle);
    }

    // Letterbox: preserve aspect ratio and center within dot area
    let (src_w, src_h) = img.dimensions();
    if src_w == 0 || src_h == 0 || dot_w == 0 || dot_h == 0 {
        return RgbImage::new(dot_w, dot_h);
    }
    let src_aspect = src_w as f32 / src_h as f32;
    let dst_aspect = dot_w as f32 / dot_h as f32;

    let (fit_w, fit_h) = if src_aspect > dst_aspect {
        // Fit by width
        let w = dot_w;
        let h = ((dot_w as f32 / src_aspect).round() as u32).max(1);
        (w, h.min(dot_h))
    } else {
        // Fit by height
        let h = dot_h;
        let w = ((dot_h as f32 * src_aspect).round() as u32).max(1);
        (w.min(dot_w), h)
    };

    let resized = imageops::resize(&img.to_rgb8(), fit_w, fit_h, imageops::FilterType::Triangle);

    // Compose into full-size RGB image with black bars
    let mut out = RgbImage::new(dot_w, dot_h);
    let off_x = ((dot_w - fit_w) / 2) as i64;
    let off_y = ((dot_h - fit_h) / 2) as i64;
    imageops::overlay(&mut out, &resized, off_x, off_y);
    out
}


#[cfg(feature = "image")]
fn fill_braille_from_luma(braille: &mut BrailleGrid, luma: &image::GrayImage, threshold: u8) {
    braille.clear();
    // If dimensions match dot dims, blit is effectively 1:1
    blit_luma_to_braille(
        luma.as_raw(),
        luma.width() as usize,
        luma.height() as usize,
        threshold,
        braille,
    );
}

#[cfg(feature = "image")]
fn copy_braille_to_grid(
    grid: &mut GridBuffer,
    braille: &BrailleGrid,
    luma: &image::GrayImage,
    color_mode: ColorMode,
    rgb_opt: Option<&image::RgbImage>,
) {
    let w_cells = braille.width();
    let h_cells = braille.height();

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
                    // Average luma over this cell's 2x4 dots
                    let base_x = (cx * 2) as u32;
                    let base_y = (cy * 4) as u32;
                    let mut acc: u32 = 0;
                    let mut count: u32 = 0;
                    for dy in 0..4 {
                        for dx in 0..2 {
                            let x = base_x + dx;
                            let y = base_y + dy;
                            if x < luma.width() && y < luma.height() {
                                acc += luma.get_pixel(x, y)[0] as u32;
                                count += 1;
                            }
                        }
                    }
                    let v = if count > 0 { (acc / count) as u8 } else { 0 };
                    let color = Color::new(v, v, v);
                    grid.set_cell_with_color(cx, cy, ch, color);
                }
            }
        }
        ColorMode::Full => {
            let rgb = rgb_opt.expect("RGB image required for full color mode");
            for cy in 0..h_cells {
                for cx in 0..w_cells {
                    let ch = braille.get_char(cx, cy);
                    // Average RGB over this cell's 2x4 dots
                    let base_x = (cx * 2) as u32;
                    let base_y = (cy * 4) as u32;
                    let mut r_acc: u32 = 0;
                    let mut g_acc: u32 = 0;
                    let mut b_acc: u32 = 0;
                    let mut count: u32 = 0;
                    for dy in 0..4 {
                        for dx in 0..2 {
                            let x = base_x + dx;
                            let y = base_y + dy;
                            if x < rgb.width() && y < rgb.height() {
                                let p = rgb.get_pixel(x, y);
                                r_acc += p[0] as u32;
                                g_acc += p[1] as u32;
                                b_acc += p[2] as u32;
                                count += 1;
                            }
                        }
                    }
                    if count > 0 {
                        let r = (r_acc / count) as u8;
                        let g = (g_acc / count) as u8;
                        let b = (b_acc / count) as u8;
                        let color = Color::new(r, g, b);
                        grid.set_cell_with_color(cx, cy, ch, color);
                    } else {
                        grid.set_cell(cx, cy, ch);
                    }
                }
            }
        }
    }
}
#[cfg(feature = "image")]
fn render_morph_frame(
    img_a: &image::DynamicImage,
    img_b: &image::DynamicImage,
    braille: &mut BrailleGrid,
    grid: &mut GridBuffer,
    t: f32,
    manual_threshold: u8,
    auto_thresh: bool,
    letterbox: bool,
    color_mode: ColorMode,
) -> u8 {
    use image::{GrayImage, RgbImage};

    let dot_w = braille.dot_width() as u32;
    let dot_h = braille.dot_height() as u32;

    // Prepare source images at current dot resolution
    let luma_a = prepare_luma_dot_sized(img_a, dot_w, dot_h, letterbox);
    let luma_b = prepare_luma_dot_sized(img_b, dot_w, dot_h, letterbox);

    // Blend luma
    let mut luma_blend = GrayImage::new(dot_w, dot_h);
    let (aw, ah) = (luma_a.width(), luma_a.height());
    let (bw, bh) = (luma_b.width(), luma_b.height());
    let alpha = t.clamp(0.0, 1.0);
    for y in 0..dot_h {
        for x in 0..dot_w {
            let a = if x < aw && y < ah { luma_a.get_pixel(x, y)[0] as f32 } else { 0.0 };
            let b = if x < bw && y < bh { luma_b.get_pixel(x, y)[0] as f32 } else { 0.0 };
            let v = ((1.0 - alpha) * a + alpha * b).round().clamp(0.0, 255.0) as u8;
            luma_blend.put_pixel(x, y, image::Luma([v]));
        }
    }

    // Determine threshold for braille dotting
    let used_threshold = if auto_thresh { otsu_threshold(luma_blend.as_raw()) } else { manual_threshold };

    // Fill braille from blended luma
    fill_braille_from_luma(braille, &luma_blend, used_threshold);

    // Optional RGB blend for colorized output
    let rgb_opt = if matches!(color_mode, ColorMode::Full) {
        let rgb_a = prepare_rgb_dot_sized(img_a, dot_w, dot_h, letterbox);
        let rgb_b = prepare_rgb_dot_sized(img_b, dot_w, dot_h, letterbox);
        let mut rgb_blend = RgbImage::new(dot_w, dot_h);
        for y in 0..dot_h {
            for x in 0..dot_w {
                let pa = if x < rgb_a.width() && y < rgb_a.height() { rgb_a.get_pixel(x, y) } else { &image::Rgb([0,0,0]) };
                let pb = if x < rgb_b.width() && y < rgb_b.height() { rgb_b.get_pixel(x, y) } else { &image::Rgb([0,0,0]) };
                let r = ((1.0 - alpha) * pa[0] as f32 + alpha * pb[0] as f32).round().clamp(0.0, 255.0) as u8;
                let g = ((1.0 - alpha) * pa[1] as f32 + alpha * pb[1] as f32).round().clamp(0.0, 255.0) as u8;
                let b = ((1.0 - alpha) * pa[2] as f32 + alpha * pb[2] as f32).round().clamp(0.0, 255.0) as u8;
                rgb_blend.put_pixel(x, y, image::Rgb([r, g, b]));
            }
        }
        Some(rgb_blend)
    } else { None };

    copy_braille_to_grid(grid, braille, &luma_blend, color_mode, rgb_opt.as_ref());
    used_threshold
}


#[cfg(feature = "image")]
fn render_with_state(
    img: &image::DynamicImage,
    braille: &mut BrailleGrid,
    grid: &mut GridBuffer,
    threshold: u8,
    auto_threshold: bool,
    letterbox: bool,
    color_mode: ColorMode,
) -> u8 {
    let dot_w = braille.dot_width() as u32;
    let dot_h = braille.dot_height() as u32;
    let luma = prepare_luma_dot_sized(img, dot_w, dot_h, letterbox);
    let used_threshold = if auto_threshold { otsu_threshold(luma.as_raw()) } else { threshold };
    fill_braille_from_luma(braille, &luma, used_threshold);
    let rgb_opt = if matches!(color_mode, ColorMode::Full) {
        Some(prepare_rgb_dot_sized(img, dot_w, dot_h, letterbox))
    } else {
        None
    };
    copy_braille_to_grid(grid, braille, &luma, color_mode, rgb_opt.as_ref());
    used_threshold
}

#[cfg(feature = "image")]
fn draw_status(
    grid: &mut GridBuffer,
    path_opt: Option<&str>,
    used_threshold: u8,
    auto_thresh: bool,
    letterbox: bool,
    color_mode: ColorMode,
    typed: &str,
) {
    let name = path_opt
        .and_then(|p| std::path::Path::new(p).file_name().and_then(|s| s.to_str()))
        .unwrap_or("<none>");
    let color_str = match color_mode {
        ColorMode::Off => "OFF",
        ColorMode::Grayscale => "GRAY",
        ColorMode::Full => "FULL",
    };
    let status = format!(
        "File: {} | +/- thr={} | a auto={} | l letterbox={} | c color={} | x max | s save | q quit",
        name,
        used_threshold,
        if auto_thresh { "ON" } else { "OFF" },
        if letterbox { "ON" } else { "OFF" },
        color_str,
    );
    draw_centered(grid, &status);
    draw_left(grid, 1, &format!("Input: {}", typed));
}

/// Helper: draw centered instruction text on first line of grid
fn draw_centered(grid: &mut GridBuffer, text: &str) {
    let start_x = (grid.width().saturating_sub(text.len())) / 2;
    for (i, ch) in text.chars().enumerate() {
        let x = start_x + i;
        if x < grid.width() {
            grid.set_cell(x, 0, ch);
        }
    }
}

/// Helper: draw left-aligned text at a specific row
fn draw_left(grid: &mut GridBuffer, row: usize, text: &str) {
    if row >= grid.height() { return; }
    for (i, ch) in text.chars().enumerate() {
        if i < grid.width() {
            grid.set_cell(i, row, ch);
        } else {
            break;
        }
    }
}

#[cfg(feature = "image")]
pub fn render_image(path: &str, morph_second: Option<&str>, morph_duration_override: Option<u64>) -> Result<()> {
    use image::DynamicImage;

    let img = image::open(path).with_context(|| format!("open image {}", path))?;

    let mut renderer = TerminalRenderer::new()?;
    let (w_cells, h_cells) = renderer.dimensions();
    let (w_cells, h_cells) = (w_cells as usize, h_cells as usize);

    let mut grid = GridBuffer::new(w_cells, h_cells);
    let mut braille = BrailleGrid::new(w_cells, h_cells);

    // Enable bracketed paste for consistency (we also accept paste to switch images)
    let mut out = stdout();
    let _ = execute!(out, EnableBracketedPaste);

    // State
    let mut typed = String::new();
    let mut current_img: Option<DynamicImage> = Some(img);
    let mut current_path: Option<String> = Some(path.to_string());

    let mut manual_threshold: u8 = 128;
    let mut used_threshold: u8 = 128;
    let mut auto_thresh = false;
    let mut letterbox = true;
    let mut color_mode = ColorMode::Off;

    // Initial render
    if let Some(ref img) = current_img {
        grid.clear();
        used_threshold = render_with_state(
            img,
            &mut braille,
            &mut grid,
            manual_threshold,
            auto_thresh,
            letterbox,
            color_mode,
        );
    }
    draw_status(
        &mut grid,
        current_path.as_deref(),
        used_threshold,
        auto_thresh,
        letterbox,
        color_mode,
        &typed,
    );
    renderer.render(&grid)?;

    // Morph/transition state (two-image crossfade)
    let mut morph_prompting: bool = false; // waiting for second path
    let mut morph_mode: bool = false;      // morph animation active
    let mut morph_paused: bool = false;
    let mut morph_other_img: Option<DynamicImage> = None;
    let mut morph_t: f32 = 0.0;            // 0.0..1.0 blend factor
    let mut morph_dir: i32 = 1;            // 1 forward, -1 backward (ping-pong)
    let mut morph_last_tick = std::time::Instant::now();
    let mut morph_duration_ms: u64 = 2500; // default 2.5s per A->B

        // Auto-start morph if a second image path was provided via CLI
        if let Some(second_path) = morph_second {
            match image::open(second_path) {
                Ok(new_img) => {
                    morph_other_img = Some(new_img);
                    morph_mode = true;
                    morph_paused = false;
                    morph_t = 0.0;
                    morph_dir = 1;
                    if let Some(ms) = morph_duration_override { morph_duration_ms = ms.max(50); }
                    if let (Some(ref img_a), Some(ref img_b)) = (current_img.as_ref(), morph_other_img.as_ref()) {
                        grid.clear();
                        used_threshold = render_morph_frame(
                            img_a,
                            img_b,
                            &mut braille,
                            &mut grid,
                            morph_t,
                            manual_threshold,
                            auto_thresh,
                            letterbox,
                            color_mode,
                        );
                        draw_status(
                            &mut grid,
                            current_path.as_deref(),
                            used_threshold,
                            auto_thresh,
                            letterbox,
                            color_mode,
                            &typed,
                        );
                        draw_left(&mut grid, 2, &format!(
                            "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                            (morph_t * 100.0).clamp(0.0, 100.0),
                            if morph_paused { "PAUSED" } else { "PLAY" },
                            morph_duration_ms,
                        ));
                        renderer.render(&grid)?;
                    }
                }
                Err(err) => {
                    grid.clear();
                    draw_centered(&mut grid, &format!("Failed to open second image (morph): {}", err));
                    draw_status(
                        &mut grid,
                        current_path.as_deref(),
                        used_threshold,
                        auto_thresh,
                        letterbox,
                        color_mode,
                        &typed,
                    );
                    renderer.render(&grid)?;
                }
            }
        }


    'outer: loop {
        let poll_ms = if morph_mode && !morph_paused { 16 } else { 100 };
        if event::poll(std::time::Duration::from_millis(poll_ms))? {
            match event::read()? {
                Event::Paste(s) => {
                    typed = s;
                    let candidate = typed.trim().trim_matches('"').to_string();
                    if !candidate.is_empty() {
                        if morph_prompting {
                            match image::open(&candidate) {
                                Ok(new_img) => {
                                    morph_other_img = Some(new_img);
                                    morph_mode = true;
                                    morph_paused = false;
                                    morph_t = 0.0;
                                    morph_dir = 1;
                                    typed.clear();
                                    if let (Some(ref img_a), Some(ref img_b)) = (current_img.as_ref(), morph_other_img.as_ref()) {
                                        grid.clear();
                                        used_threshold = render_morph_frame(
                                            img_a,
                                            img_b,
                                            &mut braille,
                                            &mut grid,
                                            morph_t,
                                            manual_threshold,
                                            auto_thresh,
                                            letterbox,
                                            color_mode,
                                        );
                                    }
                                }
                                Err(err) => {
                                    grid.clear();
                                    draw_centered(&mut grid, &format!("Failed to open second image: {}", err));
                                }
                            }
                            morph_prompting = false;
                        } else {
                            match image::open(&candidate) {
                                Ok(new_img) => {
                                    current_img = Some(new_img);
                                    current_path = Some(candidate);
                                    morph_mode = false; // cancel any active morph
                                    typed.clear();
                                    if let Some(ref img) = current_img {
                                        grid.clear();
                                        used_threshold = render_with_state(
                                            img,
                                            &mut braille,
                                            &mut grid,
                                            manual_threshold,
                                            auto_thresh,
                                            letterbox,
                                            color_mode,
                                        );
                                    }
                                }
                                Err(err) => {
                                    grid.clear();
                                    draw_centered(&mut grid, &format!("Failed to open image: {}", err));
                                }
                            }
                        }
                    }
                    draw_status(
                        &mut grid,
                        current_path.as_deref(),
                        used_threshold,
                        auto_thresh,
                        letterbox,
                        color_mode,
                        &typed,
                    );
                    if morph_prompting {
                        draw_left(&mut grid, 2, "Paste second image path to start morph...");
                    } else if morph_mode {
                        draw_left(&mut grid, 2, &format!(
                            "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                            (morph_t * 100.0).clamp(0.0, 100.0),
                            if morph_paused { "PAUSED" } else { "PLAY" },
                            morph_duration_ms,
                        ));
                    }
                    renderer.render(&grid)?;
                }
                Event::Resize(new_w, new_h) => {
                    // Rebuild grids to match new terminal size and re-render
                    let (w_cells, h_cells) = (new_w as usize, new_h as usize);
                    grid = GridBuffer::new(w_cells, h_cells);
                    braille = BrailleGrid::new(w_cells, h_cells);
                    if morph_mode {
                        if let (Some(ref img_a), Some(ref img_b)) = (current_img.as_ref(), morph_other_img.as_ref()) {
                            grid.clear();
                            used_threshold = render_morph_frame(
                                img_a,
                                img_b,
                                &mut braille,
                                &mut grid,
                                morph_t,
                                manual_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                            );
                        }
                    } else if let Some(ref img) = current_img {
                        grid.clear();
                        used_threshold = render_with_state(
                            img,
                            &mut braille,
                            &mut grid,
                            manual_threshold,
                            auto_thresh,
                            letterbox,
                            color_mode,
                        );
                    }
                    draw_status(
                        &mut grid,
                        current_path.as_deref(),
                        used_threshold,
                        auto_thresh,
                        letterbox,
                        color_mode,
                        &typed,
                    );
                    if morph_mode {
                        draw_left(&mut grid, 2, &format!(
                            "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                            (morph_t * 100.0).clamp(0.0, 100.0),
                            if morph_paused { "PAUSED" } else { "PLAY" },
                            morph_duration_ms,
                        ));
                    }
                    renderer.render(&grid)?;
                }

                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    match k.code {
                        KeyCode::Esc => {
                            if !typed.is_empty() || morph_prompting {
                                morph_prompting = false;
                                typed.clear();
                                draw_status(
                                    &mut grid,
                                    current_path.as_deref(),
                                    used_threshold,
                                    auto_thresh,
                                    letterbox,
                                    color_mode,
                                    &typed,
                                );
                                if morph_mode {
                                    draw_left(&mut grid, 2, &format!(
                                        "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                                        (morph_t * 100.0).clamp(0.0, 100.0),
                                        if morph_paused { "PAUSED" } else { "PLAY" },
                                        morph_duration_ms,
                                    ));
                                }
                                renderer.render(&grid)?;
                            } else {
                                break 'outer;
                            }
                        },
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                if typed.is_empty() {
                                    break 'outer;
                                } else {
                                    if let KeyCode::Char(c) = k.code { typed.push(c); }
                                    draw_status(
                                        &mut grid,
                                        current_path.as_deref(),
                                        used_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                        &typed,
                                    );
                                    renderer.render(&grid)?;
                                }
                            }
                        KeyCode::Enter => {
                            // Allow typing a new path and pressing Enter to switch
                            let candidate = typed.trim().trim_matches('"').to_string();
                            if !candidate.is_empty() {
                                match image::open(&candidate) {
                                    Ok(new_img) => {
                                        current_img = Some(new_img);
                                        current_path = Some(candidate);
                                        typed.clear();

                                        if let Some(ref img) = current_img {
                                            grid.clear();
                                            used_threshold = render_with_state(
                                                img,
                                                &mut braille,
                                                &mut grid,
                                                manual_threshold,
                                                auto_thresh,
                                                letterbox,
                                                color_mode,
                                            );
                                        }
                                    }
                                    Err(err) => {
                                        grid.clear();
                                        draw_centered(&mut grid, &format!("Failed to open image: {}", err));
                                    }
                                }
                            }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Backspace => {
                            typed.pop();
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            if typed.is_empty() && current_img.is_some() {
                                if manual_threshold < 250 {
                                    manual_threshold = manual_threshold.saturating_add(5);
                                }
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            if typed.is_empty() && current_img.is_some() {
                                if manual_threshold > 5 {
                                    manual_threshold = manual_threshold.saturating_sub(5);
                                }
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            if typed.is_empty() && current_img.is_some() {
                                auto_thresh = !auto_thresh;
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }


                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            if typed.is_empty() && current_img.is_some() {
                                letterbox = !letterbox;
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if typed.is_empty() && current_img.is_some() {
                                color_mode = match color_mode {
                                    ColorMode::Off => ColorMode::Grayscale,
                                    ColorMode::Grayscale => ColorMode::Full,
                                    ColorMode::Full => ColorMode::Off,
                                };
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }

                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            if typed.is_empty() && current_img.is_some() {
                                if morph_mode {
                                    // Stop morph
                                    morph_mode = false;
                                    morph_prompting = false;
                                    grid.clear();
                                    if let Some(ref img) = current_img {
                                        used_threshold = render_with_state(
                                            img,
                                            &mut braille,
                                            &mut grid,
                                            manual_threshold,
                                            auto_thresh,
                                            letterbox,
                                            color_mode,
                                        );
                                    }
                                    draw_status(
                                        &mut grid,
                                        current_path.as_deref(),
                                        used_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                        &typed,
                                    );
                                    renderer.render(&grid)?;
                                } else {
                                    // Prompt for second image
                                    morph_prompting = true;
                                    typed.clear();
                                    draw_status(
                                        &mut grid,
                                        current_path.as_deref(),
                                        used_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                        &typed,
                                    );
                                    draw_left(&mut grid, 2, "Paste second image path to start morph...");
                                    renderer.render(&grid)?;
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }

                        KeyCode::Char(' ') => {
                            if typed.is_empty() && morph_mode {
                                morph_paused = !morph_paused;
                                draw_status(
                                    &mut grid,
                                    current_path.as_deref(),
                                    used_threshold,
                                    auto_thresh,
                                    letterbox,
                                    color_mode,
                                    &typed,
                                );
                                draw_left(&mut grid, 2, &format!(
                                    "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                                    (morph_t * 100.0).clamp(0.0, 100.0),
                                    if morph_paused { "PAUSED" } else { "PLAY" },
                                    morph_duration_ms,
                                ));
                                renderer.render(&grid)?;
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }

                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            if typed.is_empty() && morph_mode {
                                morph_dir *= -1;
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }

                        KeyCode::Char('[') => {
                            if typed.is_empty() {
                                morph_duration_ms = (morph_duration_ms.saturating_sub(200)).max(100);
                                if morph_mode {
                                    draw_status(
                                        &mut grid,
                                        current_path.as_deref(),
                                        used_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                        &typed,
                                    );
                                    draw_left(&mut grid, 2, &format!(
                                        "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                                        (morph_t * 100.0).clamp(0.0, 100.0),
                                        if morph_paused { "PAUSED" } else { "PLAY" },
                                        morph_duration_ms,
                                    ));
                                    renderer.render(&grid)?;
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }

                        KeyCode::Char(']') => {
                            if typed.is_empty() {
                                morph_duration_ms = (morph_duration_ms.saturating_add(200)).min(20000);
                                if morph_mode {
                                    draw_status(
                                        &mut grid,
                                        current_path.as_deref(),
                                        used_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                        &typed,
                                    );
                                    draw_left(&mut grid, 2, &format!(
                                        "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                                        (morph_t * 100.0).clamp(0.0, 100.0),
                                        if morph_paused { "PAUSED" } else { "PLAY" },
                                        morph_duration_ms,
                                    ));
                                    renderer.render(&grid)?;
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }


                        KeyCode::Char('x') | KeyCode::Char('X') => {
                            if typed.is_empty() {
                                let before = renderer.dimensions();
                                let _ = renderer.maximize_canvas();
                                let after = renderer.dimensions();
                                let (nw, nh) = after;
                                let (w_cells, h_cells) = (nw as usize, nh as usize);
                                grid = GridBuffer::new(w_cells, h_cells);
                                braille = BrailleGrid::new(w_cells, h_cells);
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                                if after == before {
                                    draw_left(
                                        &mut grid,
                                        2,
                                        "Maximize not supported by this terminal. For a larger canvas: start cmd /k \"mode con: cols=240 lines=120 && cargo run -- --image-drop\"",
                                    );
                                }
                                draw_status(
                                    &mut grid,
                                    current_path.as_deref(),
                                    used_threshold,
                                    auto_thresh,
                                    letterbox,
                                    color_mode,
                                    &typed,
                                );
                                renderer.render(&grid)?;
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }

                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            if typed.is_empty() && current_img.is_some() {
                                if let Some(ref path) = current_path {
                                    let mut out_str = String::new();
                                    for cy in 0..braille.height() {
                                        for cx in 0..braille.width() {
                                            out_str.push(braille.get_char(cx, cy));
                                        }
                                        out_str.push('\n');
                                    }
                                    let p = std::path::Path::new(path);
                                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                                    let parent = p.parent().unwrap_or(std::path::Path::new("."));
                                    let save_path = parent.join(format!("{}.braille.txt", stem));
                                    let msg = match std::fs::write(&save_path, out_str) {
                                        Ok(_) => format!("Saved: {}", save_path.display()),
                                        Err(err) => format!("Save failed: {}", err),
                                    };
                                    draw_left(&mut grid, 2, &msg);
                                    renderer.render(&grid)?;
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }
                        KeyCode::Char(c) => {
                            typed.push(c);
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        // Morph animation tick (runs even when no events)
        if morph_mode && !morph_paused {
            if let (Some(ref img_a), Some(ref img_b)) = (current_img.as_ref(), morph_other_img.as_ref()) {
                let now = std::time::Instant::now();
                let dt = now.duration_since(morph_last_tick);
                morph_last_tick = now;
                let dur = std::time::Duration::from_millis(morph_duration_ms.max(1));
                let step = (dt.as_secs_f32() / dur.as_secs_f32()) * (morph_dir as f32);
                morph_t += step;
                if morph_t > 1.0 { morph_t = 2.0 - morph_t; morph_dir = -morph_dir; }
                if morph_t < 0.0 { morph_t = -morph_t; morph_dir = -morph_dir; }
                grid.clear();
                used_threshold = render_morph_frame(
                    img_a,
                    img_b,
                    &mut braille,
                    &mut grid,
                    morph_t,
                    manual_threshold,
                    auto_thresh,
                    letterbox,
                    color_mode,
                );
                draw_status(
                    &mut grid,
                    current_path.as_deref(),
                    used_threshold,
                    auto_thresh,
                    letterbox,
                    color_mode,
                    &typed,
                );
                draw_left(&mut grid, 2, &format!(
                    "Morph t={:.0}% {} dur={}ms | [Space] pause | m stop | [ faster | ] slower | r reverse",
                    (morph_t * 100.0).clamp(0.0, 100.0),
                    if morph_paused { "PAUSED" } else { "PLAY" },
                    morph_duration_ms,
                ));
                renderer.render(&grid)?;
            }
        }

    }

    let _ = execute!(stdout(), DisableBracketedPaste);

    renderer.cleanup()?;
    Ok(())
}

#[cfg(not(feature = "image"))]
pub fn render_image(_path: &str) -> Result<()> {
    anyhow::bail!("Image support requires building with `--features image`")
}

#[cfg(feature = "image")]
pub fn drop_loop() -> Result<()> {
    use image::DynamicImage;

    let mut renderer = TerminalRenderer::new()?;
    let (w_cells, h_cells) = renderer.dimensions();
    let (w_cells, h_cells) = (w_cells as usize, h_cells as usize);
    let mut grid = GridBuffer::new(w_cells, h_cells);
    let mut braille = BrailleGrid::new(w_cells, h_cells);

    // Try to enable bracketed paste (needed for Event::Paste in many terminals)
    let mut out = stdout();
    let _ = execute!(out, EnableBracketedPaste);

    // State
    let mut typed = String::new();
    let mut current_img: Option<DynamicImage> = None;
    let mut current_path: Option<String> = None;

    let mut manual_threshold: u8 = 128;
    let mut used_threshold: u8 = 128;
    let mut auto_thresh = false;
    let mut letterbox = true;
    let mut color_mode = ColorMode::Off;

    // Initial prompt
    grid.clear();
    draw_status(
        &mut grid,
        current_path.as_deref(),
        used_threshold,
        auto_thresh,
        letterbox,
        color_mode,
        &typed,
    );
    renderer.render(&grid)?;

    'outer: loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Paste(s) => {
                    typed = s;
                    let candidate = typed.trim().trim_matches('"').to_string();
                    if !candidate.is_empty() {
                        match image::open(&candidate) {
                            Ok(img) => {
                                typed.clear();

                                current_img = Some(img);
                                current_path = Some(candidate.clone());
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            }
                            Err(err) => {
                                grid.clear();
                                draw_centered(&mut grid, &format!("Failed to open image: {}", err));
                            }
                        }
                    }
                    draw_status(
                        &mut grid,
                        current_path.as_deref(),
                        used_threshold,
                        auto_thresh,
                        letterbox,
                        color_mode,
                        &typed,
                    );
                    renderer.render(&grid)?;
                }
                Event::Resize(new_w, new_h) => {
                    // Rebuild grids to match new terminal size and re-render current image (if any)
                    let (w_cells, h_cells) = (new_w as usize, new_h as usize);
                    grid = GridBuffer::new(w_cells, h_cells);
                    braille = BrailleGrid::new(w_cells, h_cells);
                    if let Some(ref img) = current_img {
                        grid.clear();
                        used_threshold = render_with_state(
                            img,
                            &mut braille,
                            &mut grid,
                            manual_threshold,
                            auto_thresh,
                            letterbox,
                            color_mode,
                        );
                    }
                    draw_status(
                        &mut grid,
                        current_path.as_deref(),
                        used_threshold,
                        auto_thresh,
                        letterbox,
                        color_mode,
                        &typed,
                    );
                    renderer.render(&grid)?;
                }

                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    match k.code {
                        KeyCode::Esc => {
                            if !typed.is_empty() {
                                typed.clear();
                                draw_status(
                                    &mut grid,
                                    current_path.as_deref(),
                                    used_threshold,
                                    auto_thresh,
                                    letterbox,
                                    color_mode,
                                    &typed,
                                );
                                renderer.render(&grid)?;
                            } else {
                                break 'outer;
                            }
                        },
                        KeyCode::Enter => {
                            let candidate = typed.trim().trim_matches('"').to_string();
                            if !candidate.is_empty() {
                                match image::open(&candidate) {
                                    Ok(img) => {
                                        typed.clear();

                                        current_img = Some(img);
                                        current_path = Some(candidate.clone());
                                        if let Some(ref img) = current_img {
                                            grid.clear();
                                            used_threshold = render_with_state(
                                                img,
                                                &mut braille,
                                                &mut grid,
                                                manual_threshold,
                                                auto_thresh,
                                                letterbox,
                                                color_mode,
                                            );
                                        }
                                    }
                                    Err(err) => {
                                        grid.clear();
                                        draw_centered(&mut grid, &format!("Failed to open image: {}", err));
                                    }
                                }
                            }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Backspace => {
                            typed.pop();
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') if typed.is_empty() => break 'outer,
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            if typed.is_empty() && current_img.is_some() {
                                if manual_threshold < 250 {
                                    manual_threshold = manual_threshold.saturating_add(5);
                                }
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            if typed.is_empty() && current_img.is_some() {
                                if manual_threshold > 5 {
                                    manual_threshold = manual_threshold.saturating_sub(5);
                                }
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            if typed.is_empty() && current_img.is_some() {
                                auto_thresh = !auto_thresh;
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            if typed.is_empty() && current_img.is_some() {
                                letterbox = !letterbox;
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }

                        KeyCode::Char('x') | KeyCode::Char('X') => {
                            if typed.is_empty() {
                                let before = renderer.dimensions();
                                let _ = renderer.maximize_canvas();
                                let after = renderer.dimensions();
                                let (nw, nh) = after;
                                let (w_cells, h_cells) = (nw as usize, nh as usize);
                                grid = GridBuffer::new(w_cells, h_cells);
                                braille = BrailleGrid::new(w_cells, h_cells);
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                                if after == before {
                                    draw_left(
                                        &mut grid,
                                        2,
                                        "Maximize not supported by this terminal. For a larger canvas: start cmd /k \"mode con: cols=240 lines=120 && cargo run -- --image-drop\"",
                                    );
                                }
                                draw_status(
                                    &mut grid,
                                    current_path.as_deref(),
                                    used_threshold,
                                    auto_thresh,
                                    letterbox,
                                    color_mode,
                                    &typed,
                                );
                                renderer.render(&grid)?;
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if typed.is_empty() && current_img.is_some() {
                                color_mode = match color_mode {
                                    ColorMode::Off => ColorMode::Grayscale,
                                    ColorMode::Grayscale => ColorMode::Full,
                                    ColorMode::Full => ColorMode::Off,
                                };
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        color_mode,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            if typed.is_empty() && current_img.is_some() {
                                if let Some(ref path) = current_path {
                                    // Save Braille art (characters only) from BrailleGrid
                                    let mut out_str = String::new();
                                    for cy in 0..braille.height() {
                                        for cx in 0..braille.width() {
                                            out_str.push(braille.get_char(cx, cy));
                                        }
                                        out_str.push('\n');
                                    }
                                    let p = std::path::Path::new(path);
                                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                                    let parent = p.parent().unwrap_or(std::path::Path::new("."));
                                    let save_path = parent.join(format!("{}.braille.txt", stem));
                                    let msg = match std::fs::write(&save_path, out_str) {
                                        Ok(_) => format!("Saved: {}", save_path.display()),
                                        Err(err) => format!("Save failed: {}", err),
                                    };
                                    draw_left(&mut grid, 2, &msg);
                                    renderer.render(&grid)?;
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                        }
                        KeyCode::Char(c) => {
                            typed.push(c);
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                color_mode,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // Best-effort disable bracketed paste
    let _ = execute!(stdout(), DisableBracketedPaste);

    renderer.cleanup()?;
    Ok(())
}

#[cfg(not(feature = "image"))]
pub fn drop_loop() -> Result<()> {
    anyhow::bail!("Image drag-and-drop requires building with `--features image`")
}

