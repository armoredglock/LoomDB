# LoomDB Technical Specification 📐

This document specifies the exact on-disk binary container file format (`.loom`), memory representations, network wire protocol, and index structures for LoomDB.

---

## 1. The Single-File `.loom` Container Format

All tables, indexes, vectors, and WAL journals can be encapsulated within a single portable file:

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
68      8         sequence_number        Latest committed MVCC transaction timestamp
76      44        reserved               Zeroed for future cryptographic / layout expansion
120     8         header_checksum        xxHash3-64 checksum of bytes 0..119
```

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
