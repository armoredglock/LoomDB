use crate::error::{LoomError, LoomResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetGovernor {
    pub max_ram_bytes: u64,
    pub max_wal_bytes: u64,
    pub max_page_cache_bytes: u64,
    pub max_io_per_query: u64,

    pub current_ram_bytes: u64,
    pub current_wal_bytes: u64,
    pub current_io_count: u64,
}

impl Default for BudgetGovernor {
    fn default() -> Self {
        Self::new(512 * 1024, 4 * 1024 * 1024, 256 * 1024, 1000)
    }
}

impl BudgetGovernor {
    pub fn new(
        max_ram_bytes: u64,
        max_wal_bytes: u64,
        max_page_cache_bytes: u64,
        max_io_per_query: u64,
    ) -> Self {
        Self {
            max_ram_bytes,
            max_wal_bytes,
            max_page_cache_bytes,
            max_io_per_query,
            current_ram_bytes: 42 * 1024, // Baseline idle metadata footprint
            current_wal_bytes: 0,
            current_io_count: 0,
        }
    }

    /// Hard enforcement: verify requested allocation against maximum RAM cap.
    pub fn track_alloc(&mut self, bytes: u64) -> LoomResult<()> {
        let projected = self.current_ram_bytes.saturating_add(bytes);
        if projected > self.max_ram_bytes {
            return Err(LoomError::BudgetExceeded {
                metric: "RAM".to_string(),
                used: projected,
                cap: self.max_ram_bytes,
            });
        }
        self.current_ram_bytes = projected;
        Ok(())
    }

    pub fn track_free(&mut self, bytes: u64) {
        self.current_ram_bytes = self.current_ram_bytes.saturating_sub(bytes);
    }

    /// Hard enforcement: verify query I/O reads against per-query cap.
    pub fn track_io(&mut self, pages: u64) -> LoomResult<()> {
        let projected = self.current_io_count.saturating_add(pages);
        if projected > self.max_io_per_query {
            return Err(LoomError::BudgetExceeded {
                metric: "Query I/O pages".to_string(),
                used: projected,
                cap: self.max_io_per_query,
            });
        }
        self.current_io_count = projected;
        Ok(())
    }

    pub fn reset_query_io(&mut self) {
        self.current_io_count = 0;
    }

    /// Return system table rows for `_sys_budget`.
    pub fn system_rows(&self) -> Vec<serde_json::Map<String, serde_json::Value>> {
        let mut rows = Vec::new();

        let metrics = [
            ("max_ram_bytes", self.current_ram_bytes, self.max_ram_bytes),
            ("max_wal_bytes", self.current_wal_bytes, self.max_wal_bytes),
            ("max_page_cache_bytes", 64 * 1024, self.max_page_cache_bytes),
            ("max_io_per_query", self.current_io_count, self.max_io_per_query),
        ];

        for (metric, used, cap) in metrics {
            let mut row = serde_json::Map::new();
            row.insert("metric".into(), serde_json::Value::String(metric.into()));
            row.insert("used".into(), serde_json::Value::Number(used.into()));
            row.insert("cap".into(), serde_json::Value::Number(cap.into()));
            let pct = if cap > 0 { (used as f64 / cap as f64) * 100.0 } else { 0.0 };
            row.insert("utilization_pct".into(), serde_json::json!(format!("{:.1}%", pct)));
            let status = if used > cap { "THROTTLED" } else { "NORMAL" };
            row.insert("status".into(), serde_json::Value::String(status.into()));
            rows.push(row);
        }

        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_governor_hard_enforcement() {
        let mut gov = BudgetGovernor::new(100 * 1024, 1024, 1024, 50);
        // Allocation within budget passes
        assert!(gov.track_alloc(10 * 1024).is_ok());

        // Allocation exceeding hard budget fails with BudgetExceeded
        let err = gov.track_alloc(200 * 1024);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Hard resource budget exceeded"));
    }
}
