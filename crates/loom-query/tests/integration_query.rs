use loom_core::catalog::DatabaseCatalog;
use loom_query::executor::Executor;
use loom_query::parser::Parser;

#[test]
fn test_end_to_end_multimodal_sql_workflow() {
    let mut catalog = DatabaseCatalog::new();

    // 1. Create a multi-modal table
    let ddl = "CREATE TABLE sensor_readings (id TEXT PRIMARY KEY, location TEXT, payload JSON, embedding VECTOR(4));";
    let stmt = Parser::parse(ddl).unwrap();
    let res = Executor::execute(stmt, &mut catalog).unwrap();
    assert!(res.message.unwrap().contains("created"));

    // 2. Insert rows
    let insert_sql = "INSERT INTO sensor_readings (id, location, payload, embedding) VALUES ('s_100', 'datacenter-us-east', '{\"temp\": 23.4}', [0.5, 0.5, 0.5, 0.5]);";
    let stmt2 = Parser::parse(insert_sql).unwrap();
    let res2 = Executor::execute(stmt2, &mut catalog).unwrap();
    assert_eq!(res2.message.unwrap(), "1 row inserted");

    // 3. Query with SIMD vector similarity and projection
    let select_sql = "SELECT id, location, cosine_similarity(embedding, [0.5, 0.5, 0.5, 0.5]) AS score FROM sensor_readings WHERE location LIKE '%datacenter%' LIMIT 1;";
    let stmt3 = Parser::parse(select_sql).unwrap();
    let res3 = Executor::execute(stmt3, &mut catalog).unwrap();
    assert_eq!(res3.rows.len(), 1);
    assert_eq!(res3.rows[0].get("id").unwrap(), "s_100");
    let score = res3.rows[0].get("score").unwrap().as_f64().unwrap();
    assert!((score - 1.0).abs() < 1e-4);

    // 4. Query system tables
    let budget_sql = "SELECT * FROM _sys_budget;";
    let stmt4 = Parser::parse(budget_sql).unwrap();
    let res4 = Executor::execute(stmt4, &mut catalog).unwrap();
    assert!(!res4.rows.is_empty());

    let telem_sql = "SELECT * FROM _sys_telemetry;";
    let stmt5 = Parser::parse(telem_sql).unwrap();
    let res5 = Executor::execute(stmt5, &mut catalog).unwrap();
    assert!(!res5.rows.is_empty());
}

#[test]
fn test_cow_branching_sql_lifecycle() {
    let mut catalog = DatabaseCatalog::new();

    // Fork branch
    let fork_sql = "FORK BRANCH 'staging_v2';";
    let stmt = Parser::parse(fork_sql).unwrap();
    let res = Executor::execute(stmt, &mut catalog).unwrap();
    assert!(res.message.unwrap().contains("staging_v2"));

    // Inspect _sys_branches
    let branches_sql = "SELECT * FROM _sys_branches;";
    let stmt2 = Parser::parse(branches_sql).unwrap();
    let res2 = Executor::execute(stmt2, &mut catalog).unwrap();
    assert_eq!(res2.rows.len(), 2);

    // Switch branch
    let checkout_sql = "CHECKOUT 'staging_v2';";
    let stmt3 = Parser::parse(checkout_sql).unwrap();
    let res3 = Executor::execute(stmt3, &mut catalog).unwrap();
    assert!(res3.message.unwrap().contains("staging_v2"));

    // Switch back to main before discarding
    let checkout_main = "CHECKOUT 'main';";
    let stmt_main = Parser::parse(checkout_main).unwrap();
    let res_main = Executor::execute(stmt_main, &mut catalog).unwrap();
    assert!(res_main.message.unwrap().contains("main"));

    // Discard branch
    let discard_sql = "DISCARD 'staging_v2';";
    let stmt4 = Parser::parse(discard_sql).unwrap();
    let res4 = Executor::execute(stmt4, &mut catalog).unwrap();
    assert!(res4.message.unwrap().contains("Discarded"));
}
