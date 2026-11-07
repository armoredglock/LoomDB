# LoomDB Features & Engineering Roadmap 🗺️

LoomDB is built to be the most lightweight, resourceful, and developer-friendly multi-modal database in the open-source ecosystem.

---

## 1. Feature Classification Matrix

### Category A: The Ultra-Lightweight & Resourceful Foundation
Capabilities that ensure LoomDB runs everywhere with minimal resource consumption:

| Feature | LoomDB Implementation | Target Footprint |
| :--- | :--- | :--- |
| **Micro-Binary Size** | Pure Rust, zero C-runtime bloat, dead-code elimination, LTO. | **< 850 KB (stripped)** |
| **Micro-Memory Footprint** | Paged Clock/LRU cache, zero-copy `mmap`, zero background thread idle. | **< 512 KB RAM (idle)** |
| **Single-File `.loom` Container** | Slotted page container encapsulating WAL, B+Trees, Vectors, & Docs. | **1 portable file** |
| **Wasm In-Browser Engine** | Compiles to `wasm32-unknown-unknown` with OPFS browser storage. | **Pure browser native** |
| **Embedded Web Studio** | Bundled zero-dependency single-page HTML/CSS/JS query console. | **~40 KB embedded** |
| **Zero-Config Import/Export** | Native 1-command import and export for CSV, JSONL, and Parquet. | **Built-in CLI** |

---

### Category B: Multi-Modal Swiss Army Knife
Combine the power of 5 databases into 1 engine without external dependencies:

* **SQL Relational Engine:** Transactions, joins, filters, grouping, foreign keys, secondary indexes.
* **JSON Document Engine (LoomBSON):** Unstructured/schemaless collections, nested JSON path querying (`doc->'user'->>'name'`).
* **AI Vector Search (HNSW):** High-dimensional cosine, Euclidean, and dot product search with SIMD acceleration and 8-bit scalar quantization.
* **Full-Text Search (BM25):** Tokenized inverted index with stemming and BM25 relevance scoring.
* **Time-Series Telemetry:** Gorilla-compressed float points with automatic time-window partitioning.
* **Graph Traversal:** Lightweight adjacency edge lists for relationship queries without needing a dedicated graph database.

---

### Category C: The "Missing" Next-Gen Innovations
Features rarely found in existing databases:

1. **Single-Pass Fused Vector Search:** Traverses HNSW graph nodes while evaluating relational filters in the inner loop (no pre-filter graph fragmentation, no post-filter missing results).
2. **Time-Travel & Git-Style Branching:** Query past snapshots (`AS OF TIMESTAMP '...'`) and create zero-copy copy-on-write branches for safe testing.
3. **Reactive Live Push Subscriptions:** Native Change Data Capture (CDC) streaming query deltas directly over WebSockets or channels without polling.
4. **Local-First CRDT Synchronization:** Offline-first mobile and desktop nodes sync conflict-free when online.
5. **Fluid Typing:** Start 100% schemaless for rapid prototyping, add strict schema constraints on the fly without database downtime or data rewrites.
6. **Crypto-Shredding:** Per-tenant encryption keys enable immediate, legally compliant tenant data destruction by destroying the key.

---

## 2. Engineering Roadmap

```
[Phase 0] Micro-Storage Core & Single-File Container (.loom v0.1)
   │
   ▼
[Phase 1] Concurrency, MVCC & Basic LoomQL (v0.2)
   │
   ▼
[Phase 2] Multi-Modal Engines: Vectors, JSON, & FTS (v0.3)
   │
   ▼
[Phase 3] Embedded Web Studio, CLI & Standalone Server (v0.4)
   │
   ▼
[Phase 4] WebAssembly (Wasm) Browser Port & Time-Travel (v0.5)
   │
   ▼
[Phase 5] Local-First Delta-CRDT Sync & Production v1.0
```

### Phase 0: The Single-File Micro Pager (`loom-storage` & `loom-core`) — v0.1
* [x] Design container layout and file header specification.
* [x] Set up modular workspace and zero-dependency configuration.
* [ ] Implement virtual 4KB Pager with Clock/LRU page cache.
* [ ] Implement in-file circular WAL ring with CRC32 checksums.
* [ ] Slotted page layout for variable-sized keys and values.
* [ ] Micro memory profile (<512KB RAM budget enforcement).

### Phase 1: Transactions & Basic LoomQL (`loom-query`) — v0.2
* [ ] Multi-Version Concurrency Control (MVCC) with Snapshot Isolation.
* [ ] B+Tree primary and secondary index algorithms.
* [ ] LoomQL Lexer, Parser, and AST for basic SQL statements.
* [ ] In-memory execution planner and filter operators.

### Phase 2: Vector & Multi-Modal Engine (`loom-index`) — v0.3
* [ ] SIMD-accelerated HNSW vector index (cosine, Euclidean, dot product).
* [ ] 8-bit Scalar Quantization (SQ8) to cut vector memory by 75%.
* [ ] Fused single-pass hybrid search (Vector + Filter bitmask).
* [ ] LoomBSON nested document projection.
* [ ] Inverted index with BM25 full-text ranking.

### Phase 3: Developer Experience & Web Studio (`loom-cli`, `loom-server`) — v0.4
* [ ] Embedded Web Studio UI (zero-dependency HTML/CSS/JS dashboard).
* [ ] Interactive CLI REPL with query syntax highlighting.
* [ ] HTTP REST & WebSocket live streaming endpoints.
* [ ] Native CSV, JSONL, and Parquet import/export commands.
* [ ] Python SDK (`loomdb`) and TypeScript SDK (`@loomdb/client`).

### Phase 4: WebAssembly (Wasm) & Time-Travel — v0.5
* [ ] Target `wasm32-unknown-unknown` compilation with OPFS backing.
* [ ] In-browser interactive demo playground.
* [ ] Time-travel query execution (`AS OF TIMESTAMP`).
* [ ] Zero-copy database branching.

### Phase 5: CRDT Sync & Production v1.0
* [ ] Delta-state CRDT engine for local-first peer synchronization.
* [ ] Column-level and tenant-level encryption (Crypto-shredding).
* [ ] Formal fuzzing, crash-consistency, and Jepsen-style fault verification.
