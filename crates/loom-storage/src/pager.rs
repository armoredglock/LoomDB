use crate::abi::{ContainerHeader, PageHeader, PageType, HEADER_SIZE, PAGE_SIZE};
use crc32fast::Hasher;
use loom_core::error::{LoomError, LoomResult};
use loom_core::governor::BudgetGovernor;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Single-File 64-Bit Virtual Paging Engine for `.loom` containers.
pub struct Pager {
    file: File,
    pub header: ContainerHeader,
    page_cache: HashMap<u32, [u8; PAGE_SIZE]>,
    dirty_pages: HashSet<u32>,
    /// Physical page reference counts across branches for Copy-on-Write (COW) management
    page_ref_counts: HashMap<u32, usize>,
    governor: BudgetGovernor,
}

impl Pager {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> LoomResult<Self> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(p)?;

        let metadata = file.metadata()?;
        let mut page_ref_counts = HashMap::new();
        let governor = BudgetGovernor::default();

        let header = if metadata.len() < HEADER_SIZE as u64 {
            let h = ContainerHeader::new();
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&h.serialize())?;
            // Pad to full first page (4096 bytes)
            let padding = vec![0u8; PAGE_SIZE - HEADER_SIZE];
            file.write_all(&padding)?;
            file.sync_all()?;
            page_ref_counts.insert(0, 1);
            h
        } else {
            let mut h_buf = [0u8; HEADER_SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut h_buf)?;
            let h = ContainerHeader::deserialize(&h_buf)?;
            for i in 0..h.total_pages {
                page_ref_counts.insert(i, 1);
            }
            h
        };

        Ok(Self {
            file,
            header,
            page_cache: HashMap::new(),
            dirty_pages: HashSet::new(),
            page_ref_counts,
            governor,
        })
    }

    pub fn set_governor(&mut self, governor: BudgetGovernor) {
        self.governor = governor;
    }

    pub fn governor(&self) -> &BudgetGovernor {
        &self.governor
    }

    pub fn governor_mut(&mut self) -> &mut BudgetGovernor {
        &mut self.governor
    }

    /// Allocate a new 4096-byte physical page slot in the container.
    /// Strictly enforced against BudgetGovernor RAM limits.
    pub fn allocate_page(&mut self, page_type: PageType) -> LoomResult<u32> {
        // Enforce hard RAM budget
        self.governor.track_alloc(PAGE_SIZE as u64)?;

        let page_id = self.header.total_pages;
        self.header.total_pages += 1;

        let mut page_buf = [0u8; PAGE_SIZE];
        let ph = PageHeader::new(page_id, page_type);
        page_buf[0..crate::abi::PAGE_HEADER_SIZE].copy_from_slice(&ph.serialize());

        self.page_cache.insert(page_id, page_buf);
        self.dirty_pages.insert(page_id);
        self.page_ref_counts.insert(page_id, 1);

        Ok(page_id)
    }

    /// Read a 4096-byte page. Enforced against per-query I/O budget limits.
    pub fn read_page(&mut self, page_id: u32) -> LoomResult<[u8; PAGE_SIZE]> {
        if page_id >= self.header.total_pages {
            return Err(LoomError::PageNotFound(page_id));
        }

        // Check I/O budget cap
        self.governor.track_io(1)?;

        if let Some(cached) = self.page_cache.get(&page_id) {
            return Ok(*cached);
        }

        // Cache miss: read from disk container
        let mut buf = [0u8; PAGE_SIZE];
        let offset = page_id as u64 * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut buf)?;

        self.page_cache.insert(page_id, buf);
        Ok(buf)
    }

    /// True Copy-on-Write (COW) Branch Mutation:
    /// If the page is shared by multiple branches (ref_count > 1), allocates a new physical slot,
    /// copies parent contents, and rebinds the branch page directory table.
    /// If unshared (ref_count == 1), modifies the page directly.
    pub fn write_page_cow(
        &mut self,
        logical_page_id: u32,
        branch_page_map: &mut HashMap<u32, u32>,
        data: &[u8],
        page_type: PageType,
    ) -> LoomResult<u32> {
        let current_phys = branch_page_map.get(&logical_page_id).copied();

        match current_phys {
            Some(phys_id) => {
                let ref_count = self.page_ref_counts.get(&phys_id).copied().unwrap_or(1);
                if ref_count > 1 {
                    // Page is shared across multiple branches! Perform Copy-on-Write allocation.
                    let new_phys_id = self.allocate_page(page_type)?;
                    let mut new_buf = self.read_page(phys_id)?;

                    // Copy payload into new slot
                    let copy_len = data.len().min(PAGE_SIZE - crate::abi::PAGE_HEADER_SIZE);
                    new_buf[crate::abi::PAGE_HEADER_SIZE..crate::abi::PAGE_HEADER_SIZE + copy_len]
                        .copy_from_slice(&data[..copy_len]);

                    // Recompute checksum
                    let mut hasher = Hasher::new();
                    hasher.update(&new_buf[crate::abi::PAGE_HEADER_SIZE..]);
                    let cs = hasher.finalize();

                    let mut ph = PageHeader::new(new_phys_id, page_type);
                    ph.checksum = cs;
                    new_buf[0..crate::abi::PAGE_HEADER_SIZE].copy_from_slice(&ph.serialize());

                    self.page_cache.insert(new_phys_id, new_buf);
                    self.dirty_pages.insert(new_phys_id);

                    // Decrement old physical page ref count
                    self.page_ref_counts.insert(phys_id, ref_count.saturating_sub(1));
                    // Point branch mapping to new physical slot
                    branch_page_map.insert(logical_page_id, new_phys_id);

                    Ok(new_phys_id)
                } else {
                    // Unshared page: overwrite in place
                    let mut buf = self.read_page(phys_id)?;
                    let copy_len = data.len().min(PAGE_SIZE - crate::abi::PAGE_HEADER_SIZE);
                    buf[crate::abi::PAGE_HEADER_SIZE..crate::abi::PAGE_HEADER_SIZE + copy_len]
                        .copy_from_slice(&data[..copy_len]);

                    let mut hasher = Hasher::new();
                    hasher.update(&buf[crate::abi::PAGE_HEADER_SIZE..]);
                    let cs = hasher.finalize();

                    let mut ph = PageHeader::new(phys_id, page_type);
                    ph.checksum = cs;
                    buf[0..crate::abi::PAGE_HEADER_SIZE].copy_from_slice(&ph.serialize());

                    self.page_cache.insert(phys_id, buf);
                    self.dirty_pages.insert(phys_id);
                    Ok(phys_id)
                }
            }
            None => {
                // New logical page allocation
                let new_phys_id = self.allocate_page(page_type)?;
                let mut buf = [0u8; PAGE_SIZE];
                let copy_len = data.len().min(PAGE_SIZE - crate::abi::PAGE_HEADER_SIZE);
                buf[crate::abi::PAGE_HEADER_SIZE..crate::abi::PAGE_HEADER_SIZE + copy_len]
                    .copy_from_slice(&data[..copy_len]);

                let mut hasher = Hasher::new();
                hasher.update(&buf[crate::abi::PAGE_HEADER_SIZE..]);
                let cs = hasher.finalize();

                let mut ph = PageHeader::new(new_phys_id, page_type);
                ph.checksum = cs;
                buf[0..crate::abi::PAGE_HEADER_SIZE].copy_from_slice(&ph.serialize());

                self.page_cache.insert(new_phys_id, buf);
                self.dirty_pages.insert(new_phys_id);
                branch_page_map.insert(logical_page_id, new_phys_id);
                Ok(new_phys_id)
            }
        }
    }

    /// Register a new branch sharing existing pages with parent branch (increments ref counts).
    pub fn register_branch_cow(
        &mut self,
        parent_page_map: &HashMap<u32, u32>,
    ) -> HashMap<u32, u32> {
        let child_map = parent_page_map.clone();
        for phys_id in child_map.values() {
            let count = self.page_ref_counts.get(phys_id).copied().unwrap_or(0);
            self.page_ref_counts.insert(*phys_id, count + 1);
        }
        child_map
    }

    /// Sync in-flight dirty pages and commit generation seqlock header to disk container.
    pub fn sync(&mut self) -> LoomResult<()> {
        self.header.begin_write();

        // Flush dirty pages
        for page_id in self.dirty_pages.drain() {
            if let Some(buf) = self.page_cache.get(&page_id) {
                let offset = page_id as u64 * PAGE_SIZE as u64;
                self.file.seek(SeekFrom::Start(offset))?;
                self.file.write_all(buf)?;
            }
        }

        self.header.end_write();

        // Write header (offset 0)
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&self.header.serialize())?;
        self.file.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pager_creation_and_cow_branching() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("test_cow_{}.loom", ts));

        let mut pager = Pager::open_or_create(&path).unwrap();
        assert_eq!(pager.header.total_pages, 1);

        let mut main_pages = HashMap::new();
        // Allocate page for logical ID 1 in main branch
        let phys_1 = pager
            .write_page_cow(1, &mut main_pages, b"Initial data for main", PageType::BTreeLeaf)
            .unwrap();
        assert_eq!(phys_1, 1);

        // Fork branch 'experiment'
        let mut exp_pages = pager.register_branch_cow(&main_pages);
        assert_eq!(exp_pages.get(&1), Some(&1));
        assert_eq!(*pager.page_ref_counts.get(&1).unwrap(), 2);

        // Write to child branch: triggers Copy-on-Write!
        let phys_new = pager
            .write_page_cow(1, &mut exp_pages, b"Modified data in branch", PageType::BTreeLeaf)
            .unwrap();
        assert_ne!(phys_new, 1);
        assert_eq!(phys_new, 2); // Allocated new physical slot 2!

        // Main branch still points to physical page 1
        assert_eq!(main_pages.get(&1), Some(&1));
        // Experiment branch now points to physical page 2
        assert_eq!(exp_pages.get(&1), Some(&2));

        // Read both pages and assert bit-for-bit isolation
        let main_data = pager.read_page(1).unwrap();
        let exp_data = pager.read_page(2).unwrap();

        assert_eq!(
            &main_data[crate::abi::PAGE_HEADER_SIZE..crate::abi::PAGE_HEADER_SIZE + 21],
            b"Initial data for main"
        );
        assert_eq!(
            &exp_data[crate::abi::PAGE_HEADER_SIZE..crate::abi::PAGE_HEADER_SIZE + 23],
            b"Modified data in branch"
        );

        // Commit to file
        assert!(pager.sync().is_ok());
        assert!(pager.header.is_reader_view_valid());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_pager_budget_governor_enforcement() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("budget_test_{}.loom", ts));

        let mut pager = Pager::open_or_create(&path).unwrap();
        // Set hard RAM budget to allow at most 2 additional pages (8KB)
        pager.set_governor(BudgetGovernor::new(
            42 * 1024 + 2 * PAGE_SIZE as u64,
            4 * 1024 * 1024,
            256 * 1024,
            10,
        ));

        assert!(pager.allocate_page(PageType::BTreeLeaf).is_ok());
        assert!(pager.allocate_page(PageType::BTreeLeaf).is_ok());

        // Third page exceeds budget -> hard error!
        let err = pager.allocate_page(PageType::BTreeLeaf);
        assert!(err.is_err());
        match err.unwrap_err() {
            LoomError::BudgetExceeded { metric, .. } => assert_eq!(metric, "RAM"),
            other => panic!("Expected BudgetExceeded, got {:?}", other),
        }

        let _ = std::fs::remove_file(path);
    }
}
