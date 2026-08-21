#![forbid(unsafe_code)]

//! Advanced caching strategies
//!
//! This module provides various caching strategies for optimal performance:
//! - LRU (Least Recently Used) caching with lock-free optimizations
//! - Tiered caching (L1/L2/L3) with automatic promotion
//! - Predictive caching with access pattern analysis
//! - Compression strategies for memory efficiency

use spin::Mutex as SpinMutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry<K, V> {
    pub key: K,
    pub value: V,
    pub access_count: u64,
    pub last_access: Instant,
    pub size: u64,
}

pub struct LruCache<K, V> {
    capacity: usize,
    current_size: u64,
    max_size: u64,
    hits: AtomicU64,
    misses: AtomicU64,
    entries: Arc<RwLock<HashMap<K, CacheEntry<K, V>>>>,
    access_order: Arc<RwLock<VecDeque<K>>>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LruCache<K, V> {
    pub fn new(capacity: usize, max_size: u64) -> Self {
        Self {
            capacity,
            current_size: 0,
            max_size,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            entries: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.get_with_size(key).map(|(value, _)| value)
    }

    /// Get an entry and its size on a hit
    pub fn get_with_size(&self, key: &K) -> Option<(V, u64)> {
        let mut entries = self.entries.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();

        if let Some(entry) = entries.get_mut(key) {
            entry.access_count += 1;
            entry.last_access = Instant::now();
            self.hits.fetch_add(1, Ordering::Relaxed);

            // Move to front of access order
            if let Some(pos) = access_order.iter().position(|k| k == key) {
                access_order.remove(pos);
                access_order.push_front(key.clone());
            }

            Some((entry.value.clone(), entry.size))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    pub fn put(&mut self, key: K, value: V, size: u64) -> Option<V> {
        self.put_with_evictions(key, value, size).0
    }

    /// Insert an entry, returning the replaced value (if the key already
    /// existed) and the entries evicted to make room
    pub fn put_with_evictions(
        &mut self,
        key: K,
        value: V,
        size: u64,
    ) -> (Option<V>, Vec<(K, V, u64)>) {
        let mut entries = self.entries.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();

        // Check if key already exists
        if let Some(entry) = entries.get_mut(&key) {
            let old_value = entry.value.clone();
            self.current_size -= entry.size;

            entry.value = value;
            entry.size = size;
            entry.last_access = Instant::now();
            self.current_size += size;

            // Update access order
            if let Some(pos) = access_order.iter().position(|k| k == &key) {
                access_order.remove(pos);
                access_order.push_front(key);
            }

            return (Some(old_value), Vec::new());
        }

        // Evict if necessary
        let mut evicted = Vec::new();
        while (entries.len() >= self.capacity || self.current_size + size > self.max_size)
            && !access_order.is_empty()
        {
            if let Some(oldest_key) = access_order.pop_back()
                && let Some(entry) = entries.remove(&oldest_key)
            {
                self.current_size -= entry.size;
                evicted.push((entry.key, entry.value, entry.size));
            }
        }

        // Add new entry
        let entry = CacheEntry {
            key: key.clone(),
            value,
            access_count: 0,
            last_access: Instant::now(),
            size,
        };

        self.current_size += size;
        entries.insert(key.clone(), entry);
        access_order.push_front(key);

        (None, evicted)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();

        if let Some(entry) = entries.remove(key) {
            self.current_size -= entry.size;
            access_order.retain(|k| k != key);
            Some(entry.value)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        let mut entries = self.entries.write().unwrap();
        let mut access_order = self.access_order.write().unwrap();

        entries.clear();
        access_order.clear();
        self.current_size = 0;
    }

    pub fn stats(&self) -> LruCacheStats {
        let entries = self.entries.read().unwrap();
        let _access_order = self.access_order.read().unwrap();

        let total_accesses: u64 = entries.values().map(|e| e.access_count).sum();
        let avg_access_count = if !entries.is_empty() {
            total_accesses as f64 / entries.len() as f64
        } else {
            0.0
        };

        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let accesses = hits + misses;
        let hit_rate = if accesses == 0 {
            0.0
        } else {
            hits as f64 / accesses as f64
        };

        LruCacheStats {
            entries: entries.len(),
            capacity: self.capacity,
            current_size: self.current_size as usize,
            max_size: self.max_size,
            hit_rate,
            avg_access_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LruCacheStats {
    pub entries: usize,
    pub capacity: usize,
    pub current_size: usize,
    pub max_size: u64,
    pub hit_rate: f64,
    pub avg_access_count: f64,
}

/// Tiered caching with L1 (fast, small), L2 (medium), L3 (slow, large)
pub struct TieredCache<K, V> {
    l1: Arc<RwLock<LruCache<K, V>>>,
    l2: Arc<RwLock<LruCache<K, V>>>,
    l3: Arc<RwLock<LruCache<K, V>>>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> Default for TieredCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> TieredCache<K, V> {
    pub fn new() -> Self {
        Self {
            l1: Arc::new(RwLock::new(LruCache::new(100, 1024 * 1024))), // 100 entries, 1MB
            l2: Arc::new(RwLock::new(LruCache::new(1000, 100 * 1024 * 1024))), // 1000 entries, 100MB
            l3: Arc::new(RwLock::new(LruCache::new(10000, 10 * 1024 * 1024 * 1024))), // 10000 entries, 10GB
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        // Try L1 first
        if let Some(value) = self.l1.read().unwrap().get(key) {
            return Some(value);
        }

        // Try L2, promote to L1 and demote L1's evictions back to L2
        let l2_hit = {
            let l2 = self.l2.read().unwrap();
            l2.get_with_size(key)
        };
        if let Some((value, size)) = l2_hit {
            let evicted =
                self.l1
                    .write()
                    .unwrap()
                    .put_with_evictions(key.clone(), value.clone(), size);
            drop(evicted.0); // replaced value is dropped, not demoted
            if !evicted.1.is_empty() {
                let mut l2 = self.l2.write().unwrap();
                for (k, v, s) in evicted.1 {
                    l2.put(k, v, s);
                }
            }
            return Some(value);
        }

        // Try L3, promote to L2 and demote L2's evictions down to L3
        let l3_hit = {
            let l3 = self.l3.read().unwrap();
            l3.get_with_size(key)
        };
        if let Some((value, size)) = l3_hit {
            let evicted =
                self.l2
                    .write()
                    .unwrap()
                    .put_with_evictions(key.clone(), value.clone(), size);
            drop(evicted.0); // replaced value is dropped, not demoted
            if !evicted.1.is_empty() {
                let mut l3 = self.l3.write().unwrap();
                for (k, v, s) in evicted.1 {
                    l3.put(k, v, s);
                }
            }
            return Some(value);
        }

        None
    }

    pub fn put(&mut self, key: K, value: V, size: u64) {
        // Write to L1 only; demote L1's evictions to L2, then L2's to L3
        let evicted = self
            .l1
            .write()
            .unwrap()
            .put_with_evictions(key, value, size);
        drop(evicted.0); // replaced value is dropped, not demoted

        if evicted.1.is_empty() {
            return;
        }

        let mut l2 = self.l2.write().unwrap();
        let mut l3_evictions = Vec::new();
        for (k, v, s) in evicted.1 {
            let from_l2 = l2.put_with_evictions(k, v, s);
            drop(from_l2.0);
            l3_evictions.extend(from_l2.1);
        }
        drop(l2);

        if !l3_evictions.is_empty() {
            let mut l3 = self.l3.write().unwrap();
            for (k, v, s) in l3_evictions {
                let replaced = l3.put_with_evictions(k, v, s);
                drop(replaced.0);
                drop(replaced.1); // L3 evictions fall off the bottom of the cache
            }
        }
    }

    pub fn invalidate(&self, key: &K) {
        self.l1.write().unwrap().remove(key);
        self.l2.write().unwrap().remove(key);
        self.l3.write().unwrap().remove(key);
    }

    pub fn clear(&mut self) {
        self.l1.write().unwrap().clear();
        self.l2.write().unwrap().clear();
        self.l3.write().unwrap().clear();
    }

    pub fn stats(&self) -> TieredCacheStats {
        TieredCacheStats {
            l1: self.l1.read().unwrap().stats(),
            l2: self.l2.read().unwrap().stats(),
            l3: self.l3.read().unwrap().stats(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TieredCacheStats {
    pub l1: LruCacheStats,
    pub l2: LruCacheStats,
    pub l3: LruCacheStats,
}

/// Predictive caching based on access patterns
pub struct PredictiveCache<K, V> {
    cache: Arc<RwLock<LruCache<K, V>>>,
    access_patterns: Arc<RwLock<HashMap<K, AccessPattern>>>,
}

#[derive(Debug, Clone)]
struct AccessPattern {
    access_count: u64,
    last_access: Instant,
    access_interval: Duration,
    predicted_next_access: Option<Instant>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> PredictiveCache<K, V> {
    pub fn new(capacity: usize, max_size: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity, max_size))),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        // Check if key exists in patterns first with read lock
        let needs_update = {
            let patterns = self.access_patterns.read().unwrap();
            patterns.contains_key(key)
        };

        // Update pattern stats if key exists
        if needs_update {
            let mut patterns = self.access_patterns.write().unwrap();
            if let Some(pattern) = patterns.get_mut(key) {
                let now = Instant::now();
                let elapsed = now.saturating_duration_since(pattern.last_access);

                pattern.access_count += 1;

                // Only update the interval once we have at least two accesses,
                // and ignore sub-millisecond noise
                if pattern.access_count > 1 && elapsed >= Duration::from_millis(1) {
                    let observed = elapsed.as_secs_f64();
                    let previous = pattern.access_interval.as_secs_f64();
                    // EMA: 80% history, 20% latest observation
                    pattern.access_interval =
                        Duration::from_secs_f64(previous * 0.8 + observed * 0.2);
                    pattern.predicted_next_access = Some(now + pattern.access_interval);
                }

                pattern.last_access = now;
            }
        }

        self.cache.read().unwrap().get(key)
    }

    pub fn put(&self, key: K, value: V, size: u64) {
        let mut patterns = self.access_patterns.write().unwrap();

        patterns
            .entry(key.clone())
            .or_insert_with(|| AccessPattern {
                access_count: 0,
                last_access: Instant::now(),
                access_interval: Duration::ZERO,
                predicted_next_access: None,
            });

        self.cache.write().unwrap().put(key, value, size);
    }

    pub fn predict_next_access(&self, key: &K) -> Option<Instant> {
        let patterns = self.access_patterns.read().unwrap();
        patterns.get(key).and_then(|p| p.predicted_next_access)
    }

    pub fn prefetch(&self, keys: Vec<K>, fetch_fn: impl Fn(&K) -> Option<V>) {
        for key in keys {
            if self
                .predict_next_access(&key)
                .is_some_and(|t| t <= Instant::now() + Duration::from_secs(60))
                && let Some(value) = fetch_fn(&key)
            {
                let size = 1024; // Simplified size calculation
                self.put(key, value, size);
            }
        }
    }
}

/// Spin-lock optimized LRU cache using spin locks for high performance
/// in high-contention scenarios with short critical sections
pub struct SpinLockLruCache<K, V> {
    capacity: usize,
    current_size: Arc<SpinMutex<u64>>,
    max_size: u64,
    entries: Arc<SpinMutex<HashMap<K, CacheEntry<K, V>>>>,
    access_order: Arc<SpinMutex<VecDeque<K>>>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> SpinLockLruCache<K, V> {
    pub fn new(capacity: usize, max_size: u64) -> Self {
        Self {
            capacity,
            current_size: Arc::new(SpinMutex::new(0)),
            max_size,
            entries: Arc::new(SpinMutex::new(HashMap::new())),
            access_order: Arc::new(SpinMutex::new(VecDeque::with_capacity(capacity))),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.lock();
        let mut access_order = self.access_order.lock();

        if let Some(entry) = entries.get_mut(key) {
            entry.access_count += 1;
            entry.last_access = Instant::now();

            // Move to front of access order
            if let Some(pos) = access_order.iter().position(|k| k == key) {
                access_order.remove(pos);
                access_order.push_front(key.clone());
            }

            Some(entry.value.clone())
        } else {
            None
        }
    }

    pub fn put(&self, key: K, value: V, size: u64) -> Option<V> {
        let mut entries = self.entries.lock();
        let mut access_order = self.access_order.lock();
        let mut current_size = self.current_size.lock();

        // Check if key already exists
        if let Some(entry) = entries.get_mut(&key) {
            let old_value = entry.value.clone();
            *current_size -= entry.size;

            entry.value = value;
            entry.size = size;
            entry.last_access = Instant::now();
            *current_size += size;

            // Update access order
            if let Some(pos) = access_order.iter().position(|k| k == &key) {
                access_order.remove(pos);
                access_order.push_front(key);
            }

            return Some(old_value);
        }

        // Evict if necessary
        while (entries.len() >= self.capacity || *current_size + size > self.max_size)
            && !access_order.is_empty()
        {
            if let Some(oldest_key) = access_order.pop_back()
                && let Some(entry) = entries.remove(&oldest_key)
            {
                *current_size -= entry.size;
            }
        }

        // Add new entry
        let entry = CacheEntry {
            key: key.clone(),
            value,
            access_count: 0,
            last_access: Instant::now(),
            size,
        };

        *current_size += size;
        entries.insert(key.clone(), entry);
        access_order.push_front(key);

        None
    }

    pub fn size(&self) -> u64 {
        *self.current_size.lock()
    }

    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LruCache::new(3, 1000);

        cache.put("key1".to_string(), "value1".to_string(), 100);
        cache.put("key2".to_string(), "value2".to_string(), 100);
        cache.put("key3".to_string(), "value3".to_string(), 100);

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        cache.put("key4".to_string(), "value4".to_string(), 100);

        // key2 should be evicted (LRU) since key1 was just accessed
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
        assert_eq!(cache.get(&"key4".to_string()), Some("value4".to_string()));
    }

    #[test]
    fn lru_cache_reports_accurate_hit_rate() {
        let mut cache = LruCache::new(3, 1000);
        cache.put("a".to_string(), "va".to_string(), 10);
        cache.put("b".to_string(), "vb".to_string(), 10);

        let _ = cache.get(&"a".to_string()); // hit
        let _ = cache.get(&"c".to_string()); // miss

        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_tiered_cache() {
        let mut cache = TieredCache::new();

        cache.put("key".to_string(), "value".to_string(), 100);

        // Should be in L1
        assert_eq!(
            cache.l1.read().unwrap().get(&"key".to_string()),
            Some("value".to_string())
        );

        // Get should work from any tier
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_predictive_cache() {
        let cache = PredictiveCache::new(10, 1000);

        cache.put("key".to_string(), "value".to_string(), 100);
        cache.get(&"key".to_string());
        cache.get(&"key".to_string());

        let patterns = cache.access_patterns.read().unwrap();
        assert!(patterns.contains_key(&"key".to_string()));
    }

    #[test]
    fn test_tiered_cache_demotes_overflowing_entries() {
        let mut cache = TieredCache::new();

        for i in 0..200 {
            cache.put(format!("key{i}"), format!("value{i}"), 100);
        }

        assert_eq!(
            cache.get(&"key199".to_string()),
            Some("value199".to_string())
        );
        assert_eq!(cache.get(&"key0".to_string()), Some("value0".to_string()));

        let l1 = cache.l1.read().unwrap();
        assert_eq!(l1.get(&"key199".to_string()), Some("value199".to_string()));
        assert_eq!(l1.get(&"key0".to_string()), Some("value0".to_string()));
        assert_eq!(l1.get(&"key100".to_string()), None);
    }

    #[test]
    fn test_predictive_cache_predicts_intervals() {
        let cache = PredictiveCache::new(10, 1000);

        cache.put("key".to_string(), "value".to_string(), 100);
        cache.get(&"key".to_string());
        cache.get(&"key".to_string());

        std::thread::sleep(Duration::from_millis(100));

        cache.get(&"key".to_string());

        let now = Instant::now();
        let predicted = cache.predict_next_access(&"key".to_string()).unwrap();
        assert!(predicted > now);
        assert!(predicted <= now + Duration::from_millis(200));
    }

    #[test]
    fn test_spin_lock_lru_cache() {
        let cache = SpinLockLruCache::new(3, 1000);

        cache.put("key1".to_string(), "value1".to_string(), 100);
        cache.put("key2".to_string(), "value2".to_string(), 100);
        cache.put("key3".to_string(), "value3".to_string(), 100);

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.len(), 3);

        cache.put("key4".to_string(), "value4".to_string(), 100);

        // key2 should be evicted (LRU) since key1 was just accessed
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
        assert_eq!(cache.get(&"key4".to_string()), Some("value4".to_string()));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_spin_lock_concurrent_access() {
        let cache = Arc::new(SpinLockLruCache::new(100, 10_000_000));
        let mut handles = vec![];

        for thread_id in 0..4 {
            let cache_clone = cache.clone();
            let handle = std::thread::spawn(move || {
                for i in 0..50 {
                    let key = format!("t{}-k{}", thread_id, i);
                    let value = format!("v{}", i);
                    cache_clone.put(key.clone(), value, 100);
                    let _ = cache_clone.get(&key);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(cache.len() <= 100);
    }
}
