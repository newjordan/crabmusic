# [AUDIO-003] Ring Buffer for Audio Pipeline

**Epic**: Audio Capture System
**Priority**: P0 (Blocking)
**Estimated Effort**: 1-2 days
**Status**: Not Started

---

## Description

Implement a lock-free ring buffer for passing audio samples from the audio capture thread to the main processing thread. This is critical for real-time audio performance - the audio thread must never block.

**Agent Instructions**: Create a lock-free ring buffer that:
- Supports single-producer (audio thread) single-consumer (main thread) pattern
- Provides non-blocking write and read operations
- Uses the `ringbuf` crate for proven lock-free implementation
- Wraps AudioBuffer objects for type-safe audio data transfer
- Handles buffer overflow gracefully (overwrite oldest data)

---

## Acceptance Criteria

- [ ] RingBuffer wrapper struct created using `ringbuf` crate
- [ ] Non-blocking `push()` method for audio thread to write AudioBuffer
- [ ] Non-blocking `pop()` method for main thread to read AudioBuffer
- [ ] Buffer capacity configurable (default 8192 samples = ~185ms at 44100 Hz)
- [ ] Thread-safe using Arc for shared ownership
- [ ] No allocations in push/pop operations (pre-allocated at creation)
- [ ] Overflow handling: oldest data overwritten when full
- [ ] Comprehensive unit tests with multi-threaded scenarios
- [ ] Benchmarks showing <1μs push/pop latency
- [ ] Documentation with threading examples
- [ ] Code follows docs/architecture/coding-standards.md
- [ ] All tests pass with `cargo test`
- [ ] No clippy warnings

---

## Technical Approach

### Ring Buffer Wrapper

Reference: **docs/architecture.md - Producer-Consumer with Ring Buffer**

```rust
use ringbuf::{HeapRb, Rb};
use std::sync::Arc;
use crate::audio::AudioBuffer;

/// Lock-free ring buffer for audio pipeline
///
/// Provides single-producer single-consumer communication between
/// audio capture thread and main processing thread.
///
/// # Thread Safety
/// - Producer (audio thread): Calls `push()` to write samples
/// - Consumer (main thread): Calls `pop()` to read samples
/// - Both operations are lock-free and non-blocking
///
/// # Examples
///
/// ```
/// use crabmusic::audio::{AudioRingBuffer, AudioBuffer};
/// use std::sync::Arc;
///
/// let ring_buffer = Arc::new(AudioRingBuffer::new(8192));
/// let producer = ring_buffer.clone();
/// let consumer = ring_buffer.clone();
///
/// // Audio thread
/// std::thread::spawn(move || {
///     let buffer = AudioBuffer::new(1024, 44100, 2);
///     producer.push(buffer);
/// });
///
/// // Main thread
/// if let Some(buffer) = consumer.pop() {
///     println!("Received {} samples", buffer.len());
/// }
/// ```
pub struct AudioRingBuffer {
    inner: HeapRb<AudioBuffer>,
}

impl AudioRingBuffer {
    /// Create a new ring buffer with specified capacity
    ///
    /// # Arguments
    /// * `capacity` - Number of AudioBuffer objects the ring can hold
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::audio::AudioRingBuffer;
    ///
    /// // Buffer for ~200ms of audio at 44100 Hz with 1024 sample buffers
    /// let ring_buffer = AudioRingBuffer::new(10);
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: HeapRb::new(capacity),
        }
    }

    /// Push an audio buffer into the ring (non-blocking)
    ///
    /// If the ring is full, the oldest buffer is overwritten.
    /// This ensures the audio thread never blocks.
    ///
    /// # Arguments
    /// * `buffer` - AudioBuffer to push
    ///
    /// # Returns
    /// `true` if buffer was pushed, `false` if ring was full (oldest overwritten)
    pub fn push(&mut self, buffer: AudioBuffer) -> bool {
        match self.inner.push_overwrite(buffer) {
            None => true,  // Successfully pushed
            Some(_) => false,  // Overwrote oldest buffer
        }
    }

    /// Pop an audio buffer from the ring (non-blocking)
    ///
    /// Returns None if no buffers are available.
    ///
    /// # Returns
    /// `Some(AudioBuffer)` if data available, `None` if ring is empty
    pub fn pop(&mut self) -> Option<AudioBuffer> {
        self.inner.pop()
    }

    /// Check if the ring buffer is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if the ring buffer is full
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Get the number of buffers currently in the ring
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get the total capacity of the ring buffer
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

// Implement Send + Sync for thread safety
unsafe impl Send for AudioRingBuffer {}
unsafe impl Sync for AudioRingBuffer {}
```

### Integration with Audio Capture

The ring buffer will be used in AUDIO-002 like this:

```rust
// In CpalAudioDevice
pub struct CpalAudioDevice {
    stream: Option<cpal::Stream>,
    ring_buffer: Arc<Mutex<AudioRingBuffer>>,
    // ... other fields
}

// In audio callback
let ring_buffer = ring_buffer.clone();
let stream = device.build_input_stream(
    &config.into(),
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let buffer = AudioBuffer::with_samples(
            data.to_vec(),
            sample_rate,
            channels
        );
        
        // Non-blocking push
        ring_buffer.lock().unwrap().push(buffer);
    },
    |err| eprintln!("Audio stream error: {}", err),
)?;
```

### File Organization

Create `src/audio/ring_buffer.rs` with the AudioRingBuffer implementation.

Update `src/audio/mod.rs` to export:
```rust
mod ring_buffer;
pub use ring_buffer::AudioRingBuffer;
```

---

## Dependencies

- **Depends on**:
  - FOUND-001 (project structure exists)
  - AUDIO-001 (AudioBuffer struct defined)
- **Blocks**: 
  - AUDIO-002 (CPAL implementation needs ring buffer)

---

## Architecture References

- **Pattern**: docs/architecture.md - "Producer-Consumer with Ring Buffer"
- **Coding Standards**: docs/architecture.md - "Audio thread must never block"
- **Performance**: docs/architecture.md - "Pre-allocate buffers"

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_creation() {
        let ring = AudioRingBuffer::new(10);
        assert_eq!(ring.capacity(), 10);
        assert_eq!(ring.len(), 0);
        assert!(ring.is_empty());
        assert!(!ring.is_full());
    }

    #[test]
    fn test_push_pop() {
        let mut ring = AudioRingBuffer::new(5);
        let buffer = AudioBuffer::new(1024, 44100, 2);
        
        assert!(ring.push(buffer.clone()));
        assert_eq!(ring.len(), 1);
        
        let popped = ring.pop().unwrap();
        assert_eq!(popped.sample_rate, 44100);
        assert!(ring.is_empty());
    }

    #[test]
    fn test_overflow_behavior() {
        let mut ring = AudioRingBuffer::new(3);
        
        // Fill the ring
        for i in 0..3 {
            let buffer = AudioBuffer::with_samples(vec![i as f32], 44100, 1);
            assert!(ring.push(buffer));
        }
        assert!(ring.is_full());
        
        // Push one more - should overwrite oldest
        let buffer = AudioBuffer::with_samples(vec![99.0], 44100, 1);
        assert!(!ring.push(buffer)); // Returns false on overwrite
        
        // First buffer should be gone, second should be first
        let first = ring.pop().unwrap();
        assert_eq!(first.samples[0], 1.0);
    }

    #[test]
    fn test_multi_threaded() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let ring = Arc::new(Mutex::new(AudioRingBuffer::new(100)));
        let producer = ring.clone();
        let consumer = ring.clone();
        
        // Producer thread
        let producer_handle = thread::spawn(move || {
            for i in 0..50 {
                let buffer = AudioBuffer::with_samples(
                    vec![i as f32],
                    44100,
                    1
                );
                producer.lock().unwrap().push(buffer);
            }
        });
        
        // Consumer thread
        let consumer_handle = thread::spawn(move || {
            let mut count = 0;
            while count < 50 {
                if let Some(_buffer) = consumer.lock().unwrap().pop() {
                    count += 1;
                }
            }
            count
        });
        
        producer_handle.join().unwrap();
        let consumed = consumer_handle.join().unwrap();
        assert_eq!(consumed, 50);
    }
}
```

### Benchmarks

Create `benches/ring_buffer_benchmark.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crabmusic::audio::{AudioRingBuffer, AudioBuffer};

fn bench_push(c: &mut Criterion) {
    c.bench_function("ring_buffer_push", |b| {
        let mut ring = AudioRingBuffer::new(1000);
        let buffer = AudioBuffer::new(1024, 44100, 2);
        
        b.iter(|| {
            ring.push(black_box(buffer.clone()));
        });
    });
}

fn bench_pop(c: &mut Criterion) {
    c.bench_function("ring_buffer_pop", |b| {
        let mut ring = AudioRingBuffer::new(1000);
        
        // Pre-fill
        for _ in 0..500 {
            ring.push(AudioBuffer::new(1024, 44100, 2));
        }
        
        b.iter(|| {
            black_box(ring.pop());
        });
    });
}

criterion_group!(benches, bench_push, bench_pop);
criterion_main!(benches);
```

---

## Dev Notes

- Using `ringbuf` crate for proven lock-free SPSC implementation
- `HeapRb` provides heap-allocated ring buffer (vs stack-allocated)
- `push_overwrite` ensures audio thread never blocks on full buffer
- Mutex wrapper needed for Arc sharing (ringbuf doesn't impl Sync directly)
- Alternative: Could use `crossbeam-channel` bounded channel
- Buffer capacity calculation: (sample_rate / buffer_size) * desired_latency_ms
  - Example: (44100 / 1024) * 200ms ≈ 8.6 buffers → use 10

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code reviewed against coding standards
- [ ] All unit tests passing
- [ ] Multi-threaded tests passing
- [ ] Benchmarks show <1μs latency
- [ ] Documentation complete with examples
- [ ] No compiler warnings
- [ ] Clippy passes with no warnings
- [ ] Code formatted with rustfmt

