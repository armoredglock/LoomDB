use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightRecord {
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub seed: u64,
    pub statement: String,
    pub state_hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlightRecorder {
    pub sequence: u64,
    pub records: Vec<FlightRecord>,
}

impl FlightRecorder {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            records: Vec::new(),
        }
    }

    /// Record a mutation with deterministic execution entropy (seed & clock).
    pub fn record(&mut self, statement: &str, seed: u64, state_hash: u64) -> u64 {
        self.sequence += 1;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let entry = FlightRecord {
            sequence: self.sequence,
            timestamp_ns: ts,
            seed,
            statement: statement.trim().to_string(),
            state_hash,
        };

        self.records.push(entry);
        self.sequence
    }

    /// Replay the recorded stream bit-for-bit to verify deterministic state integrity.
    pub fn verify_replay<F>(&self, mut apply_fn: F) -> Result<usize, String>
    where
        F: FnMut(&FlightRecord) -> Result<u64, String>,
    {
        for (i, rec) in self.records.iter().enumerate() {
            let resulting_hash = apply_fn(rec)?;
            if resulting_hash != rec.state_hash {
                return Err(format!(
                    "Non-deterministic replay diverge at sequence #{}: expected state hash 0x{:016x}, got 0x{:016x}",
                    rec.sequence, rec.state_hash, resulting_hash
                ));
            }
            if i % 1000 == 0 {
                // Yield/continue
            }
        }
        Ok(self.records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_recorder_deterministic_replay() {
        let mut rec = FlightRecorder::new();
        rec.record("INSERT INTO t VALUES (1)", 42, 0x1234);
        rec.record("INSERT INTO t VALUES (2)", 43, 0x5678);

        let replayed = rec.verify_replay(|item| {
            if item.sequence == 1 {
                assert_eq!(item.seed, 42);
                Ok(0x1234)
            } else {
                assert_eq!(item.seed, 43);
                Ok(0x5678)
            }
        });
        assert_eq!(replayed.unwrap(), 2);
    }
}
