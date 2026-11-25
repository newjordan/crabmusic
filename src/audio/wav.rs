//! Minimal WAV reader and writer.
//!
//! Supports PCM integer (8, 16, 24, 32-bit) and IEEE float 32-bit files so the
//! visualizers can run from a recording instead of a live device.

use super::AudioError;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Decoded WAV audio. Samples are interleaved and normalized to -1.0..1.0.
#[derive(Debug, Clone, PartialEq)]
pub struct WavData {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

/// Read a WAV file from disk.
pub fn read_wav(path: impl AsRef<Path>) -> Result<WavData, AudioError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|err| {
        AudioError::FileError(format!("failed to read {}: {err}", path.display()))
    })?;
    parse_wav(&bytes).map_err(|err| AudioError::FileError(format!("{}: {err}", path.display())))
}

/// Write 16-bit PCM WAV. Samples are clamped to -1.0..1.0.
pub fn write_wav(path: impl AsRef<Path>, data: &WavData) -> Result<(), AudioError> {
    let path = path.as_ref();
    if data.channels == 0 || data.sample_rate == 0 {
        return Err(AudioError::InvalidFormat(
            "WAV sample rate and channel count must be non-zero".into(),
        ));
    }
    if data.samples.len() % data.channels as usize != 0 {
        return Err(AudioError::InvalidFormat(
            "WAV sample count is not divisible by the channel count".into(),
        ));
    }

    let mut file = File::create(path).map_err(|err| {
        AudioError::FileError(format!("failed to create {}: {err}", path.display()))
    })?;
    file.write_all(&encode_wav_pcm16(data))
        .map_err(|err| AudioError::FileError(format!("failed to write {}: {err}", path.display())))
}

pub fn parse_wav(bytes: &[u8]) -> Result<WavData, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("not a RIFF/WAVE file".into());
    }

    let mut offset = 12usize;
    let mut format = None;
    let mut data = None;

    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let start = offset + 8;
        let end = start.saturating_add(size).min(bytes.len());
        let chunk = &bytes[start..end];

        if id == b"fmt " && chunk.len() >= 16 {
            format = Some(FmtChunk {
                audio_format: u16::from_le_bytes(chunk[0..2].try_into().unwrap()),
                channels: u16::from_le_bytes(chunk[2..4].try_into().unwrap()),
                sample_rate: u32::from_le_bytes(chunk[4..8].try_into().unwrap()),
                bits_per_sample: u16::from_le_bytes(chunk[14..16].try_into().unwrap()),
            });
        } else if id == b"data" {
            data = Some(chunk.to_vec());
        }

        offset = start + size;
        if size % 2 == 1 {
            offset += 1;
        }
    }

    let fmt = format.ok_or("missing fmt chunk")?;
    let data = data.ok_or("missing data chunk")?;
    if fmt.channels == 0 || fmt.sample_rate == 0 {
        return Err("invalid channel count or sample rate".into());
    }

    let samples = decode_samples(&fmt, &data)?;
    Ok(WavData {
        sample_rate: fmt.sample_rate,
        channels: fmt.channels,
        samples,
    })
}

pub fn encode_wav_pcm16(data: &WavData) -> Vec<u8> {
    let channels = data.channels.max(1);
    let mut pcm = Vec::with_capacity(data.samples.len() * 2);
    for sample in &data.samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let value = (clamped * i16::MAX as f32).round() as i16;
        pcm.extend_from_slice(&value.to_le_bytes());
    }

    let byte_rate = data.sample_rate * channels as u32 * 2;
    let block_align = channels * 2;
    let data_size = pcm.len() as u32;
    let riff_size = 36 + data_size;

    let mut out = Vec::with_capacity(44 + pcm.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&data.sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());
    out.extend_from_slice(&pcm);
    out
}

struct FmtChunk {
    audio_format: u16,
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
}

fn decode_samples(fmt: &FmtChunk, data: &[u8]) -> Result<Vec<f32>, String> {
    let bytes_per_sample = (fmt.bits_per_sample / 8) as usize;
    if bytes_per_sample == 0 {
        return Err(format!("unsupported bit depth {}", fmt.bits_per_sample));
    }
    let frame_bytes = bytes_per_sample * fmt.channels as usize;
    if frame_bytes == 0 || data.len() < frame_bytes {
        return Err("WAV data chunk is empty".into());
    }
    let frame_count = data.len() / frame_bytes;
    let mut samples = Vec::with_capacity(frame_count * fmt.channels as usize);

    match (fmt.audio_format, fmt.bits_per_sample) {
        (1, 8) => {
            for byte in data.iter().take(frame_count * fmt.channels as usize) {
                samples.push((*byte as f32 - 128.0) / 128.0);
            }
        }
        (1, 16) => {
            for chunk in data
                .chunks_exact(2)
                .take(frame_count * fmt.channels as usize)
            {
                let value = i16::from_le_bytes(chunk.try_into().unwrap());
                samples.push(value as f32 / i16::MAX as f32);
            }
        }
        (1, 24) => {
            for chunk in data
                .chunks_exact(3)
                .take(frame_count * fmt.channels as usize)
            {
                let value = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]) >> 8;
                samples.push(value as f32 / 8_388_608.0);
            }
        }
        (1, 32) => {
            for chunk in data
                .chunks_exact(4)
                .take(frame_count * fmt.channels as usize)
            {
                let value = i32::from_le_bytes(chunk.try_into().unwrap());
                samples.push(value as f32 / i32::MAX as f32);
            }
        }
        (3, 32) => {
            for chunk in data
                .chunks_exact(4)
                .take(frame_count * fmt.channels as usize)
            {
                samples.push(f32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }
        (format, bits) => {
            return Err(format!(
                "unsupported WAV encoding format {format} at {bits} bits (need PCM or float)"
            ));
        }
    }

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm16_round_trip_preserves_a_sine() {
        let sample_rate = 8000;
        let samples: Vec<f32> = (0..sample_rate)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sample_rate as f32).sin())
            .collect();
        let original = WavData {
            sample_rate: sample_rate as u32,
            channels: 1,
            samples,
        };
        let parsed = parse_wav(&encode_wav_pcm16(&original)).unwrap();
        assert_eq!(parsed.sample_rate, original.sample_rate);
        assert_eq!(parsed.channels, 1);
        assert_eq!(parsed.samples.len(), original.samples.len());
        for (got, expected) in parsed.samples.iter().zip(original.samples.iter()) {
            assert!((got - expected).abs() < 1.0 / 1000.0);
        }
    }

    #[test]
    fn rejects_non_wav_bytes() {
        assert!(parse_wav(b"not a wav").is_err());
    }
}
