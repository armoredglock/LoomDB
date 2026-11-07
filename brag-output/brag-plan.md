# LoomDB Launch Video Plan (/brag-slim)

## 1. Project Profile & Angle
* **Product:** LoomDB &mdash; The World's Lightest Multi-Modal Database.
* **Core Positioning Sentence:**
  > *LoomDB is a forkable, deterministic, budget-enforced, zero-copy embedded multi-modal database with a stable single-file ABI and native OPFS multi-tab coordination.*
* **Core Novelty Bundle:**
  1. **Forkable COW Snapshots:** Cheap copy-on-write page branching (`fork`, `checkout`, `discard`) inside a single `.loom` file.
  2. **Deterministic Flight Recorder + Replay:** WAL capturing non-deterministic seeds, timestamps, and queries for bit-for-bit replay.
  3. **Hard Resource-Budget Governor:** Strict RAM/WAL caps enforced via backpressure and errors, exposed via queryable `_sys_budget`.
  4. **Zero-Copy Mmap Read Path + Stable ABI:** Lock-free readers with generation seqlock and typed slice views over memory-mapped pages.
  5. **Native OPFS Multi-Tab Coordination:** Single-writer / multi-reader coordination in browsers via Web Locks API and BroadcastChannel CDC.
* **Credibility Multipliers:**
  - Composable index framework ("meta-index")
  - Public file format spec & crash/fuzz test suite
  - Portable footprint benchmark suite (<512KB idle RAM, <850KB binary)
* **Target Audience:** Systems engineers, edge device developers, local AI agent developers, browser-first desktop apps (Tauri/Electron/Wasm).
* **Tone:** `polished` &middot; Razor-sharp, confident, technical, high-craft systems design.
* **Format:** Landscape (1920x1080), 30 fps, exactly 20.0 seconds (600 frames).
* **Music Tempo:** 120 BPM (0.5s per beat, 15 frames per beat). Every scene cut aligns strictly with a downbeat.

---

## 2. Storyboard & Scene Breakdown

| Scene | Time | Frames | Musical Beat | Visual Action & Content |
|---|---|---|---|---|
| **Scene 1: The Problem** | 0.0s &ndash; 3.0s | 0 &ndash; 89 | Beat 1 &ndash; 6 | Dark blueprint grid. "EMBEDDED DATABASES ARE MISSING CORE GUARANTEES." Red contrast cards highlighting lack of cheap branching, non-deterministic edge crashes, and un-governed OOM panics. Hook question slams in: "WHAT IF AN EMBEDDED ENGINE GAVE YOU HARD GUARANTEES?" |
| **Scene 2: Core Novelty Reveal** | 3.0s &ndash; 7.0s | 90 &ndash; 209 | Beat 7 &ndash; 14 | Downbeat impact hit. LoomDB logo and positioning statement reveal: "Forkable, Deterministic, Budget-Enforced, Zero-Copy Embedded Multi-Modal Database." 5 novelty cards illuminate: COW Snapshots, Flight Recorder, Budget Governor, Zero-Copy ABI, OPFS Coordination. |
| **Scene 3: Live Verification** | 7.0s &ndash; 12.0s | 210 &ndash; 359 | Beat 15 &ndash; 24 | High-resolution terminal window. Live query typing: `FORK BRANCH 'experiment'`, `SELECT * FROM _sys_budget;` showing 8.2% RAM enforced cap, and `SELECT replay_verify(tx_id=402);` showing bit-for-bit replay verified in 0.21 ms. |
| **Scene 4: OPFS & Benchmarks** | 12.0s &ndash; 16.0s | 360 &ndash; 479 | Beat 25 &ndash; 32 | Split view. Left: Browser OPFS multi-tab coordination diagram showing Web Locks & BroadcastChannel CDC. Right: Footprint benchmark knockdown bar chart: LoomDB (<0.48 MB) vs SQLite (4.2 MB) vs DuckDB (32 MB) vs MongoDB (350 MB). |
| **Scene 5: CTA & GitHub Launch** | 16.0s &ndash; 20.0s | 480 &ndash; 600 | Beat 33 &ndash; 40 | Climax outro. "THE REINVENTED EMBEDDED DATABASE." Standalone installers for Windows (loom-setup.exe), macOS, and Linux/Wasm. GitHub repo URL: `github.com/armoredglock/LoomDB`. |

---

## 3. Audio & Soundtrack Architecture
* **Tempo:** 120 BPM (4/4 time signature).
* **Key:** D minor (Deep electronic synth bass, tight 808 kick on beats 1 & 3, crisp snare on beats 2 & 4, subtle 16th-note metallic hi-hats).
* **Sound Effects:**
  * Downbeat impact hits at Frame 0, 90, 210, 360, 480.
  * Keystroke mechanical chatter during terminal query execution.
  * Rising synth riser leading up to the climax outro at Frame 460-480.
