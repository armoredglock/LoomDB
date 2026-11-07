use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use loom_core::types::InternalKey;
use std::sync::atomic::{AtomicUsize, Ordering};

/// An in-memory, lock-free sorted table of recent mutations.
pub struct MemTable {
    map: SkipMap<InternalKey, Option<Bytes>>,
    byte_size: AtomicUsize,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            map: SkipMap::new(),
            byte_size: AtomicUsize::new(0),
        }
    }

    /// Insert or update a key-value pair. A value of `None` signifies a deletion tombstone.
    pub fn put(&self, key: InternalKey, value: Option<Bytes>) {
        let entry_size = key.user_key.len() + value.as_ref().map_or(0, |v| v.len()) + 16;
        self.byte_size.fetch_add(entry_size, Ordering::Relaxed);
        self.map.insert(key, value);
    }

    /// Retrieve the value for a given key.
    pub fn get(&self, key: &InternalKey) -> Option<Option<Bytes>> {
        self.map.get(key).map(|entry| entry.value().clone())
    }

    /// Approximate total bytes occupied in memory.
    pub fn approximate_size(&self) -> usize {
        self.byte_size.load(Ordering::Relaxed)
    }

    /// Check if the MemTable is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}
