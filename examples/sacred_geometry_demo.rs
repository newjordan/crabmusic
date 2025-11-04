// Sacred Geometry Demo
// Showcases both Flower of Life and Mandala visualizers

use crabmusic::dsp::AudioParameters;
use crabmusic::rendering::TerminalRenderer;
use crabmusic::visualization::{
    color_schemes::{ColorScheme, ColorSchemeType},
    FlowerOfLifeConfig, FlowerOfLifeVisualizer, GridBuffer, MandalaConfig, MandalaVisualizer,
    Visualizer,
};
use crossterm::event::{self, Event, KeyCode};
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

enum VisualizerMode {
    FlowerOfLife,
    Mandala,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("✨ Sacred Geometry Demo");
    println!("Press 'q' to quit, 'v' to switch visualizer, 'c' to cycle colors");
    println!("Starting in 1 second...\n");
    thread::sleep(Duration::from_secs(1));

    let mut renderer = TerminalRenderer::new()?;
    let (width_u16, height_u16) = renderer.dimensions();
    let width = width_u16 as usize;
    let height = height_u16 as usize;

    let mut grid = GridBuffer::new(width, height);

    // Create both visualizers
    let flower_config = FlowerOfLifeConfig {
        num_rings: 2,
        base_radius: 15.0,
        rotation_speed: 1.0,
        pulse_intensity: 0.7,
        use_color: true,
    };
    let mut flower_viz = FlowerOfLifeVisualizer::new(flower_config);

    let mandala_config = MandalaConfig {
        symmetry: 8,
        num_layers: 3,
        base_radius: 15.0,
        rotation_speed: 1.0,
        pulse_intensity: 0.7,
        use_color: true,
    };
    let mut mandala_viz = MandalaVisualizer::new(mandala_config);

    // Color schemes to cycle through
    let color_schemes = vec![
        ColorSchemeType::Rainbow,
        ColorSchemeType::HeatMap,
        ColorSchemeType::BluePurple,
        ColorSchemeType::CyanMagenta,
        ColorSchemeType::GreenYellow,
    ];
    let mut color_idx = 0;

    let mut mode = VisualizerMode::FlowerOfLife;

    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut last_fps_update = Instant::now();
    let mut fps = 0.0;

    loop {
        let frame_start = Instant::now();

        // Check for keyboard input (non-blocking)
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('v') => {
                        mode = match mode {
                            VisualizerMode::FlowerOfLife => VisualizerMode::Mandala,
                            VisualizerMode::Mandala => VisualizerMode::FlowerOfLife,
                        };
                    }
                    KeyCode::Char('c') => {
                        color_idx = (color_idx + 1) % color_schemes.len();
                        let scheme = ColorScheme::new(color_schemes[color_idx]);
                        flower_viz.set_color_scheme(scheme.clone());
                        mandala_viz.set_color_scheme(scheme);
                    }
                    _ => {}
                }
            }
        }

        // Simulate audio parameters with smooth animation
        let time = start_time.elapsed().as_secs_f32();
        let params = AudioParameters {
            amplitude: 0.7 + 0.3 * (time * 0.5).sin(),
            bass: 0.5 + 0.5 * (time * 0.8).sin(),
            mid: 0.5 + 0.5 * (time * 1.2).cos(),
            treble: 0.5 + 0.5 * (time * 1.5).sin(),
            beat: (time * 2.0).sin() > 0.95, // Beat every ~0.5 seconds
            beat_flux: false,
            bpm: 120.0,
            tempo_confidence: 0.8,
            spectrum: vec![],
            waveform: vec![],
        };

        // Update and render based on mode
        match mode {
            VisualizerMode::FlowerOfLife => {
                flower_viz.update(&params);
                flower_viz.render(&mut grid);
            }
            VisualizerMode::Mandala => {
                mandala_viz.update(&params);
                mandala_viz.render(&mut grid);
            }
        }

        // Render to terminal
        renderer.render(&grid)?;

        // Calculate FPS
        frame_count += 1;
        if last_fps_update.elapsed() >= Duration::from_secs(1) {
            fps = frame_count as f32 / last_fps_update.elapsed().as_secs_f32();
            frame_count = 0;
            last_fps_update = Instant::now();
        }

        // Print status
        let viz_name = match mode {
            VisualizerMode::FlowerOfLife => "🌸 Flower of Life",
            VisualizerMode::Mandala => "🕉️  Mandala",
        };
        print!(
            "\r{} | Color: {} | FPS: {:.1} | Press 'v' to switch, 'c' for colors, 'q' to quit",
            viz_name,
            color_schemes[color_idx].name(),
            fps
        );
        io::stdout().flush()?;

        // Target 60 FPS
        let frame_time = frame_start.elapsed();
        let target_frame_time = Duration::from_millis(16); // ~60 FPS
        if frame_time < target_frame_time {
            thread::sleep(target_frame_time - frame_time);
        }
    }

    println!("\n\n✨ Demo complete!");
    Ok(())
}

