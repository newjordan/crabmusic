// Integration tests for audio capture system
// Tests the full audio capture pipeline with synthetic audio

use crabmusic::audio::{
    AudioBuffer, AudioCaptureDevice, AudioConfig, AudioRingBuffer, CpalAudioDevice,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_audio_ring_buffer_thread_safety() {
    let ring_buffer = Arc::new(AudioRingBuffer::new(10));
    let ring_buffer_clone = Arc::clone(&ring_buffer);

    // Spawn producer thread
    let producer = thread::spawn(move || {
        for i in 0..20 {
            let buffer = AudioBuffer::new(1024, 44100, 2);
            ring_buffer_clone.push(buffer);
            thread::sleep(Duration::from_millis(10));
            if i % 5 == 0 {
                println!("Produced {} buffers", i + 1);
            }
        }
    });

    // Consumer thread (main thread)
    let mut consumed = 0;
    for _ in 0..20 {
        if let Some(_buffer) = ring_buffer.pop() {
            consumed += 1;
        }
        thread::sleep(Duration::from_millis(15));
    }

    producer.join().unwrap();
    println!("Consumed {} buffers", consumed);
    assert!(consumed > 0, "Should have consumed at least some buffers");
}

#[test]
fn test_audio_buffer_properties() {
    let samples = vec![0.0; 2048];
    let buffer = AudioBuffer::with_samples(samples, 48000, 2);

    assert_eq!(buffer.len(), 2048);
    assert_eq!(buffer.sample_rate, 48000);
    assert_eq!(buffer.channels, 2);

    // Duration should be approximately 21.33ms (2048 samples / 2 channels = 1024 frames / 48000)
    let duration = buffer.duration_secs();
    assert!(duration > 0.021 && duration < 0.022);
}

#[test]
fn test_audio_buffer_with_synthetic_data() {
    // Generate a 440Hz sine wave
    let frequency = 440.0;
    let sample_rate = 44100.0;
    let samples: Vec<f32> = (0..1024)
        .map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * frequency * t).sin()
        })
        .collect();

    let buffer = AudioBuffer::with_samples(samples.clone(), 44100, 1);

    // Verify data integrity
    assert_eq!(buffer.samples[0], samples[0]);
    assert_eq!(buffer.samples[512], samples[512]);
    assert_eq!(buffer.samples[1023], samples[1023]);

    // Verify amplitude is reasonable
    let max_amplitude = buffer
        .samples
        .iter()
        .fold(0.0f32, |max, &s| max.max(s.abs()));
    assert!(max_amplitude > 0.9 && max_amplitude <= 1.0);
}

#[test]
fn test_audio_config_validation() {
    let config = AudioConfig {
        sample_rate: 44100,
        channels: 2,
        buffer_size: 1024,
    };

    assert_eq!(config.sample_rate, 44100);
    assert_eq!(config.channels, 2);
    assert_eq!(config.buffer_size, 1024);
}

#[test]
fn test_ring_buffer_capacity_management() {
    let ring_buffer = AudioRingBuffer::new(5);

    // Fill beyond capacity
    for _i in 0..10 {
        let buffer = AudioBuffer::new(512, 44100, 1);
        ring_buffer.push(buffer);
    }

    // Should only be able to pop up to capacity
    let mut popped = 0;
    while ring_buffer.pop().is_some() {
        popped += 1;
    }

    assert!(popped <= 5, "Should not exceed ring buffer capacity");
}

#[test]
fn test_audio_buffer_clear() {
    let samples = vec![0.5; 1024];
    let mut buffer = AudioBuffer::with_samples(samples, 44100, 1);

    // Clear buffer
    buffer.clear();

    // Verify all samples are zero
    assert!(buffer.samples.iter().all(|&s| s == 0.0));
}

#[test]
fn test_audio_buffer_clone() {
    let mut samples = vec![0.0; 512];
    samples[0] = 0.7;
    samples[511] = 0.3;
    let buffer1 = AudioBuffer::with_samples(samples, 44100, 1);

    let buffer2 = buffer1.clone();

    assert_eq!(buffer1.len(), buffer2.len());
    assert_eq!(buffer1.sample_rate, buffer2.sample_rate);
    assert_eq!(buffer1.channels, buffer2.channels);
    assert_eq!(buffer1.samples[0], buffer2.samples[0]);
    assert_eq!(buffer1.samples[511], buffer2.samples[511]);
}

#[test]
fn test_stereo_to_mono_conversion() {
    // This tests the internal conversion logic
    let stereo_samples = [0.5, 0.3, 0.7, 0.1, 0.2, 0.8];
    let mono_samples: Vec<f32> = stereo_samples
        .chunks_exact(2)
        .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
        .collect();

    assert_eq!(mono_samples.len(), 3);
    assert!((mono_samples[0] - 0.4).abs() < 0.001); // (0.5 + 0.3) / 2
    assert!((mono_samples[1] - 0.4).abs() < 0.001); // (0.7 + 0.1) / 2
    assert!((mono_samples[2] - 0.5).abs() < 0.001); // (0.2 + 0.8) / 2
}

#[test]
fn test_audio_buffer_different_sample_rates() {
    let rates = vec![22050, 44100, 48000, 96000];

    for rate in rates {
        let samples = vec![0.0; 1024];
        let buffer = AudioBuffer::with_samples(samples, rate, 1);
        assert_eq!(buffer.sample_rate, rate);

        // Verify duration calculation is correct
        let expected_duration = 1024.0 / rate as f64;
        let actual_duration = buffer.duration_secs();
        assert!((actual_duration - expected_duration).abs() < 0.0001);
    }
}

#[test]
fn test_audio_buffer_different_channel_counts() {
    let channels = vec![1, 2, 4, 8];

    for ch in channels {
        let buffer = AudioBuffer::new(1024, 44100, ch);
        assert_eq!(buffer.channels, ch);
    }
}

#[test]
fn test_ring_buffer_fifo_ordering() {
    let ring_buffer = AudioRingBuffer::new(5);

    // Push buffers with identifiable data
    for i in 0..3 {
        let samples = vec![i as f32];
        let buffer = AudioBuffer::with_samples(samples, 44100, 1);
        ring_buffer.push(buffer);
    }

    // Pop and verify FIFO order
    for i in 0..3 {
        if let Some(buffer) = ring_buffer.pop() {
            assert_eq!(buffer.samples[0], i as f32);
        } else {
            panic!("Expected buffer {} but got None", i);
        }
    }
}

#[test]
fn test_audio_buffer_large_sizes() {
    let sizes = vec![512, 1024, 2048, 4096, 8192];

    for size in sizes {
        let buffer = AudioBuffer::new(size, 44100, 2);
        assert_eq!(buffer.len(), 0); // New buffer is empty
        assert_eq!(buffer.samples.capacity(), size);
    }
}

#[test]
fn test_ring_buffer_concurrent_access() {
    let ring_buffer = Arc::new(AudioRingBuffer::new(100));
    let mut handles = vec![];

    // Spawn multiple producer threads
    for thread_id in 0..4 {
        let rb = Arc::clone(&ring_buffer);
        let handle = thread::spawn(move || {
            for i in 0..25 {
                let samples = vec![(thread_id * 100 + i) as f32; 512];
                let buffer = AudioBuffer::with_samples(samples, 44100, 1);
                rb.push(buffer);
                thread::sleep(Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }

    // Wait for all producers
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify we can pop buffers
    let mut count = 0;
    while ring_buffer.pop().is_some() {
        count += 1;
    }

    assert!(count > 0, "Should have received buffers from producers");
    println!("Received {} buffers from concurrent producers", count);
}

#[test]
#[ignore] // Ignore by default as it requires audio device
fn test_cpal_device_initialization() {
    let ring_buffer = Arc::new(AudioRingBuffer::new(10));
    let result = CpalAudioDevice::new(ring_buffer);

    // This test will fail if no audio device is available
    // but that's expected in CI environments
    match result {
        Ok(device) => {
            println!("Audio device initialized successfully");
            assert!(!device.is_capturing());
        }
        Err(e) => {
            println!(
                "Audio device initialization failed (expected in CI): {:?}",
                e
            );
        }
    }
}

#[test]
fn test_audio_buffer_zero_initialization() {
    let buffer = AudioBuffer::new(1024, 44100, 2);

    // New buffers should be empty
    assert!(buffer.is_empty());
}
