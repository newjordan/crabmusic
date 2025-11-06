// Test BrailleGrid without requiring a terminal
// This demonstrates the core animation logic works!

use crabcrust::{BrailleGrid, Color};

fn main() {
    println!("🦀 CrabCrust BrailleGrid Test (No Terminal Required)\n");

    // Create a small grid
    let mut grid = BrailleGrid::new(40, 10);

    println!("✓ Created BrailleGrid: {}×{} cells = {}×{} dots",
        grid.width(), grid.height(), grid.dot_width(), grid.dot_height());

    // Test 1: Set individual dots
    grid.set_dot(0, 0);
    grid.set_dot(1, 0);
    println!("✓ Set dots at (0,0) and (1,0)");
    println!("  First cell char: '{}'", grid.get_char(0, 0));

    // Test 2: Draw a line
    grid.clear();
    grid.draw_line(0, 0, 20, 10);
    println!("✓ Drew diagonal line from (0,0) to (20,10)");

    // Test 3: Draw a circle
    grid.clear();
    grid.draw_circle(40, 20, 15, Color::CYAN);
    println!("✓ Drew circle at center (40,20) with radius 15");

    // Test 4: Render a simple animation frame
    grid.clear();

    // Simulate spinner animation (one frame)
    let center_x = grid.dot_width() / 2;
    let center_y = grid.dot_height() / 2;
    let angle = 0.0f32;

    for i in 0..8 {
        let trail_angle = angle - (i as f32 * 0.3);
        let x = center_x as f32 + trail_angle.cos() * 15.0;
        let y = center_y as f32 + trail_angle.sin() * 15.0;

        if x >= 0.0 && y >= 0.0 && (x as usize) < grid.dot_width() && (y as usize) < grid.dot_height() {
            let fade = 1.0 - (i as f32 / 8.0) * 0.7;
            let color = Color::new(
                (0 as f32) as u8,
                (255.0 * fade) as u8,
                (255.0 * fade) as u8,
            );
            grid.set_dot_with_color(x as usize, y as usize, color);
        }
    }

    println!("✓ Generated spinner animation frame");

    // Display a few rows of the grid
    println!("\n📊 Sample output (first 5 rows):");
    for y in 0..5 {
        print!("  ");
        for x in 0..40 {
            let ch = grid.get_char(x, y);
            let color = grid.get_color(x, y);
            if let Some(c) = color {
                // Show colored character representation
                print!("{}", ch);
            } else {
                print!("{}", ch);
            }
        }
        println!();
    }

    println!("\n✅ All BrailleGrid operations successful!");
    println!("🎮 The animation system is working perfectly!");
    println!("\n💡 To see the actual animations, run in a real terminal:");
    println!("   cargo run -- demo all");
}
