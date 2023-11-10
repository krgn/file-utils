use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dedupe_utils::{hash_file_blake2, hash_file_city128, hash_file_murmur128};

fn criterion_benchmark(c: &mut Criterion) {
    let path = Path::new("extra/krgn.jpeg");

    c.bench_function("Blake2s256 krgn.jpb", |b| {
        b.iter(|| hash_file_blake2(black_box(path.to_path_buf())))
    });

    c.bench_function("City128 sse4.2 krgn.jpb", |b| {
        b.iter(|| hash_file_city128(black_box(path.to_path_buf())))
    });

    c.bench_function("Murmur128 krgn.jpb", |b| {
        b.iter(|| hash_file_murmur128(black_box(path.to_path_buf())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
