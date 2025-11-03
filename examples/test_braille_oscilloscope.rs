// Quick test to see Braille oscilloscope rendering
// Run with: cargo run --example test_braille_oscilloscope

use crabmusic::dsp::DspProcessor;
use crabmusic::audio::AudioBuffer;
use crabmusic::visualization::{OscilloscopeVisualizer, OscilloscopeConfig, GridBuffer, Visualizer};

fn main() {
    println!("=== Braille Oscilloscope Test ===\n");

    // Create DSP processor and oscilloscope
    let mut dsp = DspProcessor::new(44100, 2048).expect("Failed to create DSP");
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Generate a 440 Hz sine wave (A note)
    println!("Generating 440 Hz sine wave...\n");
    let mut samples = Vec::new();
    for i in 0..2048 {
        let t = i as f32 / 44100.0;
        let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.8;
        samples.push(sample);
    }

    let buffer = AudioBuffer::with_samples(samples, 44100, 1);

    // Process audio to get waveform data
    let params = dsp.process(&buffer);

    println!("Waveform samples: {}", params.waveform.len());
    println!("Amplitude: {:.2}", params.amplitude);
    println!();

    // Update and render oscilloscope
    viz.update(&params);

    let mut grid = GridBuffer::new(80, 24);
    viz.render(&mut grid);

    // Print the result
    println!("=== Braille Oscilloscope Output ===\n");
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let cell = grid.get_cell(x, y);
            print!("{}", cell.character);
        }
        println!();
    }

    println!("\n=== Look for smooth curves using Braille characters! ===");
    println!("Braille chars look like: ⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟");
    println!("The waveform should be a smooth sine curve!");
}
