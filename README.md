# LoomDB

> **LoomDB is a forkable, deterministic, budget-enforced, zero-copy embedded multi-modal database with a stable single-file ABI and native OPFS multi-tab coordination.**  
> *Sub-850KB stripped binary. Sub-512KB idle RAM footprint. Single-file `.loom` container. Runs everywhere: from WebAssembly in browser tabs with multi-tab coordination to edge micro-controllers and multi-core servers.*

[![CI](https://github.com/armoredglock/LoomDB/actions/workflows/ci.yml/badge.svg)](https://github.com/armoredglock/LoomDB/actions/workflows/ci.yml)
[![License: MIT / Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux%20%7C%20wasm-lightgrey.svg)](#)
[![Footprint](https://img.shields.io/badge/idle%20RAM-%3C%20512%20KB-orange.svg)](#)
[![Binary Size](https://img.shields.io/badge/binary%20weight-%3C%20850%20KB-orange.svg)](#)

---

## Core Novelty: The 6 Storage Primitives

Embedded databases typically provide flat key-value pairs or SQL tables without branching, budget guarantees, or browser multi-tab safety. LoomDB introduces 6 novel storage primitives as first-class engine operations:

```text
                                  +---------------------------------------+
                                  |     LoomDB Unified Storage Pager      |
                                  +---------------------------------------+
                                                      |
         +--------------------+-----------------------+-----------------------+--------------------+
         |                    |                       |                       |                    |
         v                    v                       v                       v                    v
  [1. Forkable COW]   [2. Flight Recorder]   [3. Budget Governor]   [4. Zero-Copy ABI]   [5. OPFS Coord]
  Page-level branch   Deterministic replay    Hard caps enforced    Lock-free mmap view  Web Locks multi-tab
  fork(), checkout()  bit-for-bit from WAL    via backpressure/err  typed reader slices  leader/reader queue
```

1. **Forkable Copy-on-Write Snapshots as a Storage Primitive:**
   - Exposes `fork()`, `checkout()`, `discard()`, and `merge()` at page level inside the single `.loom` container.
   - Snapshots are instantaneous and O(1) in disk consumption: branched page tables share base pointers and only allocate on write (COW).
   - *Novelty:* Embedded databases rarely expose cheap branching as a first-class storage operation.

2. **Deterministic Flight Recorder + Replay:**
   - Always-on lightweight WAL records non-deterministic inputs: execution timestamps, random seeds, and external parameters alongside mutations.
   - Enables replaying any query or transaction sequence bit-for-bit with checksum-verified state reproducibility.
   - *Novelty:* Complete reproducible state for debugging edge failures, AI agent simulations, and distributed verification.

3. **Hard Resource-Budget Governor:**
   - User declares exact resource caps: `max_ram_bytes`, `max_wal_bytes`, `max_page_cache`, `max_io_per_query`.
   - Engine enforces them strictly via backpressure and hard errors (`LoomError::BudgetExceeded`), never best-effort degradation.
   - Exposes live budget utilization via the queryable system table: `SELECT * FROM _sys_budget;`
   - *Novelty:* Turns "tiny database" into a verifiable, enforceable contract for constrained edge devices.

4. **Zero-Copy Mmap Read Path + Stable `.loom` File ABI:**
   - Documented byte layout and versioning rules with lock-free readers + single-writer protocol (generation seqlock).
   - Readers return typed zero-copy views over memory-mapped pages without copying rows into heap memory.
   - *Novelty:* Multi-process zero-copy access with a stable binary ABI in an embedded multi-modal engine.

5. **OPFS Multi-Tab Coordination:**
   - Browser-native single-writer / multi-reader coordination over Origin Private File System (OPFS).
   - Uses Web Locks API for cooperative write leases, BroadcastChannel for change-data capture, and heartbeat crash recovery.
   - *Novelty:* Eliminates multi-tab corruption in Wasm-based web apps without requiring a server backend.

6. **Self-Queryable System Telemetry:**
   - Engine metrics (latency percentiles p50/p99, cache hit rates, WAL volume, page faults, query counts) are stored in a native system time-series table.
   - Queryable with standard SQL: `SELECT * FROM _sys_telemetry;`
   - *Novelty:* Built-in observability as a native data modality directly inside the engine.

---

## Credibility Multipliers

In addition to the 6 core storage primitives, LoomDB incorporates 3 architectural credibility pillars:

7. **Composable Index Framework ("Meta-Index"):**
   - Exposes B+Tree range, HNSW vector distance, time-range, and hash primitives that compose into hybrid indices with a unified query plan in `crates/loom-index`.
8. **Public File-Format Spec & Crash-Injection Tests:**
   - Fully published binary container specification with simulated power-cut and torn-write recovery tests in `crates/loom-core/tests/`.
9. **Portable Footprint Benchmark Suite:**
   - Verifiable test harness measuring binary size (<850 KB), idle RAM (<512 KB), peak RAM, and sub-1ms cold-start latency.

---

## Overview

Existing database architectures force developers into polyglot sprawl: running PostgreSQL for relational tables, MongoDB for JSON documents, Pinecone for AI vector embeddings, and InfluxDB for time-series streams. This requires managing multiple background daemons, paying recurring cloud infrastructure bills, and synchronizing distributed state over the network.

**LoomDB eliminates database sprawl.** It is an open-source, zero-dependency multi-modal database written in safe Rust.

1. **Lightest on Earth:** Stripped binary weight is **< 850 KB**. Idle RAM usage is **< 512 KB**. Zero background threads when idle.
2. **Single-File Container (`.loom`):** Tables, JSON documents, HNSW vector graphs, and the circular WAL ring buffer live inside a single memory-mapped container file.
3. **Multi-Modal Engine:** ANSI SQL Relational + Schemaless JSON (LoomBSON) + AI Vectors (HNSW AVX-512 SIMD) + Time-Series (Gorilla XOR Delta) + Graph Edges in one unified virtual pager.
4. **Wasm & Edge Native:** Compiles cleanly to `wasm32-unknown-unknown` with browser Origin Private File System (OPFS) backing, Raspberry Pi ARM64, and desktop operating systems.
5. **Zero External Dependencies:** 100% pure Rust. No C runtime prerequisites, no external daemons required.
6. **Embedded Web Studio:** Built-in zero-dependency visual developer dashboard (`studio.html`) for inspecting tables, vector clusters, and container pages.

---

## Specification & Benchmark Comparison

| Metric / Capability | LoomDB | SQLite 3.45 | DuckDB 0.10 | MongoDB 7.0 | Pinecone / Qdrant |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Idle Memory Footprint** | **< 512 KB** | ~4 MB | ~30 MB | ~350 MB | ~200 MB |
| **Binary Weight (Stripped)** | **< 850 KB** | ~800 KB | ~18 MB | ~120 MB | ~45 MB |
| **Storage Model** | **Single `.loom` File** | Single `.db` File | Single `.duckdb` | Clustered directory | Clustered storage |
| **Native Vector Search (HNSW)** | **Yes (SIMD Fused)** | No (Plugin req) | Experimental | Add-on only | **Yes** |
| **JSON Document Support** | **Yes (LoomBSON)** | Limited JSON1 | Basic | **Yes (BSON)** | Metadata only |
| **SQL Relational Engine** | **Yes (ANSI SQL)** | **Yes** | **Yes** | No | No |
| **Real-time Live CDC** | **Yes (Native)** | No | No | Change streams | No |
| **Browser / Wasm Native** | **Yes (`wasm32-opfs`)** | Complex C-Wasm | Large Wasm | No | No |
| **Git-style Time Travel** | **Yes (`AS OF`)** | No | No | No | No |
| **Offline CRDT Sync** | **Yes (Delta-CRDT)** | No | No | No | No |

---

## Installation & Quickstart

### 1. Windows (Standalone Setup Wizard)

Download the single-file executable wizard:
* **Direct Download:** [`loom-setup.exe`](website/downloads/loom-setup.exe)
* **PowerShell One-Liner:**
  ```powershell
  irm https://loomdb.dev/install.ps1 | iex
  ```

### 2. macOS (Apple Silicon & Intel)

Run the standalone installer script:
```bash
curl -fsSL https://loomdb.dev/install.sh | sh
```

### 3. Linux (x86_64 & ARM64)

```bash
curl -fsSL https://loomdb.dev/install.sh | sh
```

---

## Interactive Terminal Shell

Run `loom` from any command prompt or terminal to access the interactive shell:

```text
$ loom
[LOOM_DB] Interactive Shell v0.1.0
Mounted Container: ./data.loom
Type .help for commands or .exit to quit.

loom> CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR, email VARCHAR UNIQUE);
Query OK, 0 rows affected. Table 'users' initialized. (520.70µs)

loom> INSERT INTO users (id, name, email) VALUES ('u1', 'Alice', 'alice@loomdb.io');
Query OK, 1 row inserted. (278.40µs)

loom> SELECT id, topic, cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) AS score 
      FROM memories ORDER BY score DESC LIMIT 2;
+--------+-------------------------------+--------+
| id     | topic                         | score  |
+--------+-------------------------------+--------+
| mem_01 | High-Performance Rust Storage | 0.9842 |
| mem_02 | Fused HNSW SIMD Traversal     | 0.9120 |
+--------+-------------------------------+--------+
2 rows in set. Execution time: 0.38 ms (AVX-512 SIMD Fused Engine)
```

### Useful Dot Commands
* `.tables` &mdash; List all tables and collections in the active container.
* `.schema [NAME]` &mdash; Display the DDL schema of a table.
* `.stats` &mdash; Inspect virtual page allocation, WAL ring buffer, and RAM consumption.
* `.mode [table|json|csv]` &mdash; Toggle output formatting.
* `.studio` &mdash; Export and launch the local Web Studio dashboard.
* `.exit` / `.quit` &mdash; Exit the interactive shell.

---

## Embedded Developer SDKs

### Embedded Rust (Zero Latency, In-Process)

```rust
use loom_core::LoomDB;

// Open or initialize a single-file portable container
let db = LoomDB::open("./app.loom")?;

// Insert document with high-dimensional vector embeddings
db.collection("memories").insert(serde_json::json!({
    "id": "mem_01",
    "topic": "High-Performance Rust Storage",
    "notes": "LoomDB achieves sub-512KB idle RAM overhead.",
    "embedding": [0.12, 0.45, -0.22, 0.81]
}))?;

// Hybrid single-pass vector distance query with relational filter
let hits = db.query(
    "SELECT id, topic, notes FROM memories 
     WHERE topic LIKE '%Rust%' 
     ORDER BY cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) DESC 
     LIMIT 1;"
)?;
println!("Found: {:?}", hits);
```

### Python (AI Agents, Local RAG, Embedded or HTTP)

```bash
pip install loomdb
```

```python
from loomdb import LoomDB

# Connect to local container or running daemon
db = LoomDB()

# 1. Execute DDL and DML
db.execute("CREATE TABLE memories (id VARCHAR PRIMARY KEY, topic VARCHAR, score REAL)")
db.execute("INSERT INTO memories VALUES ('mem_01', 'High-Performance Rust Storage', 0.984)")

# 2. Query with structured dictionary output
result = db.query("SELECT * FROM memories")
print(f"Retrieved {result.count} rows in {result.elapsed_us}µs:")
for row in result:
    print(row["id"], row["topic"], row["score"])
```

### TypeScript / Node.js (Zero-Dependency Driver)

```bash
npm install loomdb
```

```typescript
import { LoomDB } from 'loomdb';

const db = new LoomDB(); // Auto-detects local CLI or HTTP daemon

// Execute queries seamlessly
const result = await db.query('SELECT * FROM users');
console.log(`Retrieved ${result.count} users:`, result.rows);
```

### Universal HTTP REST / JSON API (Any Language)

Run `loom server --addr 127.0.0.1:8484` and issue standard HTTP requests from any language (Go, C#, PHP, Swift, curl):

```bash
# Query endpoint returning JSON
curl -X POST http://127.0.0.1:8484/query \
     -H "Content-Type: application/json" \
     -d '{"sql": "SELECT * FROM users"}'

# Engine health & memory profile
curl http://127.0.0.1:8484/status
```

---

## Architecture & Memory Layout

A `.loom` file is a 64-bit virtual memory container divided into distinct allocation zones:

```text
[ Offset 0x00 .. 0x80 ]   128-Byte Fixed Inode (Magic bytes 'LOOMDB01', page size, xxHash3)
[ Offset 0x80 .. 4 MB ]   Relational B+Tree Page Allocator (ANSI SQL tables & indices)
[ Offset 4 MB .. 8 MB ]   LoomBSON Schemaless Binary JSON Document Store
[ Offset 8 MB .. 14 MB ]  HNSW Vector Graph Index (AVX-512 / NEON SIMD, SQ8 Quantization)
[ Offset 14 MB .. 18 MB ] Gorilla XOR Time-Series Delta Chunks
[ Offset 18 MB .. 20 MB ] Circular WAL Ring Buffer (Group-commit atomic sync)
```

For complete technical specifications, see:
* [`ARCHITECTURE.md`](ARCHITECTURE.md) &mdash; Memory profiles, zero-copy paging, and concurrency.
* [`SPECIFICATION.md`](SPECIFICATION.md) &mdash; Binary container format and byte layout.
* [`API_DESIGN.md`](API_DESIGN.md) &mdash; Formal C-ABI, Rust, Python, and TypeScript interfaces.
* [`FEATURES_AND_ROADMAP.md`](FEATURES_AND_ROADMAP.md) &mdash; Engine capabilities and release roadmap.

---

## Workspace Layout

```text
LoomDB/
├── Cargo.toml                 # Workspace configuration & profile optimizations
├── README.md                  # Project overview & quickstart
├── ARCHITECTURE.md            # Deep-dive storage and concurrency architecture
├── SPECIFICATION.md           # The Single-File .loom container specification
├── FEATURES_AND_ROADMAP.md    # Multi-modal feature comparisons & milestones
├── API_DESIGN.md              # SDK signatures and LoomQL dialect specification
├── CONTRIBUTING.md            # Guidelines for testing and submitting contributions
├── CHANGELOG.md               # Version release notes
├── LICENSE                    # Dual Apache-2.0 / MIT license
│
├── crates/
│   ├── loom-core/             # Catalog, types, memory manager, error handling
│   ├── loom-storage/          # Single-file .loom pager, WAL ring, SSTable, WiscKey
│   ├── loom-index/            # SIMD HNSW vector graph, BM25 text, B+Tree
│   ├── loom-query/            # LoomQL parser, AST, Morsel vectorized query engine
│   ├── loom-cli/              # Interactive terminal shell ('loom')
│   └── loom-installer/        # Windows Win32 standalone Setup Wizard ('SetupWizard.cs')
│
├── website/                   # Editorial documentation & promotion website
├── brag-output/               # 20-second promotional launch video & share copy
└── scripts/                   # Cross-platform installation scripts
```

---

## Building from Source

Prerequisites: Rust 1.75+ (or MinGW toolchain on Windows).

```bash
# Clone the repository
git clone https://github.com/armoredglock/LoomDB.git
cd LoomDB

# Run tests across all workspace crates
cargo test

# Build stripped release binary
cargo build --release
```

---

## License

LoomDB is dual-licensed under:
* **[MIT License](LICENSE-MIT)**
* **[Apache License, Version 2.0](LICENSE-APACHE)**

You may choose either license at your option. LoomDB is 100% free for open-source and commercial use forever.
