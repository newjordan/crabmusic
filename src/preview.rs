//! Headless visualizer catalog and text captures.
//!
//! This is the library side of the single `crabmusic --visualizer <name>`
//! entry point. It builds each existing visualizer with its normal constructor
//! and renders it from a synthetic tone or a WAV file, without opening a
//! terminal or an audio device.

use crate::audio::{AudioBuffer, WavData};
use crate::dsp::DspProcessor;
use crate::visualization::character_sets::{get_character_set, CharacterSetType};
use crate::visualization::color_schemes::{ColorScheme, ColorSchemeType};
use crate::visualization::ray_tracer::RenderMode;
use crate::visualization::{
    GravityWellVisualizer, GridBuffer, GridTunnelVisualizer, ImageChannelVisualizer,
    NightNightVisualizer, ObjViewerVisualizer, OscilloscopeConfig, OscilloscopeVisualizer,
    PrimitivesVisualizer, Raycaster3DVisualizer, ScrollDirection, SineWaveConfig,
    SineWaveVisualizer, SpectrogramVisualizer, SpectrumConfig, SpectrumVisualizer,
    StarfieldVisualizer, TerrainLandscapeVisualizer, VideoChannelVisualizer, Visualizer,
    WaveformTunnelVisualizer, XYOscilloscopeConfig, XYOscilloscopeVisualizer,
};
use std::fs;
use std::path::Path;

/// A visualizer the app can start by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualizerInfo {
    /// Stable CLI id, for example `spectrum`.
    pub id: &'static str,
    /// Human-readable name shown in the terminal.
    pub title: &'static str,
}

const CATALOG: &[VisualizerInfo] = &[
    VisualizerInfo {
        id: "sine-wave",
        title: "Sine Wave",
    },
    VisualizerInfo {
        id: "spectrum",
        title: "Spectrum",
    },
    VisualizerInfo {
        id: "oscilloscope",
        title: "Oscilloscope",
    },
    VisualizerInfo {
        id: "xy-oscilloscope",
        title: "XY Oscilloscope",
    },
    VisualizerInfo {
        id: "raycaster3d",
        title: "Raycaster 3D",
    },
    VisualizerInfo {
        id: "obj-viewer",
        title: "3D Model Viewer",
    },
    VisualizerInfo {
        id: "primitives",
        title: "Primitives 3D",
    },
    VisualizerInfo {
        id: "grid-tunnel",
        title: "Grid Tunnel",
    },
    VisualizerInfo {
        id: "gravity-well",
        title: "Gravity Well",
    },
    VisualizerInfo {
        id: "waveform-tunnel",
        title: "Waveform Tunnel",
    },
    VisualizerInfo {
        id: "terrain",
        title: "Terrain Landscape",
    },
    VisualizerInfo {
        id: "starfield",
        title: "Starfield",
    },
    VisualizerInfo {
        id: "spectrogram",
        title: "Spectrogram",
    },
    VisualizerInfo {
        id: "night-night",
        title: "Night Night",
    },
    VisualizerInfo {
        id: "image",
        title: "Image Mode",
    },
    VisualizerInfo {
        id: "video",
        title: "Video Mode",
    },
    VisualizerInfo {
        id: "archive-tv",
        title: "Archive TV",
    },
    VisualizerInfo {
        id: "archive-cooking",
        title: "Archive Cooking",
    },
    VisualizerInfo {
        id: "archive-public-access",
        title: "Archive Public Access",
    },
    VisualizerInfo {
        id: "archive-industrial",
        title: "Archive Industrial Films",
    },
    VisualizerInfo {
        id: "archive-educational",
        title: "Archive Educational",
    },
    VisualizerInfo {
        id: "archive-local-news",
        title: "Archive Local News",
    },
];

/// Every visualizer the CLI can launch, in channel order.
pub fn visualizers() -> &'static [VisualizerInfo] {
    CATALOG
}

/// Resolve a CLI name. Accepts the canonical id, spaces, or underscores.
pub fn find_visualizer(name: &str) -> Option<&'static VisualizerInfo> {
    let key = normalize_id(name);
    CATALOG.iter().find(|info| info.id == key)
}

fn normalize_id(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['_', ' '], "-")
}

/// Build the same visualizer the interactive app starts for this id.
pub fn create_visualizer(id: &str, sample_rate: u32) -> Result<Box<dyn Visualizer>, String> {
    let info = find_visualizer(id).ok_or_else(|| {
        format!("unknown visualizer '{id}'. Run with --list-visualizers to see the names.")
    })?;
    let scheme = ColorScheme::new(ColorSchemeType::Monochrome);
    let charset = get_character_set(CharacterSetType::Blocks);
    let visualizer: Box<dyn Visualizer> = match info.id {
        "sine-wave" => Box::new(SineWaveVisualizer::new(SineWaveConfig::default(), charset)),
        "spectrum" => {
            let mut viz = SpectrumVisualizer::new(SpectrumConfig::default(), sample_rate, charset);
            viz.set_peak_hold(true);
            viz.set_show_labels(true);
            Box::new(viz)
        }
        "oscilloscope" => Box::new(OscilloscopeVisualizer::new(OscilloscopeConfig::default())),
        "xy-oscilloscope" => Box::new(XYOscilloscopeVisualizer::new(
            XYOscilloscopeConfig::default(),
        )),
        "raycaster3d" => Box::new(Raycaster3DVisualizer::new_with(
            RenderMode::Wireframe {
                step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
                tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            },
            0.0,
        )),
        "obj-viewer" => Box::new(ObjViewerVisualizer::new()),
        "primitives" => Box::new(PrimitivesVisualizer::new(scheme)),
        "grid-tunnel" => Box::new(GridTunnelVisualizer::new(scheme)),
        "gravity-well" => Box::new(GravityWellVisualizer::new(scheme)),
        "waveform-tunnel" => Box::new(WaveformTunnelVisualizer::new(scheme)),
        "terrain" => Box::new(TerrainLandscapeVisualizer::new(scheme)),
        "starfield" => Box::new(StarfieldVisualizer::new(scheme)),
        "spectrogram" => Box::new(SpectrogramVisualizer::new(scheme, ScrollDirection::Up)),
        "night-night" => Box::new(NightNightVisualizer::new(scheme)),
        "image" => Box::new(ImageChannelVisualizer::new(scheme)),
        "video" => Box::new(VideoChannelVisualizer::new(scheme)),
        "archive-tv" => Box::new(VideoChannelVisualizer::new_named(
            scheme,
            "Archive TV",
            "Random old TV from Internet Archive",
        )),
        "archive-cooking" => Box::new(VideoChannelVisualizer::new_named(
            scheme,
            "Archive Cooking",
            "Random vintage cooking shows",
        )),
        "archive-public-access" => Box::new(VideoChannelVisualizer::new_named(
            scheme,
            "Archive Public Access",
            "Random public-access oddities",
        )),
        "archive-industrial" => Box::new(VideoChannelVisualizer::new_named(
            scheme,
            "Archive Industrial Films",
            "Random industrial and training films",
        )),
        "archive-educational" => Box::new(VideoChannelVisualizer::new_named(
            scheme,
            "Archive Educational",
            "Random educational films",
        )),
        "archive-local-news" => Box::new(VideoChannelVisualizer::new_named(
            scheme,
            "Archive Local News",
            "Random local news oddities",
        )),
        _ => return Err(format!("visualizer '{}' has no constructor", info.id)),
    };
    Ok(visualizer)
}

/// A chord with a slow amplitude pulse, used for captures and DSP tests.
///
/// The mix is bass (82 Hz), mid (440 Hz), a fifth (659 Hz), and treble (3520 Hz)
/// so band-sensitive visualizers all have something to draw.
pub fn synthetic_test_tone(sample_rate: u32, duration_secs: f32) -> Vec<f32> {
    let count = (sample_rate as f32 * duration_secs).max(1.0) as usize;
    (0..count)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let pulse = 0.55 + 0.45 * (2.0 * std::f32::consts::PI * 2.0 * t).sin().max(0.0);
            let bass = 0.70 * (2.0 * std::f32::consts::PI * 82.41 * t).sin();
            let mid = 0.45 * (2.0 * std::f32::consts::PI * 440.0 * t).sin();
            let fifth = 0.28 * (2.0 * std::f32::consts::PI * 659.25 * t).sin();
            let treble = 0.16 * (2.0 * std::f32::consts::PI * 3520.0 * t).sin();
            (pulse * (bass + mid + fifth + treble)).clamp(-1.0, 1.0)
        })
        .collect()
}

/// Write the synthetic demo tone as 16-bit PCM WAV.
pub fn write_synthetic_tone(path: impl AsRef<Path>) -> Result<(), String> {
    let data = WavData {
        sample_rate: 44_100,
        channels: 1,
        samples: synthetic_test_tone(44_100, 2.0),
    };
    crate::audio::write_wav(path, &data).map_err(|err| err.to_string())
}

/// Drive one visualizer with audio and return the last frame as text.
pub fn render_capture(
    id: &str,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    width: usize,
    height: usize,
    frames: usize,
) -> Result<String, String> {
    let mut visualizer = create_visualizer(id, sample_rate)?;
    let frame_limit = frame_budget(id, frames);
    let mut grid = GridBuffer::new(width, height);
    drive_visualizer(
        visualizer.as_mut(),
        samples,
        sample_rate,
        channels,
        frame_limit,
        &mut grid,
    )?;
    Ok(grid_to_text(&grid))
}

/// Render every visualizer (or one, when `only` is set) into `dir` as `.txt` files.
pub fn write_captures(
    dir: impl AsRef<Path>,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    width: usize,
    height: usize,
    frames: usize,
    only: Option<&str>,
) -> Result<Vec<VisualizerInfo>, String> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir).map_err(|err| format!("failed to create {}: {err}", dir.display()))?;

    let selected: Vec<&VisualizerInfo> = if let Some(name) = only {
        vec![find_visualizer(name).ok_or_else(|| {
            format!("unknown visualizer '{name}'. Run with --list-visualizers to see the names.")
        })?]
    } else {
        CATALOG.iter().collect()
    };

    let mut written = Vec::new();
    for info in selected {
        let text = render_capture(
            info.id,
            samples,
            sample_rate,
            channels,
            width,
            height,
            frames,
        )?;
        let path = dir.join(format!("{}.txt", info.id));
        fs::write(&path, &text)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        written.push(*info);
    }
    Ok(written)
}

fn frame_budget(id: &str, frames: usize) -> usize {
    let frames = frames.max(1);
    // Ray-traced modes stay on their existing renderers; a few frames is enough
    // for a still and keeps capture time bounded.
    if matches!(id, "raycaster3d" | "obj-viewer" | "primitives") {
        frames.min(3)
    } else {
        frames
    }
}

fn drive_visualizer(
    visualizer: &mut dyn Visualizer,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    frames: usize,
    grid: &mut GridBuffer,
) -> Result<(), String> {
    let channels = channels.max(1);
    let window = 2048usize;
    let mut processor = DspProcessor::new(sample_rate, window)
        .map_err(|err| format!("failed to create DSP processor: {err}"))?;
    let frame_samples = window;
    let frames_per_chunk = (frame_samples / channels as usize).max(1);
    let total_frames = samples.len() / channels as usize;
    if total_frames == 0 {
        return Err("audio file has no frames".into());
    }

    let mut rendered = 0usize;
    let mut cursor = 0usize;
    while rendered < frames {
        if cursor >= total_frames {
            cursor = 0;
        }
        let take = frames_per_chunk.min(total_frames - cursor).max(1);
        let start = cursor * channels as usize;
        let end = (start + take * channels as usize).min(samples.len());
        let buffer = AudioBuffer::with_samples(samples[start..end].to_vec(), sample_rate, channels);
        let params = processor.process(&buffer);
        visualizer.update(&params);
        grid.clear();
        visualizer.render(grid);
        cursor += take;
        rendered += 1;
    }
    Ok(())
}

/// Render a grid as plain terminal text. Trailing spaces are kept so columns line up.
pub fn grid_to_text(grid: &GridBuffer) -> String {
    let mut out = String::new();
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            out.push(grid.get_cell(x, y).character);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_ids_are_unique_and_constructible() {
        let mut seen = std::collections::BTreeSet::new();
        for info in visualizers() {
            assert!(seen.insert(info.id), "duplicate id {}", info.id);
            assert!(
                create_visualizer(info.id, 44_100).is_ok(),
                "failed to construct {}",
                info.id
            );
        }
        assert_eq!(visualizers().len(), 22);
    }

    #[test]
    fn name_lookup_accepts_spaces_and_underscores() {
        assert_eq!(
            find_visualizer("XY Oscilloscope").unwrap().id,
            "xy-oscilloscope"
        );
        assert_eq!(find_visualizer("grid_tunnel").unwrap().id, "grid-tunnel");
        assert!(find_visualizer("not-a-mode").is_none());
    }

    #[test]
    fn sine_wave_capture_is_not_blank() {
        let tone = synthetic_test_tone(44_100, 0.25);
        let text = render_capture("sine-wave", &tone, 44_100, 1, 48, 12, 4).unwrap();
        assert!(text.chars().any(|ch| ch != ' ' && ch != '\n'));
        assert_eq!(text.lines().count(), 12);
    }
}
