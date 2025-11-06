// CrabCrust: Add arcade-style animations to your CLI tools 🦀✨

pub mod braille;
pub mod rendering;
pub mod animation;
pub mod executor;
pub mod wrapper;

// Re-export commonly used types
pub use braille::{BrailleGrid, Color};
pub use rendering::TerminalRenderer;
pub use animation::{Animation, AnimationPlayer, SpinnerAnimation, RocketAnimation, SaveAnimation};
pub use executor::{CommandExecutor, CommandResult};

/// CrabCrust result type
pub type Result<T> = std::result::Result<T, anyhow::Error>;
