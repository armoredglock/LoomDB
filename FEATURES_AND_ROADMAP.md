# LoomDB Features & Engineering Roadmap 🗺️

This document catalogs every feature planned for LoomDB, categorizing both industry-standard database features and **missing next-generation features**, followed by a phased milestone roadmap.

---

## 1. Feature Classification Matrix

### Category A: Core Database Standards (The Table Stakes)
Features required to match the reliability, durability, and convenience of engines like SQLite, PostgreSQL, and MongoDB:

| Feature | Description | Status |
| :--- | :--- | :--- |
| **ACID Guarantees** | Strict Atomicity, Consistency, Isolation, and Durability. | Planned (v0.1) |
| **Write-Ahead Logging (WAL)** | Append-only transaction logging with group-commit and crash recovery. | Planned (v0.1) |
| **MVCC (Snapshot Isolation)** | Readers never block writers, and writers never block readers. | Planned (v0.1) |
| **B+Tree & Hash Indexes** | Primary and secondary indexes for \(O(\log N)\) and \(O(1)\) lookups. | Planned (v0.1) |
| **Full-Text Search (BM25)** | Inverted index with tokenization, stemming, and BM25 relevance ranking. | Planned (v0.3) |
| **Crash Recovery & Checkpointing** | Automatic WAL replay and fuzzy checkpointing on crash recovery. | Planned (v0.1) |
| **Zero-Downtime Backup & Export** | Online consistent backups and JSON/Parquet/CSV import & export. | Planned (v0.4) |
| **Role-Based Access Control (RBAC)** | Granular user roles, permissions, and authentication mechanisms. | Planned (v0.4) |
| **Comprehensive Health & Metrics** | Prometheus-compatible metrics endpoint (IOPS, cache hit ratios, latency).| Planned (v0.4) |

---

### Category B: The "Missing" Next-Generation Features
Features that developers today constantly wish for, but are missing or fragmented across existing databases:

#### 1. Dual Deployment: Embedded & Standalone
* **The Problem Today:** SQLite is embedded but lacks clustering and networked APIs. Postgres/Mongo are powerful but cannot be embedded inside a mobile app, CLI tool, or desktop application without running a heavy background service.
* **LoomDB Solution:** Seamless dual mode. Use it as a zero-dependency crate/library (`loom::open("./data")`) in-process, OR run `loomdb server --config loom.toml` to serve thousands of remote clients via gRPC and HTTP.

#### 2. Unified Multi-Model Fabric (No More Polyglot Sprawl)
* **The Problem Today:** Teams use 4–5 different databases (Postgres + MongoDB + Redis + Pinecone + InfluxDB).
* **LoomDB Solution:** A single engine natively understands:
  * **Relational Tables** (joins, constraints, transactions).
  * **Document Trees** (flexible nested JSON/LoomBSON).
  * **Vector Embeddings** (HNSW index, cosine/dot similarity).
  * **Time-Series Streams** (Gorilla-compressed, auto-partitioned).
  * **Graph Edges** (adjacency index for traversal without external graph DBs).

#### 3. Single-Pass Hybrid Retrieval (Vector + Metadata + Full-Text)
* **The Problem Today:** Doing vector search with filters in traditional systems requires either **pre-filtering** (which can destroy graph connectivity in HNSW) or **post-filtering** (which wastes search k-nearest neighbors on disqualified records).
* **LoomDB Solution:** **Single-pass fused traversal**. The HNSW graph search traverses nodes while simultaneously validating relational and inverted index bitmasks in the inner loop.

#### 4. Time-Travel & Git-Style Database Branching
* **The Problem Today:** Testing migrations on production data is risky; historical queries require tedious append-only audit tables or complex temporal tables.
* **LoomDB Solution:**
  * **Query as of timestamp:** `SELECT * FROM accounts AS OF '2026-10-15T12:00:00Z'`.
  * **Zero-copy branching:** `loomdb branch create staging` creates an instantaneous Copy-on-Write clone of the database to test migrations or AI pipelines without duplicating disk space.

#### 5. Native Reactive Live Queries (Push CDC)
* **The Problem Today:** Applications poll the DB or set up Kafka + Debezium + Redis to react to data changes.
* **LoomDB Solution:** Live queries directly in the client SDK. Run `db.subscribe(query)` and receive a continuous WebSocket/channel stream of inserts, updates, and deletes calculated directly from the WAL.

#### 6. Local-First & Edge Sync (Delta-CRDTs)
* **The Problem Today:** Offline-first mobile/desktop apps require complex synchronization logic that frequently causes conflict errors or data overwrites.
* **LoomDB Solution:** Native Conflict-free Replicated Data Types (CRDTs). Embedded clients write locally with zero network latency; when an internet connection is established, state vectors synchronize automatically.

#### 7. Fluid Typing (Schemaless to Strict Schema)
* **The Problem Today:** Startups love schemaless NoSQL for rapid prototyping, but regret it later when data corruption happens. Relational DBs are rigid from day one.
* **LoomDB Solution:** Start completely schemaless. As your models mature, add validation contracts:
  ```sql
  ALTER TABLE users ENFORCE SCHEMA (
      id UUID PRIMARY KEY,
      email VARCHAR NOT NULL UNIQUE,
      profile JSON -- still flexible!
  );
  ```

#### 8. Tiered Storage: Hot, Warm, Cold
* **The Problem Today:** Keeping years of historical data in fast NVMe drives is cost-prohibitive. Archiving to S3 requires custom batch export jobs.
* **LoomDB Solution:** Automatic, transparent block tiering:
  * **Hot:** In-memory block cache & MemTable.
  * **Warm:** Local NVMe compressed SSTables.
  * **Cold:** Seamless, background offloading of older SSTable blocks to AWS S3, Google Cloud Storage, or MinIO with zero query code changes.

#### 9. Crypto-Shredding & Multi-Tenant Keys
* **The Problem Today:** Complying with GDPR / CCPA "Right to be Forgotten" requires expensive vacuuming and rewriting of database tables.
* **LoomDB Solution:** Column and tenant-level encryption. Deleting a customer's tenant key instantly renders all their historical data cryptographically unrecoverable across all SSTables, WALs, and backups.

---

## 2. Engineering Roadmap

```
[Phase 0] Core Storage Engine (LoomStore v0.1)
   │
   ▼
[Phase 1] Concurrency, MVCC & Basic LoomQL (v0.2)
   │
   ▼
[Phase 2] Multi-Model & Vector Engine (v0.3)
   │
   ▼
[Phase 3] Server Mode, gRPC, HTTP & SDKs (v0.4)
   │
   ▼
[Phase 4] Time-Travel, Branching & Live Subscriptions (v0.5)
   │
   ▼
[Phase 5] Local-First CRDT Sync, Raft Clustering & Tiered Storage (v1.0)
```

### Phase 0: Storage Core (`loom-storage` & `loom-core`) — v0.1
* [ ] Implement basic memory management, typed keys, and error types.
* [ ] Append-only Write-Ahead Log (WAL) with xxHash3 checksums.
* [ ] Lock-free in-memory MemTable (SkipList implementation).
* [ ] SSTable writer and reader with two-level index and Blocked Bloom Filters.
* [ ] Basic key-value separation (vLog) for large values.
* [ ] Leveled compaction strategy to merge SSTables.
* [ ] Crash recovery and point-in-time replay from WAL.

### Phase 1: Transactions & Basic LoomQL (`loom-query`) — v0.2
* [ ] MVCC implementation with snapshot isolation.
* [ ] Transaction manager: `begin()`, `commit()`, `rollback()`.
* [ ] LoomQL Lexer, Parser, and AST (SQL-compatible syntax).
* [ ] Relational schema catalog and primary/secondary B+Tree indexes.
* [ ] Basic vectorized execution operators (`Filter`, `Project`, `Scan`, `HashJoin`).

### Phase 2: Multi-Model & Vector Engine (`loom-index`) — v0.3
* [ ] LoomBSON document format with nested projection.
* [ ] SIMD-accelerated HNSW index for high-dimensional vector search.
* [ ] Single-pass hybrid search (Vector distance + inverted full-text + relational filter).
* [ ] Time-series partition manager with Gorilla compression.

### Phase 3: Server & Client Ecosystem (`loom-server`, `loom-cli`, `sdk`) — v0.4
* [ ] Standalone server daemon with Tokio async runtime.
* [ ] gRPC interface with protobuf definitions.
* [ ] HTTP/JSON REST interface and WebSocket streaming.
* [ ] Interactive CLI (`loom-cli`) with syntax-highlighted REPL.
* [ ] Official SDKs: Python (`loomdb`), TypeScript/Node.js (`@loomdb/client`), and Rust crate.

### Phase 4: Time-Travel & Live Subscriptions — v0.5
* [ ] Multi-version snapshot time-travel queries (`AS OF TIMESTAMP`).
* [ ] Zero-copy snapshot branching for developer staging workflows.
* [ ] WAL-integrated Change Data Capture (CDC).
* [ ] Reactive query subscriptions over WebSockets.

### Phase 5: Distributed & Production Hardening — v1.0
* [ ] Delta-state CRDT engine for local-first peer synchronization.
* [ ] Raft distributed consensus for high-availability leader-follower clusters.
* [ ] Tiered storage offloader (transparent S3/Cloud Storage integration).
* [ ] Column-level encryption and per-tenant cryptographic isolation.
* [ ] Jepsen fault-injection testing and fuzzing suites.
