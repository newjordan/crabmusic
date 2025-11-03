// Night Night Animation - A soothing bedtime visualization
// Features: Twinkling stars, moon phases, sunset-to-night color transitions

use crabmusic::visualization::{Color, GridBuffer};
use crabmusic::rendering::TerminalRenderer;
use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};
use rand::Rng;

struct Star {
    x: usize,
    y: usize,
    brightness: f32,
    twinkle_speed: f32,
    phase: f32,
}

impl Star {
    fn new(x: usize, y: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x,
            y,
            brightness: rng.gen_range(0.3..1.0),
            twinkle_speed: rng.gen_range(0.02..0.08),
            phase: rng.gen_range(0.0..std::f32::consts::TAU),
        }
    }

    fn update(&mut self) {
        self.phase += self.twinkle_speed;
        if self.phase > std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }
    }

    fn get_brightness(&self) -> f32 {
        let twinkle = (self.phase.sin() * 0.5 + 0.5).powi(2);
        self.brightness * twinkle
    }

    fn get_char(&self) -> char {
        let b = self.get_brightness();
        if b > 0.8 { '✦' }
        else if b > 0.6 { '*' }
        else if b > 0.4 { '·' }
        else { '.' }
    }

    fn get_color(&self) -> Color {
        let b = self.get_brightness();
        let base = (200.0 + b * 55.0) as u8;
        Color::new(base, base, base)
    }
}

struct Moon {
    x: usize,
    y: usize,
    phase: f32, // 0.0 to 1.0 - controls visibility/glow
}

impl Moon {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y, phase: 0.0 }
    }

    fn update(&mut self) {
        self.phase = (self.phase + 0.003).min(1.0);
    }

    fn draw(&self, grid: &mut GridBuffer) {
        if self.x < 3 || self.y < 2 || self.x + 3 >= grid.width() || self.y + 2 >= grid.height() {
            return;
        }

        let glow = (self.phase * 255.0) as u8;
        let moon_color = Color::new(glow, glow, (glow as f32 * 0.9) as u8);

        // Moon shape (crescent -> full)
        let shapes = [
            // Crescent
            vec![
                "  ▄▀ ",
                " █   ",
                "  ▀▄ ",
            ],
            // Half
            vec![
                " ▄▀▀ ",
                "█▀▀  ",
                " ▀▀▄ ",
            ],
            // Full
            vec![
                " ▄▀▀▄ ",
                "█▀▀▀█",
                " ▀▀▀▄ ",
            ],
        ];

        let shape_idx = ((self.phase * 2.0).min(2.0) as usize).min(2);
        let shape = &shapes[shape_idx];

        for (dy, line) in shape.iter().enumerate() {
            for (dx, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let x = self.x + dx;
                    let y = self.y + dy;
                    if x < grid.width() && y < grid.height() {
                        grid.set_cell_with_color(x, y, ch, moon_color);
                    }
                }
            }
        }

        // Subtle glow around moon
        if self.phase > 0.5 {
            let glow_strength = ((self.phase - 0.5) * 2.0 * 100.0) as u8;
            let glow_color = Color::new(
                glow_strength,
                glow_strength,
                (glow_strength as f32 * 0.8) as u8,
            );

            // Add dim glow pixels around moon
            for dy in -1..=4 {
                for dx in -1..=6 {
                    let x = (self.x as i32 + dx) as usize;
                    let y = (self.y as i32 + dy) as usize;

                    if x < grid.width() && y < grid.height()
                        && grid.get_cell(x, y).character == ' '
                        && rand::random::<f32>() < 0.15 {
                        grid.set_cell_with_color(x, y, '·', glow_color);
                    }
                }
            }
        }
    }
}

struct SkyGradient {
    progress: f32, // 0.0 = sunset, 1.0 = night
}

impl SkyGradient {
    fn new() -> Self {
        Self { progress: 0.0 }
    }

    fn update(&mut self) {
        self.progress = (self.progress + 0.001).min(1.0);
    }

    fn get_color_for_row(&self, row: usize, total_rows: usize) -> Color {
        let row_factor = row as f32 / total_rows as f32;

        // Sunset colors (orange/pink to purple)
        let sunset_top = Color::new(20, 10, 40);      // Deep purple-blue
        let sunset_mid = Color::new(180, 60, 100);    // Purple-pink
        let sunset_bottom = Color::new(255, 140, 60); // Orange

        // Night colors (dark blue to black)
        let night_top = Color::new(5, 5, 15);         // Almost black
        let night_mid = Color::new(15, 15, 40);       // Deep blue
        let night_bottom = Color::new(10, 20, 50);    // Dark blue

        // Interpolate between sunset and night based on progress
        let top = Self::lerp_color(sunset_top, night_top, self.progress);
        let mid = Self::lerp_color(sunset_mid, night_mid, self.progress);
        let bottom = Self::lerp_color(sunset_bottom, night_bottom, self.progress);

        // Interpolate between top/mid/bottom based on row
        if row_factor < 0.4 {
            let t = row_factor / 0.4;
            Self::lerp_color(top, mid, t)
        } else {
            let t = (row_factor - 0.4) / 0.6;
            Self::lerp_color(mid, bottom, t)
        }
    }

    fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
        Color::new(
            (c1.r as f32 + (c2.r as f32 - c1.r as f32) * t) as u8,
            (c1.g as f32 + (c2.g as f32 - c1.g as f32) * t) as u8,
            (c1.b as f32 + (c2.b as f32 - c1.b as f32) * t) as u8,
        )
    }
}

struct NightNightAnimation {
    stars: Vec<Star>,
    moon: Moon,
    sky: SkyGradient,
    message_phase: f32,
    show_message: bool,
}

impl NightNightAnimation {
    fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::thread_rng();
        let num_stars = (width * height / 20).max(30);

        let mut stars = Vec::new();
        for _ in 0..num_stars {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height.saturating_sub(8)); // Keep top area for stars
            stars.push(Star::new(x, y));
        }

        let moon = Moon::new(width - 12, 3);

        Self {
            stars,
            moon,
            sky: SkyGradient::new(),
            message_phase: 0.0,
            show_message: false,
        }
    }

    fn update(&mut self) {
        self.sky.update();

        for star in &mut self.stars {
            star.update();
        }

        self.moon.update();

        // Show message after sky is mostly dark
        if self.sky.progress > 0.6 && !self.show_message {
            self.show_message = true;
        }

        if self.show_message {
            self.message_phase = (self.message_phase + 0.02).min(1.0);
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        let width = grid.width();
        let height = grid.height();

        // Draw gradient sky
        for y in 0..height {
            let color = self.sky.get_color_for_row(y, height);
            for x in 0..width {
                grid.set_cell_with_color(x, y, ' ', color);
            }
        }

        // Draw stars (only visible when sky is dark enough)
        let star_visibility = (self.sky.progress - 0.3).max(0.0) / 0.7;
        if star_visibility > 0.0 {
            for star in &self.stars {
                if star.x < width && star.y < height {
                    let brightness = star.get_brightness() * star_visibility;
                    if brightness > 0.2 {
                        let mut color = star.get_color();
                        // Dim the star based on overall visibility
                        color.r = (color.r as f32 * star_visibility) as u8;
                        color.g = (color.g as f32 * star_visibility) as u8;
                        color.b = (color.b as f32 * star_visibility) as u8;
                        grid.set_cell_with_color(star.x, star.y, star.get_char(), color);
                    }
                }
            }
        }

        // Draw moon
        self.moon.draw(grid);

        // Draw "night night" message
        if self.show_message {
            let messages = [
                "✨  n i g h t   n i g h t  ✨",
                "      s w e e t   d r e a m s      ",
            ];

            let alpha = self.message_phase;
            let text_brightness = (alpha * 200.0 + 55.0) as u8;
            let text_color = Color::new(
                text_brightness,
                (text_brightness as f32 * 0.9) as u8,
                text_brightness,
            );

            for (i, msg) in messages.iter().enumerate() {
                let y = height / 2 + i * 2;
                if y < height {
                    let x_start = (width.saturating_sub(msg.len())) / 2;
                    for (dx, ch) in msg.chars().enumerate() {
                        let x = x_start + dx;
                        if x < width {
                            // Add a subtle pulse effect
                            let pulse = (self.message_phase * 6.0 + dx as f32 * 0.1).sin() * 0.1 + 0.9;
                            let pulse_color = Color::new(
                                (text_color.r as f32 * pulse) as u8,
                                (text_color.g as f32 * pulse) as u8,
                                (text_color.b as f32 * pulse) as u8,
                            );
                            grid.set_cell_with_color(x, y, ch, pulse_color);
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut renderer = TerminalRenderer::new()?;
    let (width, height) = renderer.dimensions();
    let mut grid = GridBuffer::new(width as usize, height as usize);

    // Create animation
    let mut animation = NightNightAnimation::new(width as usize, height as usize);

    // Main loop
    let frame_duration = Duration::from_millis(33); // ~30 FPS for smooth animation
    let mut last_frame = Instant::now();

    println!("Starting night night animation... Press 'q' or ESC to exit.");
    std::thread::sleep(Duration::from_millis(500));

    loop {
        // Check for key press (non-blocking)
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        // Update animation
        animation.update();

        // Render
        grid.clear();
        animation.render(&mut grid);
        renderer.render(&grid)?;

        // Frame timing
        let elapsed = last_frame.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        last_frame = Instant::now();
    }

    // Cleanup
    renderer.cleanup()?;
    println!("\nSweet dreams! 😴");

    Ok(())
}
