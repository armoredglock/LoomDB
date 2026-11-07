use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Uuid,
    Json,
    Vector(usize),
    Boolean,
    Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub is_nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDef {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub ddl: String,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
}

use crate::cow_branch::BranchManager;
use crate::flight_recorder::FlightRecorder;
use crate::governor::BudgetGovernor;
use crate::telemetry::SystemTelemetry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCatalog {
    pub tables: HashMap<String, TableDef>,
    #[serde(default)]
    pub branches: BranchManager,
    #[serde(default)]
    pub governor: BudgetGovernor,
    #[serde(default)]
    pub telemetry: SystemTelemetry,
    #[serde(default)]
    pub flight_recorder: FlightRecorder,
}

impl DatabaseCatalog {
    pub fn new() -> Self {
        let mut cat = Self {
            tables: HashMap::new(),
            branches: BranchManager::new(),
            governor: BudgetGovernor::default(),
            telemetry: SystemTelemetry::new(),
            flight_recorder: FlightRecorder::new(),
        };
        // Initialize default sample multi-modal collection
        cat.create_table(
            "memories",
            vec![
                ColumnDef { name: "id".into(), data_type: DataType::Text, is_primary_key: true, is_unique: true, is_nullable: false },
                ColumnDef { name: "topic".into(), data_type: DataType::Text, is_primary_key: false, is_unique: false, is_nullable: false },
                ColumnDef { name: "notes".into(), data_type: DataType::Text, is_primary_key: false, is_unique: false, is_nullable: true },
                ColumnDef { name: "embedding".into(), data_type: DataType::Vector(4), is_primary_key: false, is_unique: false, is_nullable: true },
            ],
            "CREATE TABLE memories (id TEXT PRIMARY KEY, topic TEXT, notes TEXT, embedding VECTOR(4));".into()
        );

        let mut row1 = HashMap::new();
        row1.insert("id".into(), serde_json::json!("mem_01"));
        row1.insert("topic".into(), serde_json::json!("High-Performance Rust Storage"));
        row1.insert("notes".into(), serde_json::json!("Micro profile <512KB RAM"));
        row1.insert("embedding".into(), serde_json::json!([0.12, 0.45, -0.22, 0.81]));

        let mut row2 = HashMap::new();
        row2.insert("id".into(), serde_json::json!("mem_02"));
        row2.insert("topic".into(), serde_json::json!("Fused HNSW SIMD Traversal"));
        row2.insert("notes".into(), serde_json::json!("Single-pass filter search"));
        row2.insert("embedding".into(), serde_json::json!([0.10, 0.40, -0.20, 0.79]));

        if let Some(t) = cat.tables.get_mut("memories") {
            t.rows.push(row1);
            t.rows.push(row2);
        }

        cat
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<ColumnDef>, ddl: String) -> bool {
        let lower = name.to_lowercase();
        if self.tables.contains_key(&lower) {
            return false;
        }
        self.tables.insert(
            lower.clone(),
            TableDef {
                name: lower,
                columns,
                ddl,
                rows: Vec::new(),
            },
        );
        true
    }

    pub fn drop_table(&mut self, name: &str) -> bool {
        self.tables.remove(&name.to_lowercase()).is_some()
    }

    pub fn table_exists(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        lower == "_sys_budget" || lower == "_sys_telemetry" || lower == "_sys_branches" || self.tables.contains_key(&lower)
    }

    /// Dynamically query internal system tables
    pub fn get_system_table(&self, name: &str) -> Option<(Vec<ColumnDef>, Vec<HashMap<String, serde_json::Value>>)> {
        match name.to_lowercase().as_str() {
            "_sys_budget" => {
                let cols = vec![
                    ColumnDef { name: "metric".into(), data_type: DataType::Text, is_primary_key: true, is_unique: true, is_nullable: false },
                    ColumnDef { name: "used".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "cap".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "utilization_pct".into(), data_type: DataType::Text, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "status".into(), data_type: DataType::Text, is_primary_key: false, is_unique: false, is_nullable: false },
                ];
                let rows: Vec<HashMap<String, serde_json::Value>> = self.governor.system_rows()
                    .into_iter()
                    .map(|m| m.into_iter().collect())
                    .collect();
                Some((cols, rows))
            }
            "_sys_telemetry" => {
                let cols = vec![
                    ColumnDef { name: "timestamp_us".into(), data_type: DataType::Integer, is_primary_key: true, is_unique: false, is_nullable: false },
                    ColumnDef { name: "query_count".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "latency_p50_us".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "latency_p99_us".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "cache_hits".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "wal_bytes".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "idle_ram_kb".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                ];
                let rows: Vec<HashMap<String, serde_json::Value>> = self.telemetry.system_rows()
                    .into_iter()
                    .map(|m| m.into_iter().collect())
                    .collect();
                Some((cols, rows))
            }
            "_sys_branches" => {
                let cols = vec![
                    ColumnDef { name: "name".into(), data_type: DataType::Text, is_primary_key: true, is_unique: true, is_nullable: false },
                    ColumnDef { name: "parent".into(), data_type: DataType::Text, is_primary_key: false, is_unique: false, is_nullable: true },
                    ColumnDef { name: "is_active".into(), data_type: DataType::Boolean, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "table_count".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                    ColumnDef { name: "created_at_us".into(), data_type: DataType::Integer, is_primary_key: false, is_unique: false, is_nullable: false },
                ];
                let rows: Vec<HashMap<String, serde_json::Value>> = self.branches.list()
                    .into_iter()
                    .map(|b| {
                        let mut map = HashMap::new();
                        map.insert("name".into(), serde_json::json!(b.name));
                        map.insert("parent".into(), serde_json::json!(b.parent));
                        map.insert("is_active".into(), serde_json::json!(b.is_active));
                        map.insert("table_count".into(), serde_json::json!(b.table_count));
                        map.insert("created_at_us".into(), serde_json::json!(b.created_at_us));
                        map
                    })
                    .collect();
                Some((cols, rows))
            }
            _ => None,
        }
    }

    pub fn get_table(&self, name: &str) -> Option<&TableDef> {
        self.tables.get(&name.to_lowercase())
    }

    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut TableDef> {
        self.tables.get_mut(&name.to_lowercase())
    }

    pub fn list_tables(&self) -> Vec<(&str, usize, usize)> {
        let mut list: Vec<_> = self.tables
            .iter()
            .map(|(name, def)| (name.as_str(), def.columns.len(), def.rows.len()))
            .collect();
        list.sort_by_key(|a| a.0);
        list
    }

    pub fn insert_row(&mut self, table_name: &str, row: HashMap<String, serde_json::Value>) -> Result<(), String> {
        let table = self.tables.get_mut(&table_name.to_lowercase())
            .ok_or_else(|| format!("Table '{}' does not exist", table_name))?;
        table.rows.push(row);
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_vec(self)?;
        let mut file = File::create(p)?;

        // Write 128-byte Container Header (Offset 0x00 .. 0x80)
        let magic = b"LOOMDB\x01\x00";
        let version_seq = (self.flight_recorder.sequence * 2 + 2) as u64; // Even = stable seqlock
        let page_size = 4096u32;
        let cat_pages = ((json.len() + 32 + 4095) / 4096) as u32;
        let total_pages = 1 + cat_pages;
        let active_root_page = 0u32;
        let catalog_page = 1u32;
        let branch_dir_page = 0u32;
        let wal_seq = self.flight_recorder.sequence;
        let flags = 0u32;

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(magic);
        hasher.update(&version_seq.to_le_bytes());
        hasher.update(&page_size.to_le_bytes());
        hasher.update(&total_pages.to_le_bytes());
        hasher.update(&active_root_page.to_le_bytes());
        hasher.update(&catalog_page.to_le_bytes());
        hasher.update(&branch_dir_page.to_le_bytes());
        hasher.update(&wal_seq.to_le_bytes());
        hasher.update(&flags.to_le_bytes());
        let checksum = hasher.finalize();

        let mut header = [0u8; 128];
        header[0..8].copy_from_slice(magic);
        header[8..16].copy_from_slice(&version_seq.to_le_bytes());
        header[16..20].copy_from_slice(&page_size.to_le_bytes());
        header[20..24].copy_from_slice(&total_pages.to_le_bytes());
        header[24..28].copy_from_slice(&active_root_page.to_le_bytes());
        header[28..32].copy_from_slice(&catalog_page.to_le_bytes());
        header[32..36].copy_from_slice(&branch_dir_page.to_le_bytes());
        header[36..44].copy_from_slice(&wal_seq.to_le_bytes());
        header[44..48].copy_from_slice(&flags.to_le_bytes());
        header[48..52].copy_from_slice(&checksum.to_le_bytes());

        // Write Header page (Page 0: 4096 bytes)
        file.write_all(&header)?;
        let pad0 = vec![0u8; 4096 - 128];
        file.write_all(&pad0)?;

        // Write Catalog page (Page 1: 32-byte PageHeader + serialized catalog data)
        let mut page1_hdr = [0u8; 32];
        page1_hdr[0..4].copy_from_slice(&1u32.to_le_bytes()); // page_id = 1
        page1_hdr[4] = 2; // PageType::Catalog = 2
        page1_hdr[6..8].copy_from_slice(&(self.tables.len() as u16).to_le_bytes());
        file.write_all(&page1_hdr)?;
        file.write_all(&json)?;

        // Pad to end of page boundary
        let remainder = (32 + json.len()) % 4096;
        if remainder > 0 {
            let pad = vec![0u8; 4096 - remainder];
            file.write_all(&pad)?;
        }

        file.sync_all()?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let p = path.as_ref();
        if !p.exists() {
            let cat = Self::new();
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }
            cat.save_to_file(p)?;
            return Ok(cat);
        }
        let mut file = File::open(p)?;
        let mut magic = [0u8; 8];
        if file.read_exact(&mut magic).is_ok() && &magic == b"LOOMDB\x01\x00" {
            // Read from Page 1 (offset 4096)
            file.seek(SeekFrom::Start(4096 + 32))?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            // Trim trailing null padding
            while let Some(&0) = data.last() {
                data.pop();
            }
            if let Ok(cat) = serde_json::from_slice(&data) {
                return Ok(cat);
            }
        }

        // Fallback for plain json or legacy files
        file.seek(SeekFrom::Start(0))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let cat: Self = serde_json::from_slice(&data)
            .unwrap_or_else(|_| Self::new());
        Ok(cat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_table_creation() {
        let mut cat = DatabaseCatalog::new();
        assert!(cat.table_exists("memories"));

        cat.create_table(
            "users",
            vec![
                ColumnDef { name: "id".into(), data_type: DataType::Integer, is_primary_key: true, is_unique: true, is_nullable: false },
                ColumnDef { name: "name".into(), data_type: DataType::Text, is_primary_key: false, is_unique: false, is_nullable: false },
            ],
            "CREATE TABLE users (id INT, name TEXT);".into()
        );

        assert!(cat.table_exists("users"));
        assert_eq!(cat.get_table("users").unwrap().columns.len(), 2);
    }

    #[test]
    fn test_catalog_insert_row() {
        let mut cat = DatabaseCatalog::new();
        let mut row = HashMap::new();
        row.insert("id".into(), serde_json::json!("mem_99"));
        row.insert("topic".into(), serde_json::json!("Test Vector Storage"));

        assert!(cat.insert_row("memories", row).is_ok());
        assert_eq!(cat.get_table("memories").unwrap().rows.len(), 3);
    }
}
