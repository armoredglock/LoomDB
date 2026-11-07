use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub parent: Option<String>,
    pub created_at_us: u64,
    pub wal_seq: u64,
    pub table_count: usize,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchState {
    pub name: String,
    pub parent: Option<String>,
    pub created_at_us: u64,
    pub wal_seq: u64,
    /// Copy-on-Write page directory mapping logical table/page IDs to file offsets
    pub page_map: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchManager {
    pub active_branch: String,
    pub branches: HashMap<String, BranchState>,
}

impl Default for BranchManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BranchManager {
    pub fn new() -> Self {
        let mut branches = HashMap::new();
        branches.insert(
            "main".to_string(),
            BranchState {
                name: "main".to_string(),
                parent: None,
                created_at_us: 0,
                wal_seq: 0,
                page_map: HashMap::new(),
            },
        );

        Self {
            active_branch: "main".to_string(),
            branches,
        }
    }

    /// Fork a new branch from the active branch using Copy-on-Write page mapping.
    pub fn fork(&mut self, new_branch: &str) -> Result<(), String> {
        let branch_name = new_branch.trim().to_string();
        if branch_name.is_empty() {
            return Err("Branch name cannot be empty".into());
        }
        if self.branches.contains_key(&branch_name) {
            return Err(format!("Branch '{}' already exists", branch_name));
        }

        let parent_state = self
            .branches
            .get(&self.active_branch)
            .ok_or_else(|| format!("Active branch '{}' missing", self.active_branch))?;

        // COW snapshot: shallow clone the page table mapping with reference to parent
        let new_state = BranchState {
            name: branch_name.clone(),
            parent: Some(self.active_branch.clone()),
            created_at_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64,
            wal_seq: parent_state.wal_seq,
            page_map: parent_state.page_map.clone(),
        };

        self.branches.insert(branch_name, new_state);
        Ok(())
    }

    /// Checkout an existing branch.
    pub fn checkout(&mut self, branch_name: &str) -> Result<(), String> {
        let name = branch_name.trim();
        if !self.branches.contains_key(name) {
            return Err(format!("Branch '{}' does not exist", name));
        }
        self.active_branch = name.to_string();
        Ok(())
    }

    /// Discard a branch and free unshared COW pages.
    pub fn discard(&mut self, branch_name: &str) -> Result<(), String> {
        let name = branch_name.trim();
        if name == "main" {
            return Err("Cannot discard default 'main' branch".into());
        }
        if self.active_branch == name {
            return Err(format!("Cannot discard currently active branch '{}'. Checkout another branch first.", name));
        }
        if self.branches.remove(name).is_none() {
            return Err(format!("Branch '{}' not found", name));
        }
        Ok(())
    }

    /// Return an immutable listing of all registered snapshot branches.
    pub fn list(&self) -> Vec<BranchInfo> {
        let mut list: Vec<BranchInfo> = self
            .branches
            .values()
            .map(|b| BranchInfo {
                name: b.name.clone(),
                parent: b.parent.clone(),
                created_at_us: b.created_at_us,
                wal_seq: b.wal_seq,
                table_count: b.page_map.len(),
                is_active: b.name == self.active_branch,
            })
            .collect();

        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_branching_workflow() {
        let mut mgr = BranchManager::new();
        assert_eq!(mgr.active_branch, "main");

        // Fork snapshot
        assert!(mgr.fork("experiment_a").is_ok());
        assert_eq!(mgr.list().len(), 2);

        // Checkout snapshot
        assert!(mgr.checkout("experiment_a").is_ok());
        assert_eq!(mgr.active_branch, "experiment_a");

        // Switch back and discard
        assert!(mgr.checkout("main").is_ok());
        assert!(mgr.discard("experiment_a").is_ok());
        assert_eq!(mgr.list().len(), 1);
    }
}
