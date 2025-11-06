// CrabCrust: Add arcade-style animations to your CLI tools 🦀✨

use anyhow::Result;
use clap::{Parser, Subcommand};
use crabcrust::wrapper::git::GitWrapper;
use crabcrust::{AnimationPlayer, RocketAnimation, SaveAnimation, SpinnerAnimation};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "crabcrust")]
#[command(about = "Add arcade-style animations to your CLI tools 🦀✨", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Git wrapper with animations
    Git {
        /// Git arguments (e.g., commit -m "message")
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Test animations
    Demo {
        /// Which animation to demo: spinner, rocket, save, all
        #[arg(default_value = "all")]
        animation: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Git { args } => {
            let mut wrapper = GitWrapper::new()?;
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let result = wrapper.run(&args_refs)?;

            // Exit with the same code as git
            std::process::exit(result.exit_code);
        }

        Commands::Demo { animation } => {
            let mut player = AnimationPlayer::new()?;

            match animation.as_str() {
                "spinner" => {
                    println!("🌀 Spinner Animation Demo");
                    player.play_for(SpinnerAnimation::new(), Duration::from_secs(3))?;
                }
                "rocket" => {
                    println!("🚀 Rocket Animation Demo");
                    player.play(RocketAnimation::new(Duration::from_secs(2)))?;
                }
                "save" => {
                    println!("💾 Save Animation Demo");
                    player.play(SaveAnimation::default())?;
                }
                "all" | _ => {
                    println!("🎮 Running all animations...\n");

                    println!("1. Spinner Animation");
                    player.play_for(SpinnerAnimation::new(), Duration::from_secs(2))?;
                    std::thread::sleep(Duration::from_millis(500));

                    println!("\n2. Save Animation");
                    player.play(SaveAnimation::default())?;
                    std::thread::sleep(Duration::from_millis(500));

                    println!("\n3. Rocket Animation");
                    player.play(RocketAnimation::new(Duration::from_secs(2)))?;

                    println!("\n✨ Demo complete!");
                }
            }
        }
    }

    Ok(())
}
