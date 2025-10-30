// Ring buffer for lock-free audio pipeline communication

use super::AudioBuffer;
use ringbuf::{traits::*, HeapRb};
use std::sync::{Arc, Mutex};

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
    inner: Arc<Mutex<HeapRb<AudioBuffer>>>,
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
            inner: Arc::new(Mutex::new(HeapRb::new(capacity))),
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
    pub fn push(&self, buffer: AudioBuffer) -> bool {
        let mut ring = self.inner.lock().unwrap();
        if ring.is_full() {
            // Drop oldest buffer to make room
            let _ = ring.try_pop();
        }
        ring.try_push(buffer).is_ok()
    }

    /// Pop an audio buffer from the ring (non-blocking)
    ///
    /// Returns None if no buffers are available.
    ///
    /// # Returns
    /// `Some(AudioBuffer)` if data available, `None` if ring is empty
    pub fn pop(&self) -> Option<AudioBuffer> {
        let mut ring = self.inner.lock().unwrap();
        ring.try_pop()
    }

    /// Check if the ring buffer is empty
    pub fn is_empty(&self) -> bool {
        let ring = self.inner.lock().unwrap();
        ring.is_empty()
    }

    /// Check if the ring buffer is full
    pub fn is_full(&self) -> bool {
        let ring = self.inner.lock().unwrap();
        ring.is_full()
    }

    /// Get the number of buffers currently in the ring
    pub fn len(&self) -> usize {
        let ring = self.inner.lock().unwrap();
        ring.occupied_len()
    }

    /// Get the total capacity of the ring buffer
    pub fn capacity(&self) -> usize {
        let ring = self.inner.lock().unwrap();
        ring.capacity().get()
    }
}

impl Clone for AudioRingBuffer {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// Thread safety: Arc<Mutex<T>> is Send + Sync
unsafe impl Send for AudioRingBuffer {}
unsafe impl Sync for AudioRingBuffer {}

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
        let ring = AudioRingBuffer::new(5);
        let buffer = AudioBuffer::new(1024, 44100, 2);

        assert!(ring.push(buffer.clone()));
        assert_eq!(ring.len(), 1);

        let popped = ring.pop().unwrap();
        assert_eq!(popped.sample_rate, 44100);
        assert!(ring.is_empty());
    }

    #[test]
    fn test_overflow_behavior() {
        let ring = AudioRingBuffer::new(3);

        // Fill the ring
        for i in 0..3 {
            let buffer = AudioBuffer::with_samples(vec![i as f32], 44100, 1);
            assert!(ring.push(buffer));
        }
        assert!(ring.is_full());

        // Push one more - should overwrite oldest
        let buffer = AudioBuffer::with_samples(vec![99.0], 44100, 1);
        assert!(ring.push(buffer)); // Should succeed by dropping oldest

        // First buffer should be gone, second should be first
        let first = ring.pop().unwrap();
        assert_eq!(first.samples[0], 1.0); // Second buffer (index 1)
    }

    #[test]
    fn test_pop_empty() {
        let ring = AudioRingBuffer::new(5);
        assert!(ring.pop().is_none());
    }

    #[test]
    fn test_multiple_push_pop() {
        let ring = AudioRingBuffer::new(10);

        // Push 5 buffers
        for i in 0..5 {
            let buffer = AudioBuffer::with_samples(vec![i as f32], 44100, 1);
            ring.push(buffer);
        }
        assert_eq!(ring.len(), 5);

        // Pop 3 buffers
        for i in 0..3 {
            let buffer = ring.pop().unwrap();
            assert_eq!(buffer.samples[0], i as f32);
        }
        assert_eq!(ring.len(), 2);

        // Push 2 more
        for i in 5..7 {
            let buffer = AudioBuffer::with_samples(vec![i as f32], 44100, 1);
            ring.push(buffer);
        }
        assert_eq!(ring.len(), 4);
    }

    #[test]
    fn test_multi_threaded() {
        use std::thread;

        let ring = AudioRingBuffer::new(100);
        let producer = ring.clone();
        let consumer = ring.clone();

        // Producer thread
        let producer_handle = thread::spawn(move || {
            for i in 0..50 {
                let buffer = AudioBuffer::with_samples(vec![i as f32], 44100, 1);
                producer.push(buffer);
                // Small delay to simulate audio callback timing
                thread::sleep(std::time::Duration::from_micros(10));
            }
        });

        // Consumer thread
        let consumer_handle = thread::spawn(move || {
            let mut count = 0;
            let mut attempts = 0;
            while count < 50 && attempts < 10000 {
                if let Some(_buffer) = consumer.pop() {
                    count += 1;
                } else {
                    // Small delay if no data available
                    thread::sleep(std::time::Duration::from_micros(10));
                }
                attempts += 1;
            }
            count
        });

        producer_handle.join().unwrap();
        let consumed = consumer_handle.join().unwrap();
        assert_eq!(consumed, 50);
    }

    #[test]
    fn test_clone_shares_buffer() {
        let ring1 = AudioRingBuffer::new(5);
        let ring2 = ring1.clone();

        let buffer = AudioBuffer::new(1024, 44100, 2);
        ring1.push(buffer);

        assert_eq!(ring2.len(), 1);
        let popped = ring2.pop().unwrap();
        assert_eq!(popped.sample_rate, 44100);
        assert!(ring1.is_empty());
    }
}
