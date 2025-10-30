use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crabmusic::audio::{AudioBuffer, AudioRingBuffer};

fn bench_push(c: &mut Criterion) {
    c.bench_function("ring_buffer_push", |b| {
        let ring = AudioRingBuffer::new(1000);
        let buffer = AudioBuffer::new(1024, 44100, 2);

        b.iter(|| {
            ring.push(black_box(buffer.clone()));
        });
    });
}

fn bench_pop(c: &mut Criterion) {
    c.bench_function("ring_buffer_pop", |b| {
        let ring = AudioRingBuffer::new(1000);

        // Pre-fill
        for _ in 0..500 {
            ring.push(AudioBuffer::new(1024, 44100, 2));
        }

        b.iter(|| {
            black_box(ring.pop());
        });
    });
}

fn bench_push_pop_cycle(c: &mut Criterion) {
    c.bench_function("ring_buffer_push_pop_cycle", |b| {
        let ring = AudioRingBuffer::new(100);
        let buffer = AudioBuffer::new(1024, 44100, 2);

        b.iter(|| {
            ring.push(black_box(buffer.clone()));
            black_box(ring.pop());
        });
    });
}

criterion_group!(benches, bench_push, bench_pop, bench_push_pop_cycle);
criterion_main!(benches);

