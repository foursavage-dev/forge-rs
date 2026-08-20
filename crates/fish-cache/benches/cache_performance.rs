#![cfg_attr(not(test), forbid(unsafe_code))]

//! Performance benchmarks for cache operations
//!
//! This module provides comprehensive benchmarks to measure the impact
//! of performance optimizations.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fish_cache::{BufferPool, LocalCache, ScopedBuffer};
use std::time::Duration;
use tempfile::TempDir;

fn bench_buffer_pool_basic(c: &mut Criterion) {
    let pool = BufferPool::new();

    let mut group = c.benchmark_group("buffer_pool");

    for size in [256, 1024, 4096, 16384, 65536].iter() {
        group.bench_with_input(BenchmarkId::new("get_return", size), size, |b, &size| {
            b.iter(|| {
                let buffer = pool.get_buffer(size);
                std::hint::black_box(&buffer);
                pool.return_buffer(buffer);
            })
        });
    }

    group.finish();
}

fn bench_buffer_pool_scoped(c: &mut Criterion) {
    let pool = std::sync::Arc::new(BufferPool::new());

    let mut group = c.benchmark_group("scoped_buffer");

    for size in [256, 1024, 4096, 16384, 65536].iter() {
        group.bench_with_input(BenchmarkId::new("scoped", size), size, |b, &size| {
            b.iter(|| {
                let mut scoped = ScopedBuffer::new(size, pool.clone());
                scoped.as_mut().resize(size, 0);
                std::hint::black_box(scoped.as_ref().len());
            })
        });
    }

    group.finish();
}

fn bench_cache_put_get(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let cache = LocalCache::new(temp_dir.path()).unwrap();

    c.bench_function("cache_put_get", |b| {
        b.iter(|| {
            let key = format!("test_key_{}", std::hint::black_box(42));
            let fingerprint = format!("fingerprint_{}", std::hint::black_box(42));
            cache.put(&key, &fingerprint).unwrap();
            let result = cache.get(&key);
            std::hint::black_box(result);
        })
    });
}

fn bench_cache_matches(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let cache = LocalCache::new(temp_dir.path()).unwrap();

    // Pre-populate cache
    for i in 0..100 {
        cache
            .put(&format!("key_{}", i), &format!("fp_{}", i))
            .unwrap();
    }

    c.bench_function("cache_matches_hit", |b| {
        b.iter(|| {
            let result = cache.matches("key_42", "fp_42");
            std::hint::black_box(result);
        })
    });

    c.bench_function("cache_matches_miss", |b| {
        b.iter(|| {
            let result = cache.matches("key_42", "wrong_fp");
            std::hint::black_box(result);
        })
    });
}

fn bench_cache_put_object(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let cache = LocalCache::new(temp_dir.path()).unwrap();

    let mut group = c.benchmark_group("cache_objects");

    for size in [1024, 4096, 16384, 65536].iter() {
        let data = vec![0u8; *size];
        group.bench_with_input(BenchmarkId::new("put_object", size), size, |b, &size| {
            let hash = format!("hash_{}", size);
            b.iter(|| {
                cache
                    .put_object(&hash, std::hint::black_box(&data))
                    .unwrap();
            })
        });
    }

    group.finish();
}

fn bench_cache_disk_stats(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let cache = LocalCache::new(temp_dir.path()).unwrap();

    // Pre-populate cache
    for i in 0..100 {
        cache
            .put(&format!("key_{}", i), &format!("fp_{}", i))
            .unwrap();
        cache
            .put_object(&format!("obj_{}", i), &vec![0u8; 1024])
            .unwrap();
    }

    c.bench_function("cache_disk_stats", |b| {
        b.iter(|| {
            let stats = cache.disk_stats();
            std::hint::black_box(stats);
        })
    });
}

fn bench_cache_prune(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let cache = LocalCache::new(temp_dir.path()).unwrap();

    // Pre-populate cache with old records
    for i in 0..100 {
        cache
            .put(&format!("key_{}", i), &format!("fp_{}", i))
            .unwrap();
        cache
            .put_object(&format!("obj_{}", i), &vec![0u8; 1024])
            .unwrap();
    }

    c.bench_function("cache_prune_age", |b| {
        b.iter(|| {
            let report = cache.prune(Some(Duration::from_secs(3600)), None).unwrap();
            std::hint::black_box(report);
        })
    });

    c.bench_function("cache_prune_size", |b| {
        b.iter(|| {
            let report = cache.prune(None, Some(1024)).unwrap();
            std::hint::black_box(report);
        })
    });
}

criterion_group!(
    benches,
    bench_buffer_pool_basic,
    bench_buffer_pool_scoped,
    bench_cache_put_get,
    bench_cache_matches,
    bench_cache_put_object,
    bench_cache_disk_stats,
    bench_cache_prune
);
criterion_main!(benches);
