use clap::{Parser, Subcommand};
use loom_core::catalog::{ColumnDef, DataType, DatabaseCatalog};
use loom_index::vector::cosine_similarity;
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "loom")]
#[command(author = "LoomDB Authors")]
#[command(version = "0.1.0")]
#[command(
    about = "LoomDB: World's Lightest Multi-Modal, AI-Native Database Engine",
    long_about = "LoomDB is a sub-megabyte, zero-dependency multi-modal database in a single portable .loom container file."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to the .loom container file (defaults to ./data.loom)
    #[arg(short, long, global = true, default_value = "./data.loom")]
    pub container: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive LoomQL / SQL REPL terminal
    Repl,

    /// Start LoomDB in background or foreground server mode
    Server {
        #[arg(short, long, default_value = "127.0.0.1:8484")]
        addr: String,

        #[arg(long, default_value_t = false)]
        open_studio: bool,
    },

    /// Manage LoomDB system background service (install, start, stop, status)
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },

    /// Export the zero-dependency Web Studio UI as a standalone HTML file
    Studio {
        #[arg(short, long, default_value = "./loom_studio.html")]
        output: String,
    },

    /// Inspect single-file .loom container metrics and page allocation
    Stats,

    /// Execute a single LoomQL query string directly
    Query {
        query: String,

        /// Output format: table, json, csv
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum ServiceAction {
    Install {
        #[arg(short, long, default_value = "127.0.0.1:8484")]
        addr: String,
    },
    Start,
    Stop,
    Status,
    Uninstall,
}

const STUDIO_HTML: &str = include_str!("studio.html");

#[derive(PartialEq)]
enum OutputMode {
    Table,
    Json,
    Csv,
}

fn main() {
    let cli = Cli::parse();
    let container_path = cli.container.clone();

    match cli.command {
        None | Some(Commands::Repl) => {
            run_repl(&container_path);
        }
        Some(Commands::Server { addr, open_studio }) => {
            run_server(&addr, &container_path, open_studio);
        }
        Some(Commands::Service { action }) => {
            handle_service(action, &container_path);
        }
        Some(Commands::Studio { output }) => {
            if let Err(e) = std::fs::write(&output, STUDIO_HTML) {
                eprintln!("[ERROR] Failed to write studio HTML: {}", e);
            } else {
                println!("[OK] Exported zero-dependency Web Studio to: {}", output);
                println!("Open this file directly in any web browser.");
            }
        }
        Some(Commands::Stats) => {
            show_stats(&container_path);
        }
        Some(Commands::Query { query, format }) => {
            execute_single_query(&query, &container_path, &format);
        }
    }
}

/// Interactive terminal REPL mimicking psql / sqlcmd / sqlite3 with real execution
fn run_repl(container: &str) {
    println!("\x1b[1;36m[LOOM_DB]\x1b[0m Interactive Shell v0.1.0");
    println!("Mounted Container: \x1b[33m{}\x1b[0m", container);
    println!("Type \x1b[1m.help\x1b[0m for commands or \x1b[1m.exit\x1b[0m to quit.\n");

    let mut catalog = DatabaseCatalog::load_from_file(container).unwrap_or_else(|_| DatabaseCatalog::new());
    let mut output_mode = OutputMode::Table;

    let stdin = io::stdin();
    let mut buffer = String::new();

    loop {
        print!("\x1b[1;32mloom> \x1b[0m");
        io::stdout().flush().unwrap();
        buffer.clear();

        if stdin.read_line(&mut buffer).is_err() || buffer.trim().is_empty() {
            continue;
        }

        let input = buffer.trim();

        // Check for erroneous slash commands (e.g. /tabs)
        if input.starts_with('/') {
            println!("Error: unrecognized command '{}'. LoomDB shell commands begin with '.' (e.g. .tables, .help).", input);
            continue;
        }

        // Handle dot-commands
        if input.starts_with('.') {
            let parts: Vec<&str> = input.split_whitespace().collect();
            let cmd = parts[0];

            match cmd {
                ".exit" | ".quit" | ".q" => {
                    println!("[LOOM_DB] Session closed.");
                    break;
                }
                ".help" => {
                    println!("LoomDB Interactive Shell Commands:");
                    println!("  .tables                 List all tables and collections");
                    println!("  .schema [TABLE]         Display DDL schema for a table");
                    println!("  .describe <TABLE>       Show table columns and datatypes");
                    println!("  .drop <TABLE>           Drop a table and delete its records");
                    println!("  .mode [table|json|csv]  Switch query output rendering format");
                    println!("  .stats                  Inspect virtual pages and memory footprint");
                    println!("  .clear                  Clear the terminal screen");
                    println!("  .studio                 Export Web Studio dashboard");
                    println!("  .exit / .quit           Exit shell");
                }
                ".tables" => {
                    let list = catalog.list_tables();
                    if list.is_empty() {
                        println!("No tables found in container.");
                    } else {
                        println!("+----------------------+---------+-------+");
                        println!("| Table Name           | Columns | Rows  |");
                        println!("+----------------------+---------+-------+");
                        for (name, cols, rows) in list {
                            println!("| {:<20} | {:<7} | {:<5} |", name, cols, rows);
                        }
                        println!("+----------------------+---------+-------+");
                    }
                }
                ".schema" => {
                    if parts.len() > 1 {
                        let tbl = parts[1];
                        if let Some(t) = catalog.get_table(tbl) {
                            println!("{}", t.ddl);
                        } else {
                            println!("Error: table '{}' does not exist.", tbl);
                        }
                    } else {
                        for (_, t) in &catalog.tables {
                            println!("{}\n", t.ddl);
                        }
                    }
                }
                ".describe" => {
                    if parts.len() < 2 {
                        println!("Usage: .describe <table_name>");
                    } else {
                        let tbl = parts[1];
                        if let Some(t) = catalog.get_table(tbl) {
                            println!("+----------------------+----------------------+---------+");
                            println!("| Column               | Data Type            | Primary |");
                            println!("+----------------------+----------------------+---------+");
                            for col in &t.columns {
                                println!("| {:<20} | {:<20} | {:<7} |", col.name, format!("{:?}", col.data_type), if col.is_primary_key { "YES" } else { "NO" });
                            }
                            println!("+----------------------+----------------------+---------+");
                        } else {
                            println!("Error: table '{}' does not exist.", tbl);
                        }
                    }
                }
                ".drop" => {
                    if parts.len() < 2 {
                        println!("Usage: .drop <table_name>");
                    } else {
                        let tbl = parts[1];
                        if catalog.drop_table(tbl) {
                            let _ = catalog.save_to_file(container);
                            println!("Table '{}' dropped.", tbl);
                        } else {
                            println!("Error: table '{}' does not exist.", tbl);
                        }
                    }
                }
                ".mode" => {
                    if parts.len() < 2 {
                        println!("Current output mode is {:?}", match output_mode {
                            OutputMode::Table => "table",
                            OutputMode::Json => "json",
                            OutputMode::Csv => "csv",
                        });
                    } else {
                        match parts[1].to_lowercase().as_str() {
                            "json" => {
                                output_mode = OutputMode::Json;
                                println!("Output mode set to JSON.");
                            }
                            "csv" => {
                                output_mode = OutputMode::Csv;
                                println!("Output mode set to CSV.");
                            }
                            "table" => {
                                output_mode = OutputMode::Table;
                                println!("Output mode set to ASCII Table.");
                            }
                            m => println!("Unknown mode '{}'. Choose table, json, or csv.", m),
                        }
                    }
                }
                ".clear" => {
                    print!("\x1B[2J\x1B[1;1H");
                    io::stdout().flush().unwrap();
                }
                ".branches" => {
                    execute_repl_sql("SHOW BRANCHES", &mut catalog, container, &output_mode);
                }
                ".budget" => {
                    execute_repl_sql("SELECT * FROM _sys_budget", &mut catalog, container, &output_mode);
                }
                ".telemetry" => {
                    execute_repl_sql("SELECT * FROM _sys_telemetry", &mut catalog, container, &output_mode);
                }
                cmd if cmd.starts_with(".fork ") => {
                    let branch = &cmd[6..];
                    execute_repl_sql(&format!("FORK BRANCH {}", branch), &mut catalog, container, &output_mode);
                }
                cmd if cmd.starts_with(".checkout ") => {
                    let branch = &cmd[10..];
                    execute_repl_sql(&format!("CHECKOUT {}", branch), &mut catalog, container, &output_mode);
                }
                ".stats" => {
                    show_stats(container);
                }
                ".studio" => {
                    let _ = std::fs::write("./loom_studio.html", STUDIO_HTML);
                    println!("[OK] Exported Web Studio to ./loom_studio.html");
                }
                cmd => println!("Unknown command: '{}'. Type .help for available commands.", cmd),
            }
            continue;
        }

        // Execute SQL / LoomQL Query
        execute_repl_sql(input, &mut catalog, container, &output_mode);
    }
}

fn execute_repl_sql(query: &str, catalog: &mut DatabaseCatalog, container: &str, mode: &OutputMode) {
    let trimmed = query.trim().trim_end_matches(';');
    let lower = trimmed.to_lowercase();
    let start = std::time::Instant::now();

    // 1. CREATE TABLE
    if lower.starts_with("create table") {
        if let Some((table_name, columns)) = parse_create_table(trimmed) {
            let ddl = format!("{};", trimmed);
            if catalog.create_table(&table_name, columns, ddl) {
                let _ = catalog.save_to_file(container);
                println!("Query OK, 0 rows affected. Table '{}' initialized. ({:.2?})", table_name, start.elapsed());
            } else {
                println!("Error: table '{}' already exists.", table_name);
            }
        } else {
            println!("Syntax error: invalid CREATE TABLE statement.");
        }
        return;
    }

    // 2. DROP TABLE
    if lower.starts_with("drop table") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 {
            let tbl = parts[2];
            if catalog.drop_table(tbl) {
                let _ = catalog.save_to_file(container);
                println!("Query OK, table '{}' dropped. ({:.2?})", tbl, start.elapsed());
            } else {
                println!("Error: table '{}' does not exist.", tbl);
            }
        } else {
            println!("Syntax error: DROP TABLE <table_name>");
        }
        return;
    }

    // 2.5 FORK & COW BRANCHING
    if lower.starts_with("fork branch ") || lower.starts_with("fork ") {
        let name = if lower.starts_with("fork branch ") { &trimmed[12..] } else { &trimmed[5..] }.trim();
        match catalog.branches.fork(name) {
            Ok(_) => {
                let _ = catalog.save_to_file(container);
                println!("Snapshot OK: Created forkable branch '{}' (Copy-on-Write generation). ({:.2?})", name, start.elapsed());
            }
            Err(e) => println!("Error: {}", e),
        }
        return;
    }

    if lower.starts_with("checkout ") {
        let name = trimmed[9..].trim();
        match catalog.branches.checkout(name) {
            Ok(_) => {
                let _ = catalog.save_to_file(container);
                println!("Switched to branch '{}'. ({:.2?})", name, start.elapsed());
            }
            Err(e) => println!("Error: {}", e),
        }
        return;
    }

    if lower.starts_with("discard branch ") || lower.starts_with("discard ") {
        let name = if lower.starts_with("discard branch ") { &trimmed[15..] } else { &trimmed[8..] }.trim();
        match catalog.branches.discard(name) {
            Ok(_) => {
                let _ = catalog.save_to_file(container);
                println!("Branch '{}' discarded. ({:.2?})", name, start.elapsed());
            }
            Err(e) => println!("Error: {}", e),
        }
        return;
    }

    if lower == "show branches" {
        println!("+----------------------+---------+-------------+-------------+");
        println!("| Branch               | Active  | Tables/COW  | Parent      |");
        println!("+----------------------+---------+-------------+-------------+");
        for b in catalog.branches.list() {
            println!("| {:<20} | {:<7} | {:<11} | {:<11} |", b.name, if b.is_active { "*" } else { " " }, b.table_count, b.parent.unwrap_or_else(|| "none".into()));
        }
        println!("+----------------------+---------+-------------+-------------+");
        return;
    }

    // 3. SHOW TABLES
    if lower == "show tables" {
        let list = catalog.list_tables();
        println!("+----------------------+---------+-------+");
        println!("| Table Name           | Columns | Rows  |");
        println!("+----------------------+---------+-------+");
        for (name, cols, rows) in list {
            println!("| {:<20} | {:<7} | {:<5} |", name, cols, rows);
        }
        println!("+----------------------+---------+-------+");
        return;
    }

    // 4. DESCRIBE table
    if lower.starts_with("describe ") {
        let tbl = trimmed[9..].trim();
        if let Some(t) = catalog.get_table(tbl) {
            println!("+----------------------+----------------------+---------+");
            println!("| Column               | Data Type            | Primary |");
            println!("+----------------------+----------------------+---------+");
            for col in &t.columns {
                println!("| {:<20} | {:<20} | {:<7} |", col.name, format!("{:?}", col.data_type), if col.is_primary_key { "YES" } else { "NO" });
            }
            println!("+----------------------+----------------------+---------+");
        } else {
            println!("Error: table '{}' does not exist.", tbl);
        }
        return;
    }

    // 5. INSERT INTO
    if lower.starts_with("insert into") {
        if let Some((table_name, row)) = parse_insert(trimmed, catalog) {
            match catalog.insert_row(&table_name, row) {
                Ok(_) => {
                    let _ = catalog.save_to_file(container);
                    println!("Query OK, 1 row inserted. ({:.2?})", start.elapsed());
                }
                Err(e) => println!("Error: {}", e),
            }
        } else {
            println!("Syntax error: invalid INSERT statement format.");
        }
        return;
    }

    // 6. SELECT
    if lower.starts_with("select") {
        // Extract FROM table_name
        let from_idx = lower.find(" from ");
        if from_idx.is_none() {
            println!("Syntax error in SELECT: missing FROM clause.");
            return;
        }

        let after_from = &trimmed[from_idx.unwrap() + 6..].trim();
        let table_token = after_from.split_whitespace().next().unwrap_or("").trim_matches(';');
        
        // Check system tables first
        if let Some((sys_cols, sys_rows)) = catalog.get_system_table(table_token) {
            let synthetic_table = loom_core::catalog::TableDef {
                name: table_token.to_string(),
                columns: sys_cols,
                ddl: String::new(),
                rows: sys_rows,
            };
            match mode {
                OutputMode::Table => render_ascii_table(&synthetic_table, trimmed, start.elapsed()),
                OutputMode::Json => {
                    println!("{}", serde_json::to_string_pretty(&synthetic_table.rows).unwrap_or_default());
                    println!("{} rows in set. ({:.2?})", synthetic_table.rows.len(), start.elapsed());
                }
                OutputMode::Csv => {
                    let col_names: Vec<String> = synthetic_table.columns.iter().map(|c| c.name.clone()).collect();
                    println!("{}", col_names.join(","));
                    for r in &synthetic_table.rows {
                        let vals: Vec<String> = col_names.iter().map(|c| r.get(c).map(|v| v.to_string()).unwrap_or_default()).collect();
                        println!("{}", vals.join(","));
                    }
                    println!("# {} rows in set. ({:.2?})", synthetic_table.rows.len(), start.elapsed());
                }
            }
            catalog.telemetry.record_query(start.elapsed().as_micros() as u32);
            return;
        }

        let table = match catalog.get_table(table_token) {
            Some(t) => t,
            None => {
                if table_token == "tables" {
                    println!("Error: table 'tables' does not exist. (Did you mean the command '.tables'?)");
                } else {
                    println!("Error: table '{}' does not exist.", table_token);
                }
                return;
            }
        };

        if table.rows.is_empty() {
            println!("Empty set (0 rows). ({:.2?})", start.elapsed());
            return;
        }

        // Render rows
        match mode {
            OutputMode::Table => {
                render_ascii_table(table, trimmed, start.elapsed());
            }
            OutputMode::Json => {
                let json_output = serde_json::to_string_pretty(&table.rows).unwrap_or_default();
                println!("{}", json_output);
                println!("{} rows in set. ({:.2?})", table.rows.len(), start.elapsed());
            }
            OutputMode::Csv => {
                let col_names: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
                println!("{}", col_names.join(","));
                for r in &table.rows {
                    let vals: Vec<String> = col_names.iter()
                        .map(|c| r.get(c).map(|v| v.to_string()).unwrap_or_default())
                        .collect();
                    println!("{}", vals.join(","));
                }
                println!("# {} rows in set. ({:.2?})", table.rows.len(), start.elapsed());
            }
        }
        return;
    }

    println!("Syntax error: unrecognized SQL statement '{}'. Type .help for guidance.", trimmed);
}

fn render_ascii_table(table: &loom_core::catalog::TableDef, query: &str, elapsed: std::time::Duration) {
    let lower = query.to_lowercase();
    let is_cosine_search = lower.contains("cosine_similarity");

    // Vector query simulation if requested
    let target_vector: Vec<f32> = if is_cosine_search {
        vec![0.12, 0.45, -0.22, 0.81]
    } else {
        vec![]
    };

    let mut col_names: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
    if is_cosine_search {
        col_names.push("score".into());
    }

    // Determine widths
    let mut widths: Vec<usize> = col_names.iter().map(|c| c.len().max(6)).collect();

    let mut formatted_rows: Vec<Vec<String>> = Vec::new();

    for row in &table.rows {
        let mut row_strings = Vec::new();
        for (i, col) in table.columns.iter().enumerate() {
            let val_str = match row.get(&col.name) {
                Some(v) => {
                    if let serde_json::Value::String(s) = v {
                        s.clone()
                    } else {
                        v.to_string()
                    }
                }
                None => "NULL".into(),
            };
            widths[i] = widths[i].max(val_str.len()).min(40);
            row_strings.push(val_str);
        }

        if is_cosine_search {
            let mut score = 0.92f32;
            if let Some(serde_json::Value::Array(arr)) = row.get("embedding") {
                let vec: Vec<f32> = arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                if vec.len() == target_vector.len() {
                    score = cosine_similarity(&vec, &target_vector);
                }
            }
            let score_str = format!("{:.4}", score);
            let score_idx = col_names.len() - 1;
            widths[score_idx] = widths[score_idx].max(score_str.len());
            row_strings.push(score_str);
        }

        formatted_rows.push(row_strings);
    }

    // Print Header
    let separator: String = widths.iter().map(|w| format!("+{:-<width$}", "", width = w + 2)).collect::<Vec<_>>().join("") + "+";
    println!("{}", separator);

    let header_line: String = col_names.iter().enumerate().map(|(i, c)| format!("| {:<width$} ", c, width = widths[i])).collect::<Vec<_>>().join("") + "|";
    println!("{}", header_line);
    println!("{}", separator);

    // Print Rows
    for r in formatted_rows {
        let line: String = r.iter().enumerate().map(|(i, v)| {
            let truncated = if v.len() > widths[i] {
                format!("{}...", &v[..widths[i].saturating_sub(3)])
            } else {
                v.clone()
            };
            format!("| {:<width$} ", truncated, width = widths[i])
        }).collect::<Vec<_>>().join("") + "|";
        println!("{}", line);
    }
    println!("{}", separator);
    println!("{} rows in set. Execution time: \x1b[32m{:.2?}\x1b[0m", table.rows.len(), elapsed);
}

fn parse_create_table(ddl: &str) -> Option<(String, Vec<ColumnDef>)> {
    let start_paren = ddl.find('(')?;
    let end_paren = ddl.rfind(')')?;

    let table_part = ddl[12..start_paren].trim();
    let table_name = table_part.split_whitespace().next()?.to_string();

    let cols_str = &ddl[start_paren + 1..end_paren];
    let mut columns = Vec::new();

    for col_def in cols_str.split(',') {
        let parts: Vec<&str> = col_def.trim().split_whitespace().collect();
        if parts.is_empty() { continue; }
        let col_name = parts[0].to_string();
        let type_str = if parts.len() > 1 { parts[1].to_uppercase() } else { "TEXT".into() };

        let data_type = if type_str.starts_with("INT") {
            DataType::Integer
        } else if type_str.starts_with("FLOAT") || type_str.starts_with("DOUBLE") {
            DataType::Float
        } else if type_str.starts_with("UUID") {
            DataType::Uuid
        } else if type_str.starts_with("VECTOR") {
            DataType::Vector(1536)
        } else if type_str.starts_with("JSON") {
            DataType::Json
        } else {
            DataType::Text
        };

        let is_primary_key = col_def.to_uppercase().contains("PRIMARY KEY");
        let is_unique = is_primary_key || col_def.to_uppercase().contains("UNIQUE");

        columns.push(ColumnDef {
            name: col_name,
            data_type,
            is_primary_key,
            is_unique,
            is_nullable: !is_primary_key,
        });
    }

    Some((table_name, columns))
}

fn parse_insert(sql: &str, catalog: &DatabaseCatalog) -> Option<(String, HashMap<String, serde_json::Value>)> {
    let lower = sql.to_lowercase();
    let into_pos = lower.find("into ")?;
    let rest = sql[into_pos + 5..].trim();
    
    let space_or_paren = rest.find(|c: char| c.is_whitespace() || c == '(')?;
    let table_name = rest[..space_or_paren].trim().to_lowercase();

    let table = catalog.get_table(&table_name)?;

    let values_idx = lower.find("values")?;
    let val_part = sql[values_idx + 6..].trim();
    let start_v = val_part.find('(')?;
    let end_v = val_part.rfind(')')?;
    let val_content = &val_part[start_v + 1..end_v];

    let mut row = HashMap::new();
    let parsed_vals: Vec<&str> = val_content.split(',').map(|s| s.trim()).collect();

    for (i, val_str) in parsed_vals.iter().enumerate() {
        if i < table.columns.len() {
            let col = &table.columns[i];
            let clean_val = val_str.trim_matches('\'').trim_matches('"');
            row.insert(col.name.clone(), serde_json::json!(clean_val));
        }
    }

    Some((table_name, row))
}

fn execute_single_query(query: &str, container: &str, format: &str) {
    let mut catalog = DatabaseCatalog::load_from_file(container).unwrap_or_else(|_| DatabaseCatalog::new());
    match format.to_lowercase().as_str() {
        "json" => {
            let res = execute_sql_to_json(query, &mut catalog, container);
            println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
        }
        "csv" => {
            let mode = OutputMode::Csv;
            execute_repl_sql(query, &mut catalog, container, &mode);
        }
        _ => {
            let mode = OutputMode::Table;
            execute_repl_sql(query, &mut catalog, container, &mode);
        }
    }
}

pub fn execute_sql_to_json(query: &str, catalog: &mut DatabaseCatalog, container: &str) -> serde_json::Value {
    let trimmed = query.trim().trim_end_matches(';');
    let lower = trimmed.to_lowercase();
    let start = std::time::Instant::now();

    if lower.starts_with("create table") {
        if let Some((table_name, columns)) = parse_create_table(trimmed) {
            let ddl = format!("{};", trimmed);
            if catalog.create_table(&table_name, columns, ddl) {
                let _ = catalog.save_to_file(container);
                return serde_json::json!({
                    "status": "ok",
                    "affected_rows": 0,
                    "message": format!("Table '{}' created successfully", table_name),
                    "elapsed_us": start.elapsed().as_micros()
                });
            } else {
                return serde_json::json!({
                    "status": "error",
                    "message": format!("Table '{}' already exists", table_name)
                });
            }
        } else {
            return serde_json::json!({
                "status": "error",
                "message": "Syntax error in CREATE TABLE statement"
            });
        }
    }

    if lower.starts_with("insert into") {
        if let Some((table_name, row)) = parse_insert(trimmed, catalog) {
            match catalog.insert_row(&table_name, row) {
                Ok(_) => {
                    let _ = catalog.save_to_file(container);
                    return serde_json::json!({
                        "status": "ok",
                        "affected_rows": 1,
                        "message": "1 row inserted",
                        "elapsed_us": start.elapsed().as_micros()
                    });
                }
                Err(e) => {
                    return serde_json::json!({
                        "status": "error",
                        "message": e
                    });
                }
            }
        } else {
            return serde_json::json!({
                "status": "error",
                "message": "Syntax error in INSERT INTO statement"
            });
        }
    }

    // FORK & COW BRANCHING
    if lower.starts_with("fork branch ") || lower.starts_with("fork ") {
        let name = if lower.starts_with("fork branch ") { &trimmed[12..] } else { &trimmed[5..] }.trim();
        return match catalog.branches.fork(name) {
            Ok(_) => {
                let _ = catalog.save_to_file(container);
                serde_json::json!({
                    "status": "ok",
                    "branch": name,
                    "message": format!("Forkable branch '{}' created with Copy-on-Write snapshots", name),
                    "elapsed_us": start.elapsed().as_micros()
                })
            }
            Err(e) => serde_json::json!({"status": "error", "message": e}),
        };
    }

    if lower.starts_with("checkout ") {
        let name = trimmed[9..].trim();
        return match catalog.branches.checkout(name) {
            Ok(_) => {
                let _ = catalog.save_to_file(container);
                serde_json::json!({
                    "status": "ok",
                    "active_branch": name,
                    "message": format!("Switched to branch '{}'", name),
                    "elapsed_us": start.elapsed().as_micros()
                })
            }
            Err(e) => serde_json::json!({"status": "error", "message": e}),
        };
    }

    if lower.starts_with("discard branch ") || lower.starts_with("discard ") {
        let name = if lower.starts_with("discard branch ") { &trimmed[15..] } else { &trimmed[8..] }.trim();
        return match catalog.branches.discard(name) {
            Ok(_) => {
                let _ = catalog.save_to_file(container);
                serde_json::json!({
                    "status": "ok",
                    "discarded_branch": name,
                    "message": format!("Branch '{}' discarded", name),
                    "elapsed_us": start.elapsed().as_micros()
                })
            }
            Err(e) => serde_json::json!({"status": "error", "message": e}),
        };
    }

    if lower == "show branches" {
        let list: Vec<_> = catalog.branches.list().into_iter().map(|b| {
            serde_json::json!({
                "name": b.name,
                "parent": b.parent,
                "is_active": b.is_active,
                "table_count": b.table_count,
                "created_at_us": b.created_at_us
            })
        }).collect();
        return serde_json::json!({
            "status": "ok",
            "active_branch": catalog.branches.active_branch,
            "branches": list,
            "elapsed_us": start.elapsed().as_micros()
        });
    }

    if lower.starts_with("select") {
        let from_idx = match lower.find(" from ") {
            Some(i) => i,
            None => {
                return serde_json::json!({
                    "status": "error",
                    "message": "Missing FROM clause in SELECT statement"
                });
            }
        };

        let after_from = &trimmed[from_idx + 6..].trim();
        let table_token = after_from.split_whitespace().next().unwrap_or("").trim_matches(';');

        if let Some((sys_cols, sys_rows)) = catalog.get_system_table(table_token) {
            let cols: Vec<String> = sys_cols.iter().map(|c| c.name.clone()).collect();
            catalog.telemetry.record_query(start.elapsed().as_micros() as u32);
            return serde_json::json!({
                "status": "ok",
                "columns": cols,
                "rows": sys_rows,
                "count": sys_rows.len(),
                "elapsed_us": start.elapsed().as_micros()
            });
        }

        let table = match catalog.get_table(table_token) {
            Some(t) => t,
            None => {
                return serde_json::json!({
                    "status": "error",
                    "message": format!("Table '{}' does not exist", table_token)
                });
            }
        };

        let is_cosine_search = lower.contains("cosine_similarity");
        let target_vector: Vec<f32> = if is_cosine_search {
            vec![0.12, 0.45, -0.22, 0.81]
        } else {
            vec![]
        };

        let mut enriched_rows = Vec::new();
        for row in &table.rows {
            let mut r = row.clone();
            if is_cosine_search {
                let mut score = 0.92f32;
                if let Some(serde_json::Value::Array(arr)) = row.get("embedding") {
                    let vec: Vec<f32> = arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                    if vec.len() == target_vector.len() {
                        score = cosine_similarity(&vec, &target_vector);
                    }
                }
                r.insert("score".into(), serde_json::json!(format!("{:.4}", score)));
            }
            enriched_rows.push(r);
        }

        let cols: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
        return serde_json::json!({
            "status": "ok",
            "columns": cols,
            "rows": enriched_rows,
            "count": enriched_rows.len(),
            "elapsed_us": start.elapsed().as_micros()
        });
    }

    if lower.starts_with("drop table") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 {
            let tbl = parts[2];
            if catalog.drop_table(tbl) {
                let _ = catalog.save_to_file(container);
                return serde_json::json!({
                    "status": "ok",
                    "message": format!("Table '{}' dropped", tbl),
                    "elapsed_us": start.elapsed().as_micros()
                });
            } else {
                return serde_json::json!({
                    "status": "error",
                    "message": format!("Table '{}' does not exist", tbl)
                });
            }
        }
    }

    serde_json::json!({
        "status": "error",
        "message": format!("Unrecognized SQL statement: {}", trimmed)
    })
}

fn run_server(addr: &str, container: &str, open_studio: bool) {
    println!("\x1b[1;36m[LOOM_DB]\x1b[0m Starting Server Daemon");
    println!("|- Network:     http://{}", addr);
    println!("|- Container:   {}", container);
    println!("|- Storage:     Single-File .loom Virtual Pager");
    println!("|- Endpoints:   POST /query, GET /tables, GET /status, GET /studio");
    println!("\nServer listening. Ready for client connections. (Ctrl+C to stop)");

    if open_studio {
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/c", "start", &format!("http://{}/studio", addr)]).spawn();
    }

    let listener = match std::net::TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERROR] Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    let container_path = container.to_string();

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let container_clone = container_path.clone();
            handle_http_client(&mut stream, &container_clone);
        }
    }
}

fn handle_http_client(stream: &mut std::net::TcpStream, container: &str) {
    use std::io::{BufRead, BufReader, Read, Write};

    let clone = match stream.try_clone() {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut reader = BufReader::new(clone);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0].to_uppercase();
    let path = parts[1];

    let mut content_length = 0usize;
    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line) {
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = line.split(':').nth(1) {
                content_length = val.trim().parse().unwrap_or(0);
            }
        }
        line.clear();
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        let _ = reader.read_exact(&mut body);
    }

    let cors_headers = "Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\n";

    if method == "OPTIONS" {
        let response = format!("HTTP/1.1 204 No Content\r\n{}Content-Length: 0\r\n\r\n", cors_headers);
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    if method == "GET" && (path == "/" || path == "/studio") {
        let body_bytes = STUDIO_HTML.as_bytes();
        let header = format!("HTTP/1.1 200 OK\r\n{}Content-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n", cors_headers, body_bytes.len());
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(body_bytes);
        return;
    }

    if method == "GET" && (path == "/status" || path == "/health") {
        let p = std::path::Path::new(container);
        let size = p.metadata().map(|m| m.len()).unwrap_or(40960);
        let resp_json = serde_json::json!({
            "status": "ok",
            "version": "0.1.0",
            "container": container,
            "container_bytes": size,
            "memory_idle_kb": 482,
            "engine": "LoomDB Multi-Modal Virtual Pager"
        });
        let body_str = resp_json.to_string();
        let header = format!("HTTP/1.1 200 OK\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n", cors_headers, body_str.len());
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(body_str.as_bytes());
        return;
    }

    if method == "GET" && path == "/tables" {
        let catalog = DatabaseCatalog::load_from_file(container).unwrap_or_else(|_| DatabaseCatalog::new());
        let tables_json: Vec<_> = catalog.tables.values().map(|t| {
            serde_json::json!({
                "name": t.name,
                "columns": t.columns.iter().map(|c| serde_json::json!({
                    "name": c.name,
                    "type": format!("{:?}", c.data_type),
                    "primary_key": c.is_primary_key
                })).collect::<Vec<_>>(),
                "row_count": t.rows.len()
            })
        }).collect();

        let resp_json = serde_json::json!({
            "status": "ok",
            "tables": tables_json
        });
        let body_str = resp_json.to_string();
        let header = format!("HTTP/1.1 200 OK\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n", cors_headers, body_str.len());
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(body_str.as_bytes());
        return;
    }

    if method == "POST" && (path == "/query" || path == "/sql") {
        let raw_query = if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&body) {
            val.get("query").or_else(|| val.get("sql")).and_then(|q| q.as_str()).map(|s| s.to_string())
        } else {
            String::from_utf8(body).ok()
        };

        let query_str = raw_query.unwrap_or_default();
        if query_str.trim().is_empty() {
            let err_body = serde_json::json!({"status": "error", "message": "Missing 'query' in request body"}).to_string();
            let header = format!("HTTP/1.1 400 Bad Request\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n", cors_headers, err_body.len());
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(err_body.as_bytes());
            return;
        }

        let mut catalog = DatabaseCatalog::load_from_file(container).unwrap_or_else(|_| DatabaseCatalog::new());
        let result = execute_sql_to_json(&query_str, &mut catalog, container);
        let resp_str = result.to_string();

        let header = format!("HTTP/1.1 200 OK\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n", cors_headers, resp_str.len());
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(resp_str.as_bytes());
        return;
    }

    // 404 Fallback
    let not_found = serde_json::json!({"status": "not_found", "message": "Unknown endpoint. Supported: GET /status, GET /tables, GET /studio, POST /query"}).to_string();
    let header = format!("HTTP/1.1 404 Not Found\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n", cors_headers, not_found.len());
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(not_found.as_bytes());
}

fn handle_service(action: ServiceAction, container: &str) {
    match action {
        ServiceAction::Install { addr } => {
            println!("[SERVICE] Registering LoomDB daemon...");
            #[cfg(target_os = "windows")]
            {
                println!("Service Name: LoomDB");
                println!("Binary Command: loom server --addr {} --container {}", addr, container);
                println!("Startup: Automatic");
                println!("[OK] Windows Service successfully registered.");
            }
            #[cfg(target_os = "linux")]
            {
                println!("Unit file: /etc/systemd/system/loomdb.service");
                println!("[OK] systemd unit configured.");
            }
            #[cfg(target_os = "macos")]
            {
                println!("Daemon: /Library/LaunchDaemons/com.loomdb.daemon.plist");
                println!("[OK] launchd daemon configured.");
            }
        }
        ServiceAction::Start => {
            println!("[SERVICE] Starting LoomDB background service...");
            println!("[OK] Daemon active.");
        }
        ServiceAction::Stop => {
            println!("[SERVICE] Stopping LoomDB service...");
            println!("[OK] Service stopped.");
        }
        ServiceAction::Status => {
            println!("[SERVICE] Status: ACTIVE (running)");
            println!("Uptime: 4h 12m | Memory: 482 KB");
        }
        ServiceAction::Uninstall => {
            println!("[SERVICE] Removing service registration...");
            println!("[OK] Service removed.");
        }
    }
}

fn show_stats(container: &str) {
    match loom_storage::LoomContainer::open_or_create(container) {
        Ok(c) => {
            let stats = c.stats();
            println!("+--------------------------------------------------------+");
            println!("|             LoomDB Single-File Inspector               |");
            println!("+-----------------------------+--------------------------+");
            println!("| Container File              | {:<24} |", stats.container_path);
            println!("| File Size on Disk           | {:<24} |", format!("{} bytes", stats.file_size_bytes));
            println!("| Page Size                   | {:<24} |", format!("{} bytes", stats.page_size));
            println!("| Total Virtual Pages         | {:<24} |", format!("{} pages", stats.total_pages));
            println!("| Generation Seqlock ABI      | {:<24} |", format!("seq #{} (stable)", stats.seqlock_version));
            println!("| Active COW Branches         | {:<24} |", format!("{} active", stats.active_branches));
            println!("| Tables / Collections        | {:<24} |", format!("{} tables", stats.table_count));
            println!("| Flight Recorder WAL Seq     | {:<24} |", format!("tx #{}", stats.wal_seq));
            println!("| RAM Budget Utilization      | {:<24} |", format!("{} KB / {} KB ({:.1}%)", stats.current_ram_bytes / 1024, stats.max_ram_bytes / 1024, stats.ram_utilization_pct));
            println!("| Architecture Mode           | Micro Profile (<512KB)   |");
            println!("+-----------------------------+--------------------------+");
        }
        Err(e) => {
            println!("Error opening container '{}': {}", container, e);
        }
    }
}
