use crate::ast::*;
use loom_core::error::{LoomError, LoomResult};

fn split_comma_top_level(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for c in s.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

pub struct Parser;

impl Parser {
    pub fn parse(sql: &str) -> LoomResult<Statement> {
        let trimmed = sql.trim().trim_end_matches(';').trim();
        let upper = trimmed.to_uppercase();

        if upper.starts_with("EXPLAIN") {
            let inner_sql = trimmed[7..].trim();
            let inner_stmt = Self::parse(inner_sql)?;
            return Ok(Statement::Explain(Box::new(inner_stmt)));
        }

        if upper.starts_with("FORK BRANCH") || upper.starts_with("FORK") {
            let branch = trimmed
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_matches('\'')
                .trim_matches('"');
            return Ok(Statement::ForkBranch(branch.to_string()));
        }

        if upper.starts_with("CHECKOUT") {
            let branch = trimmed
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_matches('\'')
                .trim_matches('"');
            return Ok(Statement::CheckoutBranch(branch.to_string()));
        }

        if upper.starts_with("DISCARD") {
            let branch = trimmed
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_matches('\'')
                .trim_matches('"');
            return Ok(Statement::DiscardBranch(branch.to_string()));
        }

        if upper.starts_with("DROP TABLE") {
            let name = trimmed
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_matches('\'')
                .trim_matches('"');
            return Ok(Statement::DropTable(name.to_string()));
        }

        if upper.starts_with("CREATE TABLE") {
            return Self::parse_create_table(trimmed);
        }

        if upper.starts_with("INSERT INTO") {
            return Self::parse_insert(trimmed);
        }

        if upper.starts_with("SELECT") {
            return Self::parse_select(trimmed);
        }

        Err(LoomError::ParseError(format!("Unsupported SQL statement: {}", sql)))
    }

    fn parse_create_table(sql: &str) -> LoomResult<Statement> {
        let paren_start = sql.find('(').ok_or_else(|| LoomError::ParseError("Missing '(' in CREATE TABLE".into()))?;
        let paren_end = sql.rfind(')').ok_or_else(|| LoomError::ParseError("Missing ')' in CREATE TABLE".into()))?;

        let header = sql[..paren_start].trim();
        let table_name = header.split_whitespace().nth(2).ok_or_else(|| LoomError::ParseError("Missing table name in CREATE TABLE".into()))?;

        let body = &sql[paren_start + 1..paren_end];
        let mut columns = Vec::new();

        for col_def in body.split(',') {
            let parts: Vec<&str> = col_def.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            let name = parts[0].to_string();
            let data_type = parts.get(1).unwrap_or(&"TEXT").to_string();
            let is_primary_key = col_def.to_uppercase().contains("PRIMARY KEY");
            columns.push(ColumnDefinition {
                name,
                data_type,
                is_primary_key,
            });
        }

        Ok(Statement::CreateTable(CreateTableStatement {
            table: table_name.to_string(),
            columns,
        }))
    }

    fn parse_insert(sql: &str) -> LoomResult<Statement> {
        let upper = sql.to_uppercase();
        let values_idx = upper.find("VALUES").ok_or_else(|| LoomError::ParseError("Missing VALUES clause in INSERT".into()))?;

        let prefix = sql[..values_idx].trim();
        let suffix = sql[values_idx + 6..].trim();

        // Extract table name and optional column names
        let table_part = prefix.strip_prefix("INSERT INTO").or_else(|| prefix.strip_prefix("insert into")).unwrap_or(prefix).trim();
        let (table_name, columns) = if let Some(col_start) = table_part.find('(') {
            let col_end = table_part.find(')').ok_or_else(|| LoomError::ParseError("Unclosed '(' in column list".into()))?;
            let name = table_part[..col_start].trim();
            let cols = table_part[col_start + 1..col_end]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            (name, cols)
        } else {
            (table_part.trim(), Vec::new())
        };

        // Extract values inside parenthesis
        let val_start = suffix.find('(').ok_or_else(|| LoomError::ParseError("Missing '(' in VALUES".into()))?;
        let val_end = suffix.rfind(')').ok_or_else(|| LoomError::ParseError("Missing ')' in VALUES".into()))?;
        let val_body = &suffix[val_start + 1..val_end];

        let mut values = Vec::new();
        for val_str in split_comma_top_level(val_body) {
            let v = val_str.trim();
            if (v.starts_with('\'') && v.ends_with('\'')) || (v.starts_with('"') && v.ends_with('"')) {
                values.push(serde_json::Value::String(v[1..v.len() - 1].to_string()));
            } else if let Ok(n) = v.parse::<i64>() {
                values.push(serde_json::Value::Number(n.into()));
            } else if let Ok(f) = v.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(f) {
                    values.push(serde_json::Value::Number(num));
                } else {
                    values.push(serde_json::Value::Null);
                }
            } else if v.eq_ignore_ascii_case("true") {
                values.push(serde_json::Value::Bool(true));
            } else if v.eq_ignore_ascii_case("false") {
                values.push(serde_json::Value::Bool(false));
            } else if let Ok(json_arr) = serde_json::from_str::<serde_json::Value>(v) {
                values.push(json_arr);
            } else {
                values.push(serde_json::Value::String(v.to_string()));
            }
        }

        Ok(Statement::Insert(InsertStatement {
            table: table_name.to_string(),
            columns,
            values,
        }))
    }

    fn parse_select(sql: &str) -> LoomResult<Statement> {
        let upper = sql.to_uppercase();
        let from_idx = upper.find("FROM").ok_or_else(|| LoomError::ParseError("Missing FROM in SELECT".into()))?;

        let proj_str = sql[6..from_idx].trim();
        let mut projections = Vec::new();

        if proj_str == "*" {
            projections.push(SelectItem::Wildcard);
        } else {
            // Check for vector distance function: cosine_similarity(col, [vec]) AS alias
            for item in split_comma_top_level(proj_str) {
                let item = item.trim();
                let item_upper = item.to_uppercase();
                if item_upper.starts_with("COSINE_SIMILARITY") {
                    if let Some(start_vec) = item.find('[') {
                        if let Some(end_vec) = item.find(']') {
                            let vec_str = &item[start_vec..=end_vec];
                            let target_vec: Vec<f32> = serde_json::from_str(vec_str).unwrap_or_default();
                            let alias = item.split_whitespace().last().unwrap_or("score").to_string();
                            projections.push(SelectItem::VectorScore {
                                field: "embedding".into(),
                                target: target_vec,
                                alias,
                            });
                            continue;
                        }
                    }
                }
                projections.push(SelectItem::Column(item.to_string()));
            }
        }

        let after_from = &sql[from_idx + 4..].trim();
        let mut parts = after_from.split_whitespace();
        let table_name = parts.next().unwrap_or("").trim_matches(';').to_string();

        let mut filter = None;
        let mut order_by = None;
        let mut limit = None;

        // Extract WHERE
        if let Some(where_idx) = upper.find("WHERE") {
            let where_part = &sql[where_idx + 5..];
            let end_filter = where_part.to_uppercase().find("ORDER BY").or_else(|| where_part.to_uppercase().find("LIMIT")).unwrap_or(where_part.len());
            let cond_str = where_part[..end_filter].trim();

            if cond_str.to_uppercase().contains("LIKE") {
                let parts: Vec<&str> = cond_str.split_whitespace().collect();
                if parts.len() >= 3 {
                    let col = parts[0].to_string();
                    let pattern = parts[2].trim_matches('\'').trim_matches('"').to_string();
                    filter = Some(FilterExpr::Like(col, pattern));
                }
            } else if cond_str.contains('=') {
                let parts: Vec<&str> = cond_str.split('=').collect();
                if parts.len() == 2 {
                    let col = parts[0].trim().to_string();
                    let val = parts[1].trim().trim_matches('\'').trim_matches('"');
                    filter = Some(FilterExpr::Eq(col, serde_json::Value::String(val.to_string())));
                }
            }
        }

        // Extract ORDER BY
        if let Some(order_idx) = upper.find("ORDER BY") {
            let order_part = &sql[order_idx + 8..];
            let end_order = order_part.to_uppercase().find("LIMIT").unwrap_or(order_part.len());
            let clause = order_part[..end_order].trim();
            let mut words = clause.split_whitespace();
            if let Some(col) = words.next() {
                let asc = words.next().map(|w| !w.eq_ignore_ascii_case("DESC")).unwrap_or(true);
                order_by = Some(OrderByExpr {
                    column: col.to_string(),
                    ascending: asc,
                });
            }
        }

        // Extract LIMIT
        if let Some(limit_idx) = upper.find("LIMIT") {
            let limit_part = sql[limit_idx + 5..].trim().trim_matches(';');
            if let Ok(n) = limit_part.split_whitespace().next().unwrap_or("").parse::<usize>() {
                limit = Some(n);
            }
        }

        Ok(Statement::Select(SelectStatement {
            table: table_name,
            projections,
            filter,
            order_by,
            limit,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_table() {
        let stmt = Parser::parse("CREATE TABLE users (id INT PRIMARY KEY, name TEXT);").unwrap();
        match stmt {
            Statement::CreateTable(c) => {
                assert_eq!(c.table, "users");
                assert_eq!(c.columns.len(), 2);
                assert!(c.columns[0].is_primary_key);
            }
            _ => panic!("Expected CreateTable"),
        }
    }

    #[test]
    fn test_parse_select_with_vector_and_limit() {
        let sql = "SELECT id, topic, cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) AS score FROM memories WHERE topic LIKE '%Rust%' ORDER BY score DESC LIMIT 2;";
        let stmt = Parser::parse(sql).unwrap();
        match stmt {
            Statement::Select(s) => {
                assert_eq!(s.table, "memories");
                assert_eq!(s.limit, Some(2));
                assert!(s.filter.is_some());
                assert!(s.order_by.is_some());
            }
            _ => panic!("Expected Select"),
        }
    }
}
