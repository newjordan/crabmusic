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
    grayscale: bool,
) {
    let w_cells = braille.width();
    let h_cells = braille.height();

    if grayscale {
        for cy in 0..h_cells {
            for cx in 0..w_cells {
                let char = braille.get_char(cx, cy);
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
                grid.set_cell_with_color(cx, cy, char, color);
            }
        }
    } else {
        for cy in 0..h_cells {
            for cx in 0..w_cells {
                grid.set_cell(cx, cy, braille.get_char(cx, cy));
            }
        }
    }

    // Ensure overlay rows can draw over content; caller will write prompt/inputs after this
}

#[cfg(feature = "image")]
fn render_with_state(
    img: &image::DynamicImage,
    braille: &mut BrailleGrid,
    grid: &mut GridBuffer,
    threshold: u8,
    auto_threshold: bool,
    letterbox: bool,
    grayscale: bool,
) -> u8 {
    let dot_w = braille.dot_width() as u32;
    let dot_h = braille.dot_height() as u32;
    let luma = prepare_luma_dot_sized(img, dot_w, dot_h, letterbox);
    let used_threshold = if auto_threshold { otsu_threshold(luma.as_raw()) } else { threshold };
    fill_braille_from_luma(braille, &luma, used_threshold);
    copy_braille_to_grid(grid, braille, &luma, grayscale);
    used_threshold
}

#[cfg(feature = "image")]
fn draw_status(
    grid: &mut GridBuffer,
    path_opt: Option<&str>,
    used_threshold: u8,
    auto_thresh: bool,
    letterbox: bool,
    grayscale: bool,
    typed: &str,
) {
    let name = path_opt
        .and_then(|p| std::path::Path::new(p).file_name().and_then(|s| s.to_str()))
        .unwrap_or("<none>");
    let status = format!(
        "File: {} | +/- thr={} | a auto={} | l letterbox={} | c color={} | s save | q quit",
        name,
        used_threshold,
        if auto_thresh { "ON" } else { "OFF" },
        if letterbox { "ON" } else { "OFF" },
        if grayscale { "ON" } else { "OFF" },
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
pub fn render_image(path: &str) -> Result<()> {
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
    let mut grayscale = false;

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
            grayscale,
        );
    }
    draw_status(
        &mut grid,
        current_path.as_deref(),
        used_threshold,
        auto_thresh,
        letterbox,
        grayscale,
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
                            Ok(new_img) => {
                                current_img = Some(new_img);
                                current_path = Some(candidate);
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        grayscale,
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
                        grayscale,
                        &typed,
                    );
                    renderer.render(&grid)?;
                }
                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    match k.code {
                        KeyCode::Esc => break 'outer,
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
                                        grayscale,
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
                                                grayscale,
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
                                grayscale,
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
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if typed.is_empty() && current_img.is_some() {
                                grayscale = !grayscale;
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
                                &typed,
                            );
                            renderer.render(&grid)?;
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
                                grayscale,
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
    let mut grayscale = false;

    // Initial prompt
    grid.clear();
    draw_status(
        &mut grid,
        current_path.as_deref(),
        used_threshold,
        auto_thresh,
        letterbox,
        grayscale,
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
                                        grayscale,
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
                        grayscale,
                        &typed,
                    );
                    renderer.render(&grid)?;
                }
                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    match k.code {
                        KeyCode::Esc => break 'outer,
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
                                                grayscale,
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
                                grayscale,
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
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
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
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
                                &typed,
                            );
                            renderer.render(&grid)?;
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if typed.is_empty() && current_img.is_some() {
                                grayscale = !grayscale;
                                if let Some(ref img) = current_img {
                                    grid.clear();
                                    used_threshold = render_with_state(
                                        img,
                                        &mut braille,
                                        &mut grid,
                                        manual_threshold,
                                        auto_thresh,
                                        letterbox,
                                        grayscale,
                                    );
                                }
                            } else if let KeyCode::Char(c) = k.code { typed.push(c); }
                            draw_status(
                                &mut grid,
                                current_path.as_deref(),
                                used_threshold,
                                auto_thresh,
                                letterbox,
                                grayscale,
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
                                grayscale,
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

