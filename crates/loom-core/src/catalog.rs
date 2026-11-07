use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCatalog {
    pub tables: HashMap<String, TableDef>,
}

impl DatabaseCatalog {
    pub fn new() -> Self {
        let mut cat = Self {
            tables: HashMap::new(),
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
        self.tables.contains_key(&name.to_lowercase())
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
        let json = serde_json::to_vec_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(&json)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let p = path.as_ref();
        if !p.exists() {
            let cat = Self::new();
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent)?;
            }
            cat.save_to_file(p)?;
            return Ok(cat);
        }
        let mut file = File::open(p)?;
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
