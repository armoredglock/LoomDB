use std::path::{Path, PathBuf};

/// Configuration options for initializing a LoomDB instance.
#[derive(Debug, Clone)]
pub struct Options {
    /// Directory path where the database persists on disk.
    pub path: PathBuf,
    /// Maximum size of the active MemTable before triggering SSTable flush (default: 64 MB).
    pub max_memtable_size: usize,
    /// Capacity of the uncompressed in-memory block cache (default: 256 MB).
    pub block_cache_size: usize,
    /// Enable Write-Ahead Log (WAL) for strict durability.
    pub enable_wal: bool,
    /// Whether to force an fsync on every commit.
    pub sync_on_write: bool,
    /// Threshold (in bytes) above which values are separated into the Value Log (default: 1024 bytes).
    pub value_threshold: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./loom_data"),
            max_memtable_size: 64 * 1024 * 1024,   // 64 MB
            block_cache_size: 256 * 1024 * 1024,  // 256 MB
            enable_wal: true,
            sync_on_write: false,
            value_threshold: 1024, // 1 KB
        }
    }
}

impl Options {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    pub fn with_wal(mut self, enabled: bool) -> Self {
        self.enable_wal = enabled;
        self
    }

    pub fn with_max_memtable_size(mut self, size: usize) -> Self {
        self.max_memtable_size = size;
        self
    }

    pub fn with_block_cache_size(mut self, size: usize) -> Self {
        self.block_cache_size = size;
        self
    }

    pub fn with_sync_on_write(mut self, sync: bool) -> Self {
        self.sync_on_write = sync;
        self
    }
}
