use crate::ast::*;
use loom_core::catalog::{ColumnDef, DataType, DatabaseCatalog};
use loom_core::error::{LoomError, LoomResult};
use loom_index::vector::cosine_similarity;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub execution_time_ms: f64,
    pub message: Option<String>,
}

pub struct Executor;

impl Executor {
    pub fn execute(stmt: Statement, catalog: &mut DatabaseCatalog) -> LoomResult<QueryResult> {
        let start = std::time::Instant::now();

        let result = match stmt {
            Statement::Explain(inner) => {
                let plan_json = serde_json::to_string_pretty(&*inner).unwrap_or_default();
                let cols = vec![ColumnDef {
                    name: "QUERY PLAN".into(),
                    data_type: DataType::Text,
                    is_primary_key: false,
                    is_unique: false,
                    is_nullable: false,
                }];
                let mut row = HashMap::new();
                row.insert("QUERY PLAN".into(), serde_json::Value::String(plan_json));
                QueryResult {
                    columns: cols,
                    rows: vec![row],
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: Some("Query plan explained".into()),
                }
            }
            Statement::ForkBranch(name) => {
                catalog.branches.fork(&name).map_err(LoomError::Internal)?;
                catalog.flight_recorder.record(&format!("FORK BRANCH '{}'", name), 42, 0x101);
                QueryResult {
                    columns: vec![],
                    rows: vec![],
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: Some(format!("Branch '{}' active (COW page table allocated)", name)),
                }
            }
            Statement::CheckoutBranch(name) => {
                catalog.branches.checkout(&name).map_err(LoomError::Internal)?;
                QueryResult {
                    columns: vec![],
                    rows: vec![],
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: Some(format!("Switched to branch '{}'", name)),
                }
            }
            Statement::DiscardBranch(name) => {
                catalog.branches.discard(&name).map_err(LoomError::Internal)?;
                QueryResult {
                    columns: vec![],
                    rows: vec![],
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: Some(format!("Discarded branch '{}'", name)),
                }
            }
            Statement::CreateTable(c) => {
                let cols = c
                    .columns
                    .iter()
                    .map(|col| ColumnDef {
                        name: col.name.clone(),
                        data_type: DataType::Text,
                        is_primary_key: col.is_primary_key,
                        is_unique: col.is_primary_key,
                        is_nullable: !col.is_primary_key,
                    })
                    .collect();
                let ddl = format!("CREATE TABLE {} (...);", c.table);
                catalog.create_table(&c.table, cols, ddl);
                catalog.flight_recorder.record(&format!("CREATE TABLE {}", c.table), 42, 0x102);
                QueryResult {
                    columns: vec![],
                    rows: vec![],
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: Some(format!("Table '{}' created successfully", c.table)),
                }
            }
            Statement::DropTable(name) => {
                catalog.drop_table(&name);
                QueryResult {
                    columns: vec![],
                    rows: vec![],
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: Some(format!("Table '{}' dropped", name)),
                }
            }
            Statement::Insert(ins) => {
                let table_name = ins.table.to_lowercase();
                let mut row = HashMap::new();
                if let Some(t_def) = catalog.get_table(&table_name) {
                    for (i, val) in ins.values.into_iter().enumerate() {
                        let col_name = if i < ins.columns.len() {
                            ins.columns[i].clone()
                        } else if i < t_def.columns.len() {
                            t_def.columns[i].name.clone()
                        } else {
                            format!("col_{}", i)
                        };
                        row.insert(col_name, val);
                    }
                } else {
                    return Err(LoomError::Internal(format!("Table '{}' does not exist", ins.table)));
                }

                catalog.insert_row(&table_name, row).map_err(LoomError::Internal)?;
                catalog.flight_recorder.record(&format!("INSERT INTO {}", table_name), 42, 0x103);
                catalog.telemetry.record_query(start.elapsed().as_micros() as u32);

                QueryResult {
                    columns: vec![],
                    rows: vec![],
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: Some("1 row inserted".into()),
                }
            }
            Statement::Select(sel) => {
                let table_name = sel.table.to_lowercase();
                // Check if system table
                let (cols, source_rows) = if let Some(sys_res) = catalog.get_system_table(&table_name) {
                    sys_res
                } else if let Some(t_def) = catalog.get_table(&table_name) {
                    (t_def.columns.clone(), t_def.rows.clone())
                } else {
                    return Err(LoomError::Internal(format!("Table '{}' does not exist", sel.table)));
                };

                let mut filtered_rows = Vec::new();

                for mut row in source_rows {
                    // Check filter
                    if let Some(ref filter) = sel.filter {
                        match filter {
                            FilterExpr::Like(col, pattern) => {
                                let pat_clean = pattern.trim_matches('%');
                                let val = row.get(col).and_then(|v| v.as_str()).unwrap_or("");
                                if !val.contains(pat_clean) {
                                    continue;
                                }
                            }
                            FilterExpr::Eq(col, target_val) => {
                                if let Some(v) = row.get(col) {
                                    if v != target_val {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            _ => {}
                        }
                    }

                    // Compute vector projections if requested
                    for proj in &sel.projections {
                        if let SelectItem::VectorScore { field, target, alias } = proj {
                            if let Some(vec_val) = row.get(field) {
                                if let Ok(v) = serde_json::from_value::<Vec<f32>>(vec_val.clone()) {
                                    let score = cosine_similarity(&v, target);
                                    let rounded = (score * 10000.0).round() / 10000.0;
                                    row.insert(alias.clone(), serde_json::json!(rounded));
                                }
                            }
                        }
                    }

                    filtered_rows.push(row);
                }

                // Sorting
                if let Some(ref ob) = sel.order_by {
                    let col = ob.column.clone();
                    let asc = ob.ascending;
                    filtered_rows.sort_by(|a, b| {
                        let va = a.get(&col).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let vb = b.get(&col).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        if asc {
                            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
                        } else {
                            vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
                        }
                    });
                }

                // Limit
                if let Some(lim) = sel.limit {
                    filtered_rows.truncate(lim);
                }

                catalog.telemetry.record_query(start.elapsed().as_micros() as u32);

                QueryResult {
                    columns: cols,
                    rows: filtered_rows,
                    execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    message: None,
                }
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_execute_select_vector_and_filter() {
        let mut catalog = DatabaseCatalog::new();
        let sql = "SELECT id, topic, cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) AS score FROM memories WHERE topic LIKE '%Rust%' ORDER BY score DESC LIMIT 1;";
        let stmt = Parser::parse(sql).unwrap();
        let res = Executor::execute(stmt, &mut catalog).unwrap();

        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0].get("id").unwrap(), "mem_01");
        assert!(res.rows[0].contains_key("score"));
    }

    #[test]
    fn test_execute_fork_branch_and_query_sys_branches() {
        let mut catalog = DatabaseCatalog::new();
        let stmt = Parser::parse("FORK BRANCH 'feature_x';").unwrap();
        let res = Executor::execute(stmt, &mut catalog).unwrap();
        assert!(res.message.unwrap().contains("feature_x"));

        let q_stmt = Parser::parse("SELECT * FROM _sys_branches;").unwrap();
        let q_res = Executor::execute(q_stmt, &mut catalog).unwrap();
        assert_eq!(q_res.rows.len(), 2); // 'main' and 'feature_x'
    }
}
