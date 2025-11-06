// Example: Using the Git wrapper with animations

use crabcrust::wrapper::git::GitWrapper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Git Wrapper Example");
    println!("Running git status with CrabCrust...\n");

    let mut git = GitWrapper::new()?;

    // Run git status
    let result = git.status()?;

    println!("\n📋 Git Status Output:");
    println!("{}", result.stdout);

    if result.success {
        println!("✅ Command succeeded with exit code {}", result.exit_code);
    } else {
        println!("❌ Command failed with exit code {}", result.exit_code);
        if !result.stderr.is_empty() {
            println!("Error: {}", result.stderr);
        }
    }

    Ok(())
}
