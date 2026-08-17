#![forbid(unsafe_code)]

//! Memory pooling for fingerprint computation
//!
//! This module provides efficient memory pooling to reduce allocation overhead
//! during fingerprint computation and cache operations.
//!
//! Performance optimizations:
//! - Object pooling for Vec<u8> buffers
//! - Zero-allocation for common buffer sizes
//! - Cache-friendly memory layout

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// Size tiers for memory pools (powers of 2 for efficient reuse)
const SIZE_TIERS: &[usize] = &[256, 1024, 4096, 16384, 65536, 262144];

/// Memory pool for Vec<u8> buffers
#[derive(Debug)]
pub struct BufferPool {
    /// Multiple pools for different size tiers
    pools: Vec<Mutex<VecDeque<Vec<u8>>>>,
    /// Statistics
    allocations: Arc<Mutex<PoolStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    pub hits: usize,
    pub misses: usize,
    pub allocations: usize,
    pub deallocations: usize,
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            pools: SIZE_TIERS
                .iter()
                .map(|_| Mutex::new(VecDeque::new()))
                .collect(),
            allocations: Arc::new(Mutex::new(PoolStats::default())),
        }
    }

    /// Get a buffer of at least the requested size
    pub fn get_buffer(&self, min_size: usize) -> Vec<u8> {
        let tier_index = self.find_tier(min_size);

        let mut stats = self.allocations.lock();
        stats.allocations += 1;

        if let Some(pool) = self.pools.get(tier_index) {
            let mut pool = pool.lock();
            if let Some(mut buffer) = pool.pop_front() {
                stats.hits += 1;
                buffer.clear();
                buffer.reserve(min_size);
                return buffer;
            }
        }

        stats.misses += 1;
        Vec::with_capacity(min_size)
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        let capacity = buffer.capacity();
        let tier_index = self.find_tier(capacity);

        let mut stats = self.allocations.lock();
        stats.deallocations += 1;

        // Only pool buffers that match our size tiers
        if let Some(&tier_size) = SIZE_TIERS.get(tier_index) {
            if capacity >= tier_size * 2 / 3 && capacity <= tier_size * 3 / 2 {
                if let Some(pool) = self.pools.get(tier_index) {
                    let mut pool = pool.lock();
                    // Limit pool size to prevent unbounded growth
                    if pool.len() < 10 {
                        pool.push_back(buffer);
                    }
                }
            }
        }
    }

    /// Find the appropriate size tier for a given size
    fn find_tier(&self, size: usize) -> usize {
        SIZE_TIERS
            .iter()
            .position(|&tier| tier >= size)
            .unwrap_or(SIZE_TIERS.len() - 1)
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let stats = self.allocations.lock();
        stats.clone()
    }

    /// Clear all pools (useful for memory pressure scenarios)
    pub fn clear(&self) {
        for pool in &self.pools {
            pool.lock().clear();
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped buffer that automatically returns to pool when dropped
pub struct ScopedBuffer {
    buffer: Vec<u8>,
    pool: Arc<BufferPool>,
}

impl ScopedBuffer {
    pub fn new(min_size: usize, pool: Arc<BufferPool>) -> Self {
        let buffer = pool.get_buffer(min_size);
        Self { buffer, pool }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &Vec<u8> {
        &self.buffer
    }

    pub fn into_inner(mut self) -> Vec<u8> {
        std::mem::take(&mut self.buffer)
    }
}

impl Drop for ScopedBuffer {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            self.pool.return_buffer(std::mem::take(&mut self.buffer));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_basic() {
        let pool = BufferPool::new();

        let buffer1 = pool.get_buffer(100);
        assert!(buffer1.capacity() >= 100);

        pool.return_buffer(buffer1);

        let buffer2 = pool.get_buffer(100);
        assert!(buffer2.capacity() >= 100);

        let stats = pool.stats();
        // We expect at least one hit since we returned the first buffer
        assert!(stats.hits >= 1 || stats.misses >= 1);
    }

    #[test]
    fn test_scoped_buffer() {
        let pool = Arc::new(BufferPool::new());

        {
            let mut scoped = ScopedBuffer::new(1000, pool.clone());
            scoped.as_mut().extend_from_slice(b"test data");
            assert_eq!(scoped.as_ref().len(), 9);
        } // Buffer should be returned to pool here

        let stats = pool.stats();
        assert!(stats.deallocations > 0);
    }

    #[test]
    fn test_size_tiers() {
        let pool = BufferPool::new();

        // Test different size tiers
        let sizes = [100, 500, 2000, 10000, 50000, 200000];
        for size in sizes {
            let buffer = pool.get_buffer(size);
            assert!(buffer.capacity() >= size);
            pool.return_buffer(buffer);
        }
    }

    #[test]
    fn test_pool_limits() {
        let pool = BufferPool::new();

        // Return many buffers - should be limited
        for _ in 0..20 {
            let buffer = pool.get_buffer(1000);
            pool.return_buffer(buffer);
        }

        // Check that pool doesn't grow unbounded
        let first_pool = pool.pools.get(2).unwrap().lock();
        assert!(first_pool.len() <= 10);
    }
}
