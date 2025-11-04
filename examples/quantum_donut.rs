
// The Quantum Donut!
// A mesmerizing journey through a tunnel of light and geometry.

use crabmusic::rendering::TerminalRenderer;
use crabmusic::visualization::braille::BrailleGrid;
use crabmusic::visualization::color_schemes::{ColorScheme, ColorSchemeType};
use crabmusic::visualization::Color;
use crossterm::event::{self, Event, KeyCode};
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

const TWO_PI: f32 = 2.0 * std::f32::consts::PI;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌀 The Quantum Donut!");
    println!("Press 'q' to quit, 'c' to cycle colors.");
    println!("Starting in 1 second...");
    thread::sleep(Duration::from_secs(1));

    let mut renderer = TerminalRenderer::new()?;
    let (width, height) = renderer.dimensions();

    let mut grid = BrailleGrid::new(width as usize, height as usize);
    let dot_width = grid.dot_width();
    let dot_height = grid.dot_height();
    let center_x = dot_width as f32 / 2.0;
    let center_y = dot_height as f32 / 2.0;

    let color_schemes = vec![
        ColorSchemeType::Rainbow,
        ColorSchemeType::BluePurple,
        ColorSchemeType::CyanMagenta,
        ColorSchemeType::GreenYellow,
        ColorSchemeType::HeatMap,
    ];
    let mut color_idx = 0;
    let mut color_scheme = ColorScheme::new(color_schemes[color_idx]);

    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut last_fps_update = Instant::now();
    let mut fps = 0.0;



    loop {
        let frame_start = Instant::now();
        let time = start_time.elapsed().as_secs_f32();

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') => {
                        color_idx = (color_idx + 1) % color_schemes.len();
                        color_scheme = ColorScheme::new(color_schemes[color_idx]);
                    }
                    _ => {}
                }
            }
        }

        grid.clear();

        // --- Draw the Quantum Donut (Concentric Circles) ---
        let num_donuts = 4;
        for i in 0..num_donuts {
            let radius_base = (dot_height as f32 * 0.1) + (i as f32 * 12.0);
            let radius = radius_base + (time * 2.0 + i as f32).sin() * 5.0;
            let rotation = time * 0.5 * (if i % 2 == 0 { -1.0 } else { 1.0 });
            let color = color_scheme.get_color(i as f32 / num_donuts as f32);

            // Draw dashed circle by drawing arcs
            let num_segments = 12;
            for j in 0..num_segments {
                if j % 2 == 0 {
                    continue;
                }
                let start_angle = rotation + (j as f32 / num_segments as f32) * TWO_PI;
                let end_angle = rotation + ((j as f32 + 0.8) / num_segments as f32) * TWO_PI;
                
                draw_arc(&mut grid, center_x, center_y, radius, start_angle, end_angle, color.unwrap_or(Color::new(255,255,255)));
            }
        }

        // --- Draw the expanding grid ---
        let grid_speed = 20.0;
        let num_grid_lines = 8;
        let max_grid_radius = (dot_width.min(dot_height) as f32) / 2.0;

        // Concentric circles
        for i in 0..num_grid_lines {
            let radius = (i as f32 * 30.0 + time * grid_speed) % max_grid_radius;
            let color = color_scheme.get_color(radius / max_grid_radius).unwrap_or(Color::new(100,100,100));
            grid.draw_circle(center_x as usize, center_y as usize, radius as usize, color);
        }

        // Radial lines with flowing dots
        let num_radial_lines = 16;
        let num_dots_per_line = 15;
        let radial_speed = 40.0;

        for i in 0..num_radial_lines {
            let angle = (i as f32 / num_radial_lines as f32) * TWO_PI + time * 0.1;
            let color = color_scheme.get_color(i as f32 / num_radial_lines as f32).unwrap_or(Color::new(100,100,100));

            for j in 0..num_dots_per_line {
                let distance = (j as f32 * 20.0 + time * radial_speed) % max_grid_radius;
                let x = center_x + distance * angle.cos();
                let y = center_y + distance * angle.sin();
                if x >= 0.0 && y >= 0.0 && x < dot_width as f32 && y < dot_height as f32 {
                    grid.set_dot_with_color(x as usize, y as usize, color);
                }
            }
        }


        renderer.render_braille(&grid)?;

        frame_count += 1;
        if last_fps_update.elapsed() >= Duration::from_secs(1) {
            fps = frame_count as f32 / last_fps_update.elapsed().as_secs_f32();
            frame_count = 0;
            last_fps_update = Instant::now();
        }

        print!(
            "\r🌀 Quantum Donut | Color: {} | FPS: {:.1} | Press 'c' to cycle, 'q' to quit",
            color_schemes[color_idx].name(),
            fps
        );
        io::stdout().flush()?;

        let frame_time = frame_start.elapsed();
        let target_frame_time = Duration::from_millis(16); // ~60 FPS
        if frame_time < target_frame_time {
            thread::sleep(target_frame_time - frame_time);
        }
    }

    println!("\n\n🌀 Journey complete!");
    Ok(())
}

fn draw_arc(grid: &mut BrailleGrid, cx: f32, cy: f32, radius: f32, start_angle: f32, end_angle: f32, color: Color) {
    let steps = (radius * (end_angle - start_angle).abs() * 2.0).ceil() as usize;
    for i in 0..=steps {
        let angle = start_angle + (end_angle - start_angle) * (i as f32 / steps as f32);
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        grid.set_dot_with_color(x as usize, y as usize, color);
    }
}
