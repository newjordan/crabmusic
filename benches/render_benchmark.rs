// Rendering performance benchmarks
// TODO: Implement in RENDER-003

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn render_benchmark(c: &mut Criterion) {
    c.bench_function("render_80x24", |b| {
        b.iter(|| {
            // TODO: Benchmark rendering performance
            black_box(());
        });
    });
}

criterion_group!(benches, render_benchmark);
criterion_main!(benches);
