use bytes::{BufMut, BytesMut};
use crc32fast::Hasher;
use loom_core::error::{LoomError, LoomResult};
use loom_core::types::OpType;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;

/// Append-only Write-Ahead Log writer ensuring data durability before MemTable insertion.
pub struct WalWriter {
    file: Mutex<BufWriter<File>>,
}

impl WalWriter {
    pub fn create<P: AsRef<Path>>(path: P) -> LoomResult<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            file: Mutex::new(BufWriter::new(file)),
        })
    }

    /// Append a mutation record with CRC32 checksum.
    pub fn append(&self, op: OpType, key: &[u8], value: &[u8]) -> LoomResult<()> {
        let mut buf = BytesMut::with_capacity(1 + 2 + key.len() + 4 + value.len());
        buf.put_u8(op as u8);
        buf.put_u16_le(key.len() as u16);
        buf.put_slice(key);
        buf.put_u32_le(value.len() as u32);
        buf.put_slice(value);

        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let checksum = hasher.finalize();

        let mut writer = self.file.lock().map_err(|_| LoomError::Internal("WAL lock poisoned".into()))?;
        writer.write_all(&checksum.to_le_bytes())?;
        writer.write_all(&(buf.len() as u32).to_le_bytes())?;
        writer.write_all(&buf)?;
        Ok(())
    }

    /// Flush in-flight buffers to disk.
    pub fn sync(&self) -> LoomResult<()> {
        let mut writer = self.file.lock().map_err(|_| LoomError::Internal("WAL lock poisoned".into()))?;
        writer.flush()?;
        writer.get_ref().sync_data()?;
        Ok(())
    }
}
