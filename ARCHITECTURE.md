# LoomDB Architecture & System Design 🏛️

This document outlines the internal architecture, storage mechanics, concurrency models, query execution pipelines, and networking topology of LoomDB.

---

## 1. Design Philosophy

1. **Zero-Friction Embeddability:** LoomDB must compile as a pure Rust library without mandatory C runtime dependencies, allowing it to be statically or dynamically linked into any host language with zero overhead.
2. **Key-Value Separation for Zero Write Amplification:** Traditional LSM-trees suffer from heavy write amplification during compactions when moving large payloads. LoomDB separates keys (stored in an LSM tree) from values (stored in an append-only Value Log), inspired by WiscKey.
3. **Hardware-Sympathetic Performance:** Exploit modern hardware: AVX-512/AVX2/NEON SIMD for vector math and predicate filtering, asynchronous Direct I/O (`io_uring`/IOCP) to bypass OS page cache overhead, and cache-line aligned data structures.
4. **Unified Multi-Model Fabric:** Treat relational rows, JSON documents, vector embeddings, and time-series points not as separate storage silos, but as specialized view layers over a high-performance typed key-value core.

---

## 2. Component Hierarchy

```
┌────────────────────────────────────────────────────────────────────────┐
│                              LoomQL / APIs                             │
│       [Embedded SDK]      [gRPC Protocol]      [HTTP / WebSockets]     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│                             Query Engine                               │
│  ┌───────────────────────┐  ┌───────────────────────────────────────┐  │
│  │ Lexer / Parser / AST  │  │ Cost-Based Optimizer (CBO)            │  │
│  └──────────┬────────────┘  └──────────────────┬────────────────────┘  │
│             │                                  │                       │
│  ┌──────────▼──────────────────────────────────▼────────────────────┐  │
│  │ Vectorized Execution Engine (Morsel-driven, SIMD-accelerated)    │  │
│  └──────────────────────────────────┬───────────────────────────────┘  │
└─────────────────────────────────────┼──────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼──────────────────────────────────┐
│                           Multi-Model Fabric                           │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐  ┌──────────┐  │
│  │  Relational   │  │   Document    │  │ Vector/Embed │  │  Series  │  │
│  │  Table Schema │  │ (LoomBSON)    │  │ (HNSW Index) │  │ (Gorilla)│  │
│  └───────────────┘  └───────────────┘  └──────────────┘  └──────────┘  │
└─────────────────────────────────────┼──────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼──────────────────────────────────┐
│                         Transaction & Sync Layer                       │
│  ┌──────────────────────────────┐  ┌────────────────────────────────┐  │
│  │ MVCC Manager (Snapshot / SSI)│  │ Live Query & Reactive Push CDC │  │
│  ├──────────────────────────────┤  ├────────────────────────────────┤  │
│  │ Raft Consensus (Clustered)   │  │ Delta-CRDT Engine (Edge Sync)  │  │
│  └──────────────────────────────┘  └────────────────────────────────┘  │
└─────────────────────────────────────┼──────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼──────────────────────────────────┐
│                        LoomStore (Storage Core)                        │
│  ┌─────────────────────────────────┐  ┌─────────────────────────────┐  │
│  │ MemTable (SkipList + ART)       │  │ Write-Ahead Log (WAL)       │  │
│  ├─────────────────────────────────┤  ├─────────────────────────────┤  │
│  │ Key LSM-Tree (L0 -> Lmax)       │  │ Value Log (vLog, Append-only)│ │
│  ├─────────────────────────────────┤  ├─────────────────────────────┤  │
│  │ Block Cache (TinyLFU + 2Q)      │  │ Compaction Engine (Leveled) │  │
│  └─────────────────────────────────┘  └─────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Storage Layer (`LoomStore`)

### 3.1 Key-Value Separation (WiscKey Architecture)
In standard LSM-trees (like LevelDB/RocksDB), every compaction rewrites keys and their values across levels, leading to a write amplification factor of 10x–30x. 
LoomStore implements **Key-Value Separation**:
* **Keys + Pointers in LSM-Tree:** The LSM-Tree stores `(UserKey, VersionTimestamp) -> (ValueLogOffset, ValueLength)`. Because keys and pointers are small (typically 32–64 bytes), the LSM-tree fits almost entirely in memory, making compactions orders of magnitude faster.
* **Values in Append-Only vLog:** Large document payloads, raw vectors, text blobs, and column blocks are written sequentially to the append-only `vLog`. Sequential disk writes reach maximum NVMe throughput (several GB/sec).
* **Garbage Collection (GC):** A background cleaner scans the tail of the `vLog`, checks the LSM index to see if the value is still live, rewrites live values to the head of the log, and truncates dead data.

### 3.2 MemTable & Write-Ahead Log (WAL)
* **MemTable:** Concurrent, lock-free SkipList or Adaptive Radix Tree (ART) holding uncommitted and active transaction mutations in RAM.
* **Write-Ahead Log (WAL):**
  * Every write transaction appends to the WAL before acknowledging the client.
  * Employs **Group Commit:** Multiple concurrent transactions are batched into a single synchronous `fsync` call, maximizing IOPS under high load.
  * Checksums: Every log record is verified with **xxHash3** or **CRC32C** to detect disk corruption or torn writes during power failures.

### 3.3 SSTable Layout
Once a MemTable reaches a configurable threshold (default: 64MB), it is made immutable and flushed to disk as a Level 0 (L0) SSTable:
* **Data Blocks:** Fixed-size chunks (default: 4KB–64KB) compressed with **Zstandard** or **LZ4**.
* **Index Block:** Two-level sparse index locating the data block containing any given key.
* **Filter Block:** Block-level **Blocked Bloom Filters** or **Xor Filters** (saving ~30% space over traditional Bloom filters) to reject non-existent key lookups with zero disk I/O.
* **Meta Block:** Stores key count, min/max timestamp, min/max key boundaries, and deletion tombstones.

---

## 4. Multi-Model Representation

### 4.1 Document & Relational Storage (LoomBSON)
LoomDB uses an optimized binary format called **LoomBSON**:
* Field names are dictionary-encoded into 2-byte integer IDs to eliminate repetitive key string overhead.
* Supports nested structures, arrays, and primitive scalar types.
* Allows partial field projections directly from disk blocks without decompressing the entire document.

### 4.2 Native Vector Indexing (HNSW + SIMD)
* High-dimensional floating-point vectors (`f32`, `f16`, `int8` quantized) are indexed using a **Hierarchical Navigable Small World (HNSW)** graph.
* Search algorithms utilize explicit SIMD instructions (`_mm256_fmadd_ps` on x86-64, NEON on ARM) to compute cosine similarity, Euclidean distance, and dot products in parallel.
* **Filtered Vector Search:** LoomDB executes relational/document filters and vector distance traversal **in a single coordinated pass** through bitmap intersection, avoiding the classic dilemma of pre-filtering (which misses vector topology) vs. post-filtering (which wastes search budget).

### 4.3 Time-Series & Metrics
* Time-stamped data points are grouped into fixed-duration buckets (e.g., 2 hours).
* Timestamps are compressed using **Delta-of-Delta** encoding (often reducing timestamps to 1-2 bits per entry).
* Floating-point metric values are compressed with the **Gorilla XOR** algorithm, achieving 80–90% compression ratios for sensor and server metrics.

---

## 5. Transactions & Concurrency (MVCC)

* **Isolation Levels:**
  * **Snapshot Isolation (SI):** Each transaction reads from an immutable snapshot corresponding to the timestamp at its start. Writes are validated for write-write conflicts.
  * **Serializable Snapshot Isolation (SSI):** Tracks read-write dependency graphs (anti-dependency cycles) to guarantee full serializability without locking.
* **Record Format:** Keys are internally suffixed with an inverted 64-bit monotonically increasing sequence number:
  ```
  InternalKey = UserKey || (u64::MAX - SequenceNumber)
  ```
  This guarantees that standard range scans automatically encounter the newest version of any key first.

---

## 6. Query Engine & Optimization

### 6.1 Vectorized Execution (Morsel-Driven)
Unlike row-at-a-time (Volcano) iteration, LoomDB operates on **Vector Batches** (typically 1024 values per column chunk):
* Tight loops over contiguous arrays maximize CPU L1/L2 cache locality and instruction-level parallelism.
* Vectorized filter evaluation compiles filter conditions into branchless bitmasks.

### 6.2 LoomQL Execution Pipeline
1. **Parser:** Parses LoomQL (SQL-superset with document, vector, and graph operators) into an Abstract Syntax Tree (AST).
2. **Logical Planner:** Resolves schema identifiers, desugars operators, and binds types.
3. **Optimizer (CBO):** Applies predicate pushdown, projection pruning, index selection, and join reordering using cost statistics.
4. **Physical Planner:** Generates an executable plan tree comprising vectorized operators (`SeqScan`, `IndexScan`, `HashJoin`, `HNSWSearch`, `Aggregate`).
5. **Worker Pool:** Multi-threaded work-stealing scheduler divides data blocks into morsels processed across available CPU cores.

---

## 7. Sync, Replication & Real-Time

### 7.1 Reactive Live Queries (Continuous Query Evaluation)
* LoomDB maintains a register of active subscription queries.
* As transactions commit to the WAL, affected keys and table namespaces trigger index differential evaluation.
* Only the delta changes (insertions, updates, deletions) are pushed to listening client streams over WebSockets or gRPC streams.

### 7.2 Conflict-Free Sync (Delta-State CRDTs)
* Embedded LoomDB nodes can operate fully offline.
* Records can be tagged with Vector Clocks and Last-Write-Wins (LWW) or Delta-CRDT merge rules.
* When reconnected, nodes exchange state vectors and sync missing deltas bi-directionally without manual reconciliation code.

---

## 8. Security & Cryptographic Isolation

* **Encryption at Rest:** AES-256-GCM or ChaCha20-Poly1305 applied to SSTable blocks and WAL records.
* **Envelope Encryption & Tenant Keys:** Support for distinct key-encryption keys (KEK) per tenant, enabling immediate cryptographically-enforced tenant deletion (crypto-shredding).
* **Wire Protocol Security:** Native TLS 1.3 for standalone server mode, mutual TLS (mTLS) for cluster nodes.
