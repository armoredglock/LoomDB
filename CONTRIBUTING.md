# Contributing to LoomDB 🛠️

Thank you for your interest in contributing to LoomDB! As a high-performance database engine, reliability, data safety, and crash-resilience are our highest priorities.

---

## 1. Prerequisites & Toolchain

* **Rust:** Current stable Rust (1.75+ or newer) with `cargo`, `rustfmt`, and `clippy`.
* **Python:** Python 3.10+ (for integration test harness and benchmarks).
* **Git:** For version control and branch management.

Verify your environment:
```bash
rustc --version
cargo --version
```

---

## 2. Getting Started & Building

Clone the repository and build all workspace crates:

```bash
# Debug build (fast compilation)
cargo build --workspace

# Release build (with LTO and SIMD optimizations)
cargo build --workspace --release
```

---

## 3. Testing Requirements

A database engine must never silently corrupt data or drop committed transactions. All contributions must pass the three tiers of verification:

### Tier 1: Unit & Crate Tests
```bash
cargo test --workspace --all-targets
```

### Tier 2: Crash-Consistency & Power Failure Injection
LoomDB includes crash-injection harnesses simulating abrupt power loss, split writes, and sudden process terminations (`SIGKILL`).
```bash
# Runs the crash recovery suite under chaos conditions
cargo test --test crash_recovery_harness -- --nocapture
```

### Tier 3: Memory Safety & Sanity
```bash
# Lint checks
cargo clippy --workspace -- -D warnings

# Formatting
cargo fmt --all -- --check
```

---

## 4. Code Architecture Rules

1. **Explicit Errors:** Never use `.unwrap()` or `.expect()` in non-test production code. Always propagate errors using `LoomResult<T>` and the `?` operator.
2. **Deterministic Durability:** Every function that claims durability must guarantee a synchronized write to disk before returning `Ok(())`.
3. **No Unbounded Allocations:** Buffer managers and MemTables must enforce hard memory limits to prevent out-of-memory (OOM) panic crashes.
4. **SIMD Portability:** Whenever writing SIMD kernels (e.g. for vector distance calculations), always provide a fallback scalar implementation for architectures without hardware vector extensions.

---

## 5. Submitting Pull Requests

1. Create a feature branch: `git checkout -b feature/lsm-block-cache`.
2. Commit with conventional commit messages:
   * `feat: implement 2Q block cache replacement policy`
   * `fix: prevent torn write on partial WAL chunk replay`
   * `perf: vectorize cosine similarity with AVX2`
3. Ensure all tests and clippy checks pass cleanly.
4. Open a Pull Request detailing the changes, benchmarks, and data-safety implications.
