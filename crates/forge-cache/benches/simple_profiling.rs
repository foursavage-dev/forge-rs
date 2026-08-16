//! Simple profiling test for flamegraph analysis

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use forge_cache::{BufferPool, LocalCache};
use tempfile::TempDir;

fn bench_cache_operations(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let cache = LocalCache::new(temp_dir.path()).unwrap();

    // Pre-populate cache
    for i in 0..100 {
        cache
            .put(&format!("key_{}", i), &format!("fingerprint_{}", i))
            .unwrap();
    }

    c.bench_function("cache_operations_100_keys", |b| {
        b.iter(|| {
            for i in 0..100 {
                let key = format!("key_{}", i);
                let fp = format!("fingerprint_{}", i);
                black_box(cache.matches(&key, &fp));
            }
        });
    });
}

fn bench_buffer_pool(c: &mut Criterion) {
    let pool = BufferPool::new();

    c.bench_function("buffer_pool_1000_ops", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let buffer = pool.get_buffer(4096);
                black_box(&buffer);
                pool.return_buffer(buffer);
            }
        });
    });
}

criterion_group!(simple_profiling, bench_cache_operations, bench_buffer_pool);
criterion_main!(simple_profiling);
