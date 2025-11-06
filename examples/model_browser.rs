//! Interactive glTF Model Browser
//!
//! Browse, download, and view 3D models from the Khronos glTF Sample Assets.
//!
//! Usage:
//!   cargo run --example model_browser                    # Interactive mode
//!   cargo run --example model_browser list               # List all models
//!   cargo run --example model_browser view <model_id>    # View a specific model
//!   cargo run --example model_browser download <model_id> # Download without viewing

use crabmusic::visualization::ray_tracer::*;
use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Interactive mode
        run_interactive_browser();
    } else {
        match args[1].as_str() {
            "list" => list_models(),
            "view" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} view <model_id>", args[0]);
                    std::process::exit(1);
                }
                view_model(&args[2]);
            }
            "download" => {
                if args.len() < 3 {
                    eprintln!("Usage: {} download <model_id>", args[0]);
                    std::process::exit(1);
                }
                download_model_only(&args[2]);
            }
            "cache" => show_cache_info(),
            "clear-cache" => clear_cache(),
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!();
                print_usage(&args[0]);
                std::process::exit(1);
            }
        }
    }
}

fn print_usage(program: &str) {
    println!("glTF Model Browser - Browse and view 3D models");
    println!();
    println!("Usage:");
    println!("  {}                    # Interactive mode", program);
    println!("  {} list               # List all available models", program);
    println!("  {} view <model_id>    # Download and view a model", program);
    println!("  {} download <model_id> # Download model without viewing", program);
    println!("  {} cache              # Show cache information", program);
    println!("  {} clear-cache        # Clear downloaded models", program);
    println!();
    println!("Examples:");
    println!("  {} view cube", program);
    println!("  {} view suzanne", program);
    println!("  {} list", program);
}

fn run_interactive_browser() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║         🦀 CrabMusic glTF Model Browser 🦀               ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    loop {
        println!("What would you like to do?");
        println!("  1. List all models");
        println!("  2. View a model");
        println!("  3. Show cache info");
        println!("  4. Clear cache");
        println!("  5. Exit");
        println!();
        print!("Enter choice (1-5): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        println!();

        match choice {
            "1" => {
                list_models();
                println!();
            }
            "2" => {
                print!("Enter model ID: ");
                io::stdout().flush().unwrap();
                let mut model_id = String::new();
                io::stdin().read_line(&mut model_id).unwrap();
                view_model(model_id.trim());
                println!();
            }
            "3" => {
                show_cache_info();
                println!();
            }
            "4" => {
                clear_cache();
                println!();
            }
            "5" => {
                println!("Goodbye! 🦀");
                break;
            }
            _ => {
                println!("Invalid choice. Please enter 1-5.");
                println!();
            }
        }
    }
}

fn list_models() {
    use crabmusic::visualization::ray_tracer::model_catalog::*;

    let catalog = get_catalog();

    println!("Available Models ({} total):", catalog.len());
    println!();

    // Group by complexity
    for complexity in [Complexity::Simple, Complexity::Medium, Complexity::Complex] {
        let models: Vec<_> = catalog
            .iter()
            .filter(|m| m.complexity == complexity)
            .collect();

        if !models.is_empty() {
            println!("┌─ {} Models ─────────────────────────────────", complexity);
            for model in models {
                let cached = if crabmusic::visualization::ray_tracer::model_downloader::is_model_cached(
                    &format!("{}.gltf", model.id),
                ) {
                    "✓"
                } else {
                    " "
                };
                println!(
                    "│ [{}] {:15} - {} ({} KB)",
                    cached, model.id, model.name, model.size_kb
                );
                println!("│     {}", model.description);
            }
            println!("└────────────────────────────────────────────────────");
            println!();
        }
    }

    println!("Legend: [✓] = cached locally, [ ] = needs download");
}

fn view_model(model_id: &str) {
    use crabmusic::visualization::ray_tracer::model_catalog::*;
    use crabmusic::visualization::ray_tracer::model_downloader::*;

    // Find the model in catalog
    let model_info = match find_model(model_id) {
        Some(info) => info,
        None => {
            eprintln!("✗ Model '{}' not found in catalog", model_id);
            eprintln!();
            eprintln!("Run 'cargo run --example model_browser list' to see available models");
            return;
        }
    };

    println!("═══════════════════════════════════════════════════════════");
    println!("Model: {}", model_info.name);
    println!("Description: {}", model_info.description);
    println!("Complexity: {}", model_info.complexity);
    println!("Size: {} KB", model_info.size_kb);
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Download the model
    let filename = format!("{}.gltf", model_info.id);
    let model_path = match download_model_with_progress(model_info.url, &filename, model_info.name) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("✗ Failed to download model: {}", e);
            return;
        }
    };

    println!();
    println!("Loading and rendering model...");

    // Load and render the model
    let scene = match Scene::new_with_model(model_path.to_str().unwrap()) {
        Ok(scene) => scene,
        Err(e) => {
            eprintln!("✗ Failed to load model: {}", e);
            return;
        }
    };

    // Set up camera
    let camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0), // Camera position
        4.0,                          // Viewport width
        3.0,                          // Viewport height
    );

    // Render in solid mode
    let (w, h) = (160_usize, 96_usize);
    let buffer = render(&scene, &camera, w, h, RenderMode::Solid);

    // Convert to Braille and display
    let text = intensity_buffer_to_green_braille(&buffer);
    println!();
    print!("{}", text);
    println!();
    println!("✓ Render complete!");
}

fn download_model_only(model_id: &str) {
    use crabmusic::visualization::ray_tracer::model_catalog::*;
    use crabmusic::visualization::ray_tracer::model_downloader::*;

    let model_info = match find_model(model_id) {
        Some(info) => info,
        None => {
            eprintln!("✗ Model '{}' not found in catalog", model_id);
            return;
        }
    };

    let filename = format!("{}.gltf", model_info.id);
    match download_model_with_progress(model_info.url, &filename, model_info.name) {
        Ok(path) => {
            println!("✓ Model ready at: {}", path.display());
        }
        Err(e) => {
            eprintln!("✗ Failed to download: {}", e);
        }
    }
}

fn show_cache_info() {
    use crabmusic::visualization::ray_tracer::model_downloader::*;

    println!("Cache Information:");
    println!("─────────────────────────────────────────────────────────");

    match list_cached_models() {
        Ok(models) => {
            if models.is_empty() {
                println!("No models cached yet.");
            } else {
                println!("Cached models ({}):", models.len());
                for model in &models {
                    println!("  • {}", model.display());
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to list cache: {}", e);
        }
    }

    match get_cache_size() {
        Ok(size) => {
            let size_mb = size as f64 / 1024.0 / 1024.0;
            println!();
            println!("Total cache size: {:.2} MB", size_mb);
        }
        Err(e) => {
            eprintln!("✗ Failed to get cache size: {}", e);
        }
    }
}

fn clear_cache() {
    use crabmusic::visualization::ray_tracer::model_downloader;

    print!("Are you sure you want to clear the cache? (y/N): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().eq_ignore_ascii_case("y") {
        match model_downloader::clear_cache() {
            Ok(_) => println!("✓ Cache cleared successfully"),
            Err(e) => eprintln!("✗ Failed to clear cache: {}", e),
        }
    } else {
        println!("Cache clear cancelled");
    }
}

