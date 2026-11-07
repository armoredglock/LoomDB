use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Monotonically increasing sequence number for MVCC versions.
pub type SeqNum = u64;

/// Internal entry key encoding UserKey + SequenceNumber.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternalKey {
    pub user_key: Bytes,
    pub seq_num: SeqNum,
}

impl InternalKey {
    pub fn new(user_key: Bytes, seq_num: SeqNum) -> Self {
        Self { user_key, seq_num }
    }
}

/// Operation type tag recorded in the Write-Ahead Log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpType {
    Put = 1,
    Delete = 2,
    Merge = 3,
}

/// Pointer into the append-only Value Log for large values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValuePtr {
    pub vlog_id: u32,
    pub offset: u64,
    pub length: u32,
}
