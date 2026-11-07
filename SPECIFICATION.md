# LoomDB Technical Specification

This document specifies the exact on-disk binary container file format (`.loom`), stable ABI, memory representations, network wire protocol, and the 6 core storage novelty primitives of LoomDB.

> **Positioning:** LoomDB is a forkable, deterministic, budget-enforced, zero-copy embedded multi-modal database with a stable single-file ABI and native OPFS multi-tab coordination.

---

## 1. The Single-File `.loom` Container Format & Stable ABI

All tables, indexes, vectors, branches, and WAL journals are encapsulated within a single portable file:

```
+-------------------------------------------------------------------------+
|                        Header Page (128 Bytes)                          |
+-------------------------------------------------------------------------+
|                     Allocation Bitmap Pages (4KB each)                  |
+-------------------------------------------------------------------------+
|                     Virtual Inode Registry Page (4KB)                   |
+-------------------------------------------------------------------------+
|                     Virtual Data & Index Pages (0..N)                   |
|  - Circular WAL Ring Pages                                              |
|  - B+Tree Table Pages                                                   |
|  - HNSW Vector Graph Pages                                              |
|  - LoomBSON Document Pages                                              |
|  - Copy-on-Write (COW) Branch Page Directories                          |
|  - Free / Recycled Pages                                                |
+-------------------------------------------------------------------------+
```

### 1.1 Header Structure (Fixed 128 Bytes, Offset 0)

```
Offset  Size (B)  Field Name             Description
0       8         magic                  ASCII: "LOOMDB01" (0x4C4F4F4D44423031)
8       2         page_size              Page size in bytes (512 .. 65536, default: 4096)
10      2         format_version         Format version (currently 1)
12      8         total_pages            Total allocated pages in container
20      8         free_page_head         Page ID of the first recycled free page
28      8         catalog_root_page      Root B+Tree page ID for system schema catalog
36      8         wal_ring_start_page    First page of in-container circular WAL
44      8         wal_ring_page_count    Number of pages allocated to WAL ring (e.g. 1024 pages = 4MB)
52      8         wal_head_offset        Current append offset in WAL ring
60      8         wal_tail_offset        Oldest uncheckpointed offset in WAL ring
68      8         sequence_number        Latest committed MVCC transaction sequence
76      8         generation_seqlock     64-bit atomic counter for lock-free reader synchronization
84      8         active_branch_root     Page ID of the currently active COW branch root directory
92      28        reserved               Zeroed for future cryptographic / layout expansion
120     8         header_checksum        xxHash3-64 checksum of bytes 0..119
```

---

### 1.2 Zero-Copy Mmap Read Path & Generation Seqlock

To eliminate deserialization overhead and heap copies:
1. **Direct Memory Mapping:** Point queries and table scans map `.loom` pages directly into virtual memory via `mmap`.
2. **Lock-Free Readers + Single-Writer Protocol:**
   - The writer increments `generation_seqlock` to an odd number before mutating pages and increments it to an even number upon commit.
   - Readers sample `generation_seqlock` before and after reading typed struct views directly off the memory-mapped slice.
   - If the sequence changed or was odd, the reader retries without acquiring blocking mutexes.
3. **Zero-Copy Views:** Rows, JSON documents, and vector embeddings return zero-copy typed byte slices pointing directly into OS page cache buffers.

---

### 1.3 Forkable Copy-on-Write Snapshots

LoomDB exposes branching as a first-class storage primitive:
* **`fork(branch_name)`:** Shallow-copies the active branch's page directory table in $O(1)$ time. 100% of underlying data pages are shared by default.
* **`checkout(branch_name)`:** Updates the active branch pointer in the inode header (`active_branch_root`).
* **`discard(branch_name)`:** Decrements page reference counts and reclaims unshared pages into `free_page_head`.
* **Page-Level COW:** When an update occurs on a branch, only the specific modified 4KB page is cloned; unmodified pages remain shared.

---

### 1.4 Deterministic Flight Recorder & Replay WAL

LoomDB's Write-Ahead Log records execution entropy alongside mutations:
* **Recorded Envelope:**
  ```text
  [SeqNum: u64] [Timestamp: u64 (ns)] [Seed: u64] [Statement: String] [ExpectedStateHash: u64]
  ```
* **Bit-for-Bit Replay:** The replay engine reads the flight record stream, seeds the pseudo-random generator, sets the virtual timestamp, and verifies resulting state hashes at each step.

---

### 1.5 Hard Resource-Budget Governor

LoomDB turns "sub-512KB footprint" into an enforceable contract:
* **Declared Caps:** `max_ram_bytes`, `max_wal_bytes`, `max_page_cache_bytes`, `max_io_per_query`.
* **Enforcement:** If an operation attempts to exceed its declared ceiling, the engine raises `LoomError::BudgetExceeded` and applies backpressure rather than silently consuming host memory.
* **Queryable Budget:** Introspect live resource utilization via `SELECT * FROM _sys_budget;`.

---

### 1.6 OPFS Multi-Tab Coordination Protocol (Wasm & Browser)

Browser-native Single-Writer / Multi-Reader (SWMR) coordination over Origin Private File System:
1. **Exclusive Writer Leases:** Coordinated via the Web Locks API (`navigator.locks.request("loomdb_writer_{container}")`).
2. **Live CDC Broadcast:** When the active writer commits a transaction, it emits a commit notification across `BroadcastChannel("loomdb_cdc_{container}")`.
3. **Concurrent Multi-Reader Snapshots:** Non-writing tabs obtain shared read locks without blocking writes.
4. **Crash Recovery:** If a tab closes or crashes while holding a write lock, the browser automatically releases the Web Lock, allowing queued transactions to proceed immediately.

---

### 1.7 Self-Queryable System Telemetry

LoomDB stores internal engine metrics as a native time-series collection:
* **System Table:** `_sys_telemetry`
* **Schema:** `timestamp_us`, `query_count`, `latency_p50_us`, `latency_p99_us`, `cache_hits`, `wal_bytes`, `idle_ram_kb`.
* **Queryable via Standard SQL:**
  ```sql
  SELECT latency_p50_us, latency_p99_us, idle_ram_kb FROM _sys_telemetry LIMIT 5;
  ```

---

### 1.8 Composable Meta-Index Framework

In `crates/loom-index`, LoomDB provides primitive composability:
* Combines **B+Tree range scans** (`val BETWEEN x AND y`), **HNSW vector distance** (`cosine_similarity(vec, q)`), and **TimeRange filters** in a unified, single-pass iterator without materializing intermediate candidate sets.

---

### 1.2 Page Layouts

Every page is aligned to `page_size` (typically 4096 bytes). Each page starts with an 8-byte **Page Header**:

```
Offset  Size (B)  Field Name     Description
0       1         page_type      0x01: B+Tree Leaf, 0x02: B+Tree Internal,
                                 0x03: HNSW Node Page, 0x04: Document Page,
                                 0x05: Free Page
1       1         flags          Bit 0: Compressed, Bit 1: Overflow
2       2         cell_count     Number of entries / keys in this page
4       2         free_offset    Offset to start of unallocated page space
6       2         page_checksum  CRC16-CCITT of page payload
```

#### B+Tree Leaf Page Cell:
* `key_len (2B)` + `key_bytes` + `val_len (4B)` + `value_bytes` (or 16-byte `ValuePtr` if separated).
* Cell pointers are stored sequentially at the top of the page growing downward; actual cell bodies are stored at the bottom growing upward (Slotted Page Architecture).

#### HNSW Vector Graph Page:
* Stores graph adjacency lists compactly:
  * `node_id (u64)` + `vector_data ([f32; D] or [i8; D])` + `neighbor_count (u16)` + `neighbor_ids ([u64; M])`.

---

## 2. In-Memory Micro Architecture

### 2.1 The Micro Pager
When running in **Micro Profile** (e.g., embedded in IoT or Browser Wasm):
* No persistent background writer thread.
* In-flight mutations write to the in-container WAL ring.
* The pager caches only **16 to 64 pages** in memory (~64 KB to 256 KB RAM) using a lightweight Clock/LRU eviction algorithm.
* Point-lookups on clean pages read directly from the memory map (`mmap`) with zero heap copies.

---

## 3. Network Wire Protocol (LoomWire)

When running in Standalone Server mode:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       Magic (0x4C 0x4D)       |    Version    |   Msg Type    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Stream ID (u32)                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Flags (u32)                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Payload Length (u32)                   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Checksum (xxHash3, 8B)                 |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Payload Bytes ...                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

---

## 4. WebAssembly Execution Model

* Compiles target: `wasm32-unknown-unknown` and `wasm32-wasip1`.
* JavaScript Bridge: Pure TypeScript wrapper `@loomdb/wasm` (< 15 KB gzipped).
* Storage: Interacts with the browser's **Origin Private File System (OPFS)** using synchronous buffer reads/writes, allowing sub-millisecond local database queries in client-side web apps.
