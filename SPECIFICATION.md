# LoomDB Technical Specification 📐

This document specifies the exact on-disk binary file formats, memory representations, network wire protocol, and index structures for LoomDB.

---

## 1. Directory & File Organization

A LoomDB database directory is organized as follows:

```
<database_path>/
├── MANIFEST                    # Database metadata, active SSTables, and log pointers
├── CURRENT                     # Text pointer referencing current valid MANIFEST-XXXXXX
├── wal/
│   ├── 000001.wal              # Active Write-Ahead Log
│   └── 000002.wal              # Rotated WAL ready for flush
├── vlog/
│   ├── 000001.vlog             # Append-only Value Log for large payload separation
│   └── 000002.vlog
├── sst/
│   ├── L0/                     # Level 0 SSTables (freshly flushed from MemTable)
│   │   ├── 000010.sst
│   │   └── 000011.sst
│   ├── L1/                     # Level 1 SSTables (non-overlapping key ranges)
│   └── L2/                     # Deeper levels with larger capacity
└── indexes/
    ├── hnsw/                   # Serialized HNSW vector graphs
    └── fts/                    # Inverted term dictionary for full-text search
```

---

## 2. On-Disk Binary Formats

### 2.1 Write-Ahead Log (WAL) Format
Every write transaction generates one or more WAL records. The log consists of a sequence of 32KB blocks, each containing variable-sized chunk records.

```
┌─────────────────┬──────────────────┬─────────────────┬─────────────────┬──────────────────────┐
│  xxHash3 (8B)   │  Record Len (4B) │ Record Type(1B) │  Timestamp(8B)  │  Payload (variable)  │
└─────────────────┴──────────────────┴─────────────────┴─────────────────┴──────────────────────┘
```

* **xxHash3 (8 Bytes):** Little-endian 64-bit checksum of the record header and payload.
* **Record Len (4 Bytes):** Length of payload in bytes (maximum payload: 16MB per chunk).
* **Record Type (1 Byte):**
  * `0x01` = Complete Record (fits entirely in block)
  * `0x02` = First Chunk (spanning multiple blocks)
  * `0x03` = Middle Chunk
  * `0x04` = Last Chunk
  * `0x05` = Commit Marker
  * `0x06` = Rollback Marker
* **Timestamp (8 Bytes):** Nanosecond UNIX epoch timestamp.
* **Payload:** Binary serialized operations: `[OpCode: 1B, KeyLen: 2B, Key: bytes, ValLen: 4B, Value: bytes]`.

---

### 2.2 SSTable (.sst) File Format
An SSTable stores sorted keys with pointers into the data blocks or the Value Log.

```
┌────────────────────────────────────────────────────────┐
│                   Data Block 0                         │
├────────────────────────────────────────────────────────┤
│                   Data Block 1                         │
├────────────────────────────────────────────────────────┤
│                      ...                               │
├────────────────────────────────────────────────────────┤
│                   Data Block N                         │
├────────────────────────────────────────────────────────┤
│                   Filter Block (Blocked Bloom Filter)  │
├────────────────────────────────────────────────────────┤
│                   Index Block (Two-Level B-Tree)       │
├────────────────────────────────────────────────────────┤
│                   Meta Properties Block                │
├────────────────────────────────────────────────────────┤
│                   Footer (Fixed 48 Bytes)              │
└────────────────────────────────────────────────────────┘
```

#### SSTable Footer (Fixed 48 Bytes):
```
┌───────────────────────────────────┬──────────────┐
│ Field                             │ Size         │
├───────────────────────────────────┼──────────────┤
│ Metaindex Handle (Offset + Size)  │ 16 Bytes     │
│ Index Handle (Offset + Size)      │ 16 Bytes     │
│ Block Filter Handle (Offset+Size) │ 12 Bytes     │
│ Magic Number (0x4C4F4F4D44423031) │ 8 Bytes      │
└───────────────────────────────────┴──────────────┘
```
* **Magic Number:** `0x4C4F4F4D44423031` (ASCII: `"LOOMDB01"`).

#### Data Block Layout:
* Compressed using **Zstandard** (level 3) or **LZ4**.
* Entries are restart-point prefix-compressed to minimize key redundancy:
  * `shared_prefix_len (2B)` + `unshared_len (2B)` + `val_len (4B)` + `key_delta` + `value`.
* Restart points table located at the tail of the block for \(O(\log K)\) binary search inside the block.

---

### 2.3 Value Log (vLog) Entry Format
Values separated out of the LSM tree are appended sequentially to the `vLog`:

```
┌──────────────┬──────────────┬──────────────┬───────────────┬────────────────────────┐
│ CRC32C (4B)  │ Key Len (2B) │ Val Len (4B) │ User Key (var)│ Value Payload (var)    │
└──────────────┴──────────────┴──────────────┴───────────────┴────────────────────────┘
```

The corresponding LSM entry only needs to hold a compact 16-byte `ValuePtr`:
```rust
#[repr(C, packed)]
pub struct ValuePtr {
    pub vlog_file_id: u32,
    pub offset: u64,
    pub length: u32,
}
```

---

## 3. Vector Engine Specification (HNSW)

The Hierarchical Navigable Small World (HNSW) graph index maps multi-dimensional embeddings:

### 3.1 Graph Parameters
* **Vector Dimensions:** Dynamic (e.g., 384, 768, 1536, 3072).
* **Element Data Types:**
  * `Float32` (Standard IEEE 754)
  * `Float16` (Half precision)
  * `Int8` (Scalar Quantization with min/max calibration)
* **Default Hyperparameters:**
  * `M`: 16 (Number of bidirectional links per node)
  * `M0`: 32 (Max links at layer 0)
  * `efConstruction`: 128 (Size of dynamic candidate list during build)
  * `efSearch`: 64 (Size of candidate list during query)

### 3.2 Distance Functions
Computed using target-specific SIMD instruction sets:
1. **Cosine Similarity:**
   $$\text{Cosine}(A, B) = \frac{A \cdot B}{\|A\|_2 \|B\|_2}$$
2. **Squared Euclidean ($L_2$) Distance:**
   $$L_2^2(A, B) = \sum_{i=1}^d (A_i - B_i)^2$$
3. **Dot Product:**
   $$\text{Dot}(A, B) = \sum_{i=1}^d A_i B_i$$

---

## 4. Network Wire Protocol (LoomWire)

When running in Standalone Server mode, clients can connect via **gRPC** or the low-overhead **LoomWire** binary TCP protocol.

### 4.1 LoomWire Frame Layout

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

### 4.2 Message Types
* `0x01`: `QUERY_REQUEST` (SQL / LoomQL query string + parameters)
* `0x02`: `QUERY_RESPONSE` (Arrow record batches or row streaming)
* `0x03`: `SUBSCRIBE_REQUEST` (Live query subscription)
* `0x04`: `SUBSCRIPTION_DELTA` (Streaming change notification)
* `0x05`: `TRANSACTION_BEGIN`
* `0x06`: `TRANSACTION_COMMIT`
* `0x07`: `TRANSACTION_ROLLBACK`
* `0x08`: `HEARTBEAT_PING` / `HEARTBEAT_PONG`
* `0xFF`: `ERROR_FRAME`
