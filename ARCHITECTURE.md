# LoomDB Architecture & System Design 🏛️

> **Design Principle:** Uncompromising speed, ultra-low resource footprint, and zero operational friction.

---

## 1. Architectural Philosophy

LoomDB combines the self-contained simplicity of **SQLite** with the multi-modal richness of modern cloud databases (Document JSON, AI Vectors, Time-Series, and Live Reactive Queries) while maintaining a **sub-megabyte binary footprint** and **sub-megabyte idle RAM profile**.

### Key Architectural Pillars:
1. **Single-File Container (`.loom`):** No directory sprawl, no orphaned WAL files, no external metadata daemons. The entire database resides in a single, portable, copyable binary file.
2. **Zero-Copy Memory-Mapped Paging (`mmap`):** Direct memory mapping allows the operating system to manage physical page frames without intermediate buffer copies, enabling sub-microsecond point reads.
3. **Micro-to-Macro Elastic Scaling:** Configurable memory profiles allowing LoomDB to operate inside an ESP32 micro-controller / WebAssembly tab (<512 KB RAM) or dynamically expand to saturated multi-gigabyte NVMe server instances.
4. **Zero Idle Overhead:** When queries are not actively running, LoomDB allocates **0 background worker threads** and consumes **0 CPU cycles**.
5. **Wasm-First Compatibility:** Core storage and query engines are strictly separated from OS primitives, enabling direct compilation to `wasm32-unknown-unknown` with the Web Browser's **Origin Private File System (OPFS)**.

---

## 2. The Single-File `.loom` Container Architecture

Rather than littering disk with hundreds of SSTables and WAL files, LoomDB packages its storage hierarchy into a virtual page-allocated container:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        LoomDB Single File (.loom)                      │
├────────────────────────────────────────────────────────────────────────┤
│  File Header (128 Bytes: Magic, Version, Page Size, Root Pointers)     │
├────────────────────────────────────────────────────────────────────────┤
│  Free-Page Bitmap / Allocation Map (Tracks recycled 4KB pages)         │
├────────────────────────────────────────────────────────────────────────┤
│  Virtual Inode Table (Maps logical stores: WAL, Tables, Vector Graphs) │
├────────────────────────────────────────────────────────────────────────┤
│  Virtual Page Blocks:                                                  │
│  ├─ Page 004: In-file Circular WAL Log (Append-only write-buffer)     │
│  ├─ Page 005..060: B+Tree Internal & Leaf Pages (Relational & Primary)│
│  ├─ Page 061..120: LoomBSON Document Pages (Compressed variable JSON) │
│  ├─ Page 121..200: HNSW Vector Graph Pages (SIMD-aligned node arrays) │
│  └─ Page 201..250: Full-Text Inverted Posting Lists (BM25 Dictionary)  │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.1 File Header Specification (Fixed 128 Bytes)
* **Magic Bytes (8 Bytes):** `0x4C 0x4F 0x4F 0x4D 0x44 0x42 0x30 0x31` (`"LOOMDB01"`).
* **Page Size (2 Bytes):** Configurable from 512 bytes (Micro profile) to 64 KB (Server profile); default: 4096 bytes (4 KB).
* **Format Version (2 Bytes):** Format specification version integer.
* **Page Count (8 Bytes):** Total allocated pages in the file.
* **First Free Page (8 Bytes):** Head of the in-file free-list allocator.
* **Schema Catalog Page (8 Bytes):** Pointer to the root B+Tree of the system catalog.
* **Active WAL Head/Tail (16 Bytes):** Ring-buffer pointers for in-file atomic transactions.
* **Header xxHash3 (8 Bytes):** Checksum verifying header integrity before mounting.

---

## 3. Micro-Kernel Memory Profiles

LoomDB adapts its internal allocators dynamically based on the target platform:

```
┌────────────────────┬──────────────────┬───────────────────┬───────────────────┐
│ Metric / Component │ Micro Profile    │ Balanced Profile  │ Server Profile    │
│                    │ (IoT / Browser)  │ (Desktop / CLI)   │ (Cloud / Backend) │
├────────────────────┼──────────────────┼───────────────────┼───────────────────┤
│ Target RAM         │ < 512 KB         │ 8 MB - 32 MB      │ 64 MB - Unlimited │
│ Thread Model       │ Single-threaded  │ Thread-pool (2-4) │ Work-stealing Rayon│
│ Page Size          │ 512 B - 1 KB     │ 4 KB              │ 16 KB - 64 KB     │
│ Vector Index       │ 1-bit / SQ8 Quant│ SQ8 / FP16 Quant  │ Full FP32 + SIMD  │
│ WAL Mode           │ Sync on write    │ Group Commit (1ms)│ Async Group Commit│
│ Compaction         │ In-place vacuum  │ Background merger │ Parallel Leveled  │
└────────────────────┴──────────────────┴───────────────────┴───────────────────┘
```

---

## 4. Query Execution Engine

### 4.1 Fused Vector-Relational Traverser
In standard systems, combining a vector search with a SQL `WHERE` clause suffers from:
* **Pre-filtering:** Filters out records first, which often fragments the HNSW graph and prevents nearest-neighbor hops.
* **Post-filtering:** Searches the vector graph first, but discards non-matching results, frequently returning fewer results than `LIMIT k`.

LoomDB solves this with a **Single-Pass Bitmask Traverser**:
1. Relational and JSON indexes produce an ephemeral compressed **Roaring Bitmap** of valid candidate row IDs.
2. The HNSW greedy search algorithm steps through graph nodes: at each hop, it verifies node validity against the bitmask using a single bitwise `AND` CPU instruction.
3. Search guarantees both exact compliance with the `WHERE` clause and 100% graph reachability.

```
                  ┌──────────────────────────────┐
                  │ LoomQL Query with Hybrid Req │
                  └──────────────┬───────────────┘
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
       ┌───────────────────┐           ┌───────────────────┐
       │ Relational Filter │           │ Target Embedding  │
       │ (e.g. city='NYC') │           │ Vector [0.12, ...]│
       └─────────┬─────────┘           └─────────┬─────────┘
                 │                               │
                 ▼                               │
       ┌───────────────────┐                     │
       │ Fast Roaring      │                     │
       │ Filter Bitmask    │                     │
       └─────────┬─────────┘                     │
                 │                               │
                 └───────────────┬───────────────┘
                                 ▼
       ┌───────────────────────────────────────────────────┐
       │ Fused HNSW Graph Traverser                        │
       │ (SIMD Vector Dist + Bitmask Check in Inner Loop)  │
       └─────────────────────────┬─────────────────────────┘
                                 ▼
       ┌───────────────────────────────────────────────────┐
       │ Ranked Top-K Multi-Modal Result Stream            │
       └───────────────────────────────────────────────────┘
```

---

## 5. WebAssembly & Edge Browser Architecture

LoomDB targets `wasm32-unknown-unknown` without requiring Emscripten or bloated libc shims:
* **Storage Provider:** Uses the Web's **Origin Private File System (OPFS)** via `FileSystemSyncAccessHandle`, which provides direct block-level read/write performance identical to native desktop disk access.
* **Zero Network Dependency:** The database can run entirely client-side inside a browser tab, service worker, or mobile WebView.
* **Sync-Ready:** Client-side LoomDB instances capture local mutations with vector clocks and stream deltas to backend LoomDB cluster nodes via WebSockets when online.

---

## 6. Built-in Web Studio (Zero-Dependency UI)

To maximize developer resourcefulness, LoomDB includes an **Embedded Web Studio**:
* Consists of an optimized single-file HTML/JS/CSS application compressed and embedded directly inside the binary via `include_str!`.
* **Zero runtime dependencies:** Requires no Node.js, npm, or external CDN assets.
* When running `loomdb server --open-studio`, LoomDB spins up an HTTP server that serves the UI at `http://localhost:8484/studio`, offering:
  * Interactive SQL / LoomQL query console with syntax highlighting.
  * Real-time 3D vector space embedding visualizer.
  * Table and JSON document editor.
  * Live query change stream monitor.
  * Storage and page fragmentation analyzer.
