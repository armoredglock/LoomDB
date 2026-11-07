# LoomDB 🧶

> **A Multi-Modal, Local-First, AI-Native Database Engine.**  
> *Weaving Relational, Document, Vector, Time-Series, and Graph paradigms into a single zero-overhead storage engine.*

[![Status](https://img.shields.io/badge/status-active--design%20%26%20development-blue.svg)](#)
[![License](https://img.shields.io/badge/license-Apache--2.0%20%2F%20MIT-green.svg)](#)
[![Language](https://img.shields.io/badge/language-Rust%202021-orange.svg)](#)

---

## 🌟 Why LoomDB?

Today's application developers are forced to stitch together a fragmented database zoo:
* **PostgreSQL / MySQL** for structured relational records and transactions.
* **MongoDB / CouchDB** for flexible, schema-less JSON documents.
* **Pinecone / Milvus / Qdrant** for vector embeddings and semantic search.
* **InfluxDB / Timescale** for telemetry and time-stamped events.
* **Redis** for in-memory caching and reactive pub/sub.

**LoomDB** eliminates this fragmentation. Named after the textile loom that weaves different threads into a single fabric, LoomDB is designed from the ground up to **weave multiple data models into one unified, high-performance engine** with zero runtime penalty.

It runs in two modes:
1. **Embedded Mode (`in-process`):** Directly linked as a native zero-latency library (like SQLite or RocksDB) inside Rust, Python, Go, Node.js, C/C++, or mobile apps.
2. **Server Mode (`distributed / standalone`):** Runs as a standalone multi-threaded daemon communicating via gRPC, HTTP/REST, and a high-performance binary protocol.

---

## 🚀 Key Highlights & "Missing" Modern Features

LoomDB combines the reliability of battle-tested enterprise engines with capabilities missing in legacy databases:

### 1. Hybrid Multi-Model Engine (Zero Silos)
* **Documents & Relations:** Store schemaless JSON alongside strictly-typed relational tables with foreign keys and joins.
* **Native Vector Engine:** First-class vector embeddings with HNSW indexing. Filter metadata and search vectors in a **single atomic pass** (no two-phase fetch bottleneck).
* **Time-Series Partitioning:** Automatic time-based chunking with Gorilla/XOR compression for time-stamped events.
* **Graph Traversal:** First-class relationship edges allowing graph traversals without an external Neo4j instance.

### 2. Time-Travel & Branching ("Git for Data")
* Native Copy-on-Write (CoW) MVCC snapshotting.
* Query your database at any historical instant: `SELECT * FROM users AS OF TIMESTAMP '2026-11-01T00:00:00Z'`.
* Zero-copy database branching for safe staging, schema migrations, and CI/CD testing.

### 3. Reactive Live Queries & Push Subscriptions
* Forget polling. Clients can subscribe to any query expression:
  ```rust
  let mut stream = db.subscribe("SELECT * FROM orders WHERE status = 'pending'")?;
  while let Some(delta) = stream.next().await {
      println!("Order changed: {:?}", delta);
  }
  ```
* Push-based Change Data Capture (CDC) built into the core WAL engine.

### 4. Local-First & Edge-to-Cloud Sync (CRDT-Powered)
* Write anywhere, sync seamlessly: run LoomDB on IoT devices, edge nodes, or mobile clients with Conflict-free Replicated Data Types (CRDTs).
* Automatic delta-sync when network connectivity returns without split-brain corruption.

### 5. Hardware-Accelerated Storage Tiering
* Tiered storage engine: **Hot** (SIMD-accelerated in-memory cache), **Warm** (NVMe block-compressed LSM-tree), and **Cold** (transparent zero-overhead offload to S3/Blob storage).
* Direct I/O (`io_uring` on Linux, IOCP on Windows) bypassing OS buffer bloat.

### 6. Zero-Knowledge & Tenant-Level Cryptography
* Column-level and row-level encryption with AES-256-GCM / ChaCha20-Poly1305.
* Per-tenant keys natively partitioned for compliant multi-tenant SaaS architectures.

---

## 📂 Project Documentation Map

* 🏗️ **[ARCHITECTURE.md](ARCHITECTURE.md)** — Storage engine, memory layout, WAL, MVCC, and query execution pipelines.
* 📋 **[FEATURES_AND_ROADMAP.md](FEATURES_AND_ROADMAP.md)** — Exhaustive list of modern & cutting-edge features and release phases.
* 📐 **[SPECIFICATION.md](SPECIFICATION.md)** — File format specifications, disk layouts, binary protocols, and serialization.
* 🔌 **[API_DESIGN.md](API_DESIGN.md)** — Embedded SDKs, LoomQL syntax, REST/gRPC endpoints, and client drivers.
* 🛠️ **[CONTRIBUTING.md](CONTRIBUTING.md)** — Developer setup, build instructions, test harness, and style guides.

---

## ⚡ Quick Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Client Applications                      │
│      Embedded SDK (Rust/Py/JS)  │  gRPC / HTTP / CLI / LoomQL│
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                     LoomDB Core Engine                      │
├─────────────────────────────────────────────────────────────┤
│  Query Planner & Optimizer (Cost-based, Vectorized Exec)    │
│  Multi-Model Router (Relational | Document | Vector | Graph)│
├─────────────────────────────────────────────────────────────┤
│  Concurrency & MVCC Manager (Snapshot Isolation / SSI)      │
│  CRDT Replication & Live Query Pub/Sub Dispatcher           │
├─────────────────────────────────────────────────────────────┤
│  Index Layer                                                │
│  ├─ B+Tree / Masstree (Ordered Relational)                  │
│  ├─ HNSW Index (Vector Semantic Search)                     │
│  ├─ Inverted Index / BM25 (Full-Text Search)                │
│  └─ Radix / Hash (Point-Lookups)                            │
├─────────────────────────────────────────────────────────────┤
│  LoomStore (Hybrid LSM-Tree + WiscKey Separation)           │
│  ├─ MemTable (SkipList / Concurrent ART)                    │
│  ├─ Write-Ahead Log (WAL with Group Commit & Checkpoints)   │
│  ├─ Block Cache (Adaptive 2Q / TinyLFU)                     │
│  └─ SSTable Store (Direct I/O, ZSTD / Gorilla Compression)   │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Workspace Layout

```
LoomDB/
├── Cargo.toml                 # Root Workspace configuration
├── ARCHITECTURE.md            # Engine deep-dive & design philosophy
├── FEATURES_AND_ROADMAP.md    # Feature catalog & release milestones
├── SPECIFICATION.md           # On-disk layout, binary & index specs
├── API_DESIGN.md              # Client APIs, SDKs & LoomQL reference
├── CONTRIBUTING.md            # Dev environment & testing guidelines
├── README.md                  # This file
│
├── crates/
│   ├── loom-core/             # Shared types, errors, memory management, config
│   ├── loom-storage/          # LSM-tree, WAL, SSTable, BlockCache, MVCC
│   ├── loom-index/            # B+Tree, HNSW Vector, Inverted Full-Text
│   ├── loom-query/            # LoomQL Parser, AST, Planner, Vectorized Execution
│   ├── loom-sync/             # CRDT state reconciliation & Raft consensus
│   ├── loom-server/           # Standalone gRPC, HTTP REST & WebSocket server
│   └── loom-cli/              # Interactive REPL, admin, and migration tool
│
└── sdk/                       # Language bindings (Python, TypeScript, Go)
```

---

## 📜 License

LoomDB is licensed under dual Apache-2.0 / MIT licenses, granting full freedom to use LoomDB both as an embedded library and as a backend service in open-source and proprietary commercial projects.
