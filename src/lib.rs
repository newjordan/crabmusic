//! CrabMusic - Real-time ASCII music visualizer for terminal
//!
//! This library provides the core functionality for capturing system audio,
//! processing it through DSP (FFT), and rendering audio-reactive visualizations
//! in the terminal.
//!
//! # Architecture
//!
//! The library follows a pipeline architecture:
//! - **Audio Capture**: Capture system audio using CPAL
//! - **DSP Processing**: Extract audio parameters using FFT
//! - **Visualization**: Generate visual representations from audio parameters
//! - **Terminal Rendering**: Render visualizations to terminal using Ratatui
//!
//! # Examples
//!
//! ```no_run
//! use crabmusic::audio::{AudioBuffer, AudioRingBuffer};
//!
//! let ring_buffer = AudioRingBuffer::new(10);
//! let buffer = AudioBuffer::new(1024, 44100, 2);
//! ring_buffer.push(buffer);
//! ```

// Public modules
pub mod audio;
pub mod config;
pub mod dsp;
pub mod effects;
pub mod error;
pub mod rendering;
pub mod visualization;
