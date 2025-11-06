// Anti-Aliasing Demo
//
// Demonstrates the new anti-aliased Braille rendering capabilities
// Shows side-by-side comparison of binary vs anti-aliased rendering

use crabmusic::visualization::{BrailleGrid, Color, GridBuffer};
use std::io::{self, Write};

fn main() {
    println!("🦀 CrabMusic - Anti-Aliasing Demo\n");
    println!("Comparing Binary vs Anti-Aliased Braille Rendering\n");

    // Create two grids for comparison
    let width = 40;
    let height = 20;

    let mut binary_grid = BrailleGrid::new(width, height);
    let mut aa_grid = BrailleGrid::new(width, height);
    aa_grid.set_antialiasing(true);

    let white = Color::new(255, 255, 255);
    let cyan = Color::new(0, 255, 255);
    let magenta = Color::new(255, 0, 255);
    let yellow = Color::new(255, 255, 0);

    // Draw circles
    println!("Drawing circles...");
    let center_x = (width * 2 / 2) as f32;
    let center_y = (height * 4 / 2) as f32;

    // Binary circle (left side)
    draw_circle_binary(&mut binary_grid, center_x - 20.0, center_y, 15.0, cyan);

    // AA circle (right side)
    aa_grid.draw_circle_aa(center_x + 20.0, center_y, 15.0, cyan);

    // Draw diagonal lines
    println!("Drawing diagonal lines...");

    // Binary line
    binary_grid.draw_line_with_color(5, 5, 30, 70, magenta);

    // AA line
    aa_grid.draw_line_aa_with_color(5.0, 5.0, 30.0, 70.0, magenta);

    // Draw sine wave
    println!("Drawing sine waves...");
    for i in 0..80 {
        let x = i as f32;
        let y = 40.0 + 15.0 * ((x / 10.0) * std::f32::consts::PI).sin();

        if i > 0 {
            let prev_x = (i - 1) as f32;
            let prev_y = 40.0 + 15.0 * ((prev_x / 10.0) * std::f32::consts::PI).sin();

            // Binary
            binary_grid.draw_line_with_color(
                prev_x as usize,
                prev_y as usize,
                x as usize,
                y as usize,
                yellow,
            );

            // AA
            aa_grid.draw_line_aa_with_color(prev_x, prev_y, x, y, yellow);
        }
    }

    // Render to terminal
    println!("\n╔════════════════════════════════════════╗");
    println!("║         BINARY (Original)              ║");
    println!("╚════════════════════════════════════════╝\n");

    render_braille_grid(&binary_grid);

    println!("\n\n╔════════════════════════════════════════╗");
    println!("║      ANTI-ALIASED (Enhanced)           ║");
    println!("╚════════════════════════════════════════╝\n");

    render_braille_grid(&aa_grid);

    println!("\n\n✨ Key Improvements:");
    println!("  • Smoother circles (no jagged edges)");
    println!("  • Sub-pixel accurate lines");
    println!("  • Better curve rendering");
    println!("  • Perfect for sacred geometry!\n");
}

/// Draw a circle using binary line segments (for comparison)
fn draw_circle_binary(grid: &mut BrailleGrid, cx: f32, cy: f32, radius: f32, color: Color) {
    let num_points = (radius * 2.0 * std::f32::consts::PI).ceil() as usize;
    let num_points = num_points.max(8);

    let mut prev_x = cx + radius;
    let mut prev_y = cy;

    for i in 1..=num_points {
        let angle = (i as f32 / num_points as f32) * 2.0 * std::f32::consts::PI;
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();

        grid.draw_line_with_color(
            prev_x.round() as usize,
            prev_y.round() as usize,
            x.round() as usize,
            y.round() as usize,
            color,
        );

        prev_x = x;
        prev_y = y;
    }
}

/// Render a BrailleGrid to the terminal
fn render_braille_grid(grid: &BrailleGrid) {
    let mut stdout = io::stdout();

    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let ch = grid.get_char(x, y);

            if let Some(color) = grid.get_color(x, y) {
                // ANSI color escape
                write!(
                    stdout,
                    "\x1b[38;2;{};{};{}m{}\x1b[0m",
                    color.r, color.g, color.b, ch
                )
                .unwrap();
            } else {
                write!(stdout, "{}", ch).unwrap();
            }
        }
        writeln!(stdout).unwrap();
    }

    stdout.flush().unwrap();
}
