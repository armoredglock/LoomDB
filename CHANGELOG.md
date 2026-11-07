# Changelog

All notable changes to LoomDB will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-07

### Added
- **Multi-Modal Single-File Engine**: Unified 64-bit virtual pager housing SQL Relational, LoomBSON Documents, HNSW Vector Graphs, and Gorilla Time-Series Streams into a single `.loom` file.
- **Sub-512KB Idle RAM Profile**: Zero background threads and sub-512KB memory overhead when idle.
- **In-Page HNSW Vector Traversal**: SIMD-accelerated AVX-512/NEON cosine similarity and Euclidean distance search fused with relational SQL filters.
- **In-Container Circular WAL**: Append-only circular ring buffer with group-commit synchronization and xxHash3/CRC32C block integrity checks.
- **LoomDB Interactive Shell (`loom`)**: Native terminal CLI supporting ANSI SQL execution, catalog inspection, dot commands (`.tables`, `.describe`, `.mode`, `.schema`, `.stats`), and tabular data output.
- **Local Web Studio Dashboard (`studio.html`)**: Single-file developer dashboard with live table visualizer, vector cluster viewer, and container health telemetry.
- **Cross-Platform Standalone Installers**:
  - Windows: Native Win32 standalone Setup Wizard (`loom-setup.exe`).
  - macOS: Single-file shell installer (`loom-setup-macos.sh`) supporting Apple Silicon (M1/M2/M3) and Intel.
  - Linux: Standalone shell installer (`loom-setup-linux.sh`) supporting x86_64 and ARM64.
- **Editorial Promotion & Docs Website**: Zero-bloat high-craft website featuring interactive query sandbox, paradigm weaving visualizer, and dynamic benchmark comparisons.
- **Automated CI/CD**: Matrix testing on GitHub Actions across Windows, macOS, and Linux.
