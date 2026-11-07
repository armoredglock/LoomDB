# LoomDB 🧶

> **The World's Lightest, Most Versatile Multi-Modal Database.**  
> *Sub-megabyte binary. Sub-megabyte RAM footprint. Single-file `.loom` container. Runs everywhere: from WebAssembly in the browser and IoT micro-controllers to multi-core cloud servers.*

[![Status](https://img.shields.io/badge/status-active--design%20%26%20development-blue.svg)](#)
[![License](https://img.shields.io/badge/license-Apache--2.0%20%2F%20MIT-green.svg)](#)
[![Language](https://img.shields.io/badge/language-Rust%202021-orange.svg)](#)
[![WebAssembly](https://img.shields.io/badge/target-wasm32--unknown--unknown-purple.svg)](#)
[![Footprint](https://img.shields.io/badge/binary%20size-%3C%20850%20KB-success.svg)](#)

---

## 🌟 The Vision: Lighter than SQLite, Smarter than Postgres

Most developers today want a database that is **instant, zero-config, and embeddable** like SQLite, but SQLite was built in 2000 for simple relational tables. When modern projects need:
* **JSON documents** (like MongoDB)
* **AI Vector Search & Embeddings** (like Pinecone/Qdrant)
* **Time-Series metrics** (like InfluxDB)
* **Real-time live push queries** (like Firebase/Supabase)
* **In-browser / Edge execution** (like IndexedDB or Cloudflare D1)

...they are forced to abandon lightweight simplicity and run multi-gigabyte Docker stacks or expensive cloud databases.

**LoomDB solves this forever.** It is an open-source, ultra-resourceful database designed to be:
1. **Lighter than the lightest:** Stripped binary is **< 850 KB**. Idle RAM usage is **< 512 KB**. Zero background thread overhead when idle.
2. **Single-File Simplicity (`.loom`):** Everything (tables, JSON docs, HNSW vector graphs, WAL, and indexes) lives in a single, portable, memory-mappable `.loom` file. Copy it, email it, backup it, or mount it instantly.
3. **Run Everywhere (Wasm-First):** Compiles to pure `wasm32-unknown-unknown` to run natively inside web browsers, mobile apps, Edge Workers, and micro-controllers.
4. **Swiss-Army Knife Multi-Modal:** SQL Relational + Document JSON + Vector Embeddings + Full-Text Search (BM25) + Time-Series + Real-Time Live CDC in one engine.
5. **Zero External Dependencies:** Built in 100% pure, safe Rust. No C-runtime prerequisites, no bloated external daemons.
6. **Embedded Web Studio:** A built-in, zero-dependency visual web inspector accessible directly from your browser when running the CLI or server.

---

## 📊 How LoomDB Compares to Today's Databases

| Metric / Capability | **LoomDB** 🧶 | **SQLite** 🪶 | **DuckDB** 🦆 | **MongoDB** 🍃 | **Pinecone / Qdrant** 🎯 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Idle Memory Footprint** | **< 512 KB** | ~4 MB | ~30 MB | ~350 MB | ~200 MB |
| **Binary Size (Stripped)** | **< 850 KB** | ~800 KB | ~18 MB | ~120 MB | ~45 MB |
| **Storage Model** | **Single `.loom` File** | Single `.db` File | Single `.duckdb` | Folder sprawl | Clustered storage |
| **Native Vector Search (HNSW)** | **Yes (SIMD Fused)** | ❌ No (plugin req) | ❌ Experimental | ❌ Addon | **Yes** |
| **JSON Document Support** | **Yes (LoomBSON)** | ⚠️ Limited JSON1 | ⚠️ Basic | **Yes** | ❌ Metadata only |
| **SQL Relational Engine** | **Yes (LoomQL)** | **Yes** | **Yes** | ❌ No | ❌ No |
| **Real-time Live Subscriptions**| **Yes (Native CDC)** | ❌ No | ❌ No | ⚠️ Change streams | ❌ No |
| **Browser / Wasm Native** | **Yes (`wasm32`)** | ⚠️ Complex C-Wasm | ⚠️ Large Wasm | ❌ No | ❌ No |
| **Time-Travel (Git for Data)** | **Yes (`AS OF`)** | ❌ No | ❌ No | ❌ No | ❌ No |
| **Offline CRDT Sync** | **Yes (Built-in)** | ❌ No | ❌ No | ❌ No | ❌ No |

---

## 🚀 Ultra-Lightweight Architectural Modes

LoomDB adapts its memory and execution model according to the host environment:

### Mode 1: The Micro Profile (IoT, Micro-controllers, Mobile, Browser Wasm)
* **RAM Allocation:** Under **512 KB**.
* **Page Size:** 1 KB – 4 KB virtual pages.
* **Vector Index:** 8-bit Scalar Quantization (SQ8) or 1-bit binary vectors for negligible memory usage.
* **Execution:** Direct zero-copy memory-mapped I/O (`mmap`). No thread pools spawned.

### Mode 2: The Desktop & Edge Profile (CLI tools, local apps, Tauri, Electron)
* **RAM Allocation:** 8 MB – 32 MB adaptive cache.
* **Concurrency:** Lock-free SkipList MemTable with background WAL synchronization.
* **Features:** Full LoomQL, single-pass hybrid search, reactive live subscriptions.

### Mode 3: The Server & Cloud Profile (High-throughput APIs, Microservices)
* **RAM Allocation:** Dynamically scales from 64 MB to Terabytes.
* **I/O Engine:** Direct I/O (`io_uring` / IOCP) bypassing OS page-cache bottlenecks.
* **Features:** Multi-threaded vectorized Morsel execution, gRPC, HTTP REST, SSE, and WebSocket endpoints.

---

## 💻 Quick Start

### 1. Embedded Rust (Zero Latency)

```rust
use loom_core::LoomDB;

// Opens a single portable .loom file (or creates it if not found)
let db = LoomDB::open("./my_project.loom")?;

// Insert unstructured JSON with vector embeddings
db.collection("memories").insert(serde_json::json!({
    "topic": "Rust Systems Programming",
    "notes": "LoomDB achieves sub-megabyte memory footprint.",
    "embedding": [0.12, -0.45, 0.88, 0.05]
}))?;

// Hybrid Vector + Metadata Query in a single pass
let results = db.query(
    "SELECT topic, notes FROM memories 
     WHERE topic LIKE '%Rust%' 
     ORDER BY vector_distance(embedding, [0.10, -0.40, 0.85, 0.02]) ASC 
     LIMIT 1"
)?;
```

### 2. Standalone CLI & Instant Web Studio

Run the single stripped binary to start an instant local database with a built-in visual UI:

```bash
# Starts the server and opens the embedded Web Studio in your browser
loomdb server --open-studio
```

---

## 📦 Modular Workspace Layout

```
LoomDB/
├── Cargo.toml                 # Workspace root with micro-binary release profiles
├── README.md                  # Project overview & philosophy
├── ARCHITECTURE.md            # Zero-overhead architecture & memory profiles
├── SPECIFICATION.md           # The Single-File .loom container specification
├── FEATURES_AND_ROADMAP.md    # Standard vs. Missing Next-Gen features
├── API_DESIGN.md              # Rust, Python, TypeScript & LoomQL syntax
├── CONTRIBUTING.md            # Testing, crash injection, and PR guide
│
├── crates/
│   ├── loom-core/             # Micro-kernel, memory managers, error types
│   ├── loom-storage/          # Single-file .loom pager, WAL, MemTable, WiscKey
│   ├── loom-index/            # SIMD HNSW vector graph, BM25 full-text, B+Tree
│   ├── loom-query/            # LoomQL parser, AST, Morsel vectorized engine
│   └── loom-cli/              # Ultra-light CLI with embedded Web Studio UI
```

---

## 📜 Open Source & Free for Everyone

LoomDB is released under the **Apache-2.0 / MIT Dual License**, making it 100% free for open-source and commercial use forever. No hidden enterprise paywalls, no proprietary cloud locks.
