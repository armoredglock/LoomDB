use crate::abi::{PageType, PAGE_SIZE};
use crate::pager::Pager;
use loom_core::catalog::DatabaseCatalog;
use loom_core::error::{LoomError, LoomResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ContainerStats {
    pub container_path: String,
    pub file_size_bytes: u64,
    pub page_size: u32,
    pub total_pages: u32,
    pub seqlock_version: u64,
    pub active_root_page: u32,
    pub wal_seq: u64,
    pub current_ram_bytes: u64,
    pub max_ram_bytes: u64,
    pub ram_utilization_pct: f64,
    pub active_branches: usize,
    pub flight_recorder_entries: usize,
    pub table_count: usize,
}

/// Unified Single-File `.loom` Container Engine
pub struct LoomContainer {
    pub path: PathBuf,
    pub pager: Pager,
    pub catalog: DatabaseCatalog,
}

impl LoomContainer {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> LoomResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let mut pager = Pager::open_or_create(&path_buf)?;

        // Attempt to load catalog from container pages or legacy JSON
        let catalog = Self::load_catalog(&path_buf, &mut pager).unwrap_or_else(|_| DatabaseCatalog::new());

        // Sync governor state with pager
        pager.set_governor(catalog.governor.clone());

        Ok(Self {
            path: path_buf,
            pager,
            catalog,
        })
    }

    fn load_catalog(path: &Path, pager: &mut Pager) -> LoomResult<DatabaseCatalog> {
        if path.exists() {
            let mut file = File::open(path)?;
            let mut magic = [0u8; 8];
            if file.read_exact(&mut magic).is_ok() && &magic == crate::abi::MAGIC_BYTES {
                // Read from binary container catalog page
                if pager.header.total_pages > 1 {
                    let page_buf = pager.read_page(1)?;
                    let data = &page_buf[crate::abi::PAGE_HEADER_SIZE..];
                    if let Ok(cat) = serde_json::from_slice::<DatabaseCatalog>(data) {
                        return Ok(cat);
                    }
                }
            } else {
                // Legacy json file fallback
                let mut data = Vec::new();
                file.seek(SeekFrom::Start(0))?;
                file.read_to_end(&mut data)?;
                if let Ok(cat) = serde_json::from_slice::<DatabaseCatalog>(&data) {
                    return Ok(cat);
                }
            }
        }
        Ok(DatabaseCatalog::new())
    }

    pub fn save(&mut self) -> LoomResult<()> {
        // Serialize catalog into container pages
        let cat_bytes = serde_json::to_vec(&self.catalog)
            .map_err(|e| LoomError::Internal(format!("Catalog serialize failed: {}", e)))?;

        // Allocate or update catalog page (page 1)
        if self.pager.header.total_pages <= 1 {
            self.pager.allocate_page(PageType::Catalog)?;
        }

        let mut branch_map = std::collections::HashMap::new();
        branch_map.insert(1, 1);
        self.pager.write_page_cow(1, &mut branch_map, &cat_bytes, PageType::Catalog)?;

        self.pager.header.catalog_page = 1;
        self.pager.header.wal_seq = self.catalog.flight_recorder.sequence;

        self.pager.sync()?;
        Ok(())
    }

    pub fn stats(&self) -> ContainerStats {
        let file_size = if self.path.exists() {
            std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let ram_used = self.pager.governor().current_ram_bytes;
        let ram_cap = self.pager.governor().max_ram_bytes;
        let ram_pct = if ram_cap > 0 {
            (ram_used as f64 / ram_cap as f64) * 100.0
        } else {
            0.0
        };

        ContainerStats {
            container_path: self.path.display().to_string(),
            file_size_bytes: file_size,
            page_size: PAGE_SIZE as u32,
            total_pages: self.pager.header.total_pages,
            seqlock_version: self.pager.header.version_seq,
            active_root_page: self.pager.header.active_root_page,
            wal_seq: self.pager.header.wal_seq,
            current_ram_bytes: ram_used,
            max_ram_bytes: ram_cap,
            ram_utilization_pct: ram_pct,
            active_branches: self.catalog.branches.branches.len(),
            flight_recorder_entries: self.catalog.flight_recorder.records.len(),
            table_count: self.catalog.tables.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loom_container_lifecycle_and_stats() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("test_container_{}.loom", ts));

        let mut container = LoomContainer::open_or_create(&path).unwrap();
        assert!(container.save().is_ok());

        let stats = container.stats();
        assert_eq!(stats.page_size, 4096);
        assert!(stats.total_pages >= 2);
        assert!(stats.file_size_bytes >= 4096);
        assert_eq!(stats.active_branches, 1); // 'main'
        assert!(stats.table_count >= 1); // 'memories'

        let _ = std::fs::remove_file(path);
    }
}
