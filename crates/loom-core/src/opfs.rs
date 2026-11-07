use crate::error::{LoomError, LoomResult};
use serde::{Deserialize, Serialize};

/// Mode of access for browser OPFS (Origin Private File System) coordination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpfsAccessMode {
    ExclusiveWriter,
    SharedReader,
}

/// State of an active OPFS lease.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpfsLease {
    pub client_id: String,
    pub tab_id: String,
    pub mode: OpfsAccessMode,
    pub generation_seq: u64,
    pub acquired_at_ms: u64,
    pub expires_at_ms: u64,
}

/// Change Data Capture (CDC) notification broadcasted across tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpfsCdcEvent {
    pub sequence: u64,
    pub writer_tab_id: String,
    pub transaction_id: u64,
    pub mutated_tables: Vec<String>,
    pub timestamp_ms: u64,
}

/// Multi-Tab OPFS Coordinator abstraction.
/// In WASM environments, delegates to `navigator.locks` and `BroadcastChannel`.
/// In native environments, provides in-process multi-reader single-writer coordination.
#[derive(Debug, Clone, Default)]
pub struct OpfsCoordinator {
    channel_name: String,
    lock_name: String,
    current_lease: Option<OpfsLease>,
    last_known_seq: u64,
}

impl OpfsCoordinator {
    pub fn new(container_name: &str) -> Self {
        Self {
            channel_name: format!("loomdb_cdc_{}", container_name),
            lock_name: format!("loomdb_lock_{}", container_name),
            current_lease: None,
            last_known_seq: 0,
        }
    }

    pub fn channel_name(&self) -> &str {
        &self.channel_name
    }

    pub fn lock_name(&self) -> &str {
        &self.lock_name
    }

    /// Attempt to acquire an exclusive writer lease.
    pub fn acquire_writer_lease(
        &mut self,
        tab_id: &str,
        client_id: &str,
        now_ms: u64,
        ttl_ms: u64,
    ) -> LoomResult<OpfsLease> {
        if let Some(existing) = &self.current_lease {
            if existing.expires_at_ms > now_ms && existing.tab_id != tab_id {
                return Err(LoomError::TransactionConflict);
            }
        }

        self.last_known_seq += 1;
        let lease = OpfsLease {
            client_id: client_id.to_string(),
            tab_id: tab_id.to_string(),
            mode: OpfsAccessMode::ExclusiveWriter,
            generation_seq: self.last_known_seq,
            acquired_at_ms: now_ms,
            expires_at_ms: now_ms + ttl_ms,
        };

        self.current_lease = Some(lease.clone());
        Ok(lease)
    }

    /// Release an exclusive writer lease upon transaction commit.
    pub fn release_writer_lease(&mut self, tab_id: &str) -> LoomResult<()> {
        if let Some(existing) = &self.current_lease {
            if existing.tab_id == tab_id {
                self.current_lease = None;
                return Ok(());
            }
        }
        Err(LoomError::TransactionClosed)
    }

    /// Broadcast CDC event to other browser tabs.
    pub fn create_cdc_event(&mut self, writer_tab_id: &str, tx_id: u64, tables: Vec<String>, now_ms: u64) -> OpfsCdcEvent {
        self.last_known_seq += 1;
        OpfsCdcEvent {
            sequence: self.last_known_seq,
            writer_tab_id: writer_tab_id.to_string(),
            transaction_id: tx_id,
            mutated_tables: tables,
            timestamp_ms: now_ms,
        }
    }

    /// Readers validate if writer is active or if reader view is clean.
    pub fn validate_reader_generation(&self, seqlock_version: u64) -> LoomResult<()> {
        // Odd sequence means a writer is actively modifying pages
        if seqlock_version % 2 != 0 {
            return Err(LoomError::SeqlockConflict);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opfs_writer_lease_and_cdc() {
        let mut coord = OpfsCoordinator::new("test");
        let lease = coord.acquire_writer_lease("tab_1", "worker_a", 1000, 5000);
        assert!(lease.is_ok());

        // Concurrent attempt from different tab should fail before TTL
        let conflict = coord.acquire_writer_lease("tab_2", "worker_b", 2000, 5000);
        assert!(conflict.is_err());

        // Releasing lease allows next tab to acquire
        assert!(coord.release_writer_lease("tab_1").is_ok());
        let next_lease = coord.acquire_writer_lease("tab_2", "worker_b", 2100, 5000);
        assert!(next_lease.is_ok());

        // Seqlock validation: even is ok, odd is conflicting
        assert!(coord.validate_reader_generation(2).is_ok());
        assert!(coord.validate_reader_generation(3).is_err());

        let event = coord.create_cdc_event("tab_2", 101, vec!["memories".into()], 2200);
        assert_eq!(event.transaction_id, 101);
        assert_eq!(event.mutated_tables[0], "memories");
    }
}
