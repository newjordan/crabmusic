// Visualize what the animations look like (without needing a terminal)
// This renders animation frames to stdout

use crabcrust::{Animation, BrailleGrid, Color, SpinnerAnimation, RocketAnimation, SaveAnimation};
use std::time::Duration;

fn render_frame_to_text(grid: &BrailleGrid) -> String {
    let mut output = String::new();
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            output.push(grid.get_char(x, y));
        }
        output.push('\n');
    }
    output
}

fn main() {
    println!("🎮 CrabCrust Animation Visualization\n");
    println!("Showing 5 frames from each animation...\n");

    let width = 60;
    let height = 15;

    // === SPINNER ANIMATION ===
    println!("═══════════════════════════════════════════════════════════════");
    println!("🌀 SPINNER ANIMATION");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut spinner = SpinnerAnimation::new();
    let mut grid = BrailleGrid::new(width, height);

    for frame in 0..5 {
        spinner.update(Duration::from_millis(100));
        grid.clear();
        spinner.render(&mut grid);

        println!("Frame {}:", frame + 1);
        print!("{}", render_frame_to_text(&grid));
        println!();
    }

    // === ROCKET ANIMATION ===
    println!("═══════════════════════════════════════════════════════════════");
    println!("🚀 ROCKET ANIMATION");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut rocket = RocketAnimation::new(Duration::from_secs(2));
    grid.clear();

    for frame in 0..5 {
        rocket.update(Duration::from_millis(400));
        grid.clear();
        rocket.render(&mut grid);

        println!("Frame {}:", frame + 1);
        print!("{}", render_frame_to_text(&grid));
        println!();
    }

    // === SAVE ANIMATION ===
    println!("═══════════════════════════════════════════════════════════════");
    println!("💾 SAVE ANIMATION");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut save = SaveAnimation::default();
    grid.clear();

    for frame in 0..5 {
        save.update(Duration::from_millis(300));
        grid.clear();
        save.render(&mut grid);

        println!("Frame {}:", frame + 1);
        print!("{}", render_frame_to_text(&grid));
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("✨ Visualization Complete!");
    println!("\n💡 In a real terminal, these would be:");
    println!("   • 60 FPS smooth animation");
    println!("   • Full RGB colors");
    println!("   • Fluid motion");
    println!("\n🚀 Try it: cargo run -- demo all");
}
