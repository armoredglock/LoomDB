# LoomDB API Design & LoomQL Reference 🔌

This document details the client APIs for embedded and client-server usage, as well as the syntax reference for **LoomQL** (LoomDB's extended query language).

---

## 1. Embedded Rust SDK

### 1.1 Basic Initialization & Key-Value Operations

```rust
use loom_core::options::Options;
use loom_core::LoomDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open or create a database
    let options = Options::default()
        .with_wal(true)
        .with_max_memtable_size(64 * 1024 * 1024) // 64 MB
        .with_block_cache_size(512 * 1024 * 1024); // 512 MB

    let db = LoomDB::open("./my_database", options)?;

    // Basic Key-Value Operations
    db.put(b"user:1001", b"{\"name\": \"Alice\", \"role\": \"admin\"}")?;
    
    if let Some(value) = db.get(b"user:1001")? {
        println!("Found: {}", String::from_utf8_lossy(&value));
    }

    db.delete(b"user:1001")?;

    Ok(())
}
```

---

### 1.2 ACID Transactions

```rust
let mut txn = db.begin_transaction()?;

txn.put(b"account:A:balance", b"900")?;
txn.put(b"account:B:balance", b"1100")?;

// Commits atomically with Snapshot Isolation
txn.commit()?; 
```

---

### 1.3 Native Vector Search & Hybrid Query

```rust
use loom_index::vector::{DistanceMetric, VectorQuery};
use serde_json::json;

// Insert a document with an embedding vector
db.collection("articles").insert(json!({
    "id": "art_42",
    "title": "Modern Database Design in Rust",
    "category": "engineering",
    "embedding": [0.045, -0.231, 0.892, 0.112] // 1536-dim vector
}))?;

// Hybrid Search: Vector distance combined with metadata filter in a single pass
let results = db.collection("articles").hybrid_search(
    VectorQuery {
        vector: vec![0.041, -0.220, 0.880, 0.105],
        metric: DistanceMetric::Cosine,
        top_k: 5,
    },
    // SQL / JSON filter applied during the HNSW traversal
    "category = 'engineering' AND published_year >= 2025"
)?;

for hit in results {
    println!("ID: {}, Score: {:.4}", hit.id, hit.score);
}
```

---

### 1.4 Reactive Live Query Subscription

```rust
use futures::StreamExt;

// Subscribe to real-time changes
let mut subscription = db.subscribe_live("SELECT * FROM orders WHERE status = 'pending'")?;

tokio::spawn(async move {
    while let Some(change) = subscription.next().await {
        match change {
            Change::Insert(record) => println!("New pending order: {:?}", record),
            Change::Update(old, new) => println!("Order updated: {:?} -> {:?}", old, new),
            Change::Delete(id) => println!("Order deleted: {}", id),
        }
    }
});
```

---

## 2. Python SDK (`loomdb`)

Designed for AI/LLM engineers, data scientists, and backend developers:

```python
import loomdb

# Open embedded database
db = loomdb.open("./loom_data")

# Insert documents with vectors
db.insert("documents", [
    {
        "id": "doc_1",
        "text": "LoomDB is a multi-modal database engine.",
        "embedding": [0.12, 0.84, -0.42],
        "tags": ["database", "rust", "ai"]
    }
])

# Execute LoomQL
results = db.query("""
    SELECT id, text, cosine_similarity(embedding, [0.10, 0.80, -0.40]) AS score
    FROM documents
    WHERE 'ai' IN tags
    ORDER BY score DESC
    LIMIT 5
""")

for row in results:
    print(row["id"], row["text"], row["score"])

# Time travel query
historical = db.query("""
    SELECT * FROM documents AS OF '2026-11-01T10:00:00Z'
""")
```

---

## 3. TypeScript / Node.js SDK

```typescript
import { LoomDB } from '@loomdb/client';

// Connect to Standalone Server or use native bindings
const db = await LoomDB.connect('loom://localhost:8484', {
  token: process.env.LOOM_SECRET_KEY,
});

// Live Query with RxJS / Async Iterator
const liveOrders = db.live("SELECT * FROM telemetry WHERE temp > 85.0");

liveOrders.on('data', (point) => {
  console.warn(`Critical temperature alert: ${point.device_id} = ${point.temp}°C`);
});
```

---

## 4. LoomQL Syntax Reference

LoomQL extends standard ANSI SQL to provide first-class semantics for JSON documents, vectors, time-travel, and streaming subscriptions.

### 4.1 Schema Definition & Fluid Typing
```sql
-- Create a flexible collection with optional schema validation
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR NOT NULL UNIQUE,
    embedding VECTOR(1536) INDEX HNSW(metric = 'cosine', m = 16),
    profile JSON, -- unstructured schemaless payload
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 4.2 Single-Pass Hybrid Vector Search
```sql
-- Fused vector search with relational filter
SELECT id, email, profile->>'company' AS company,
       vector_distance(embedding, $query_vector) AS dist
FROM users
WHERE profile->>'role' = 'lead_architect'
ORDER BY dist ASC
LIMIT 10;
```

### 4.3 Time-Travel Queries
```sql
-- Query past database snapshots without backup restores
SELECT balance FROM customer_accounts 
AS OF TIMESTAMP '2026-10-31T23:59:59Z'
WHERE account_id = 'acc_9831';
```

### 4.4 Continuous Live Subscriptions
```sql
-- Push changes continuously to the connected client
LIVE SELECT id, alert_level, message 
FROM system_logs 
WHERE alert_level IN ('CRITICAL', 'FATAL');
```
