use crc32fast::Hasher;
use loom_core::error::{LoomError, LoomResult};

pub const MAGIC_BYTES: &[u8; 8] = b"LOOMDB\x01\x00";
pub const HEADER_SIZE: usize = 128; // Offset 0x00 .. 0x80
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_HEADER_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    Header = 1,
    Catalog = 2,
    BTreeLeaf = 3,
    BTreeInterior = 4,
    VectorData = 5,
    TimeSeries = 6,
    GraphCSR = 7,
    Free = 8,
    Overflow = 9,
}

impl PageType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Header),
            2 => Some(Self::Catalog),
            3 => Some(Self::BTreeLeaf),
            4 => Some(Self::BTreeInterior),
            5 => Some(Self::VectorData),
            6 => Some(Self::TimeSeries),
            7 => Some(Self::GraphCSR),
            8 => Some(Self::Free),
            9 => Some(Self::Overflow),
            _ => None,
        }
    }
}

/// Generation Seqlock Container Header (Exact 128-byte ABI at offset 0x00 .. 0x80)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerHeader {
    pub magic: [u8; 8],
    /// Generation Seqlock counter: even = valid reader view, odd = writer actively mutating
    pub version_seq: u64,
    pub page_size: u32,
    pub total_pages: u32,
    pub active_root_page: u32,
    pub catalog_page: u32,
    pub branch_dir_page: u32,
    pub wal_seq: u64,
    pub flags: u32,
    pub checksum: u32,
}

impl ContainerHeader {
    pub fn new() -> Self {
        let mut h = Self {
            magic: *MAGIC_BYTES,
            version_seq: 2, // Initial even counter
            page_size: PAGE_SIZE as u32,
            total_pages: 1, // Header page (page 0)
            active_root_page: 0,
            catalog_page: 0,
            branch_dir_page: 0,
            wal_seq: 0,
            flags: 0,
            checksum: 0,
        };
        h.checksum = h.compute_checksum();
        h
    }

    pub fn compute_checksum(&self) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&self.magic);
        hasher.update(&self.version_seq.to_le_bytes());
        hasher.update(&self.page_size.to_le_bytes());
        hasher.update(&self.total_pages.to_le_bytes());
        hasher.update(&self.active_root_page.to_le_bytes());
        hasher.update(&self.catalog_page.to_le_bytes());
        hasher.update(&self.branch_dir_page.to_le_bytes());
        hasher.update(&self.wal_seq.to_le_bytes());
        hasher.update(&self.flags.to_le_bytes());
        hasher.finalize()
    }

    pub fn serialize(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..16].copy_from_slice(&self.version_seq.to_le_bytes());
        buf[16..20].copy_from_slice(&self.page_size.to_le_bytes());
        buf[20..24].copy_from_slice(&self.total_pages.to_le_bytes());
        buf[24..28].copy_from_slice(&self.active_root_page.to_le_bytes());
        buf[28..32].copy_from_slice(&self.catalog_page.to_le_bytes());
        buf[32..36].copy_from_slice(&self.branch_dir_page.to_le_bytes());
        buf[36..44].copy_from_slice(&self.wal_seq.to_le_bytes());
        buf[44..48].copy_from_slice(&self.flags.to_le_bytes());
        let cs = self.compute_checksum();
        buf[48..52].copy_from_slice(&cs.to_le_bytes());
        buf
    }

    pub fn deserialize(buf: &[u8]) -> LoomResult<Self> {
        if buf.len() < HEADER_SIZE {
            return Err(LoomError::CorruptedHeader("Buffer shorter than 128 bytes".into()));
        }
        if &buf[0..8] != MAGIC_BYTES {
            return Err(LoomError::CorruptedHeader("Invalid magic bytes in .loom container".into()));
        }

        let version_seq = u64::from_le_bytes(buf[8..16].try_into().unwrap());
        let page_size = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        let total_pages = u32::from_le_bytes(buf[20..24].try_into().unwrap());
        let active_root_page = u32::from_le_bytes(buf[24..28].try_into().unwrap());
        let catalog_page = u32::from_le_bytes(buf[28..32].try_into().unwrap());
        let branch_dir_page = u32::from_le_bytes(buf[32..36].try_into().unwrap());
        let wal_seq = u64::from_le_bytes(buf[36..44].try_into().unwrap());
        let flags = u32::from_le_bytes(buf[44..48].try_into().unwrap());
        let checksum = u32::from_le_bytes(buf[48..52].try_into().unwrap());

        let header = Self {
            magic: *MAGIC_BYTES,
            version_seq,
            page_size,
            total_pages,
            active_root_page,
            catalog_page,
            branch_dir_page,
            wal_seq,
            flags,
            checksum,
        };

        if header.compute_checksum() != checksum {
            return Err(LoomError::ChecksumMismatch("Container header CRC32 mismatch".into()));
        }

        Ok(header)
    }

    /// Reader seqlock check: returns true if generation is even (stable)
    pub fn is_reader_view_valid(&self) -> bool {
        self.version_seq % 2 == 0
    }

    /// Writer marks write beginning (odd sequence)
    pub fn begin_write(&mut self) {
        if self.version_seq % 2 == 0 {
            self.version_seq += 1;
        } else {
            self.version_seq += 2;
        }
        self.checksum = self.compute_checksum();
    }

    /// Writer marks write complete (even sequence)
    pub fn end_write(&mut self) {
        if self.version_seq % 2 != 0 {
            self.version_seq += 1;
        }
        self.checksum = self.compute_checksum();
    }
}

/// 32-Byte Binary Page Header embedded in every 4096-byte container page
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHeader {
    pub page_id: u32,
    pub page_type: PageType,
    pub flags: u8,
    pub item_count: u16,
    pub free_offset: u16,
    pub generation: u32,
    pub next_page_id: u32,
    pub prev_page_id: u32,
    pub checksum: u32,
}

impl PageHeader {
    pub fn new(page_id: u32, page_type: PageType) -> Self {
        Self {
            page_id,
            page_type,
            flags: 0,
            item_count: 0,
            free_offset: PAGE_HEADER_SIZE as u16,
            generation: 1,
            next_page_id: 0,
            prev_page_id: 0,
            checksum: 0,
        }
    }

    pub fn serialize(&self) -> [u8; PAGE_HEADER_SIZE] {
        let mut buf = [0u8; PAGE_HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.page_id.to_le_bytes());
        buf[4] = self.page_type as u8;
        buf[5] = self.flags;
        buf[6..8].copy_from_slice(&self.item_count.to_le_bytes());
        buf[8..10].copy_from_slice(&self.free_offset.to_le_bytes());
        buf[10..14].copy_from_slice(&self.generation.to_le_bytes());
        buf[14..18].copy_from_slice(&self.next_page_id.to_le_bytes());
        buf[18..22].copy_from_slice(&self.prev_page_id.to_le_bytes());
        buf[22..26].copy_from_slice(&self.checksum.to_le_bytes());
        buf
    }

    pub fn deserialize(buf: &[u8]) -> LoomResult<Self> {
        if buf.len() < PAGE_HEADER_SIZE {
            return Err(LoomError::CorruptedHeader("Page header buffer too small".into()));
        }
        let page_id = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let page_type = PageType::from_u8(buf[4]).ok_or_else(|| {
            LoomError::CorruptedHeader(format!("Unknown page type byte: {}", buf[4]))
        })?;
        let flags = buf[5];
        let item_count = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let free_offset = u16::from_le_bytes(buf[8..10].try_into().unwrap());
        let generation = u32::from_le_bytes(buf[10..14].try_into().unwrap());
        let next_page_id = u32::from_le_bytes(buf[14..18].try_into().unwrap());
        let prev_page_id = u32::from_le_bytes(buf[18..22].try_into().unwrap());
        let checksum = u32::from_le_bytes(buf[22..26].try_into().unwrap());

        Ok(Self {
            page_id,
            page_type,
            flags,
            item_count,
            free_offset,
            generation,
            next_page_id,
            prev_page_id,
            checksum,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_header_seqlock_abi() {
        let mut header = ContainerHeader::new();
        assert!(header.is_reader_view_valid());

        header.begin_write();
        assert!(!header.is_reader_view_valid());
        assert_eq!(header.version_seq % 2, 1);

        header.end_write();
        assert!(header.is_reader_view_valid());
        assert_eq!(header.version_seq % 2, 0);

        let bytes = header.serialize();
        assert_eq!(bytes.len(), 128);

        let recovered = ContainerHeader::deserialize(&bytes).unwrap();
        assert_eq!(recovered, header);
    }

    #[test]
    fn test_page_header_roundtrip() {
        let ph = PageHeader::new(42, PageType::BTreeLeaf);
        let bytes = ph.serialize();
        let recovered = PageHeader::deserialize(&bytes).unwrap();
        assert_eq!(recovered, ph);
    }
}
